// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Error types for the display module.

use thiserror::Error;

/// Result type for display operations.
pub type Result<T> = std::result::Result<T, DisplayError>;

/// Errors that can occur during display operations.
#[derive(Debug, Error)]
pub enum DisplayError {
    /// SDL2 initialization failed.
    #[error("SDL2 initialization failed: {0}")]
    SdlInitFailed(String),

    /// Window creation failed.
    #[error("window creation failed: {0}")]
    WindowCreationFailed(String),

    /// Renderer creation failed.
    #[error("renderer creation failed: {0}")]
    RendererCreationFailed(String),

    /// Texture creation failed.
    #[error("texture creation failed: {0}")]
    TextureCreationFailed(String),

    /// Frame rendering failed.
    #[error("frame rendering failed: {0}")]
    RenderFailed(String),

    /// Invalid frame format.
    #[error("invalid frame format: {0}")]
    InvalidFrameFormat(String),

    /// Invalid configuration.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Platform error from rdcs-platform.
    #[error("platform error: {0}")]
    Platform(#[from] rdcs_platform::PlatformError),
}

impl From<String> for DisplayError {
    fn from(s: String) -> Self {
        DisplayError::RenderFailed(s)
    }
}
