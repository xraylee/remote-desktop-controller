// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Core error types shared across all RDCS crates.

use thiserror::Error;

/// Top-level error type for the RDCS system.
///
/// Each variant maps to a major subsystem so callers can match on the
/// error category without depending on every subsystem crate.
#[derive(Debug, Error)]
pub enum RdcsError {
    /// Cryptographic operation failed (key exchange, encryption, signing).
    #[error("crypto error: {0}")]
    Crypto(String),

    /// Network transport failure (UDP, relay, signaling).
    #[error("transport error: {0}")]
    Transport(String),

    /// Platform-specific failure (screen capture, input injection, audio).
    #[error("platform error: {0}")]
    Platform(String),

    /// Connection lifecycle failure (ICE, NAT traversal, heartbeat).
    #[error("connection error: {0}")]
    Connection(String),

    /// File transfer or clipboard synchronization failure.
    #[error("transfer error: {0}")]
    Transfer(String),

    /// FFI boundary error when calling from or returning to Flutter/Dart.
    #[error("ffi error: {0}")]
    Ffi(String),

    /// Configuration parsing or validation failure.
    #[error("config error: {0}")]
    Config(String),

    /// An I/O error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Catch-all for errors that do not fit other categories.
    #[error("{0}")]
    Other(String),
}

/// Convenience alias used throughout the workspace.
pub type RdcsResult<T> = Result<T, RdcsError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        let err = RdcsError::Crypto("key exchange failed".into());
        assert_eq!(err.to_string(), "crypto error: key exchange failed");

        let err = RdcsError::Transport("timeout".into());
        assert_eq!(err.to_string(), "transport error: timeout");

        let err = RdcsError::Config("missing field".into());
        assert_eq!(err.to_string(), "config error: missing field");
    }

    #[test]
    fn io_error_converts() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let rdcs_err: RdcsError = io_err.into();
        assert!(matches!(rdcs_err, RdcsError::Io(_)));
    }
}
