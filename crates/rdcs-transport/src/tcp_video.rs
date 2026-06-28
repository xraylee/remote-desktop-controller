// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 简单的 TCP 视频流传输
//!
//! Phase 2 使用简单的 TCP 传输，避免 Phase 1 的复杂性。
//!
//! 协议格式：
//! ```text
//! [4 bytes: frame_size][frame_data]
//! ```

use bytes::{Buf, BufMut, BytesMut};
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::{debug, warn};

/// TCP 视频流发送端
pub struct TcpVideoSender {
    stream: TcpStream,
    frame_count: u64,
}

impl TcpVideoSender {
    /// 从已连接的 TCP stream 创建发送端
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            frame_count: 0,
        }
    }

    /// 发送一帧编码后的视频数据
    pub async fn send_frame(&mut self, frame_data: &[u8]) -> io::Result<()> {
        // 协议格式：[4 bytes length][data]
        let frame_size = frame_data.len() as u32;

        // 写入帧大小
        self.stream.write_u32(frame_size).await?;

        // 写入帧数据
        self.stream.write_all(frame_data).await?;

        // 确保数据发送
        self.stream.flush().await?;

        self.frame_count += 1;

        debug!(
            "Sent frame #{}: {} bytes",
            self.frame_count,
            frame_data.len()
        );

        Ok(())
    }

    /// 获取已发送帧数
    pub fn frames_sent(&self) -> u64 {
        self.frame_count
    }

    /// 关闭连接
    pub async fn shutdown(&mut self) -> io::Result<()> {
        self.stream.shutdown().await
    }
}

/// TCP 视频流接收端
pub struct TcpVideoReceiver {
    stream: TcpStream,
    buffer: BytesMut,
    frame_count: u64,
}

impl TcpVideoReceiver {
    /// 从已连接的 TCP stream 创建接收端
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(4096),
            frame_count: 0,
        }
    }

    /// 接收一帧视频数据
    ///
    /// 返回 `None` 表示连接已关闭
    pub async fn recv_frame(&mut self) -> io::Result<Option<Vec<u8>>> {
        // 读取帧大小（4 字节）
        let frame_size = match self.read_u32().await? {
            Some(size) => size as usize,
            None => return Ok(None), // 连接关闭
        };

        // 验证帧大小合理性（最大 10MB）
        if frame_size > 10 * 1024 * 1024 {
            warn!("Frame size too large: {} bytes", frame_size);
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame size too large: {}", frame_size),
            ));
        }

        // 读取帧数据
        let frame_data = self.read_exact(frame_size).await?;

        self.frame_count += 1;

        debug!(
            "Received frame #{}: {} bytes",
            self.frame_count,
            frame_data.len()
        );

        Ok(Some(frame_data))
    }

    /// 获取已接收帧数
    pub fn frames_received(&self) -> u64 {
        self.frame_count
    }

    /// 读取 u32
    async fn read_u32(&mut self) -> io::Result<Option<u32>> {
        // 确保缓冲区至少有 4 字节
        while self.buffer.len() < 4 {
            let n = self.stream.read_buf(&mut self.buffer).await?;
            if n == 0 {
                // 连接关闭
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "Connection closed while reading frame size",
                    ));
                }
            }
        }

        Ok(Some(self.buffer.get_u32()))
    }

    /// 读取指定数量的字节
    async fn read_exact(&mut self, n: usize) -> io::Result<Vec<u8>> {
        // 确保缓冲区有足够数据
        while self.buffer.len() < n {
            let read = self.stream.read_buf(&mut self.buffer).await?;
            if read == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "Connection closed while reading frame data",
                ));
            }
        }

        // 提取数据
        let data = self.buffer.split_to(n).to_vec();
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn test_send_recv_frame() {
        // 创建本地 TCP 服务器
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // 生成测试数据
        let test_data = vec![1, 2, 3, 4, 5];

        // 启动发送端
        let sender_handle = tokio::spawn(async move {
            let stream = TcpStream::connect(addr).await.unwrap();
            let mut sender = TcpVideoSender::new(stream);
            sender.send_frame(&test_data).await.unwrap();
            sender.shutdown().await.unwrap();
        });

        // 接收端
        let (stream, _) = listener.accept().await.unwrap();
        let mut receiver = TcpVideoReceiver::new(stream);
        let received = receiver.recv_frame().await.unwrap().unwrap();

        assert_eq!(received, vec![1, 2, 3, 4, 5]);
        assert_eq!(receiver.frames_received(), 1);

        sender_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_frames() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let frames = vec![
            vec![1, 2, 3],
            vec![4, 5, 6, 7],
            vec![8],
        ];

        let frames_clone = frames.clone();
        let sender_handle = tokio::spawn(async move {
            let stream = TcpStream::connect(addr).await.unwrap();
            let mut sender = TcpVideoSender::new(stream);

            for frame in &frames_clone {
                sender.send_frame(frame).await.unwrap();
            }

            sender.shutdown().await.unwrap();
        });

        let (stream, _) = listener.accept().await.unwrap();
        let mut receiver = TcpVideoReceiver::new(stream);

        for (i, expected) in frames.iter().enumerate() {
            let received = receiver.recv_frame().await.unwrap().unwrap();
            assert_eq!(received, *expected, "Frame {} mismatch", i);
        }

        assert_eq!(receiver.frames_received(), 3);

        sender_handle.await.unwrap();
    }
}
