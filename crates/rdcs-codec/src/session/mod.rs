// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! WebRTC 会话管理模块。
//!
//! 集成 RTP/SRTP 传输层与编解码器管道，提供完整的 WebRTC 视频会话功能。
//!
//! # 架构
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    VideoSession                             │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
//! │  │   Encoder    │───▶│  Packetizer  │───▶│     SRTP     │ │
//! │  │              │    │     (RTP)    │    │  (Encrypt)   │ │
//! │  └──────────────┘    └──────────────┘    └──────────────┘ │
//! │         ▲                                        │         │
//! │         │                                        ▼         │
//! │  ┌──────────────┐                       ┌──────────────┐  │
//! │  │ Video Input  │                       │   Network    │  │
//! │  │   (Frames)   │                       │   (Socket)   │  │
//! │  └──────────────┘                       └──────────────┘  │
//! │                                                 │          │
//! │  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐ │
//! │  │   Decoder    │◀───│Depacketizer  │◀───│     SRTP     │ │
//! │  │              │    │     (RTP)    │    │  (Decrypt)   │ │
//! │  └──────────────┘    └──────────────┘    └──────────────┘ │
//! │         │                                                  │
//! │         ▼                                                  │
//! │  ┌──────────────┐                                         │
//! │  │ Video Output │                                         │
//! │  │  (Rendered)  │                                         │
//! │  └──────────────┘                                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```

pub mod key_exchange;
pub mod network;
pub mod peer_connection;
pub mod qos;
pub mod receiver;
pub mod sender;
pub mod stats;
pub mod video_session;

pub use key_exchange::{KeyExchange, KeyExchangeCoordinator};
pub use network::{UdpTransport, UdpTransportConfig};
pub use peer_connection::{PeerConnection, PeerConnectionConfig, PeerConnectionState};
pub use qos::{QosMonitor, QosMonitorConfig};
pub use receiver::VideoReceiver;
pub use sender::VideoSender;
pub use stats::{NetworkQuality, SessionStats};
pub use video_session::{VideoSession, VideoSessionConfig, SessionState};

use thiserror::Error;

/// Session 模块错误。
#[derive(Debug, Error)]
pub enum SessionError {
    /// 编码器错误。
    #[error("encoder error: {0}")]
    EncoderError(String),

    /// 解码器错误。
    #[error("decoder error: {0}")]
    DecoderError(String),

    /// RTP 错误。
    #[error("rtp error: {0}")]
    RtpError(#[from] crate::rtp::RtpError),

    /// 网络错误。
    #[error("network error: {0}")]
    NetworkError(String),

    /// 会话未就绪。
    #[error("session not ready: {0}")]
    NotReady(String),

    /// 会话已关闭。
    #[error("session closed")]
    SessionClosed,

    /// 配置错误。
    #[error("config error: {0}")]
    ConfigError(String),

    /// WebRTC 错误。
    #[error("webrtc error: {0}")]
    WebRtcError(#[from] webrtc::Error),

    /// 平台错误。
    #[error("platform error: {0}")]
    PlatformError(#[from] crate::CodecError),
}

/// Session 结果类型。
pub type Result<T> = std::result::Result<T, SessionError>;
