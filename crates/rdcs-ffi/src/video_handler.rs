//! Video frame delivery module for FFI layer.
//!
//! Handles receiving encoded H.264 frames, decoding to BGRA,
//! and dispatching to Flutter via EVENT_FRAME_READY callbacks.

use crate::{dispatch_event, EngineHandle, EVENT_FRAME_READY};
use rdcs_codec::platform::{OpenH264Decoder, PlatformDecoder};
use rdcs_codec::types::{Frame, VideoCodec};
use std::sync::{Arc, Mutex};

/// Video frame receiver and decoder for the FFI layer.
///
/// Receives H.264 encoded frames, decodes them to YUV/BGRA,
/// and dispatches them to Flutter via events.
pub struct VideoFrameHandler {
    decoder: Arc<Mutex<Option<OpenH264Decoder>>>,
}

impl VideoFrameHandler {
    /// Create a new video frame handler.
    pub fn new() -> Self {
        Self {
            decoder: Arc::new(Mutex::new(None)),
        }
    }

    /// Handle an incoming encoded frame.
    ///
    /// Decodes the H.264 data and dispatches a frame event to Flutter.
    pub fn handle_encoded_frame(
        &self,
        engine: &EngineHandle,
        h264_data: &[u8],
        session_id: u64,
    ) -> Result<(), String> {
        // Initialize decoder if needed
        let mut decoder_guard = self.decoder.lock().unwrap();
        if decoder_guard.is_none() {
            let decoder = OpenH264Decoder::new(VideoCodec::H264)
                .map_err(|e| format!("Failed to create decoder: {}", e))?;
            *decoder_guard = Some(decoder);
        }

        // Decode frame
        let decoder = decoder_guard.as_mut().unwrap();
        let frame = decoder
            .decode(h264_data)
            .map_err(|e| format!("Decode failed: {}", e))?;

        // Convert YUV to BGRA
        let bgra_data = yuv420_to_bgra(&frame);

        // Encode to base64 for FFI transfer
        let base64_data = base64::encode(&bgra_data);

        // Dispatch event to Flutter
        let payload = format!(
            r#"{{"session_id":{},"width":{},"height":{},"format":"bgra","data":"{}","timestamp":{}}}"#,
            session_id, frame.width, frame.height, base64_data, frame.timestamp_us
        );

        dispatch_event(engine, EVENT_FRAME_READY, &payload);

        Ok(())
    }

    /// Reset the decoder state (e.g., after stream interruption).
    pub fn reset(&self) -> Result<(), String> {
        let mut decoder_guard = self.decoder.lock().unwrap();
        if let Some(decoder) = decoder_guard.as_mut() {
            decoder
                .shutdown()
                .map_err(|e| format!("Failed to reset decoder: {}", e))?;
        }
        *decoder_guard = None;
        Ok(())
    }
}

impl Default for VideoFrameHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert YUV420 frame to BGRA8888.
///
/// This is a simple software conversion. For production, consider using
/// hardware-accelerated conversion or platform-specific APIs.
fn yuv420_to_bgra(frame: &Frame) -> Vec<u8> {
    let width = frame.width as usize;
    let height = frame.height as usize;
    let mut bgra = vec![0u8; width * height * 4];

    let y_plane = &frame.data[..width * height];
    let uv_offset = width * height;
    let u_plane = &frame.data[uv_offset..uv_offset + (width * height / 4)];
    let v_plane = &frame.data[uv_offset + (width * height / 4)..];

    for y_pos in 0..height {
        for x_pos in 0..width {
            let y_index = y_pos * width + x_pos;
            let uv_index = (y_pos / 2) * (width / 2) + (x_pos / 2);

            let y = y_plane[y_index] as i32;
            let u = u_plane[uv_index] as i32 - 128;
            let v = v_plane[uv_index] as i32 - 128;

            // YUV to RGB conversion (ITU-R BT.601)
            let r = (y + ((1436 * v) >> 10)).clamp(0, 255) as u8;
            let g = (y - ((352 * u + 731 * v) >> 10)).clamp(0, 255) as u8;
            let b = (y + ((1814 * u) >> 10)).clamp(0, 255) as u8;

            let bgra_index = y_index * 4;
            bgra[bgra_index] = b;
            bgra[bgra_index + 1] = g;
            bgra[bgra_index + 2] = r;
            bgra[bgra_index + 3] = 255; // Alpha
        }
    }

    bgra
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_frame_handler_creation() {
        let handler = VideoFrameHandler::new();
        assert!(handler.decoder.lock().unwrap().is_none());
    }

    #[test]
    fn yuv_to_bgra_conversion() {
        // Create a simple 2x2 YUV420 frame (black)
        let frame = Frame {
            data: vec![0u8; 6], // 4 Y + 1 U + 1 V
            width: 2,
            height: 2,
            timestamp_us: 0,
            is_keyframe: true,
        };

        let bgra = yuv420_to_bgra(&frame);
        assert_eq!(bgra.len(), 2 * 2 * 4);

        // Black in YUV (16, 128, 128) should be close to (0, 0, 0) in RGB
        // Our test uses 0 Y, which is darker than standard black
        for i in 0..4 {
            let b = bgra[i * 4];
            let g = bgra[i * 4 + 1];
            let r = bgra[i * 4 + 2];
            let a = bgra[i * 4 + 3];
            assert_eq!(a, 255);
            // Y=0 should produce dark colors
            assert!(r < 50 && g < 50 && b < 50);
        }
    }

    #[test]
    fn reset_uninitialized_handler() {
        let handler = VideoFrameHandler::new();
        assert!(handler.reset().is_ok());
    }
}
