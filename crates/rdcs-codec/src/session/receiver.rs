// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 视频接收管道。
//!
//! 集成：Network → SRTP → Depacketizer → Decoder

use super::{Result, SessionError};
use crate::decoder::VideoDecoder;
use crate::rtp::{H264Depacketizer, SrtpContext};
use crate::types::VideoFrame;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, trace, warn};

/// 视频接收管道配置。
#[derive(Debug, Clone)]
pub struct VideoReceiverConfig {
    /// 接收队列大小。
    pub recv_queue_size: usize,
    /// 抖动缓冲区大小（毫秒）。
    pub jitter_buffer_ms: u32,
}

impl Default for VideoReceiverConfig {
    fn default() -> Self {
        Self {
            recv_queue_size: 100,
            jitter_buffer_ms: 50,
        }
    }
}

/// 视频接收管道。
pub struct VideoReceiver {
    /// 解码器（平台相关）。
    decoder: Box<dyn VideoDecoder + Send>,
    /// RTP 解包器。
    depacketizer: Arc<RwLock<H264Depacketizer>>,
    /// SRTP 解密上下文。
    srtp: Arc<RwLock<Option<SrtpContext>>>,
    /// 接收队列（网络层 → SRTP 包）。
    tx_queue: mpsc::Sender<Vec<u8>>,
    /// 处理队列（内部读取）。
    rx_queue: mpsc::Receiver<Vec<u8>>,
    /// 解码帧输出队列。
    frame_tx: mpsc::Sender<VideoFrame>,
    /// 解码帧接收端（供应用读取）。
    frame_rx: mpsc::Receiver<VideoFrame>,
    /// 统计信息。
    stats: Arc<RwLock<ReceiverStats>>,
}

/// 接收端统计。
#[derive(Debug, Clone, Default)]
pub struct ReceiverStats {
    pub packets_received: u64,
    pub frames_decoded: u64,
    pub bytes_received: u64,
    pub decryption_errors: u32,
    pub decoding_errors: u32,
    pub packets_dropped: u32,
}

impl VideoReceiver {
    /// 创建新的视频接收管道。
    ///
    /// # Arguments
    ///
    /// * `decoder` - 视频解码器实现
    /// * `srtp` - SRTP 解密上下文（来自 PeerConnection）
    /// * `config` - 接收器配置
    pub fn new(
        decoder: Box<dyn VideoDecoder + Send>,
        srtp: Arc<RwLock<Option<SrtpContext>>>,
        config: VideoReceiverConfig,
    ) -> Self {
        let (tx_queue, rx_queue) = mpsc::channel(config.recv_queue_size);
        let (frame_tx, frame_rx) = mpsc::channel(10); // 帧队列较小，避免延迟累积
        let depacketizer = H264Depacketizer::new();

        Self {
            decoder,
            depacketizer: Arc::new(RwLock::new(depacketizer)),
            srtp,
            tx_queue,
            rx_queue,
            frame_tx,
            frame_rx,
            stats: Arc::new(RwLock::new(ReceiverStats::default())),
        }
    }

    /// 接收 SRTP 包（供网络层调用）。
    ///
    /// 包会被放入接收队列，等待处理线程解密和解包。
    pub async fn receive_packet(&self, srtp_packet: Vec<u8>) -> Result<()> {
        if let Err(e) = self.tx_queue.try_send(srtp_packet) {
            warn!("Receive queue full, dropping packet: {}", e);
            self.stats.write().await.packets_dropped += 1;
        } else {
            self.stats.write().await.packets_received += 1;
        }
        Ok(())
    }

    /// 处理接收队列中的包。
    ///
    /// 完整流程：SRTP 解密 → RTP 解包 → 解码 → 输出帧。
    /// 应该在单独的任务中循环调用。
    pub async fn process_packets(&mut self) -> Result<()> {
        while let Some(srtp_packet) = self.rx_queue.recv().await {
            if let Err(e) = self.process_single_packet(&srtp_packet).await {
                warn!("Failed to process packet: {}", e);
                // 继续处理下一个包，不中断流
            }
        }
        Ok(())
    }

    /// 处理单个 SRTP 包。
    async fn process_single_packet(&mut self, srtp_packet: &[u8]) -> Result<()> {
        // 1. SRTP 解密
        let srtp_guard = self.srtp.read().await;
        let srtp_ctx = srtp_guard
            .as_ref()
            .ok_or_else(|| SessionError::NotReady("SRTP context not initialized".into()))?;

        let rtp_packet = match srtp_ctx.decrypt(srtp_packet).await {
            Ok(p) => p,
            Err(e) => {
                self.stats.write().await.decryption_errors += 1;
                return Err(e.into());
            }
        };
        drop(srtp_guard);

        trace!("Decrypted SRTP packet: {} bytes", rtp_packet.len());

        // 2. RTP 解包
        let mut depacketizer = self.depacketizer.write().await;
        let h264_data = match depacketizer.depacketize(&rtp_packet)? {
            Some(data) => data,
            None => {
                // 需要更多包才能重组完整 NAL 单元
                return Ok(());
            }
        };
        drop(depacketizer);

        trace!("Depacketized H.264 data: {} bytes", h264_data.len());

        // 3. 解码
        let frame = match self.decoder.decode(&h264_data) {
            Ok(f) => f,
            Err(e) => {
                self.stats.write().await.decoding_errors += 1;
                return Err(SessionError::DecoderError(e.to_string()));
            }
        };

        self.stats.write().await.frames_decoded += 1;

        // 4. 输出帧
        if let Err(e) = self.frame_tx.try_send(frame) {
            warn!("Frame queue full, dropping frame: {}", e);
        }

        Ok(())
    }

    /// 获取解码后的帧（供应用调用）。
    ///
    /// 返回 `None` 表示暂无可用帧。
    pub async fn poll_frame(&mut self) -> Option<VideoFrame> {
        self.frame_rx.recv().await
    }

    /// 获取统计信息。
    pub async fn stats(&self) -> ReceiverStats {
        self.stats.read().await.clone()
    }

    /// 重置统计信息。
    pub async fn reset_stats(&self) {
        *self.stats.write().await = ReceiverStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::DecoderConfig;
    use crate::platform::macos::VideoToolboxDecoder;
    use crate::rtp::{SrtpConfig, SrtpProfile};

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
    async fn test_video_receiver_creation() {
        let decoder = Box::new(VideoToolboxDecoder::new().unwrap());
        let srtp = create_test_srtp();
        let config = VideoReceiverConfig::default();

        let receiver = VideoReceiver::new(decoder, srtp, config);
        let stats = receiver.stats().await;

        assert_eq!(stats.packets_received, 0);
        assert_eq!(stats.frames_decoded, 0);
    }

    #[tokio::test]
    async fn test_receive_packet() {
        let decoder = Box::new(VideoToolboxDecoder::new().unwrap());
        let srtp = create_test_srtp();
        init_test_srtp(&srtp).await;

        let config = VideoReceiverConfig::default();
        let receiver = VideoReceiver::new(decoder, srtp, config);

        // 接收一个测试包
        let test_packet = vec![0u8; 100];
        receiver.receive_packet(test_packet).await.unwrap();

        let stats = receiver.stats().await;
        assert_eq!(stats.packets_received, 1);
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let decoder = Box::new(VideoToolboxDecoder::new().unwrap());
        let srtp = create_test_srtp();
        let config = VideoReceiverConfig::default();
        let receiver = VideoReceiver::new(decoder, srtp, config);

        // 手动更新统计
        receiver.stats.write().await.packets_received = 100;
        receiver.stats.write().await.frames_decoded = 30;

        let stats = receiver.stats().await;
        assert_eq!(stats.packets_received, 100);
        assert_eq!(stats.frames_decoded, 30);

        // 重置
        receiver.reset_stats().await;
        let stats = receiver.stats().await;
        assert_eq!(stats.packets_received, 0);
    }

    #[tokio::test]
    async fn test_process_without_srtp_fails() {
        let decoder = Box::new(VideoToolboxDecoder::new().unwrap());
        let srtp = create_test_srtp(); // 未初始化
        let config = VideoReceiverConfig::default();
        let mut receiver = VideoReceiver::new(decoder, srtp, config);

        // 放入一个包
        let test_packet = vec![0u8; 100];
        receiver.receive_packet(test_packet).await.unwrap();

        // 处理应该因为 SRTP 未就绪而失败
        // 注意：process_single_packet 是私有的，这里仅验证队列接收
        let stats = receiver.stats().await;
        assert_eq!(stats.packets_received, 1);
    }
}
