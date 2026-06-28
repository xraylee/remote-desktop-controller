// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the codec subsystem.

use thiserror::Error;

/// Errors from the codec subsystem.
#[derive(Debug, Error)]
pub enum CodecError {
    /// The requested codec is not available on this platform.
    #[error("codec not available: {0}")]
    NotAvailable(String),

    /// Unsupported codec.
    #[error("unsupported codec: {0}")]
    UnsupportedCodec(String),

    /// Encoding failed.
    #[error("encode error: {0}")]
    EncodeError(String),

    /// Encoder initialization failed.
    #[error("encoder init failed: {0}")]
    EncoderInitFailed(String),

    /// Encoding operation failed.
    #[error("encode failed: {0}")]
    EncodeFailed(String),

    /// Decoding failed.
    #[error("decode error: {0}")]
    DecodeError(String),

    /// Decoder initialization failed.
    #[error("decoder init failed: {0}")]
    DecoderInitFailed(String),

    /// Decoding operation failed.
    #[error("decode failed: {0}")]
    DecodeFailed(String),

    /// Invalid configuration parameters.
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// Invalid frame size.
    #[error("invalid frame size: expected {expected:?}, got {actual:?}")]
    InvalidFrameSize {
        expected: (u32, u32),
        actual: (u32, u32),
    },

    /// The encoder or decoder has not been configured yet.
    #[error("not configured: call configure() before processing frames")]
    NotConfigured,

    /// A platform error occurred during hardware acceleration.
    #[error("platform error: {0}")]
    Platform(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<rdcs_platform::PlatformError> for CodecError {
    fn from(err: rdcs_platform::PlatformError) -> Self {
        CodecError::Platform(err.to_string())
    }
}
