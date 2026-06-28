// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Linux VA-API hardware-accelerated encoder stub.
//!
//! This module provides H.264 encoding using VA-API.

use crate::error::CodecError;
use crate::platform::{DecoderStats, EncoderStats, PlatformDecoder, PlatformEncoder};
use crate::types::{Frame, VideoCodec, VideoResolution};

/// Linux VA-API encoder (stub implementation).
pub struct VaapiEncoder {
    _width: u32,
    _height: u32,
    _fps: u32,
    _bitrate: u32,
}

/// Linux VA-API decoder (stub implementation).
pub struct VaapiDecoder {
    _codec: VideoCodec,
}

impl PlatformEncoder for VaapiEncoder {
    fn new(
        codec: VideoCodec,
        resolution: VideoResolution,
        fps: u32,
        bitrate: u32,
    ) -> Result<Self, CodecError> {
        if codec != VideoCodec::H264 {
            return Err(CodecError::UnsupportedCodec(format!("{:?}", codec)));
        }

        let (width, height) = resolution.dimensions();

        // TODO: Implement actual VA-API encoder
        Ok(Self {
            _width: width,
            _height: height,
            _fps: fps,
            _bitrate: bitrate,
        })
    }

    fn encode(&mut self, _frame: &Frame) -> Result<Vec<u8>, CodecError> {
        // TODO: Implement actual encoding
        Err(CodecError::EncodeFailed(
            "VA-API encoder not yet implemented".to_string()
        ))
    }

    fn request_keyframe(&mut self) {
        // TODO: Implement keyframe request
    }

    fn get_stats(&self) -> EncoderStats {
        EncoderStats::default()
    }

    fn shutdown(&mut self) -> Result<(), CodecError> {
        Ok(())
    }
}

impl PlatformDecoder for VaapiDecoder {
    fn new(codec: VideoCodec) -> Result<Self, CodecError> {
        if codec != VideoCodec::H264 {
            return Err(CodecError::UnsupportedCodec(format!("{:?}", codec)));
        }

        // TODO: Implement actual VA-API decoder
        Ok(Self { _codec: codec })
    }

    fn decode(&mut self, _data: &[u8]) -> Result<Frame, CodecError> {
        // TODO: Implement actual decoding
        Err(CodecError::DecodeFailed(
            "VA-API decoder not yet implemented".to_string(),
        ))
    }

    fn get_stats(&self) -> DecoderStats {
        DecoderStats::default()
    }

    fn shutdown(&mut self) -> Result<(), CodecError> {
        Ok(())
    }
}
