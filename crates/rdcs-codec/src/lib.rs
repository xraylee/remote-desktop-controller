// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! rdcs-codec: Video encoding/decoding pipeline for the RDCS remote desktop system.
//!
//! Provides content analysis (text/video scene detection), video encoder/decoder
//! traits with stub implementations, adaptive quality control, and encode/decode
//! pipelines that tie everything together.

pub mod adaptive;
pub mod analyzer;
pub mod decoder;
pub mod encoder;
pub mod error;
// peer_connection.rs 已废弃 - 迁移至方案 B（webrtc-rs + 平台原生编解码）
pub mod pipeline;
#[cfg(any(feature = "hardware-accel", feature = "software-encoder"))]
pub mod platform;
// pub mod rtp;  // 暂时禁用 - 等待 WebRTC 集成完成
// pub mod session;  // 暂时禁用 - 等待 WebRTC 集成完成
pub mod types;
// webrtc_decoder.rs 和 webrtc_encoder.rs 已废弃 - 使用 platform/* 模块代替

use thiserror::Error;
pub use error::CodecError as CodecErrorNew;

/// Convenience result type for codec operations.
pub type Result<T> = std::result::Result<T, CodecError>;

/// Errors from the codec subsystem.
#[derive(Debug, Error)]
pub enum CodecError {
    /// The requested codec is not available on this platform.
    #[error("codec not available: {0}")]
    NotAvailable(String),

    /// Encoding failed.
    #[error("encode error: {0}")]
    EncodeError(String),

    /// Decoding failed.
    #[error("decode error: {0}")]
    DecodeError(String),

    /// Invalid configuration parameters.
    #[error("invalid config: {0}")]
    InvalidConfig(String),

    /// The encoder or decoder has not been configured yet.
    #[error("not configured: call configure() before processing frames")]
    NotConfigured,

    /// A platform error occurred during hardware acceleration.
    #[error("platform error: {0}")]
    Platform(#[from] rdcs_platform::PlatformError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = CodecError::NotAvailable("H.265".into());
        assert_eq!(err.to_string(), "codec not available: H.265");
    }

    #[test]
    fn not_configured_error() {
        let err = CodecError::NotConfigured;
        assert!(err.to_string().contains("not configured"));
    }
}
