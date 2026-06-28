// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 端到端会话集成测试。
//!
//! 测试完整的视频会话流程：
//! Encoder → Packetizer → SRTP → Network → SRTP → Depacketizer → Decoder

use rdcs_codec::decoder::{DecoderConfig, VideoDecoder};
use rdcs_codec::encoder::{EncoderConfig, VideoEncoder};
use rdcs_codec::platform::macos::{VideoToolboxDecoder, VideoToolboxEncoder};
use rdcs_codec::rtp::{SrtpConfig, SrtpProfile};
use rdcs_codec::session::{
    PeerConnection, PeerConnectionConfig, QosMonitor, QosMonitorConfig, VideoReceiver,
    VideoReceiverConfig, VideoSender, VideoSenderConfig,
};
use rdcs_codec::types::{PixelFormat, VideoFormat, VideoFrame};
use tokio::time::{sleep, Duration};

/// 创建测试用 SRTP 配置（发送端和接收端使用相同密钥）。
fn create_test_srtp_config() -> SrtpConfig {
    SrtpConfig {
        master_key: vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
            0x0F, 0x10,
        ],
        master_salt: vec![
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        ],
        profile: SrtpProfile::Aead_Aes128Gcm,
    }
}

/// 创建测试帧（640x480 NV12）。
fn create_test_frame(frame_number: u64) -> VideoFrame {
    let width = 640;
    let height = 480;
    let format = VideoFormat {
        width,
        height,
        pixel_format: PixelFormat::Nv12,
        framerate: 30,
    };

    // NV12: Y plane + UV interleaved
    let y_size = (width * height) as usize;
    let uv_size = y_size / 2;
    let mut data = vec![0u8; y_size + uv_size];

    // 填充简单图案（渐变 + 帧号）
    let brightness = ((frame_number % 256) as u8).wrapping_add(128);
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            data[idx] = brightness.wrapping_add((x % 256) as u8);
        }
    }

    VideoFrame {
        format,
        data,
        timestamp_us: frame_number * 33333, // ~30fps
    }
}

#[tokio::test]
async fn test_end_to_end_session_loopback() {
    // 1. 创建发送端
    let mut sender_encoder = VideoToolboxEncoder::new().unwrap();
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
    sender_encoder.configure(&encoder_config).unwrap();

    let sender_pc = PeerConnection::new(PeerConnectionConfig::default());
    let srtp_config = create_test_srtp_config();
    sender_pc.connect(srtp_config.clone()).await.unwrap();
    assert!(sender_pc.is_ready().await);

    let sender_srtp = sender_pc.tx_srtp().await.unwrap();
    let mut video_sender = VideoSender::new(
        Box::new(sender_encoder),
        sender_srtp,
        VideoSenderConfig::default(),
    );

    // 2. 创建接收端
    let mut receiver_decoder = VideoToolboxDecoder::new().unwrap();
    let decoder_config = DecoderConfig {
        format: VideoFormat {
            width: 640,
            height: 480,
            pixel_format: PixelFormat::Nv12,
            framerate: 30,
        },
    };
    receiver_decoder.configure(&decoder_config).unwrap();

    let receiver_pc = PeerConnection::new(PeerConnectionConfig::default());
    receiver_pc.connect(srtp_config).await.unwrap();

    let receiver_srtp = receiver_pc.rx_srtp().await.unwrap();
    let mut video_receiver = VideoReceiver::new(
        Box::new(receiver_decoder),
        receiver_srtp,
        VideoReceiverConfig::default(),
    );

    // 3. 启动接收处理任务
    let mut receiver_handle = video_receiver;
    tokio::spawn(async move {
        receiver_handle.process_packets().await.ok();
    });

    // 4. 发送几帧
    for i in 0..5 {
        let frame = create_test_frame(i);
        video_sender.send_frame(&frame).await.unwrap();

        // 模拟网络传输
        while let Some(srtp_packet) = video_sender.poll_packet().await {
            // 在实际系统中，这里会通过 UDP socket 发送
            // 本测试中直接传递给接收端
            video_receiver.receive_packet(srtp_packet).await.unwrap();
        }

        sleep(Duration::from_millis(33)).await; // ~30fps
    }

    // 5. 验证统计
    let sender_stats = video_sender.stats().await;
    assert_eq!(sender_stats.frames_encoded, 5);
    assert!(sender_stats.packets_sent > 0);

    let receiver_stats = video_receiver.stats().await;
    assert!(receiver_stats.packets_received > 0);

    // 清理
    sender_pc.close().await.unwrap();
    receiver_pc.close().await.unwrap();
}

#[tokio::test]
async fn test_qos_monitoring() {
    let config = QosMonitorConfig {
        rtt_window_size: 5,
        stats_interval_secs: 1,
        enable_adaptive_bitrate: true,
    };
    let mut monitor = QosMonitor::new(config);

    // 模拟 RTT 样本
    monitor.record_rtt(50.0);
    monitor.record_rtt(55.0);
    monitor.record_rtt(60.0);

    assert!(monitor.average_rtt() > 0.0);
    assert!(monitor.average_rtt() < 100.0);

    // 创建测试统计
    let mut stats = rdcs_codec::session::SessionStats::default();
    stats.session_duration_secs = 10;
    stats.tx_packets_sent = 1000;
    stats.rx_packets_received = 950;

    monitor.update_stats(&mut stats);

    assert!(stats.rtt_ms.is_some());
    assert!(stats.packet_loss_rate > 0.0);
}

#[tokio::test]
async fn test_peer_connection_state_machine() {
    let config = PeerConnectionConfig::default();
    let pc = PeerConnection::new(config);

    // 初始状态
    assert_eq!(
        pc.state().await,
        rdcs_codec::session::PeerConnectionState::New
    );

    // 连接
    let srtp_config = create_test_srtp_config();
    pc.connect(srtp_config).await.unwrap();

    assert_eq!(
        pc.state().await,
        rdcs_codec::session::PeerConnectionState::Ready
    );
    assert!(pc.is_ready().await);

    // 关闭
    pc.close().await.unwrap();
    assert_eq!(
        pc.state().await,
        rdcs_codec::session::PeerConnectionState::Closed
    );
}

#[tokio::test]
async fn test_network_quality_assessment() {
    use rdcs_codec::session::{NetworkQuality, SessionStats};

    let mut stats = SessionStats::default();

    // 优秀网络
    stats.rtt_ms = Some(30.0);
    stats.packet_loss_rate = 0.5;
    assert_eq!(stats.assess_network_quality(), NetworkQuality::Excellent);

    // 良好网络
    stats.rtt_ms = Some(100.0);
    stats.packet_loss_rate = 3.0;
    assert_eq!(stats.assess_network_quality(), NetworkQuality::Good);

    // 一般网络
    stats.rtt_ms = Some(200.0);
    stats.packet_loss_rate = 8.0;
    assert_eq!(stats.assess_network_quality(), NetworkQuality::Fair);

    // 差网络
    stats.rtt_ms = Some(500.0);
    stats.packet_loss_rate = 15.0;
    assert_eq!(stats.assess_network_quality(), NetworkQuality::Poor);
}

#[tokio::test]
async fn test_sender_receiver_stats() {
    // 发送端
    let encoder = Box::new(VideoToolboxEncoder::new().unwrap());
    let sender_pc = PeerConnection::new(PeerConnectionConfig::default());
    let srtp_config = create_test_srtp_config();
    sender_pc.connect(srtp_config.clone()).await.unwrap();

    let sender_srtp = sender_pc.tx_srtp().await.unwrap();
    let sender = VideoSender::new(encoder, sender_srtp, VideoSenderConfig::default());

    let sender_stats = sender.stats().await;
    assert_eq!(sender_stats.frames_encoded, 0);

    // 接收端
    let decoder = Box::new(VideoToolboxDecoder::new().unwrap());
    let receiver_pc = PeerConnection::new(PeerConnectionConfig::default());
    receiver_pc.connect(srtp_config).await.unwrap();

    let receiver_srtp = receiver_pc.rx_srtp().await.unwrap();
    let receiver = VideoReceiver::new(decoder, receiver_srtp, VideoReceiverConfig::default());

    let receiver_stats = receiver.stats().await;
    assert_eq!(receiver_stats.packets_received, 0);
}
