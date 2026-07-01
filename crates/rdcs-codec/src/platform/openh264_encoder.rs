// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! OpenH264 软件编码器实现

use crate::error::CodecError;
use crate::types::{Frame, VideoCodec, VideoResolution};
use openh264::encoder::Encoder;
use openh264::formats::YUVBuffer;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

use crate::platform::{EncoderStats, PlatformEncoder};

/// OpenH264 编码器统计信息
#[derive(Default)]
struct OpenH264EncoderStats {
    frames_encoded: AtomicU64,
    total_encode_time_ms: AtomicU64,
    keyframes_generated: AtomicU64,
    bytes_encoded: AtomicU64,
}

/// OpenH264 软件 H.264 编码器
pub struct OpenH264Encoder {
    encoder: Encoder,
    width: u32,
    height: u32,
    #[allow(dead_code)]
    fps: u32,
    #[allow(dead_code)]
    bitrate: u32,
    stats: Arc<OpenH264EncoderStats>,
    keyframe_requested: bool,
}

impl PlatformEncoder for OpenH264Encoder {
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

        info!(
            "Creating OpenH264 software encoder: {}x{} @ {} fps, {} kbps",
            width, height, fps, bitrate / 1000
        );

        // 创建编码器（使用默认配置）
        // OpenH264 会自动在第一帧生成 SPS/PPS
        let encoder = Encoder::new().map_err(|e| {
            CodecError::EncoderInitFailed(format!("OpenH264 encoder creation failed: {:?}", e))
        })?;

        debug!("OpenH264 encoder created successfully");

        Ok(Self {
            encoder,
            width,
            height,
            fps,
            bitrate,
            stats: Arc::new(OpenH264EncoderStats::default()),
            keyframe_requested: false,
        })
    }

    fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>, CodecError> {
        let start = Instant::now();

        // 验证帧尺寸
        if frame.width != self.width || frame.height != self.height {
            return Err(CodecError::InvalidFrameSize {
                expected: (self.width, self.height),
                actual: (frame.width, frame.height),
            });
        }

        // 验证数据大小
        let expected_size = (self.width * self.height * 3 / 2) as usize; // YUV420
        if frame.data.len() < expected_size {
            return Err(CodecError::EncodeFailed(format!(
                "Frame data too small: expected {} bytes, got {}",
                expected_size,
                frame.data.len()
            )));
        }

        // 创建 YUVBuffer - from_vec 不返回 Result
        let yuv = YUVBuffer::from_vec(
            frame.data.clone(),
            self.width as usize,
            self.height as usize,
        );

        // 编码
        let bitstream = self.encoder.encode(&yuv).map_err(|e| {
            CodecError::EncodeFailed(format!("OpenH264 encode failed: {:?}", e))
        })?;

        let encode_time = start.elapsed().as_millis() as u64;

        // 提取编码数据
        let encoded_data = bitstream.to_vec();

        // 调试：记录编码数据的 NAL 单元类型（前几帧）
        if self.stats.frames_encoded.load(Ordering::Relaxed) < 3 {
            debug!("Frame {} encoded data first 32 bytes: {:02X?}",
                self.stats.frames_encoded.load(Ordering::Relaxed),
                &encoded_data[..encoded_data.len().min(32)]
            );
        }

        // 检查是否是关键帧
        let is_keyframe = Self::is_keyframe(&encoded_data);

        // 更新统计信息
        self.stats.frames_encoded.fetch_add(1, Ordering::Relaxed);
        self.stats
            .total_encode_time_ms
            .fetch_add(encode_time, Ordering::Relaxed);
        self.stats
            .bytes_encoded
            .fetch_add(encoded_data.len() as u64, Ordering::Relaxed);

        if is_keyframe {
            self.stats
                .keyframes_generated
                .fetch_add(1, Ordering::Relaxed);
        }

        self.keyframe_requested = false;

        debug!(
            "Encoded frame: {} bytes in {} ms, keyframe: {}",
            encoded_data.len(),
            encode_time,
            is_keyframe
        );

        Ok(encoded_data)
    }

    fn request_keyframe(&mut self) {
        debug!("Keyframe requested");
        self.keyframe_requested = true;
    }

    fn get_stats(&self) -> EncoderStats {
        let frames = self.stats.frames_encoded.load(Ordering::Relaxed);
        let total_time = self.stats.total_encode_time_ms.load(Ordering::Relaxed);
        let avg_time = if frames > 0 {
            total_time / frames
        } else {
            0
        };

        EncoderStats {
            frames_encoded: frames,
            total_encode_time_ms: total_time,
            average_encode_time_ms: avg_time,
            keyframes_generated: self.stats.keyframes_generated.load(Ordering::Relaxed) as u32,
            bytes_encoded: self.stats.bytes_encoded.load(Ordering::Relaxed),
        }
    }

    fn shutdown(&mut self) -> Result<(), CodecError> {
        info!("Shutting down OpenH264 encoder");
        Ok(())
    }
}

impl OpenH264Encoder {
    /// 检查编码数据是否包含关键帧（IDR）
    fn is_keyframe(data: &[u8]) -> bool {
        // H.264 Annex B 格式：查找 IDR NAL unit (type 5)
        let mut i = 0;
        while i + 4 < data.len() {
            if data[i] == 0x00
                && data[i + 1] == 0x00
                && data[i + 2] == 0x00
                && data[i + 3] == 0x01
            {
                if i + 5 < data.len() {
                    let nal_type = data[i + 4] & 0x1F;
                    if nal_type == 5 {
                        return true;
                    }
                }
                i += 4;
            } else {
                i += 1;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_keyframe() {
        let idr_data = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x00];
        assert!(OpenH264Encoder::is_keyframe(&idr_data));

        let non_idr_data = vec![0x00, 0x00, 0x00, 0x01, 0x41, 0x00];
        assert!(!OpenH264Encoder::is_keyframe(&non_idr_data));
    }
}
