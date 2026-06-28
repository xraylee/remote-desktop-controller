// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Video encoder abstraction supporting hardware-accelerated codecs,
//! plus a stub implementation for testing.

use rdcs_platform::CapturedFrame;
use serde::{Deserialize, Serialize};

use crate::{CodecError, Result};

// ---------------------------------------------------------------------------
// Codec type
// ---------------------------------------------------------------------------

/// Supported video codecs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CodecType {
    /// H.264 / AVC — widely supported, good compatibility.
    H264,
    /// H.265 / HEVC — better compression, newer hardware.
    H265,
    /// VP9 — open codec, good browser support.
    Vp9,
    /// AV1 — next-gen open codec, best compression.
    Av1,
}

/// Backward-compatible alias.
pub type VideoCodec = CodecType;

// ---------------------------------------------------------------------------
// Encoder configuration
// ---------------------------------------------------------------------------

/// Encoder configuration parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncoderConfig {
    /// Video codec to use.
    pub codec: CodecType,
    /// Output width in pixels.
    pub width: u32,
    /// Output height in pixels.
    pub height: u32,
    /// Target frame rate.
    pub target_fps: u32,
    /// Target bitrate in bits per second.
    pub target_bitrate_bps: u64,
    /// Keyframe interval in frames.
    pub keyframe_interval: u32,
    /// Whether to attempt hardware-accelerated encoding.
    pub hardware_accel: bool,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            codec: CodecType::H264,
            width: 1920,
            height: 1080,
            target_fps: 60,
            target_bitrate_bps: 4_000_000,
            keyframe_interval: 120,
            hardware_accel: false,
        }
    }
}

impl EncoderConfig {
    /// Create a config for 1080p H.264 encoding.
    pub fn full_hd() -> Self {
        Self {
            width: 1920,
            height: 1080,
            ..Self::default()
        }
    }

    /// Create a config for 720p encoding.
    pub fn hd_720p() -> Self {
        Self {
            width: 1280,
            height: 720,
            ..Self::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Encoded frame
// ---------------------------------------------------------------------------

/// An encoded video frame (NAL unit or equivalent bitstream).
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    /// Encoded bitstream data.
    pub data: Vec<u8>,
    /// Whether this is a keyframe (IDR/I-frame).
    pub is_keyframe: bool,
    /// Presentation timestamp in microseconds.
    pub pts_us: u64,
    /// Decode timestamp in microseconds.
    pub dts_us: u64,
    /// Codec used for this frame.
    pub codec: CodecType,
    /// Original frame width before encoding.
    pub width: u32,
    /// Original frame height before encoding.
    pub height: u32,
}

// ---------------------------------------------------------------------------
// VideoEncoder trait
// ---------------------------------------------------------------------------

/// Trait for video encoders (hardware or software).
pub trait VideoEncoder: Send {
    /// Configure the encoder with the given parameters.
    fn configure(&mut self, config: EncoderConfig) -> Result<()>;

    /// Encode a captured frame and return the encoded output.
    fn encode(&mut self, frame: &CapturedFrame) -> Result<EncodedFrame>;

    /// Flush any buffered frames from the encoder.
    fn flush(&mut self) -> Result<Vec<EncodedFrame>>;
}

// ---------------------------------------------------------------------------
// StubEncoder — pass-through encoder for testing
// ---------------------------------------------------------------------------

/// Magic bytes identifying a stub-encoded frame.
const STUB_MAGIC: &[u8; 4] = b"STUB";

/// A stub encoder that wraps raw pixel data with a simple header.
///
/// This is not a real codec — it preserves the original pixel data so that
/// the roundtrip encode → decode path can be tested without platform-specific
/// hardware encoder integration.
///
/// Stub wire format (little-endian):
/// ```text
/// [0..4]   magic "STUB"
/// [4..8]   width  (u32)
/// [8..12]  height (u32)
/// [12..16] stride (u32)
/// [16..24] timestamp_us (u64)
/// [24..28] pixel_data_len (u32)
/// [28..]   pixel_data
/// ```
#[derive(Debug)]
pub struct StubEncoder {
    config: Option<EncoderConfig>,
    frame_count: u64,
}

impl StubEncoder {
    /// Create a new unconfigured stub encoder.
    pub fn new() -> Self {
        Self {
            config: None,
            frame_count: 0,
        }
    }
}

impl Default for StubEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoEncoder for StubEncoder {
    fn configure(&mut self, config: EncoderConfig) -> Result<()> {
        if config.width == 0 || config.height == 0 {
            return Err(CodecError::InvalidConfig(
                "width and height must be > 0".into(),
            ));
        }
        self.config = Some(config);
        self.frame_count = 0;
        Ok(())
    }

    fn encode(&mut self, frame: &CapturedFrame) -> Result<EncodedFrame> {
        let config = self.config.as_ref().ok_or(CodecError::NotConfigured)?;

        // Build stub wire format.
        let pixel_len = frame.data.len() as u32;
        let mut data = Vec::with_capacity(28 + frame.data.len());
        data.extend_from_slice(STUB_MAGIC);
        data.extend_from_slice(&frame.width.to_le_bytes());
        data.extend_from_slice(&frame.height.to_le_bytes());
        data.extend_from_slice(&frame.stride.to_le_bytes());
        data.extend_from_slice(&frame.timestamp_us.to_le_bytes());
        data.extend_from_slice(&pixel_len.to_le_bytes());
        data.extend_from_slice(&frame.data);

        let is_keyframe = self.frame_count.is_multiple_of(config.keyframe_interval as u64);
        let pts = frame.timestamp_us;

        self.frame_count += 1;

        Ok(EncodedFrame {
            data,
            is_keyframe,
            pts_us: pts,
            dts_us: pts,
            codec: config.codec,
            width: frame.width,
            height: frame.height,
        })
    }

    fn flush(&mut self) -> Result<Vec<EncodedFrame>> {
        // Stub encoder does not buffer frames.
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rdcs_platform::PixelFormat;

    fn make_frame(w: u32, h: u32) -> CapturedFrame {
        let stride = w * 4;
        CapturedFrame {
            data: vec![42u8; (stride * h) as usize],
            width: w,
            height: h,
            pixel_format: PixelFormat::Bgra,
            stride,
            display_id: 0,
            timestamp_us: 1_000_000,
        }
    }

    #[test]
    fn default_encoder_config() {
        let config = EncoderConfig::default();
        assert_eq!(config.codec, CodecType::H264);
        assert_eq!(config.target_fps, 60);
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    #[test]
    fn stub_encoder_not_configured() {
        let mut enc = StubEncoder::new();
        let frame = make_frame(10, 10);
        let result = enc.encode(&frame);
        assert!(result.is_err());
    }

    #[test]
    fn stub_encoder_encode_roundtrip_data() {
        let mut enc = StubEncoder::new();
        enc.configure(EncoderConfig::default()).unwrap();

        let frame = make_frame(16, 16);
        let encoded = enc.encode(&frame).unwrap();

        assert!(encoded.is_keyframe); // frame 0 is always keyframe
        assert_eq!(encoded.width, 16);
        assert_eq!(encoded.height, 16);
        assert_eq!(encoded.codec, CodecType::H264);
        assert_eq!(encoded.pts_us, 1_000_000);

        // Verify stub header.
        assert_eq!(&encoded.data[0..4], STUB_MAGIC);
    }

    #[test]
    fn stub_encoder_keyframe_interval() {
        let mut enc = StubEncoder::new();
        let mut config = EncoderConfig::default();
        config.keyframe_interval = 3;
        enc.configure(config).unwrap();

        let frame = make_frame(8, 8);

        let f0 = enc.encode(&frame).unwrap();
        assert!(f0.is_keyframe);

        let f1 = enc.encode(&frame).unwrap();
        assert!(!f1.is_keyframe);

        let f2 = enc.encode(&frame).unwrap();
        assert!(!f2.is_keyframe);

        let f3 = enc.encode(&frame).unwrap();
        assert!(f3.is_keyframe);
    }

    #[test]
    fn stub_encoder_flush_returns_empty() {
        let mut enc = StubEncoder::new();
        enc.configure(EncoderConfig::default()).unwrap();
        let flushed = enc.flush().unwrap();
        assert!(flushed.is_empty());
    }

    #[test]
    fn invalid_config_zero_width() {
        let mut enc = StubEncoder::new();
        let mut config = EncoderConfig::default();
        config.width = 0;
        assert!(enc.configure(config).is_err());
    }
}
