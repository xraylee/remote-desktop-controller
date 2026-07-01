// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Platform-specific video encoder implementations.
//!
//! This module provides hardware-accelerated video encoding using native APIs:
//! - macOS: VideoToolbox
//! - Windows: Media Foundation
//! - Linux: VA-API
//!
//! # Type System Bridge
//!
//! This module bridges between two frame representations:
//! - `rdcs_platform::CapturedFrame` (BGRA from screen capture)
//! - `crate::types::Frame` (YUV420 for internal codec operations)
//! - `Vec<u8>` (H.264 Annex B NAL units for transport)

#[cfg(target_os = "macos")]
pub mod videotoolbox;

#[cfg(target_os = "windows")]
pub mod media_foundation;

#[cfg(target_os = "linux")]
pub mod vaapi;

#[cfg(feature = "software-encoder")]
pub mod openh264_encoder;

#[cfg(feature = "software-encoder")]
pub mod openh264_decoder;

// Re-export decoder for FFI layer
#[cfg(feature = "software-encoder")]
pub use openh264_decoder::OpenH264Decoder;

use crate::error::CodecError;
use crate::types::{Frame, VideoCodec, VideoResolution};
use rdcs_platform::{CapturedFrame, PixelFormat};
use std::sync::Arc;

/// Platform-specific encoder trait.
pub trait PlatformEncoder: Send + Sync {
    /// Create a new encoder with the given parameters.
    fn new(
        codec: VideoCodec,
        resolution: VideoResolution,
        fps: u32,
        bitrate: u32,
    ) -> Result<Self, CodecError>
    where
        Self: Sized;

    /// Encode a video frame.
    fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>, CodecError>;

    /// Request a keyframe on the next encode.
    fn request_keyframe(&mut self);

    /// Get current encoder statistics.
    fn get_stats(&self) -> EncoderStats;

    /// Shutdown the encoder and release resources.
    fn shutdown(&mut self) -> Result<(), CodecError>;
}

/// Platform-specific decoder trait.
pub trait PlatformDecoder: Send + Sync {
    /// Create a new decoder for the given codec.
    fn new(codec: VideoCodec) -> Result<Self, CodecError>
    where
        Self: Sized;

    /// Decode encoded data into a frame.
    fn decode(&mut self, data: &[u8]) -> Result<Frame, CodecError>;

    /// Get current decoder statistics.
    fn get_stats(&self) -> DecoderStats;

    /// Shutdown the decoder and release resources.
    fn shutdown(&mut self) -> Result<(), CodecError>;
}

/// Encoder performance statistics.
#[derive(Debug, Clone, Default)]
pub struct EncoderStats {
    pub frames_encoded: u64,
    pub total_encode_time_ms: u64,
    pub average_encode_time_ms: u64,
    pub keyframes_generated: u32,
    pub bytes_encoded: u64,
}

/// Decoder performance statistics.
#[derive(Debug, Clone, Default)]
pub struct DecoderStats {
    pub frames_decoded: u64,
    pub total_decode_time_ms: u64,
    pub average_decode_time_ms: u64,
    pub keyframes_received: u32,
    pub bytes_decoded: u64,
}

// ---------------------------------------------------------------------------
// High-level Encoder/Decoder API (bridges CapturedFrame ↔ H.264)
// ---------------------------------------------------------------------------

/// High-level platform encoder that accepts `CapturedFrame` and outputs H.264.
pub struct NativeVideoEncoder {
    inner: Box<dyn PlatformEncoder>,
}

impl NativeVideoEncoder {
    /// Create a new platform-native encoder.
    pub fn new(
        codec: VideoCodec,
        resolution: VideoResolution,
        fps: u32,
        bitrate: u32,
    ) -> Result<Self, CodecError> {
        // 优先使用软件编码器（如果启用）
        #[cfg(feature = "software-encoder")]
        {
            let inner = Box::new(openh264_encoder::OpenH264Encoder::new(
                codec, resolution, fps, bitrate,
            )?) as Box<dyn PlatformEncoder>;
            return Ok(Self { inner });
        }

        // 否则使用平台硬件编码器
        #[cfg(all(target_os = "macos", not(feature = "software-encoder")))]
        {
            let inner = Box::new(videotoolbox::VideoToolboxEncoder::new(
                codec, resolution, fps, bitrate,
            )?) as Box<dyn PlatformEncoder>;
            return Ok(Self { inner });
        }

        #[cfg(all(target_os = "windows", not(feature = "software-encoder")))]
        {
            let inner = Box::new(media_foundation::MediaFoundationEncoder::new(
                codec, resolution, fps, bitrate,
            )?) as Box<dyn PlatformEncoder>;
            return Ok(Self { inner });
        }

        #[cfg(all(target_os = "linux", not(feature = "software-encoder")))]
        {
            let inner = Box::new(vaapi::VaapiEncoder::new(
                codec, resolution, fps, bitrate,
            )?) as Box<dyn PlatformEncoder>;
            return Ok(Self { inner });
        }

        #[cfg(not(any(
            feature = "software-encoder",
            target_os = "macos",
            target_os = "windows",
            target_os = "linux"
        )))]
        return Err(CodecError::NotAvailable("no encoder available".into()));
    }

    /// Encode a captured frame from screen capture.
    ///
    /// Converts BGRA → YUV420, encodes, and returns H.264 Annex B NAL units.
    pub fn encode_captured_frame(
        &mut self,
        captured: &CapturedFrame,
    ) -> Result<Vec<u8>, CodecError> {
        let yuv_frame = captured_frame_to_yuv420(captured)?;
        self.inner.encode(&yuv_frame)
    }

    /// Request a keyframe (IDR) on the next encode.
    pub fn request_keyframe(&mut self) {
        self.inner.request_keyframe();
    }

    /// Get encoder statistics.
    pub fn stats(&self) -> EncoderStats {
        self.inner.get_stats()
    }

    /// Shutdown the encoder.
    pub fn shutdown(&mut self) -> Result<(), CodecError> {
        self.inner.shutdown()
    }
}

/// High-level platform decoder that accepts H.264 and outputs `CapturedFrame`.
pub struct NativeVideoDecoder {
    inner: Box<dyn PlatformDecoder>,
}

impl NativeVideoDecoder {
    /// Create a new platform-native decoder.
    pub fn new(codec: VideoCodec) -> Result<Self, CodecError> {
        // 优先使用软件解码器（如果启用）
        #[cfg(feature = "software-encoder")]
        {
            let inner = Box::new(openh264_decoder::OpenH264Decoder::new(codec)?)
                as Box<dyn PlatformDecoder>;
            return Ok(Self { inner });
        }

        // 否则使用平台硬件解码器
        #[cfg(all(target_os = "macos", not(feature = "software-encoder")))]
        {
            let inner =
                Box::new(videotoolbox::VideoToolboxDecoder::new(codec)?) as Box<dyn PlatformDecoder>;
            return Ok(Self { inner });
        }

        #[cfg(all(target_os = "windows", not(feature = "software-encoder")))]
        {
            let inner = Box::new(media_foundation::MediaFoundationDecoder::new(codec)?)
                as Box<dyn PlatformDecoder>;
            return Ok(Self { inner });
        }

        #[cfg(all(target_os = "linux", not(feature = "software-encoder")))]
        {
            let inner =
                Box::new(vaapi::VaapiDecoder::new(codec)?) as Box<dyn PlatformDecoder>;
            return Ok(Self { inner });
        }

        #[allow(unreachable_code)]
        {
            Err(CodecError::NotAvailable("no decoder available".into()))
        }
    }

    /// Decode H.264 Annex B NAL units into a captured frame.
    ///
    /// Decodes to YUV420, then converts to BGRA for rendering.
    pub fn decode_to_captured_frame(&mut self, data: &[u8]) -> Result<CapturedFrame, CodecError> {
        // Decode using platform decoder
        let yuv_frame = self.inner.decode(data)?;

        // Convert Frame (YUV420) to CapturedFrame (BGRA)
        yuv420_to_captured_frame(&yuv_frame)
    }

    /// Get decoder statistics.
    pub fn stats(&self) -> DecoderStats {
        self.inner.get_stats()
    }

    /// Shutdown the decoder.
    pub fn shutdown(&mut self) -> Result<(), CodecError> {
        self.inner.shutdown()
    }
}

// ---------------------------------------------------------------------------
// Pixel Format Conversion Functions
// ---------------------------------------------------------------------------

/// Convert `CapturedFrame` (BGRA) to `Frame` (YUV420 planar).
///
/// This is a software conversion using the standard BT.601 matrix.
fn captured_frame_to_yuv420(captured: &CapturedFrame) -> Result<Frame, CodecError> {
    if captured.pixel_format != PixelFormat::Bgra {
        return Err(CodecError::InvalidConfig(format!(
            "Expected BGRA pixel format, got {:?}",
            captured.pixel_format
        )));
    }

    let width = captured.width;
    let height = captured.height;
    let stride = captured.stride as usize;

    // Allocate YUV420 buffer: Y plane + U plane + V plane
    let y_size = (width * height) as usize;
    let uv_size = (width * height / 4) as usize;
    let mut yuv_data = vec![0u8; y_size + uv_size + uv_size];

    let (y_plane, uv_planes) = yuv_data.split_at_mut(y_size);
    let (u_plane, v_plane) = uv_planes.split_at_mut(uv_size);

    // Convert BGRA to YUV420 using BT.601 matrix
    for y in 0..height as usize {
        for x in 0..width as usize {
            let src_offset = y * stride + x * 4;
            if src_offset + 3 >= captured.data.len() {
                continue;
            }

            let b = captured.data[src_offset] as f32;
            let g = captured.data[src_offset + 1] as f32;
            let r = captured.data[src_offset + 2] as f32;
            // Alpha channel ignored

            // BT.601 RGB to YUV conversion
            let y_val = (0.299 * r + 0.587 * g + 0.114 * b) as u8;
            y_plane[y * width as usize + x] = y_val;

            // Subsample U and V (4:2:0)
            if y % 2 == 0 && x % 2 == 0 {
                let u_val = ((-0.169 * r - 0.331 * g + 0.500 * b) + 128.0) as u8;
                let v_val = ((0.500 * r - 0.419 * g - 0.081 * b) + 128.0) as u8;

                let uv_x = x / 2;
                let uv_y = y / 2;
                let uv_index = uv_y * (width as usize / 2) + uv_x;

                if uv_index < uv_size {
                    u_plane[uv_index] = u_val;
                    v_plane[uv_index] = v_val;
                }
            }
        }
    }

    Ok(Frame {
        width,
        height,
        data: yuv_data,
        timestamp_us: captured.timestamp_us,
        is_keyframe: false,
    })
}

/// Convert `Frame` (YUV420 planar) to `CapturedFrame` (BGRA).
///
/// This is the reverse of `captured_frame_to_yuv420`.
fn yuv420_to_captured_frame(frame: &Frame) -> Result<CapturedFrame, CodecError> {
    let width = frame.width;
    let height = frame.height;
    let y_size = (width * height) as usize;
    let uv_size = (width * height / 4) as usize;

    if frame.data.len() < y_size + uv_size + uv_size {
        return Err(CodecError::DecodeError(
            "Invalid YUV420 frame data size".into(),
        ));
    }

    let y_plane = &frame.data[0..y_size];
    let u_plane = &frame.data[y_size..y_size + uv_size];
    let v_plane = &frame.data[y_size + uv_size..y_size + uv_size + uv_size];

    // Allocate BGRA buffer
    let stride = width * 4;
    let mut bgra_data = vec![0u8; (stride * height) as usize];

    // Convert YUV420 to BGRA using BT.601 matrix
    for y in 0..height as usize {
        for x in 0..width as usize {
            let y_val = y_plane[y * width as usize + x] as f32;

            let uv_x = x / 2;
            let uv_y = y / 2;
            let uv_index = uv_y * (width as usize / 2) + uv_x;

            let u_val = u_plane[uv_index] as f32 - 128.0;
            let v_val = v_plane[uv_index] as f32 - 128.0;

            // BT.601 YUV to RGB conversion
            let r = (y_val + 1.402 * v_val).clamp(0.0, 255.0) as u8;
            let g = (y_val - 0.344 * u_val - 0.714 * v_val).clamp(0.0, 255.0) as u8;
            let b = (y_val + 1.772 * u_val).clamp(0.0, 255.0) as u8;

            let dst_offset = (y * stride as usize + x * 4) as usize;
            bgra_data[dst_offset] = b;
            bgra_data[dst_offset + 1] = g;
            bgra_data[dst_offset + 2] = r;
            bgra_data[dst_offset + 3] = 255; // Alpha
        }
    }

    Ok(CapturedFrame {
        data: Arc::from(bgra_data.into_boxed_slice()),
        width,
        height,
        pixel_format: PixelFormat::Bgra,
        stride,
        display_id: 0,
        timestamp_us: frame.timestamp_us,
    })
}
