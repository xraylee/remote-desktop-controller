// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 会话统计信息。
//!
//! 聚合发送端和接收端的统计数据，用于 QoS 监控和自适应码率。

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 会话级别统计信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    /// 会话开始时间。
    #[serde(skip)]
    pub session_start: Option<Instant>,
    /// 会话持续时间（秒）。
    pub session_duration_secs: u64,

    // 发送端统计
    pub tx_frames_encoded: u64,
    pub tx_packets_sent: u64,
    pub tx_bytes_sent: u64,
    pub tx_encoding_errors: u32,
    pub tx_network_errors: u32,

    // 接收端统计
    pub rx_packets_received: u64,
    pub rx_frames_decoded: u64,
    pub rx_bytes_received: u64,
    pub rx_decryption_errors: u32,
    pub rx_decoding_errors: u32,
    pub rx_packets_dropped: u32,

    // 网络质量指标
    pub rtt_ms: Option<f64>,
    pub packet_loss_rate: f64,
    pub jitter_ms: Option<f64>,
    pub estimated_bandwidth_kbps: Option<u32>,
}

impl Default for SessionStats {
    fn default() -> Self {
        Self {
            session_start: Some(Instant::now()),
            session_duration_secs: 0,
            tx_frames_encoded: 0,
            tx_packets_sent: 0,
            tx_bytes_sent: 0,
            tx_encoding_errors: 0,
            tx_network_errors: 0,
            rx_packets_received: 0,
            rx_frames_decoded: 0,
            rx_bytes_received: 0,
            rx_decryption_errors: 0,
            rx_decoding_errors: 0,
            rx_packets_dropped: 0,
            rtt_ms: None,
            packet_loss_rate: 0.0,
            jitter_ms: None,
            estimated_bandwidth_kbps: None,
        }
    }
}

impl SessionStats {
    /// 更新会话持续时间。
    pub fn update_duration(&mut self) {
        if let Some(start) = self.session_start {
            self.session_duration_secs = start.elapsed().as_secs();
        }
    }

    /// 计算发送码率（kbps）。
    pub fn tx_bitrate_kbps(&self) -> f64 {
        if self.session_duration_secs == 0 {
            return 0.0;
        }
        (self.tx_bytes_sent as f64 * 8.0) / (self.session_duration_secs as f64 * 1000.0)
    }

    /// 计算接收码率（kbps）。
    pub fn rx_bitrate_kbps(&self) -> f64 {
        if self.session_duration_secs == 0 {
            return 0.0;
        }
        (self.rx_bytes_received as f64 * 8.0) / (self.session_duration_secs as f64 * 1000.0)
    }

    /// 计算发送帧率（fps）。
    pub fn tx_framerate(&self) -> f64 {
        if self.session_duration_secs == 0 {
            return 0.0;
        }
        self.tx_frames_encoded as f64 / self.session_duration_secs as f64
    }

    /// 计算接收帧率（fps）。
    pub fn rx_framerate(&self) -> f64 {
        if self.session_duration_secs == 0 {
            return 0.0;
        }
        self.rx_frames_decoded as f64 / self.session_duration_secs as f64
    }

    /// 计算实际丢包率（基于统计）。
    pub fn calculate_packet_loss(&mut self) {
        if self.tx_packets_sent == 0 {
            self.packet_loss_rate = 0.0;
            return;
        }

        // 丢包 = 发送 - 接收 - 主动丢弃
        let expected = self.tx_packets_sent;
        let received = self.rx_packets_received;
        let dropped = self.rx_packets_dropped as u64;

        if received + dropped > expected {
            // 接收端统计异常，保持现有值
            return;
        }

        let lost = expected.saturating_sub(received + dropped);
        self.packet_loss_rate = (lost as f64 / expected as f64) * 100.0;
    }

    /// 生成人类可读的统计摘要。
    pub fn summary(&self) -> String {
        format!(
            "Session Stats [{}s]:\n\
             TX: {:.2} fps, {:.0} kbps, {} packets, {} errors\n\
             RX: {:.2} fps, {:.0} kbps, {} packets, {} errors\n\
             Network: {:.2}% loss, RTT: {:?} ms, Jitter: {:?} ms",
            self.session_duration_secs,
            self.tx_framerate(),
            self.tx_bitrate_kbps(),
            self.tx_packets_sent,
            self.tx_encoding_errors + self.tx_network_errors,
            self.rx_framerate(),
            self.rx_bitrate_kbps(),
            self.rx_packets_received,
            self.rx_decryption_errors + self.rx_decoding_errors,
            self.packet_loss_rate,
            self.rtt_ms,
            self.jitter_ms,
        )
    }
}

/// 网络质量评估。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkQuality {
    /// 优秀（低延迟，无丢包）。
    Excellent,
    /// 良好（轻微延迟/丢包）。
    Good,
    /// 一般（明显延迟/丢包）。
    Fair,
    /// 差（严重网络问题）。
    Poor,
    /// 未知（数据不足）。
    Unknown,
}

impl SessionStats {
    /// 评估当前网络质量。
    pub fn assess_network_quality(&self) -> NetworkQuality {
        let rtt = self.rtt_ms.unwrap_or(f64::MAX);
        let loss = self.packet_loss_rate;

        if rtt < 50.0 && loss < 1.0 {
            NetworkQuality::Excellent
        } else if rtt < 150.0 && loss < 5.0 {
            NetworkQuality::Good
        } else if rtt < 300.0 && loss < 10.0 {
            NetworkQuality::Fair
        } else if rtt < f64::MAX {
            NetworkQuality::Poor
        } else {
            NetworkQuality::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_default_stats() {
        let stats = SessionStats::default();
        assert_eq!(stats.tx_frames_encoded, 0);
        assert_eq!(stats.rx_packets_received, 0);
        assert!(stats.session_start.is_some());
    }

    #[test]
    fn test_duration_update() {
        let mut stats = SessionStats::default();
        thread::sleep(Duration::from_millis(100));
        stats.update_duration();
        assert!(stats.session_duration_secs >= 0);
    }

    #[test]
    fn test_bitrate_calculation() {
        let mut stats = SessionStats::default();
        stats.session_duration_secs = 10;
        stats.tx_bytes_sent = 1_000_000; // 1 MB

        let bitrate = stats.tx_bitrate_kbps();
        assert!((bitrate - 800.0).abs() < 1.0); // 1MB * 8 / 10s = 800 kbps
    }

    #[test]
    fn test_framerate_calculation() {
        let mut stats = SessionStats::default();
        stats.session_duration_secs = 5;
        stats.tx_frames_encoded = 150;

        let fps = stats.tx_framerate();
        assert_eq!(fps, 30.0); // 150 frames / 5s = 30 fps
    }

    #[test]
    fn test_packet_loss_calculation() {
        let mut stats = SessionStats::default();
        stats.tx_packets_sent = 1000;
        stats.rx_packets_received = 950;
        stats.rx_packets_dropped = 10;

        stats.calculate_packet_loss();
        // Lost: 1000 - 950 - 10 = 40
        // Rate: 40 / 1000 = 4%
        assert!((stats.packet_loss_rate - 4.0).abs() < 0.1);
    }

    #[test]
    fn test_zero_duration_bitrate() {
        let stats = SessionStats::default();
        // session_duration_secs = 0
        assert_eq!(stats.tx_bitrate_kbps(), 0.0);
        assert_eq!(stats.rx_bitrate_kbps(), 0.0);
    }

    #[test]
    fn test_network_quality_assessment() {
        let mut stats = SessionStats::default();

        // Excellent
        stats.rtt_ms = Some(30.0);
        stats.packet_loss_rate = 0.5;
        assert_eq!(stats.assess_network_quality(), NetworkQuality::Excellent);

        // Good
        stats.rtt_ms = Some(100.0);
        stats.packet_loss_rate = 3.0;
        assert_eq!(stats.assess_network_quality(), NetworkQuality::Good);

        // Fair
        stats.rtt_ms = Some(200.0);
        stats.packet_loss_rate = 8.0;
        assert_eq!(stats.assess_network_quality(), NetworkQuality::Fair);

        // Poor
        stats.rtt_ms = Some(500.0);
        stats.packet_loss_rate = 15.0;
        assert_eq!(stats.assess_network_quality(), NetworkQuality::Poor);

        // Unknown
        stats.rtt_ms = None;
        assert_eq!(stats.assess_network_quality(), NetworkQuality::Unknown);
    }

    #[test]
    fn test_summary_format() {
        let mut stats = SessionStats::default();
        stats.session_duration_secs = 10;
        stats.tx_frames_encoded = 300;
        stats.tx_packets_sent = 1000;
        stats.rx_packets_received = 950;

        let summary = stats.summary();
        assert!(summary.contains("10s"));
        assert!(summary.contains("30.00 fps")); // 300/10
    }
}
