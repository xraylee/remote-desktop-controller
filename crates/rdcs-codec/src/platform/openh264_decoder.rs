// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! OpenH264 软件解码器实现

use crate::error::CodecError;
use crate::types::{Frame, VideoCodec};
use openh264::decoder::Decoder;
use openh264::formats::YUVSource;  // 导入 trait 以使用其方法
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info};

use crate::platform::{DecoderStats, PlatformDecoder};

/// OpenH264 解码器统计信息
#[derive(Default)]
struct OpenH264DecoderStats {
    frames_decoded: AtomicU64,
    total_decode_time_ms: AtomicU64,
    keyframes_received: AtomicU64,
    bytes_decoded: AtomicU64,
}

/// OpenH264 软件 H.264 解码器
pub struct OpenH264Decoder {
    decoder: Decoder,
    stats: Arc<OpenH264DecoderStats>,
}

impl PlatformDecoder for OpenH264Decoder {
    fn new(codec: VideoCodec) -> Result<Self, CodecError> {
        if codec != VideoCodec::H264 {
            return Err(CodecError::UnsupportedCodec(format!("{:?}", codec)));
        }

        info!("Creating OpenH264 software decoder");

        let decoder = Decoder::new().map_err(|e| {
            CodecError::DecoderInitFailed(format!("OpenH264 decoder creation failed: {:?}", e))
        })?;

        debug!("OpenH264 decoder created successfully");

        Ok(Self {
            decoder,
            stats: Arc::new(OpenH264DecoderStats::default()),
        })
    }

    fn decode(&mut self, data: &[u8]) -> Result<Frame, CodecError> {
        let start = Instant::now();

        // 解码
        let yuv_option = self.decoder.decode(data).map_err(|e| {
            CodecError::DecodeFailed(format!("OpenH264 decode failed: {:?}", e))
        })?;

        // 解包 Option<DecodedYUV>
        let yuv = yuv_option.ok_or_else(|| {
            CodecError::DecodeFailed("OpenH264 decoder returned None".into())
        })?;

        let decode_time = start.elapsed().as_millis() as u64;

        // 提取 YUV 数据
        let (width, height) = yuv.dimensions();
        let width = width as u32;
        let height = height as u32;

        // 获取 YUV 平面数据
        let y_plane = yuv.y();
        let u_plane = yuv.u();
        let v_plane = yuv.v();

        // YUV420 planar 数据大小
        let y_size = (width * height) as usize;
        let u_size = (width * height / 4) as usize;
        let v_size = u_size;

        // 构建紧凑的 YUV420 数据（无 padding）
        let mut frame_data = Vec::with_capacity(y_size + u_size + v_size);

        // 直接拷贝整个 Y 平面（假设连续存储）
        let y_copy_len = y_size.min(y_plane.len());
        frame_data.extend_from_slice(&y_plane[..y_copy_len]);

        // 填充到期望大小
        if frame_data.len() < y_size {
            frame_data.resize(y_size, 0);
        }

        // 拷贝 U 平面
        let u_copy_len = u_size.min(u_plane.len());
        frame_data.extend_from_slice(&u_plane[..u_copy_len]);
        if frame_data.len() < y_size + u_size {
            frame_data.resize(y_size + u_size, 128);
        }

        // 拷贝 V 平面
        let v_copy_len = v_size.min(v_plane.len());
        frame_data.extend_from_slice(&v_plane[..v_copy_len]);
        if frame_data.len() < y_size + u_size + v_size {
            frame_data.resize(y_size + u_size + v_size, 128);
        }

        // 检查是否是关键帧
        let is_keyframe = Self::is_keyframe(data);

        // 更新统计信息
        self.stats.frames_decoded.fetch_add(1, Ordering::Relaxed);
        self.stats
            .total_decode_time_ms
            .fetch_add(decode_time, Ordering::Relaxed);
        self.stats
            .bytes_decoded
            .fetch_add(data.len() as u64, Ordering::Relaxed);

        if is_keyframe {
            self.stats
                .keyframes_received
                .fetch_add(1, Ordering::Relaxed);
        }

        debug!(
            "Decoded frame: {}x{} in {} ms, keyframe: {}",
            width, height, decode_time, is_keyframe
        );

        Ok(Frame {
            width,
            height,
            data: frame_data,
            timestamp_us: 0,
            is_keyframe,
        })
    }

    fn get_stats(&self) -> DecoderStats {
        let frames = self.stats.frames_decoded.load(Ordering::Relaxed);
        let total_time = self.stats.total_decode_time_ms.load(Ordering::Relaxed);
        let avg_time = if frames > 0 {
            total_time / frames
        } else {
            0
        };

        DecoderStats {
            frames_decoded: frames,
            total_decode_time_ms: total_time,
            average_decode_time_ms: avg_time,
            keyframes_received: self.stats.keyframes_received.load(Ordering::Relaxed) as u32,
            bytes_decoded: self.stats.bytes_decoded.load(Ordering::Relaxed),
        }
    }

    fn shutdown(&mut self) -> Result<(), CodecError> {
        info!("Shutting down OpenH264 decoder");
        Ok(())
    }
}

impl OpenH264Decoder {
    /// 检查编码数据是否包含关键帧（IDR）
    fn is_keyframe(data: &[u8]) -> bool {
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
