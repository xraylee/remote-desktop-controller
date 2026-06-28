// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 完整的视频会话 API。
//!
//! 整合所有组件，提供简单的 start/stop 接口。

use super::{
    key_exchange::{KeyExchange, KeyExchangeCoordinator},
    network::{UdpTransport, UdpTransportConfig},
    peer_connection::{PeerConnection, PeerConnectionConfig},
    qos::{QosMonitor, QosMonitorConfig},
    receiver::{VideoReceiver, VideoReceiverConfig},
    sender::{VideoSender, VideoSenderConfig},
    stats::SessionStats,
    SessionError,
};
use crate::decoder::{DecoderConfig, VideoDecoder};
use crate::encoder::{EncoderConfig, VideoEncoder};
use crate::types::{VideoFormat, VideoFrame};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{debug, error, info};

/// 视频会话配置。
#[derive(Debug, Clone)]
pub struct VideoSessionConfig {
    /// 编码器配置。
    pub encoder: EncoderConfig,
    /// 解码器配置。
    pub decoder: DecoderConfig,
    /// 发送器配置。
    pub sender: VideoSenderConfig,
    /// 接收器配置。
    pub receiver: VideoReceiverConfig,
    /// 网络配置。
    pub network: UdpTransportConfig,
    /// QoS 配置。
    pub qos: QosMonitorConfig,
    /// 是否为控制端（offer 方）。
    pub is_controller: bool,
}

impl Default for VideoSessionConfig {
    fn default() -> Self {
        let format = VideoFormat {
            width: 1280,
            height: 720,
            pixel_format: crate::types::PixelFormat::Nv12,
            framerate: 30,
        };

        Self {
            encoder: EncoderConfig {
                format: format.clone(),
                bitrate: 2_000_000,
                keyframe_interval: 30,
            },
            decoder: DecoderConfig { format },
            sender: VideoSenderConfig::default(),
            receiver: VideoReceiverConfig::default(),
            network: UdpTransportConfig::default(),
            qos: QosMonitorConfig::default(),
            is_controller: true,
        }
    }
}

/// 视频会话状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// 未启动。
    Idle,
    /// 正在启动。
    Starting,
    /// 运行中。
    Running,
    /// 正在停止。
    Stopping,
    /// 已停止。
    Stopped,
    /// 错误状态。
    Failed,
}

/// 完整的视频会话。
pub struct VideoSession {
    config: VideoSessionConfig,
    state: Arc<RwLock<SessionState>>,

    // 核心组件
    peer_connection: Option<PeerConnection>,
    video_sender: Option<VideoSender>,
    video_receiver: Option<VideoReceiver>,
    network: Option<UdpTransport>,
    qos_monitor: Option<QosMonitor>,

    // 统计信息
    stats: Arc<RwLock<SessionStats>>,

    // 后台任务
    send_task: Option<JoinHandle<()>>,
    recv_task: Option<JoinHandle<()>>,
    stats_task: Option<JoinHandle<()>>,

    // 控制通道
    frame_tx: mpsc::Sender<VideoFrame>,
    frame_rx: Option<mpsc::Receiver<VideoFrame>>,
    output_tx: mpsc::Sender<VideoFrame>,
    output_rx: Option<mpsc::Receiver<VideoFrame>>,
}

impl VideoSession {
    /// 创建新的视频会话。
    pub fn new(config: VideoSessionConfig) -> Self {
        let (frame_tx, frame_rx) = mpsc::channel(10);
        let (output_tx, output_rx) = mpsc::channel(10);

        Self {
            config,
            state: Arc::new(RwLock::new(SessionState::Idle)),
            peer_connection: None,
            video_sender: None,
            video_receiver: None,
            network: None,
            qos_monitor: None,
            stats: Arc::new(RwLock::new(SessionStats::default())),
            send_task: None,
            recv_task: None,
            stats_task: None,
            frame_tx,
            frame_rx: Some(frame_rx),
            output_tx,
            output_rx: Some(output_rx),
        }
    }

    /// 启动会话。
    ///
    /// 执行完整的初始化流程：
    /// 1. 密钥交换
    /// 2. 建立 PeerConnection
    /// 3. 创建编解码器
    /// 4. 启动网络传输
    /// 5. 启动发送/接收任务
    pub async fn start(
        &mut self,
        encoder: Box<dyn VideoEncoder + Send>,
        decoder: Box<dyn VideoDecoder + Send>,
    ) -> Result<(), SessionError> {
        let mut state = self.state.write().await;
        if *state != SessionState::Idle {
            return Err(SessionError::NotReady(format!(
                "Cannot start from state {:?}",
                *state
            )));
        }
        *state = SessionState::Starting;
        drop(state);

        info!("Starting video session");

        // 1. 密钥交换（简化版：使用协调器）
        let mut kex_coordinator = KeyExchangeCoordinator::new();
        let (srtp_config, _) = kex_coordinator
            .perform_handshake(crate::rtp::SrtpProfile::Aead_Aes128Gcm)
            .map_err(|e| SessionError::ConfigError(e.to_string()))?;

        debug!("Key exchange completed");

        // 2. 建立 PeerConnection
        let pc_config = PeerConnectionConfig {
            is_controller: self.config.is_controller,
            ..Default::default()
        };
        let pc = PeerConnection::new(pc_config);
        pc.connect(srtp_config).await?;

        debug!("PeerConnection established");

        // 3. 创建发送器和接收器
        let tx_srtp = pc.tx_srtp().await?;
        let rx_srtp = pc.rx_srtp().await?;

        let video_sender = VideoSender::new(encoder, tx_srtp, self.config.sender.clone());
        let video_receiver = VideoReceiver::new(decoder, rx_srtp, self.config.receiver.clone());

        // 4. 创建网络传输
        let network = UdpTransport::new(self.config.network.clone()).await
            .map_err(|e| SessionError::NetworkError(e.to_string()))?;

        info!(
            "Network transport: {} -> {}",
            network.local_addr().unwrap(),
            network.remote_addr()
        );

        // 5. 创建 QoS 监控器
        let qos_monitor = QosMonitor::new(self.config.qos.clone());

        // 保存组件
        self.peer_connection = Some(pc);
        self.video_sender = Some(video_sender);
        self.video_receiver = Some(video_receiver);
        self.network = Some(network);
        self.qos_monitor = Some(qos_monitor);

        // 6. 启动后台任务
        self.start_background_tasks().await?;

        *self.state.write().await = SessionState::Running;
        info!("Video session started successfully");

        Ok(())
    }

    /// 启动后台任务。
    async fn start_background_tasks(&mut self) -> Result<(), SessionError> {
        // 发送任务：帧输入 → 编码 → RTP → SRTP → 网络
        let mut sender = self.video_sender.take().unwrap();
        let mut network_send = self.network.as_ref().unwrap().clone(); // TODO: 需要实现 Clone
        let mut frame_rx = self.frame_rx.take().unwrap();

        let send_task = tokio::spawn(async move {
            while let Some(frame) = frame_rx.recv().await {
                if let Err(e) = sender.send_frame(&frame).await {
                    error!("Send frame error: {}", e);
                    continue;
                }

                // 发送所有 RTP 包到网络
                while let Some(packet) = sender.poll_packet().await {
                    if let Err(e) = network_send.send(packet).await {
                        error!("Network send error: {}", e);
                    }
                }
            }
        });

        // 接收任务：网络 → SRTP → RTP → 解码 → 帧输出
        let mut receiver = self.video_receiver.take().unwrap();
        let mut network_recv = self.network.take().unwrap();
        let output_tx = self.output_tx.clone();

        let recv_task = tokio::spawn(async move {
            // 网络接收子任务
            let receiver_ref = Arc::new(RwLock::new(receiver));
            let receiver_clone = receiver_ref.clone();

            tokio::spawn(async move {
                while let Some(packet) = network_recv.recv().await {
                    let receiver = receiver_clone.read().await;
                    if let Err(e) = receiver.receive_packet(packet).await {
                        error!("Receive packet error: {}", e);
                    }
                }
            });

            // 帧输出子任务
            let mut receiver = receiver_ref.write().await;
            while let Some(frame) = receiver.poll_frame().await {
                if output_tx.send(frame).await.is_err() {
                    error!("Output queue full");
                }
            }
        });

        // 统计任务：定期更新统计信息
        let stats = self.stats.clone();
        let mut qos = self.qos_monitor.take().unwrap();

        let stats_task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let mut session_stats = stats.write().await;
                session_stats.update_duration();
                qos.update_stats(&mut session_stats);
            }
        });

        self.send_task = Some(send_task);
        self.recv_task = Some(recv_task);
        self.stats_task = Some(stats_task);

        Ok(())
    }

    /// 发送一帧视频（供应用调用）。
    pub async fn send_frame(&self, frame: VideoFrame) -> Result<(), SessionError> {
        if *self.state.read().await != SessionState::Running {
            return Err(SessionError::NotReady("session not running".into()));
        }

        self.frame_tx
            .send(frame)
            .await
            .map_err(|_| SessionError::SessionClosed)
    }

    /// 接收解码后的帧（供应用调用）。
    pub async fn recv_frame(&mut self) -> Option<VideoFrame> {
        if let Some(ref mut rx) = self.output_rx {
            rx.recv().await
        } else {
            None
        }
    }

    /// 获取当前状态。
    pub async fn state(&self) -> SessionState {
        *self.state.read().await
    }

    /// 获取统计信息。
    pub async fn stats(&self) -> SessionStats {
        self.stats.read().await.clone()
    }

    /// 停止会话。
    pub async fn stop(&mut self) -> Result<(), SessionError> {
        let mut state = self.state.write().await;
        if *state == SessionState::Stopped {
            return Ok(());
        }
        *state = SessionState::Stopping;
        drop(state);

        info!("Stopping video session");

        // 停止后台任务
        if let Some(task) = self.send_task.take() {
            task.abort();
        }
        if let Some(task) = self.recv_task.take() {
            task.abort();
        }
        if let Some(task) = self.stats_task.take() {
            task.abort();
        }

        // 关闭 PeerConnection
        if let Some(pc) = &self.peer_connection {
            pc.close().await?;
        }

        *self.state.write().await = SessionState::Stopped;
        info!("Video session stopped");

        Ok(())
    }
}

// 注意：UdpTransport 需要实现 Clone 或使用 Arc
// 这里先注释掉有问题的代码，待后续修复
