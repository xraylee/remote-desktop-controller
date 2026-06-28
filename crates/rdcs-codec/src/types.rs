// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Common types for the codec subsystem.

use serde::{Deserialize, Serialize};

/// Video codec type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VideoCodec {
    /// H.264 / AVC
    H264,
    /// H.265 / HEVC
    H265,
    /// VP8
    VP8,
    /// VP9
    VP9,
    /// AV1
    AV1,
}

/// Video resolution presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VideoResolution {
    /// 1280x720 (HD)
    HD720,
    /// 1920x1080 (Full HD)
    HD1080,
    /// 2560x1440 (2K)
    HD1440,
    /// 3840x2160 (4K)
    UHD4K,
    /// Custom resolution
    Custom(u32, u32),
}

impl VideoResolution {
    /// Get the width and height dimensions.
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            VideoResolution::HD720 => (1280, 720),
            VideoResolution::HD1080 => (1920, 1080),
            VideoResolution::HD1440 => (2560, 1440),
            VideoResolution::UHD4K => (3840, 2160),
            VideoResolution::Custom(w, h) => (*w, *h),
        }
    }

    /// Get the total number of pixels.
    pub fn pixel_count(&self) -> u32 {
        let (w, h) = self.dimensions();
        w * h
    }
}

/// Video frame data.
#[derive(Debug, Clone)]
pub struct Frame {
    /// Frame width in pixels.
    pub width: u32,

    /// Frame height in pixels.
    pub height: u32,

    /// Frame data (YUV420 format: Y plane + U plane + V plane).
    pub data: Vec<u8>,

    /// Presentation timestamp in microseconds.
    pub timestamp_us: u64,

    /// Whether this is a keyframe.
    pub is_keyframe: bool,
}

impl Frame {
    /// Create a new frame with the given dimensions.
    pub fn new(width: u32, height: u32, timestamp_us: u64) -> Self {
        let size = (width * height * 3 / 2) as usize; // YUV420
        Self {
            width,
            height,
            data: vec![0u8; size],
            timestamp_us,
            is_keyframe: false,
        }
    }

    /// Create a test frame filled with a pattern.
    pub fn test_frame(width: u32, height: u32) -> Self {
        let mut frame = Self::new(width, height, 0);

        // Fill with a gradient pattern
        let y_size = (width * height) as usize;
        for i in 0..y_size {
            frame.data[i] = ((i * 255) / y_size) as u8;
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution_dimensions() {
        assert_eq!(VideoResolution::HD720.dimensions(), (1280, 720));
        assert_eq!(VideoResolution::HD1080.dimensions(), (1920, 1080));
        assert_eq!(VideoResolution::HD1440.dimensions(), (2560, 1440));
        assert_eq!(VideoResolution::UHD4K.dimensions(), (3840, 2160));
    }

    #[test]
    fn test_custom_resolution() {
        let custom = VideoResolution::Custom(1024, 768);
        assert_eq!(custom.dimensions(), (1024, 768));
        assert_eq!(custom.pixel_count(), 1024 * 768);
    }

    #[test]
    fn test_frame_creation() {
        let frame = Frame::new(1920, 1080, 0);
        assert_eq!(frame.width, 1920);
        assert_eq!(frame.height, 1080);
        assert_eq!(frame.data.len(), 1920 * 1080 * 3 / 2); // YUV420
    }

    #[test]
    fn test_test_frame() {
        let frame = Frame::test_frame(640, 480);
        assert_eq!(frame.width, 640);
        assert_eq!(frame.height, 480);
        assert!(!frame.data.is_empty());
    }
}
