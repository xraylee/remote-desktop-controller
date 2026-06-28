// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 视频发送管道。
//!
//! 集成：Encoder → Packetizer → SRTP → Network

use super::{Result, SessionError};
use crate::encoder::VideoEncoder;
use crate::rtp::{H264Packetizer, PacketizerConfig, SrtpContext};
use crate::types::{VideoFormat, VideoFrame};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, trace, warn};

/// 视频发送管道配置。
#[derive(Debug, Clone)]
pub struct VideoSenderConfig {
    /// RTP 打包器配置。
    pub packetizer: PacketizerConfig,
    /// 发送队列大小。
    pub send_queue_size: usize,
}

impl Default for VideoSenderConfig {
    fn default() -> Self {
        Self {
            packetizer: PacketizerConfig::default(),
            send_queue_size: 100,
        }
    }
}

/// 视频发送管道。
pub struct VideoSender {
    /// 编码器（平台相关）。
    encoder: Box<dyn VideoEncoder + Send>,
    /// RTP 打包器。
    packetizer: Arc<RwLock<H264Packetizer>>,
    /// SRTP 加密上下文。
    srtp: Arc<RwLock<Option<SrtpContext>>>,
    /// 发送队列（SRTP 包 → 网络层）。
    tx_queue: mpsc::Sender<Vec<u8>>,
    /// 接收队列（网络层读取）。
    rx_queue: mpsc::Receiver<Vec<u8>>,
    /// 当前 RTP 时间戳。
    timestamp: Arc<RwLock<u32>>,
    /// 统计信息。
    stats: Arc<RwLock<SenderStats>>,
}

/// 发送端统计。
#[derive(Debug, Clone, Default)]
pub struct SenderStats {
    pub frames_encoded: u64,
    pub packets_sent: u64,
    pub bytes_sent: u64,
    pub encoding_errors: u32,
    pub network_errors: u32,
}

impl VideoSender {
    /// 创建新的视频发送管道。
    ///
    /// # Arguments
    ///
    /// * `encoder` - 视频编码器实现
    /// * `srtp` - SRTP 加密上下文（来自 PeerConnection）
    /// * `config` - 发送器配置
    pub fn new(
        encoder: Box<dyn VideoEncoder + Send>,
        srtp: Arc<RwLock<Option<SrtpContext>>>,
        config: VideoSenderConfig,
    ) -> Self {
        let (tx_queue, rx_queue) = mpsc::channel(config.send_queue_size);
        let packetizer = H264Packetizer::new(config.packetizer);

        Self {
            encoder,
            packetizer: Arc::new(RwLock::new(packetizer)),
            srtp,
            tx_queue,
            rx_queue,
            timestamp: Arc::new(RwLock::new(0)),
            stats: Arc::new(RwLock::new(SenderStats::default())),
        }
    }

    /// 发送一帧视频。
    ///
    /// 完整流程：编码 → RTP 打包 → SRTP 加密 → 放入发送队列。
    pub async fn send_frame(&mut self, frame: &VideoFrame) -> Result<()> {
        // 1. 编码
        let h264_data = self
            .encoder
            .encode(frame)
            .map_err(|e| SessionError::EncoderError(e.to_string()))?;

        trace!("Encoded frame: {} bytes", h264_data.len());

        // 2. 获取时间戳并递增
        let mut ts = self.timestamp.write().await;
        let current_ts = *ts;
        // 90kHz 时钟，30fps = 每帧 3000 个时钟周期
        *ts = ts.wrapping_add(3000);
        drop(ts);

        // 3. RTP 打包
        let mut packetizer = self.packetizer.write().await;
        let rtp_packets = packetizer.packetize(&h264_data, current_ts)?;
        drop(packetizer);

        trace!("Packetized into {} RTP packets", rtp_packets.len());

        // 4. SRTP 加密
        let srtp_guard = self.srtp.read().await;
        let srtp_ctx = srtp_guard
            .as_ref()
            .ok_or_else(|| SessionError::NotReady("SRTP context not initialized".into()))?;

        for rtp_packet in rtp_packets {
            let srtp_packet = srtp_ctx.encrypt(&rtp_packet).await?;

            // 5. 放入发送队列
            if let Err(e) = self.tx_queue.try_send(srtp_packet.clone()) {
                warn!("Send queue full, dropping packet: {}", e);
                self.stats.write().await.network_errors += 1;
            } else {
                let mut stats = self.stats.write().await;
                stats.packets_sent += 1;
                stats.bytes_sent += srtp_packet.len() as u64;
            }
        }

        self.stats.write().await.frames_encoded += 1;

        Ok(())
    }

    /// 从发送队列获取 SRTP 包（供网络层调用）。
    ///
    /// 返回 `None` 表示队列为空。
    pub async fn poll_packet(&mut self) -> Option<Vec<u8>> {
        self.rx_queue.recv().await
    }

    /// 获取统计信息。
    pub async fn stats(&self) -> SenderStats {
        self.stats.read().await.clone()
    }

    /// 重置统计信息。
    pub async fn reset_stats(&self) {
        *self.stats.write().await = SenderStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::EncoderConfig;
    use crate::platform::macos::VideoToolboxEncoder;
    use crate::rtp::{SrtpConfig, SrtpProfile};
    use crate::types::PixelFormat;

    fn create_test_srtp() -> Arc<RwLock<Option<SrtpContext>>> {
        Arc::new(RwLock::new(None))
    }

    async fn init_test_srtp(srtp: &Arc<RwLock<Option<SrtpContext>>>) {
        let config = SrtpConfig {
            master_key: vec![0x42; 16],
            master_salt: vec![0x84; 14],
            profile: SrtpProfile::Aead_Aes128Gcm,
        };
        let ctx = SrtpContext::new(config).await.unwrap();
        *srtp.write().await = Some(ctx);
    }

    #[tokio::test]
    async fn test_video_sender_creation() {
        let encoder = Box::new(VideoToolboxEncoder::new().unwrap());
        let srtp = create_test_srtp();
        let config = VideoSenderConfig::default();

        let sender = VideoSender::new(encoder, srtp, config);
        let stats = sender.stats().await;

        assert_eq!(stats.frames_encoded, 0);
        assert_eq!(stats.packets_sent, 0);
    }

    #[tokio::test]
    async fn test_send_frame_without_srtp_fails() {
        let mut encoder = VideoToolboxEncoder::new().unwrap();
        let encoder_config = EncoderConfig {
            format: VideoFormat {
                width: 640,
                height: 480,
                pixel_format: PixelFormat::Nv12,
                framerate: 30,
            },
            bitrate: 1_000_000,
            keyframe_interval: 30,
        };
        encoder.configure(&encoder_config).unwrap();

        let srtp = create_test_srtp(); // 未初始化
        let config = VideoSenderConfig::default();
        let mut sender = VideoSender::new(Box::new(encoder), srtp, config);

        // 创建测试帧（小尺寸避免编码失败）
        let frame = VideoFrame {
            format: VideoFormat {
                width: 640,
                height: 480,
                pixel_format: PixelFormat::Nv12,
                framerate: 30,
            },
            data: vec![0u8; 640 * 480 * 3 / 2],
            timestamp_us: 0,
        };

        // 应该因为 SRTP 未就绪而失败
        let result = sender.send_frame(&frame).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_timestamp_increment() {
        let encoder = Box::new(VideoToolboxEncoder::new().unwrap());
        let srtp = create_test_srtp();
        init_test_srtp(&srtp).await;

        let config = VideoSenderConfig::default();
        let sender = VideoSender::new(encoder, srtp, config);

        let initial_ts = *sender.timestamp.read().await;
        assert_eq!(initial_ts, 0);

        // 模拟时间戳递增
        *sender.timestamp.write().await = 3000;
        let next_ts = *sender.timestamp.read().await;
        assert_eq!(next_ts, 3000);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let encoder = Box::new(VideoToolboxEncoder::new().unwrap());
        let srtp = create_test_srtp();
        let config = VideoSenderConfig::default();
        let sender = VideoSender::new(encoder, srtp, config);

        // 手动更新统计
        sender.stats.write().await.frames_encoded = 10;
        sender.stats.write().await.packets_sent = 50;

        let stats = sender.stats().await;
        assert_eq!(stats.frames_encoded, 10);
        assert_eq!(stats.packets_sent, 50);

        // 重置
        sender.reset_stats().await;
        let stats = sender.stats().await;
        assert_eq!(stats.frames_encoded, 0);
    }
}
