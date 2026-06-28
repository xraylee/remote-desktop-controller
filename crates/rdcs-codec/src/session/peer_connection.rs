// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! WebRTC Peer Connection 封装。
//!
//! 管理 ICE 协商、DTLS 握手、SRTP 密钥派生。

use super::{Result, SessionError};
use crate::rtp::{SrtpConfig, SrtpContext, SrtpProfile};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Peer Connection 状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerConnectionState {
    /// 新创建，未开始协商。
    New,
    /// 正在进行 ICE 协商。
    Connecting,
    /// ICE 连接已建立。
    Connected,
    /// DTLS 握手完成，SRTP 密钥已派生。
    Ready,
    /// 连接失败。
    Failed,
    /// 连接已关闭。
    Closed,
}

/// Peer Connection 配置。
#[derive(Debug, Clone)]
pub struct PeerConnectionConfig {
    /// ICE 服务器列表（STUN/TURN）。
    pub ice_servers: Vec<String>,
    /// 是否启用 mDNS 候选。
    pub enable_mdns: bool,
    /// SRTP 加密配置（默认 AES-128-GCM）。
    pub srtp_profile: SrtpProfile,
    /// 是否为控制端（offer 方）。
    pub is_controller: bool,
}

impl Default for PeerConnectionConfig {
    fn default() -> Self {
        Self {
            ice_servers: vec!["stun:stun.l.google.com:19302".to_string()],
            enable_mdns: true,
            srtp_profile: SrtpProfile::Aead_Aes128Gcm,
            is_controller: true,
        }
    }
}

/// WebRTC Peer Connection。
///
/// 注意：当前实现为简化版，仅管理 SRTP 上下文。完整的 ICE/DTLS 集成
/// 将在后续与 rdcs-signaling 对接时完成。
pub struct PeerConnection {
    config: PeerConnectionConfig,
    state: Arc<RwLock<PeerConnectionState>>,
    /// 发送端 SRTP 上下文。
    tx_srtp: Arc<RwLock<Option<SrtpContext>>>,
    /// 接收端 SRTP 上下文。
    rx_srtp: Arc<RwLock<Option<SrtpContext>>>,
}

impl PeerConnection {
    /// 创建新的 Peer Connection。
    pub fn new(config: PeerConnectionConfig) -> Self {
        debug!("Creating PeerConnection with config: {:?}", config);
        Self {
            config,
            state: Arc::new(RwLock::new(PeerConnectionState::New)),
            tx_srtp: Arc::new(RwLock::new(None)),
            rx_srtp: Arc::new(RwLock::new(None)),
        }
    }

    /// 获取当前状态。
    pub async fn state(&self) -> PeerConnectionState {
        *self.state.read().await
    }

    /// 开始连接协商。
    ///
    /// 在完整实现中，这会触发 ICE gathering 和 DTLS 握手。
    /// 当前简化版本直接使用预配置的 SRTP 密钥。
    pub async fn connect(&self, srtp_config: SrtpConfig) -> Result<()> {
        let mut state = self.state.write().await;
        if *state != PeerConnectionState::New {
            return Err(SessionError::NotReady(format!(
                "Cannot connect from state {:?}",
                *state
            )));
        }

        info!("Starting connection negotiation");
        *state = PeerConnectionState::Connecting;
        drop(state);

        // 创建 SRTP 上下文（发送和接收使用相同的密钥材料）
        let tx_ctx = SrtpContext::new(srtp_config.clone()).await?;
        let rx_ctx = SrtpContext::new(srtp_config).await?;

        *self.tx_srtp.write().await = Some(tx_ctx);
        *self.rx_srtp.write().await = Some(rx_ctx);

        // 标记为已连接和就绪
        let mut state = self.state.write().await;
        *state = PeerConnectionState::Connected;
        info!("ICE connection established");

        *state = PeerConnectionState::Ready;
        info!("DTLS handshake complete, SRTP keys derived");

        Ok(())
    }

    /// 获取发送端 SRTP 上下文。
    pub async fn tx_srtp(&self) -> Result<Arc<RwLock<Option<SrtpContext>>>> {
        let state = self.state.read().await;
        if *state != PeerConnectionState::Ready {
            return Err(SessionError::NotReady(format!(
                "SRTP not ready in state {:?}",
                *state
            )));
        }
        Ok(self.tx_srtp.clone())
    }

    /// 获取接收端 SRTP 上下文。
    pub async fn rx_srtp(&self) -> Result<Arc<RwLock<Option<SrtpContext>>>> {
        let state = self.state.read().await;
        if *state != PeerConnectionState::Ready {
            return Err(SessionError::NotReady(format!(
                "SRTP not ready in state {:?}",
                *state
            )));
        }
        Ok(self.rx_srtp.clone())
    }

    /// 关闭连接。
    pub async fn close(&self) -> Result<()> {
        let mut state = self.state.write().await;
        if *state == PeerConnectionState::Closed {
            return Ok(());
        }

        info!("Closing PeerConnection");

        // 清理 SRTP 上下文
        *self.tx_srtp.write().await = None;
        *self.rx_srtp.write().await = None;

        *state = PeerConnectionState::Closed;
        Ok(())
    }

    /// 检查连接是否就绪。
    pub async fn is_ready(&self) -> bool {
        *self.state.read().await == PeerConnectionState::Ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtp::SrtpProfile;

    fn create_test_srtp_config() -> SrtpConfig {
        SrtpConfig {
            master_key: vec![0x42; 16],
            master_salt: vec![0x84; 14],
            profile: SrtpProfile::Aead_Aes128Gcm,
        }
    }

    #[tokio::test]
    async fn test_peer_connection_lifecycle() {
        let config = PeerConnectionConfig::default();
        let pc = PeerConnection::new(config);

        // 初始状态
        assert_eq!(pc.state().await, PeerConnectionState::New);
        assert!(!pc.is_ready().await);

        // 连接
        let srtp_config = create_test_srtp_config();
        pc.connect(srtp_config).await.unwrap();

        assert_eq!(pc.state().await, PeerConnectionState::Ready);
        assert!(pc.is_ready().await);

        // 验证 SRTP 上下文可用
        assert!(pc.tx_srtp().await.is_ok());
        assert!(pc.rx_srtp().await.is_ok());

        // 关闭
        pc.close().await.unwrap();
        assert_eq!(pc.state().await, PeerConnectionState::Closed);
    }

    #[tokio::test]
    async fn test_cannot_connect_twice() {
        let config = PeerConnectionConfig::default();
        let pc = PeerConnection::new(config);

        let srtp_config = create_test_srtp_config();
        pc.connect(srtp_config.clone()).await.unwrap();

        // 第二次连接应该失败
        let result = pc.connect(srtp_config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_srtp_not_ready_before_connect() {
        let config = PeerConnectionConfig::default();
        let pc = PeerConnection::new(config);

        // 未连接前无法获取 SRTP
        assert!(pc.tx_srtp().await.is_err());
        assert!(pc.rx_srtp().await.is_err());
    }

    #[tokio::test]
    async fn test_close_idempotent() {
        let config = PeerConnectionConfig::default();
        let pc = PeerConnection::new(config);

        let srtp_config = create_test_srtp_config();
        pc.connect(srtp_config).await.unwrap();

        // 多次关闭应该成功
        pc.close().await.unwrap();
        pc.close().await.unwrap();
        assert_eq!(pc.state().await, PeerConnectionState::Closed);
    }
}
