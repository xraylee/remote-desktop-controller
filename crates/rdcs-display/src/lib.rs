// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! rdcs-display: Video display and rendering for RDCS remote desktop.
//!
//! This module provides a cross-platform video display implementation using SDL2.
//! It accepts decoded video frames (BGRA format) and renders them in a window.
//!
//! # Features
//!
//! - Cross-platform window creation (macOS, Windows, Linux)
//! - Hardware-accelerated rendering via SDL2
//! - Automatic scaling and aspect ratio preservation
//! - Frame rate limiting and VSync support
//! - Performance monitoring (FPS, frame drops)
//!
//! # Example
//!
//! ```no_run
//! use rdcs_display::{VideoDisplay, DisplayConfig};
//! use rdcs_platform::CapturedFrame;
//!
//! let config = DisplayConfig::default()
//!     .with_title("Remote Desktop")
//!     .with_size(1920, 1080);
//!
//! let mut display = VideoDisplay::new(config)?;
//!
//! // In your render loop:
//! let frame: CapturedFrame = // ... decode from network
//! display.render_frame(&frame)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod error;
pub mod renderer;
pub mod window;

pub use error::{DisplayError, Result};
pub use renderer::VideoRenderer;
pub use window::{DisplayConfig, VideoDisplay};

// Re-export key types
pub use rdcs_platform::CapturedFrame;
