// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! UDP 网络传输层。
//!
//! 提供异步 UDP socket 发送/接收功能，连接 VideoSender 和 VideoReceiver。

use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{debug, trace, warn};

/// 网络传输错误。
#[derive(Debug, Error)]
pub enum NetworkError {
    /// Socket 绑定失败。
    #[error("bind failed: {0}")]
    BindFailed(#[from] std::io::Error),

    /// 发送失败。
    #[error("send failed: {0}")]
    SendFailed(String),

    /// 接收失败。
    #[error("receive failed: {0}")]
    ReceiveFailed(String),

    /// 传输已关闭。
    #[error("transport closed")]
    TransportClosed,
}

/// 网络传输结果。
pub type Result<T> = std::result::Result<T, NetworkError>;

/// UDP 传输配置。
#[derive(Debug, Clone)]
pub struct UdpTransportConfig {
    /// 本地绑定地址。
    pub local_addr: SocketAddr,
    /// 远程对端地址。
    pub remote_addr: SocketAddr,
    /// 接收缓冲区大小。
    pub recv_buffer_size: usize,
    /// 发送队列大小。
    pub send_queue_size: usize,
}

impl Default for UdpTransportConfig {
    fn default() -> Self {
        Self {
            local_addr: "127.0.0.1:0".parse().unwrap(), // 随机端口
            remote_addr: "127.0.0.1:9001".parse().unwrap(),
            recv_buffer_size: 1500,
            send_queue_size: 100,
        }
    }
}

/// UDP 传输层。
pub struct UdpTransport {
    /// UDP socket。
    socket: Arc<UdpSocket>,
    /// 远程地址。
    remote_addr: SocketAddr,
    /// 发送队列。
    tx_send: mpsc::Sender<Vec<u8>>,
    /// 接收队列。
    rx_recv: mpsc::Receiver<Vec<u8>>,
    /// 统计信息。
    stats: Arc<tokio::sync::RwLock<TransportStats>>,
}

/// 传输统计。
#[derive(Debug, Clone, Default)]
pub struct TransportStats {
    pub packets_sent: u64,
    pub packets_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub send_errors: u32,
    pub recv_errors: u32,
}

impl UdpTransport {
    /// 创建新的 UDP 传输层。
    ///
    /// 绑定到指定地址并准备发送/接收。
    pub async fn new(config: UdpTransportConfig) -> Result<Self> {
        let socket = UdpSocket::bind(&config.local_addr).await?;
        let local_addr = socket.local_addr()?;

        debug!("UDP transport bound to {}", local_addr);

        let (tx_send, mut rx_send) = mpsc::channel(config.send_queue_size);
        let (tx_recv, rx_recv) = mpsc::channel(config.send_queue_size);

        let socket = Arc::new(socket);
        let stats = Arc::new(tokio::sync::RwLock::new(TransportStats::default()));

        // 启动发送任务
        let send_socket = socket.clone();
        let send_stats = stats.clone();
        let remote_addr = config.remote_addr;
        tokio::spawn(async move {
            while let Some(packet) = rx_send.recv().await {
                match send_socket.send_to(&packet, remote_addr).await {
                    Ok(bytes) => {
                        let mut stats = send_stats.write().await;
                        stats.packets_sent += 1;
                        stats.bytes_sent += bytes as u64;
                        trace!("Sent {} bytes to {}", bytes, remote_addr);
                    }
                    Err(e) => {
                        warn!("Send error: {}", e);
                        send_stats.write().await.send_errors += 1;
                    }
                }
            }
        });

        // 启动接收任务
        let recv_socket = socket.clone();
        let recv_stats = stats.clone();
        let recv_buffer_size = config.recv_buffer_size;
        tokio::spawn(async move {
            let mut buf = vec![0u8; recv_buffer_size];
            loop {
                match recv_socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        let mut stats = recv_stats.write().await;
                        stats.packets_received += 1;
                        stats.bytes_received += len as u64;
                        trace!("Received {} bytes from {}", len, addr);
                        drop(stats);

                        // 发送到接收队列
                        let packet = buf[..len].to_vec();
                        if tx_recv.send(packet).await.is_err() {
                            warn!("Receive queue full, dropping packet");
                            recv_stats.write().await.recv_errors += 1;
                        }
                    }
                    Err(e) => {
                        warn!("Receive error: {}", e);
                        recv_stats.write().await.recv_errors += 1;
                    }
                }
            }
        });

        Ok(Self {
            socket,
            remote_addr: config.remote_addr,
            tx_send,
            rx_recv,
            stats,
        })
    }

    /// 发送数据包。
    pub async fn send(&self, packet: Vec<u8>) -> Result<()> {
        self.tx_send
            .send(packet)
            .await
            .map_err(|_| NetworkError::TransportClosed)
    }

    /// 接收数据包（非阻塞）。
    pub async fn recv(&mut self) -> Option<Vec<u8>> {
        self.rx_recv.recv().await
    }

    /// 获取本地地址。
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.socket.local_addr().map_err(NetworkError::BindFailed)
    }

    /// 获取远程地址。
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// 获取统计信息。
    pub async fn stats(&self) -> TransportStats {
        self.stats.read().await.clone()
    }

    /// 重置统计信息。
    pub async fn reset_stats(&self) {
        *self.stats.write().await = TransportStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_udp_transport_creation() {
        let config = UdpTransportConfig {
            local_addr: "127.0.0.1:0".parse().unwrap(),
            remote_addr: "127.0.0.1:9999".parse().unwrap(),
            ..Default::default()
        };

        let transport = UdpTransport::new(config).await.unwrap();
        let local = transport.local_addr().unwrap();

        assert!(local.port() > 0);
        assert_eq!(transport.remote_addr().port(), 9999);
    }

    #[tokio::test]
    async fn test_loopback_communication() {
        // 创建两个传输实例
        let config1 = UdpTransportConfig {
            local_addr: "127.0.0.1:0".parse().unwrap(),
            remote_addr: "127.0.0.1:0".parse().unwrap(), // 临时
            ..Default::default()
        };

        let mut transport1 = UdpTransport::new(config1).await.unwrap();
        let addr1 = transport1.local_addr().unwrap();

        let config2 = UdpTransportConfig {
            local_addr: "127.0.0.1:0".parse().unwrap(),
            remote_addr: addr1, // 指向 transport1
            ..Default::default()
        };

        let mut transport2 = UdpTransport::new(config2).await.unwrap();
        let addr2 = transport2.local_addr().unwrap();

        // 更新 transport1 的远程地址
        // 注意：当前实现不支持运行时更新，这里创建新实例
        let config1 = UdpTransportConfig {
            local_addr: addr1,
            remote_addr: addr2,
            ..Default::default()
        };
        let mut transport1 = UdpTransport::new(config1).await.unwrap();

        // 发送测试包
        let test_packet = vec![0x42; 100];
        transport1.send(test_packet.clone()).await.unwrap();

        // 等待接收
        sleep(Duration::from_millis(50)).await;

        if let Some(received) = transport2.recv().await {
            assert_eq!(received, test_packet);
        }

        // 验证统计
        let stats1 = transport1.stats().await;
        assert_eq!(stats1.packets_sent, 1);

        let stats2 = transport2.stats().await;
        assert_eq!(stats2.packets_received, 1);
    }

    #[tokio::test]
    async fn test_send_multiple_packets() {
        let config = UdpTransportConfig {
            local_addr: "127.0.0.1:0".parse().unwrap(),
            remote_addr: "127.0.0.1:9998".parse().unwrap(),
            ..Default::default()
        };

        let transport = UdpTransport::new(config).await.unwrap();

        // 发送多个包
        for i in 0..10 {
            let packet = vec![i as u8; 50];
            transport.send(packet).await.unwrap();
        }

        sleep(Duration::from_millis(100)).await;

        let stats = transport.stats().await;
        assert_eq!(stats.packets_sent, 10);
    }

    #[tokio::test]
    async fn test_stats_reset() {
        let config = UdpTransportConfig {
            local_addr: "127.0.0.1:0".parse().unwrap(),
            remote_addr: "127.0.0.1:9997".parse().unwrap(),
            ..Default::default()
        };

        let transport = UdpTransport::new(config).await.unwrap();

        // 发送一些包
        transport.send(vec![0x42; 100]).await.unwrap();
        sleep(Duration::from_millis(50)).await;

        let stats = transport.stats().await;
        assert!(stats.packets_sent > 0);

        // 重置统计
        transport.reset_stats().await;

        let stats = transport.stats().await;
        assert_eq!(stats.packets_sent, 0);
        assert_eq!(stats.bytes_sent, 0);
    }
}
