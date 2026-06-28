// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Centralized error type for the signaling server.
//!
//! [`AppError`] implements [`axum::response::IntoResponse`] so handlers can
//! return `Result<T, AppError>` and have errors automatically converted to
//! appropriate HTTP responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Application-wide error type.
///
/// Each variant maps to an HTTP status code and a JSON body so that clients
/// always receive structured error responses.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Redis connection or command error.
    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// JSON serialization / deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Requested resource was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Client exceeded a rate limit.
    #[error("rate limited, retry after {retry_after_secs}s")]
    RateLimited {
        /// Number of seconds the client should wait before retrying.
        retry_after_secs: u64,
    },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::Redis(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("redis error: {err}"),
            ),
            Self::Json(err) => (
                StatusCode::BAD_REQUEST,
                format!("json error: {err}"),
            ),
            Self::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                msg.clone(),
            ),
            Self::RateLimited { retry_after_secs } => (
                StatusCode::TOO_MANY_REQUESTS,
                format!("rate limited, retry after {retry_after_secs}s"),
            ),
        };

        let body = serde_json::json!({
            "error": message,
        });

        let mut response = (status, axum::Json(body)).into_response();

        if let Self::RateLimited { retry_after_secs } = &self {
            let header_value = format!("{retry_after_secs}");
            response.headers_mut().insert(
                axum::http::header::RETRY_AFTER,
                header_value.parse().unwrap(),
            );
        }

        // Log server-side errors (5xx) at error level, client errors at warn.
        if status.is_server_error() {
            tracing::error!(%status, %self, "internal error");
        } else {
            tracing::warn!(%status, %self, "client error");
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn not_found_status() {
        let err = AppError::NotFound("session abc".into());
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn rate_limited_status_and_header() {
        let err = AppError::RateLimited { retry_after_secs: 30 };
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(
            resp.headers().get(axum::http::header::RETRY_AFTER).unwrap(),
            "30"
        );
    }

    #[test]
    fn redis_error_is_500() {
        let redis_err: redis::RedisError =
            (redis::ErrorKind::IoError, "connection refused").into();
        let err = AppError::Redis(redis_err);
        let resp = err.into_response();
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
