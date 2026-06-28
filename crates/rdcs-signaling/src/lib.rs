// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! RDCS Signaling Server library.
//!
//! Axum-based HTTP/WebSocket signaling service for peer discovery,
//! NAT traversal negotiation, and session establishment.
//!
//! This module is the library entry point, exposing all sub-modules
//! and the router constructors used by both the binary and integration
//! tests.

pub mod config;
pub mod error;
pub mod handlers;
pub mod ice_config;
pub mod mdns;
pub mod mdns_bridge;
pub mod redis;
pub mod scaling;
pub mod ws;

use axum::routing::get;
use axum::{Json, Router};
use serde_json::Value;

use crate::ws::AppState;

/// Build the application router with a default [`AppState`] (no Redis pool).
///
/// This is the entry point used by integration tests that don't need a
/// live Redis connection.
pub fn router() -> Router {
    router_with_state(AppState::default())
}

/// Build the application router with the given [`AppState`].
///
/// Used by `main` to inject a real Redis pool, and by tests that want
/// to supply a custom [`SessionManager`](crate::ws::SessionManager).
pub fn router_with_state(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/ws", get(ws::ws_handler))
        .with_state(state)
}

/// `GET /health` — lightweight liveness probe.
///
/// Returns `{"status": "ok"}` with HTTP 200.
async fn health_handler() -> Json<Value> {
    Json(serde_json::json!({"status": "ok"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_returns_200_and_json() {
        let app = router();

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let value: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value, serde_json::json!({"status": "ok"}));
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let app = router();

        let request = Request::builder()
            .uri("/does-not-exist")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
