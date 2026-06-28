// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! QoS (Quality of Service) 监控和自适应控制。
//!
//! 监控网络质量指标（RTT、丢包率、抖动），并与 adaptive 模块集成
//! 实现动态码率调整。

use super::{SessionStats, NetworkQuality};
use crate::adaptive::{QualityController, QualityParameters};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// QoS 监控器配置。
#[derive(Debug, Clone)]
pub struct QosMonitorConfig {
    /// RTT 采样窗口大小。
    pub rtt_window_size: usize,
    /// 统计更新间隔（秒）。
    pub stats_interval_secs: u64,
    /// 是否启用自适应码率。
    pub enable_adaptive_bitrate: bool,
}

impl Default for QosMonitorConfig {
    fn default() -> Self {
        Self {
            rtt_window_size: 10,
            stats_interval_secs: 1,
            enable_adaptive_bitrate: true,
        }
    }
}

/// QoS 监控器。
pub struct QosMonitor {
    config: QosMonitorConfig,
    /// RTT 样本窗口（毫秒）。
    rtt_samples: VecDeque<f64>,
    /// 包到达间隔样本（用于计算抖动）。
    arrival_intervals: VecDeque<Duration>,
    /// 上次统计更新时间。
    last_stats_update: Instant,
    /// 质量控制器（可选）。
    quality_controller: Option<QualityController>,
}

impl QosMonitor {
    /// 创建新的 QoS 监控器。
    pub fn new(config: QosMonitorConfig) -> Self {
        let quality_controller = if config.enable_adaptive_bitrate {
            Some(QualityController::new())
        } else {
            None
        };

        Self {
            config,
            rtt_samples: VecDeque::with_capacity(config.rtt_window_size),
            arrival_intervals: VecDeque::with_capacity(config.rtt_window_size),
            last_stats_update: Instant::now(),
            quality_controller,
        }
    }

    /// 记录 RTT 样本（毫秒）。
    pub fn record_rtt(&mut self, rtt_ms: f64) {
        if self.rtt_samples.len() >= self.config.rtt_window_size {
            self.rtt_samples.pop_front();
        }
        self.rtt_samples.push_back(rtt_ms);

        debug!("RTT sample: {:.2} ms (avg: {:.2} ms)", rtt_ms, self.average_rtt());
    }

    /// 记录包到达时间（用于计算抖动）。
    pub fn record_packet_arrival(&mut self, arrival_time: Instant) {
        if let Some(last_arrival) = self.arrival_intervals.back() {
            let interval = arrival_time.duration_since(self.last_stats_update);
            if self.arrival_intervals.len() >= self.config.rtt_window_size {
                self.arrival_intervals.pop_front();
            }
            self.arrival_intervals.push_back(interval);
        }
    }

    /// 获取平均 RTT（毫秒）。
    pub fn average_rtt(&self) -> f64 {
        if self.rtt_samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.rtt_samples.iter().sum();
        sum / self.rtt_samples.len() as f64
    }

    /// 计算抖动（毫秒）。
    pub fn calculate_jitter(&self) -> f64 {
        if self.arrival_intervals.len() < 2 {
            return 0.0;
        }

        // 抖动 = 包到达间隔的标准差
        let intervals: Vec<f64> = self
            .arrival_intervals
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .collect();

        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
        let variance = intervals
            .iter()
            .map(|v| {
                let diff = v - mean;
                diff * diff
            })
            .sum::<f64>()
            / intervals.len() as f64;

        variance.sqrt()
    }

    /// 更新会话统计信息。
    pub fn update_stats(&mut self, stats: &mut SessionStats) {
        let now = Instant::now();
        if now.duration_since(self.last_stats_update).as_secs() < self.config.stats_interval_secs {
            return;
        }

        // 更新 RTT
        if !self.rtt_samples.is_empty() {
            stats.rtt_ms = Some(self.average_rtt());
        }

        // 更新抖动
        let jitter = self.calculate_jitter();
        if jitter > 0.0 {
            stats.jitter_ms = Some(jitter);
        }

        // 更新丢包率
        stats.calculate_packet_loss();

        // 评估网络质量
        let quality = stats.assess_network_quality();
        debug!("Network quality: {:?}", quality);

        // 触发自适应控制
        if let Some(ref mut controller) = self.quality_controller {
            if let Some(adjustment) = self.suggest_quality_adjustment(stats, quality) {
                info!("Quality adjustment suggested: {:?}", adjustment);
                // 应用调整（实际应用需要编码器支持）
                controller.apply_parameters(&adjustment);
            }
        }

        self.last_stats_update = now;
    }

    /// 根据网络质量建议码率调整。
    fn suggest_quality_adjustment(
        &self,
        stats: &SessionStats,
        quality: NetworkQuality,
    ) -> Option<QualityParameters> {
        match quality {
            NetworkQuality::Excellent => {
                // 网络良好，可以提高质量
                if stats.tx_bitrate_kbps() < 5000.0 {
                    Some(QualityParameters {
                        bitrate: (stats.tx_bitrate_kbps() * 1.2) as u32 * 1000, // +20%
                        framerate: 30,
                        resolution_scale: 1.0,
                    })
                } else {
                    None
                }
            }
            NetworkQuality::Good => {
                // 保持当前质量
                None
            }
            NetworkQuality::Fair => {
                // 轻微降低质量
                Some(QualityParameters {
                    bitrate: (stats.tx_bitrate_kbps() * 0.8) as u32 * 1000, // -20%
                    framerate: 30,
                    resolution_scale: 0.9,
                })
            }
            NetworkQuality::Poor => {
                // 显著降低质量
                Some(QualityParameters {
                    bitrate: (stats.tx_bitrate_kbps() * 0.5) as u32 * 1000, // -50%
                    framerate: 15,
                    resolution_scale: 0.75,
                })
            }
            NetworkQuality::Unknown => None,
        }
    }

    /// 估算可用带宽（基于发送码率和丢包率）。
    pub fn estimate_bandwidth(&self, stats: &SessionStats) -> Option<u32> {
        if stats.packet_loss_rate > 50.0 {
            // 丢包率过高，无法估算
            return None;
        }

        // 简单估算：当前码率 / (1 - 丢包率)
        let loss_factor = 1.0 - (stats.packet_loss_rate / 100.0);
        if loss_factor > 0.0 {
            Some((stats.tx_bitrate_kbps() / loss_factor) as u32)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qos_monitor_creation() {
        let config = QosMonitorConfig::default();
        let monitor = QosMonitor::new(config);

        assert_eq!(monitor.average_rtt(), 0.0);
        assert!(monitor.quality_controller.is_some());
    }

    #[test]
    fn test_rtt_recording() {
        let config = QosMonitorConfig {
            rtt_window_size: 3,
            ..Default::default()
        };
        let mut monitor = QosMonitor::new(config);

        monitor.record_rtt(50.0);
        monitor.record_rtt(60.0);
        monitor.record_rtt(70.0);

        assert_eq!(monitor.average_rtt(), 60.0);

        // 超过窗口大小，应该移除最旧的
        monitor.record_rtt(80.0);
        assert_eq!(monitor.rtt_samples.len(), 3);
        assert_eq!(monitor.average_rtt(), 70.0); // (60 + 70 + 80) / 3
    }

    #[test]
    fn test_jitter_calculation() {
        let config = QosMonitorConfig::default();
        let mut monitor = QosMonitor::new(config);

        // 添加一些间隔样本
        let base = Instant::now();
        monitor.arrival_intervals.push_back(Duration::from_millis(20));
        monitor.arrival_intervals.push_back(Duration::from_millis(25));
        monitor.arrival_intervals.push_back(Duration::from_millis(18));

        let jitter = monitor.calculate_jitter();
        assert!(jitter > 0.0);
        assert!(jitter < 10.0); // 应该是几毫秒的量级
    }

    #[test]
    fn test_bandwidth_estimation() {
        let config = QosMonitorConfig::default();
        let monitor = QosMonitor::new(config);

        let mut stats = SessionStats::default();
        stats.session_duration_secs = 10;
        stats.tx_bytes_sent = 1_000_000; // 1 MB
        stats.packet_loss_rate = 5.0; // 5% 丢包

        let bandwidth = monitor.estimate_bandwidth(&stats);
        assert!(bandwidth.is_some());

        // 800 kbps / 0.95 ≈ 842 kbps
        let bw = bandwidth.unwrap();
        assert!(bw > 800 && bw < 900);
    }

    #[test]
    fn test_quality_adjustment_excellent() {
        let config = QosMonitorConfig::default();
        let monitor = QosMonitor::new(config);

        let mut stats = SessionStats::default();
        stats.session_duration_secs = 10;
        stats.tx_bytes_sent = 3_000_000; // 3 MB → 2400 kbps
        stats.rtt_ms = Some(30.0);
        stats.packet_loss_rate = 0.5;

        let adjustment = monitor.suggest_quality_adjustment(&stats, NetworkQuality::Excellent);
        assert!(adjustment.is_some());

        let params = adjustment.unwrap();
        assert!(params.bitrate > 2_400_000); // 应该增加
    }

    #[test]
    fn test_quality_adjustment_poor() {
        let config = QosMonitorConfig::default();
        let monitor = QosMonitor::new(config);

        let mut stats = SessionStats::default();
        stats.session_duration_secs = 10;
        stats.tx_bytes_sent = 5_000_000; // 5 MB → 4000 kbps
        stats.rtt_ms = Some(500.0);
        stats.packet_loss_rate = 15.0;

        let adjustment = monitor.suggest_quality_adjustment(&stats, NetworkQuality::Poor);
        assert!(adjustment.is_some());

        let params = adjustment.unwrap();
        assert!(params.bitrate < 4_000_000); // 应该降低
        assert_eq!(params.framerate, 15); // 降低帧率
    }

    #[test]
    fn test_no_adjustment_for_good() {
        let config = QosMonitorConfig::default();
        let monitor = QosMonitor::new(config);

        let stats = SessionStats::default();
        let adjustment = monitor.suggest_quality_adjustment(&stats, NetworkQuality::Good);
        assert!(adjustment.is_none());
    }
}
