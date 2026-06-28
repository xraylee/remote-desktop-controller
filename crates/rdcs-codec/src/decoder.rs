// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Video decoder abstraction and a stub implementation for testing.

use crate::encoder::{CodecType, EncodedFrame};
use crate::{CodecError, Result};

// ---------------------------------------------------------------------------
// Decoded frame
// ---------------------------------------------------------------------------

/// A decoded video frame ready for display.
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    /// Raw pixel data (BGRA or platform-native format).
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Bytes per row (stride).
    pub stride: u32,
    /// Presentation timestamp in microseconds.
    pub pts_us: u64,
}

// ---------------------------------------------------------------------------
// Decoder configuration
// ---------------------------------------------------------------------------

/// Decoder configuration.
#[derive(Debug, Clone)]
pub struct DecoderConfig {
    /// Codec to decode.
    pub codec: CodecType,
    /// Expected output width.
    pub width: u32,
    /// Expected output height.
    pub height: u32,
}

// ---------------------------------------------------------------------------
// VideoDecoder trait
// ---------------------------------------------------------------------------

/// Trait for video decoders (hardware or software).
pub trait VideoDecoder: Send {
    /// Configure the decoder with the given parameters.
    fn configure(&mut self, config: DecoderConfig) -> Result<()>;

    /// Decode an encoded frame into raw pixel data.
    fn decode(&mut self, frame: &EncodedFrame) -> Result<DecodedFrame>;

    /// Reset the decoder state (e.g. after a stream interruption).
    fn reset(&mut self) -> Result<()>;
}

// ---------------------------------------------------------------------------
// StubDecoder — reverses StubEncoder for testing
// ---------------------------------------------------------------------------

/// Magic bytes identifying a stub-encoded frame.
const STUB_MAGIC: &[u8; 4] = b"STUB";

/// Header size of the stub wire format.
const STUB_HEADER_SIZE: usize = 28;

/// A stub decoder that reverses the [`StubEncoder`](crate::encoder::StubEncoder)
/// wire format to recover the original pixel data.
#[derive(Debug)]
pub struct StubDecoder {
    config: Option<DecoderConfig>,
}

impl StubDecoder {
    /// Create a new unconfigured stub decoder.
    pub fn new() -> Self {
        Self { config: None }
    }
}

impl Default for StubDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoDecoder for StubDecoder {
    fn configure(&mut self, config: DecoderConfig) -> Result<()> {
        if config.width == 0 || config.height == 0 {
            return Err(CodecError::InvalidConfig(
                "width and height must be > 0".into(),
            ));
        }
        self.config = Some(config);
        Ok(())
    }

    fn decode(&mut self, frame: &EncodedFrame) -> Result<DecodedFrame> {
        let _config = self.config.as_ref().ok_or(CodecError::NotConfigured)?;

        if frame.data.len() < STUB_HEADER_SIZE {
            return Err(CodecError::DecodeError(
                "encoded data too short for stub header".into(),
            ));
        }

        // Verify magic.
        if &frame.data[0..4] != STUB_MAGIC {
            return Err(CodecError::DecodeError(
                "invalid stub magic bytes".into(),
            ));
        }

        let width = u32::from_le_bytes(frame.data[4..8].try_into().unwrap());
        let height = u32::from_le_bytes(frame.data[8..12].try_into().unwrap());
        let stride = u32::from_le_bytes(frame.data[12..16].try_into().unwrap());
        let timestamp_us = u64::from_le_bytes(frame.data[16..24].try_into().unwrap());
        let pixel_len = u32::from_le_bytes(frame.data[24..28].try_into().unwrap()) as usize;

        if frame.data.len() < STUB_HEADER_SIZE + pixel_len {
            return Err(CodecError::DecodeError(format!(
                "expected {} pixel bytes, have {}",
                pixel_len,
                frame.data.len() - STUB_HEADER_SIZE
            )));
        }

        let pixel_data = frame.data[STUB_HEADER_SIZE..STUB_HEADER_SIZE + pixel_len].to_vec();

        Ok(DecodedFrame {
            data: pixel_data,
            width,
            height,
            stride,
            pts_us: timestamp_us,
        })
    }

    fn reset(&mut self) -> Result<()> {
        // Stub decoder has no state to reset.
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::StubEncoder;
    use crate::encoder::VideoEncoder as _;
    use rdcs_platform::PixelFormat;

    fn make_frame(w: u32, h: u32) -> rdcs_platform::CapturedFrame {
        let stride = w * 4;
        rdcs_platform::CapturedFrame {
            data: (0..(stride * h) as usize)
                .map(|i| (i % 256) as u8)
                .collect(),
            width: w,
            height: h,
            pixel_format: PixelFormat::Bgra,
            stride,
            display_id: 0,
            timestamp_us: 42_000,
        }
    }

    #[test]
    fn decoded_frame_debug() {
        let frame = DecodedFrame {
            data: vec![0u8; 100],
            width: 10,
            height: 10,
            stride: 40,
            pts_us: 0,
        };
        assert_eq!(frame.width, 10);
    }

    #[test]
    fn stub_decoder_not_configured() {
        let mut dec = StubDecoder::new();
        let encoded = EncodedFrame {
            data: vec![0u8; 28],
            is_keyframe: true,
            pts_us: 0,
            dts_us: 0,
            codec: CodecType::H264,
            width: 10,
            height: 10,
        };
        assert!(dec.decode(&encoded).is_err());
    }

    #[test]
    fn stub_decoder_invalid_magic() {
        let mut dec = StubDecoder::new();
        dec.configure(DecoderConfig {
            codec: CodecType::H264,
            width: 10,
            height: 10,
        })
        .unwrap();

        let encoded = EncodedFrame {
            data: vec![0u8; 28],
            is_keyframe: true,
            pts_us: 0,
            dts_us: 0,
            codec: CodecType::H264,
            width: 10,
            height: 10,
        };
        assert!(dec.decode(&encoded).is_err());
    }

    #[test]
    fn stub_decoder_reset() {
        let mut dec = StubDecoder::new();
        dec.configure(DecoderConfig {
            codec: CodecType::H264,
            width: 10,
            height: 10,
        })
        .unwrap();
        assert!(dec.reset().is_ok());
    }

    // -- Acceptance criterion: pipeline_e2e --

    #[test]
    fn encode_decode_roundtrip_preserves_dimensions() {
        let original = make_frame(64, 48);
        let original_width = original.width;
        let original_height = original.height;
        let original_stride = original.stride;
        let original_data = original.data.clone();
        let original_ts = original.timestamp_us;

        // Encode.
        let mut encoder = StubEncoder::new();
        encoder
            .configure(crate::encoder::EncoderConfig::default())
            .unwrap();
        let encoded = encoder.encode(&original).unwrap();

        // Decode.
        let mut decoder = StubDecoder::new();
        decoder
            .configure(DecoderConfig {
                codec: CodecType::H264,
                width: original_width,
                height: original_height,
            })
            .unwrap();
        let decoded = decoder.decode(&encoded).unwrap();

        // Verify dimensions and data are preserved.
        assert_eq!(decoded.width, original_width);
        assert_eq!(decoded.height, original_height);
        assert_eq!(decoded.stride, original_stride);
        assert_eq!(decoded.pts_us, original_ts);
        assert_eq!(decoded.data, original_data);
    }

    #[test]
    fn encode_decode_roundtrip_various_sizes() {
        for (w, h) in [(1, 1), (8, 8), (1920, 1080), (2560, 1440)] {
            let original = make_frame(w, h);
            let mut encoder = StubEncoder::new();
            encoder
                .configure(crate::encoder::EncoderConfig::default())
                .unwrap();
            let encoded = encoder.encode(&original).unwrap();

            let mut decoder = StubDecoder::new();
            decoder
                .configure(DecoderConfig {
                    codec: CodecType::H264,
                    width: w,
                    height: h,
                })
                .unwrap();
            let decoded = decoder.decode(&encoded).unwrap();

            assert_eq!(decoded.width, w, "width mismatch for {w}x{h}");
            assert_eq!(decoded.height, h, "height mismatch for {w}x{h}");
        }
    }
}
