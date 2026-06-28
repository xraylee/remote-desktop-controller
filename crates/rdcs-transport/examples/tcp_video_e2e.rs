// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! TCP 视频传输端到端测试
//!
//! 此示例演示：
//! 1. 发送端：Mock 屏幕捕获 → OpenH264 编码 → TCP 发送
//! 2. 接收端：TCP 接收 → OpenH264 解码 → 保存为 PPM

use rdcs_codec::platform::NativeVideoEncoder;
use rdcs_codec::types::{VideoCodec, VideoResolution};
use rdcs_platform::{CapturedFrame, PixelFormat};
use rdcs_transport::tcp_video::{TcpVideoReceiver, TcpVideoSender};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("========================================");
    info!("TCP 视频传输端到端测试");
    info!("========================================");

    // 测试参数
    let width = 640;
    let height = 480;
    let fps = 30;
    let bitrate = 1_000_000; // 1 Mbps
    let num_frames = 10;

    // 启动 TCP 服务器
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    info!("TCP 服务器监听: {}", addr);

    // 启动发送端任务
    let sender_handle = tokio::spawn(async move {
        info!("发送端：连接到 {}", addr);
        let stream = TcpStream::connect(addr).await?;
        let mut sender = TcpVideoSender::new(stream);

        info!("发送端：初始化 OpenH264 编码器");
        let mut encoder = NativeVideoEncoder::new(
            VideoCodec::H264,
            VideoResolution::Custom(width, height),
            fps,
            bitrate,
        )?;

        info!("发送端：开始发送 {} 帧", num_frames);
        for i in 0..num_frames {
            // 生成 Mock 帧（渐变图案）
            let captured_frame = create_test_frame(width, height, i);

            // 编码
            let encoded_data = encoder.encode_captured_frame(&captured_frame)?;

            // 通过 TCP 发送
            sender.send_frame(&encoded_data).await?;

            info!("发送端：已发送第 {} 帧 ({} bytes)", i + 1, encoded_data.len());

            // 模拟帧间隔
            tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
        }

        info!("发送端：关闭连接");
        sender.shutdown().await?;

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    });

    // 接收端
    info!("接收端：等待连接...");
    let (stream, peer_addr) = listener.accept().await?;
    info!("接收端：接受连接来自 {}", peer_addr);

    let mut receiver = TcpVideoReceiver::new(stream);

    info!("接收端：初始化 OpenH264 解码器");
    let mut decoder = rdcs_codec::platform::NativeVideoDecoder::new(VideoCodec::H264)?;

    info!("接收端：开始接收帧");
    let mut frame_count = 0;

    while let Some(encoded_data) = receiver.recv_frame().await? {
        frame_count += 1;

        info!("接收端：收到第 {} 帧 ({} bytes)", frame_count, encoded_data.len());

        // 解码
        match decoder.decode_to_captured_frame(&encoded_data) {
            Ok(decoded_frame) => {
                info!(
                    "接收端：解码成功 {}x{}",
                    decoded_frame.width, decoded_frame.height
                );

                // 保存最后一帧为 PPM
                if frame_count == num_frames {
                    save_ppm(&decoded_frame, "tcp_output.ppm")?;
                    info!("接收端：保存最后一帧到 tcp_output.ppm");
                }
            }
            Err(e) => {
                eprintln!("接收端：解码失败: {:?}", e);
            }
        }
    }

    info!("接收端：连接关闭，共接收 {} 帧", frame_count);

    // 等待发送端完成
    sender_handle.await??;

    info!("========================================");
    info!("✅ TCP 视频传输测试完成");
    info!("   发送端: {} 帧", num_frames);
    info!("   接收端: {} 帧", frame_count);
    info!("   输出文件: tcp_output.ppm");
    info!("========================================");

    Ok(())
}

/// 创建测试帧（渐变图案）
fn create_test_frame(width: u32, height: u32, frame_index: u32) -> CapturedFrame {
    let stride = width * 4; // BGRA
    let mut data = vec![0u8; (stride * height) as usize];

    // 生成渐变图案（随帧索引变化）
    for y in 0..height {
        for x in 0..width {
            let offset = (y * stride + x * 4) as usize;
            let intensity = ((x + y + frame_index * 10) % 256) as u8;

            data[offset] = intensity;         // B
            data[offset + 1] = 255 - intensity; // G
            data[offset + 2] = intensity / 2;   // R
            data[offset + 3] = 255;            // A
        }
    }

    CapturedFrame {
        data,
        width,
        height,
        pixel_format: PixelFormat::Bgra,
        stride,
        display_id: 0,
        timestamp_us: frame_index as u64 * 33_333, // ~30fps
    }
}

/// 保存 CapturedFrame 为 PPM 格式
fn save_ppm(
    frame: &rdcs_platform::CapturedFrame,
    path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path)?;

    // PPM header
    writeln!(file, "P6")?;
    writeln!(file, "{} {}", frame.width, frame.height)?;
    writeln!(file, "255")?;

    // 将 BGRA 转换为 RGB
    for y in 0..frame.height as usize {
        for x in 0..frame.width as usize {
            let offset = y * frame.stride as usize + x * 4;
            if offset + 2 < frame.data.len() {
                let b = frame.data[offset];
                let g = frame.data[offset + 1];
                let r = frame.data[offset + 2];
                file.write_all(&[r, g, b])?;
            }
        }
    }

    Ok(())
}
