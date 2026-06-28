// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Adaptive quality controller that adjusts encoding parameters
//! based on scene analysis, bandwidth, and latency feedback.

use serde::{Deserialize, Serialize};

use crate::analyzer::{SceneInfo, SceneType};
use crate::encoder::{CodecType, EncoderConfig};

// ---------------------------------------------------------------------------
// Resolution tiers
// ---------------------------------------------------------------------------

/// Predefined resolution tiers indexed from lowest (0) to highest.
const RESOLUTION_TIERS: &[(u32, u32)] = &[
    (1280, 720),   // 720p
    (1600, 900),   // 900p
    (1920, 1080),  // 1080p
];

/// Bandwidth thresholds (bits per second) for each tier.
const TIER_BANDWIDTH_THRESHOLDS: &[u64] = &[
    1_000_000,   // 720p needs ≤ 1 Mbps
    2_000_000,   // 900p needs ≤ 2 Mbps
    4_000_000,   // 1080p needs ≤ 4 Mbps
];

/// Network quality estimate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkQuality {
    /// Excellent: low latency, high bandwidth.
    Excellent,
    /// Good: acceptable for full quality.
    Good,
    /// Fair: reduce quality to maintain frame rate.
    Fair,
    /// Poor: aggressive quality reduction needed.
    Poor,
}

// ---------------------------------------------------------------------------
// AdaptiveController trait
// ---------------------------------------------------------------------------

/// Adaptive controller that adjusts encoding parameters in real time.
pub trait AdaptiveController: Send {
    /// Notify the controller of a scene change.
    fn on_scene_change(&mut self, info: &SceneInfo);

    /// Notify the controller of a new bandwidth estimate.
    fn on_bandwidth_update(&mut self, bitrate_bps: u64);

    /// Notify the controller of a new latency measurement.
    fn on_latency_update(&mut self, rtt_ms: u32);

    /// Return the current recommended encoder configuration.
    fn current_config(&self) -> EncoderConfig;
}

// ---------------------------------------------------------------------------
// DefaultAdaptiveController
// ---------------------------------------------------------------------------

/// The default adaptive controller implementation.
///
/// Resolution strategy:
/// - On bandwidth drop ≤ 500 kbps → immediately downscale to 720p.
/// - On bandwidth recovery → step up one resolution tier per update call.
/// - On high RTT → reduce target FPS and bitrate.
/// - On scene change → adjust quality preset based on scene type.
#[derive(Debug)]
pub struct DefaultAdaptiveController {
    /// Base encoder config (codec, keyframe interval, etc.).
    base_codec: CodecType,
    base_keyframe_interval: u32,
    base_hardware_accel: bool,

    /// Current resolution tier index (0 = lowest, max = highest).
    current_tier: usize,
    /// Maximum tier index (determined by base config resolution).
    max_tier: usize,

    /// Current bandwidth estimate (bps).
    bandwidth_bps: u64,
    /// Current RTT estimate (ms).
    rtt_ms: u32,
    /// Current network quality classification.
    quality: NetworkQuality,
    /// Last observed scene type.
    scene_type: SceneType,
}

impl DefaultAdaptiveController {
    /// Create a new controller with the given base encoder configuration.
    pub fn new(base_config: &EncoderConfig) -> Self {
        let max_tier = Self::tier_for_resolution(base_config.width, base_config.height);

        Self {
            base_codec: base_config.codec,
            base_keyframe_interval: base_config.keyframe_interval,
            base_hardware_accel: base_config.hardware_accel,
            current_tier: max_tier,
            max_tier,
            bandwidth_bps: base_config.target_bitrate_bps,
            rtt_ms: 0,
            quality: NetworkQuality::Excellent,
            scene_type: SceneType::MixedContent,
        }
    }

    /// Find the tier index matching the given resolution, or the closest lower tier.
    fn tier_for_resolution(width: u32, height: u32) -> usize {
        for (i, &(w, h)) in RESOLUTION_TIERS.iter().enumerate() {
            if width <= w && height <= h {
                return i;
            }
        }
        // Resolution exceeds all tiers — use the highest.
        RESOLUTION_TIERS.len() - 1
    }

    /// Determine the best tier for the current bandwidth.
    fn best_tier_for_bandwidth(bandwidth_bps: u64) -> usize {
        // Immediate downgrade: 500 kbps or less → 720p.
        if bandwidth_bps <= 500_000 {
            return 0;
        }
        for (i, &threshold) in TIER_BANDWIDTH_THRESHOLDS.iter().enumerate() {
            if bandwidth_bps <= threshold {
                return i;
            }
        }
        // Bandwidth exceeds all thresholds — use highest tier.
        RESOLUTION_TIERS.len() - 1
    }

    /// Classify network quality from RTT and bandwidth.
    fn classify_quality(rtt_ms: u32, bandwidth_bps: u64) -> NetworkQuality {
        if rtt_ms > 200 || bandwidth_bps < 500_000 {
            NetworkQuality::Poor
        } else if rtt_ms > 100 || bandwidth_bps < 1_000_000 {
            NetworkQuality::Fair
        } else if rtt_ms < 30 && bandwidth_bps > 4_000_000 {
            NetworkQuality::Excellent
        } else {
            NetworkQuality::Good
        }
    }

    /// Compute the target bitrate for the current tier and quality.
    fn compute_bitrate(&self) -> u64 {
        let base = match self.current_tier {
            0 => 800_000u64,     // 720p
            1 => 1_500_000,      // 900p
            _ => 4_000_000,      // 1080p
        };
        // Adjust based on network quality.
        match self.quality {
            NetworkQuality::Excellent => base,
            NetworkQuality::Good => base * 80 / 100,
            NetworkQuality::Fair => base * 60 / 100,
            NetworkQuality::Poor => base * 40 / 100,
        }
    }

    /// Compute the target FPS based on scene type and network conditions.
    fn compute_fps(&self) -> u32 {
        let scene_fps = match self.scene_type {
            SceneType::StaticText => 5,
            SceneType::MixedContent => 15,
            SceneType::Video => 30,
            SceneType::FullMotion => 60,
        };
        // Reduce FPS under poor network conditions.
        match self.quality {
            NetworkQuality::Poor => scene_fps.min(15),
            NetworkQuality::Fair => scene_fps.min(30),
            _ => scene_fps,
        }
    }

    /// Return the current resolution tier index.
    pub fn current_tier(&self) -> usize {
        self.current_tier
    }

    /// Return the current network quality estimate.
    pub fn network_quality(&self) -> NetworkQuality {
        self.quality
    }
}

impl AdaptiveController for DefaultAdaptiveController {
    fn on_scene_change(&mut self, info: &SceneInfo) {
        self.scene_type = info.scene_type;
    }

    fn on_bandwidth_update(&mut self, bitrate_bps: u64) {
        self.bandwidth_bps = bitrate_bps;
        self.quality = Self::classify_quality(self.rtt_ms, bitrate_bps);

        let best_tier = Self::best_tier_for_bandwidth(bitrate_bps);

        if best_tier < self.current_tier {
            // Downgrade immediately to the best tier for this bandwidth.
            self.current_tier = best_tier;
        } else if best_tier > self.current_tier {
            // Recovery: step up one tier per update call.
            self.current_tier = (self.current_tier + 1).min(self.max_tier).min(best_tier);
        }
    }

    fn on_latency_update(&mut self, rtt_ms: u32) {
        self.rtt_ms = rtt_ms;
        self.quality = Self::classify_quality(rtt_ms, self.bandwidth_bps);

        // High latency triggers resolution downgrade.
        if rtt_ms > 200 && self.current_tier > 0 {
            self.current_tier = self.current_tier.saturating_sub(1);
        }
    }

    fn current_config(&self) -> EncoderConfig {
        let (width, height) = RESOLUTION_TIERS[self.current_tier];
        EncoderConfig {
            codec: self.base_codec,
            width,
            height,
            target_fps: self.compute_fps(),
            target_bitrate_bps: self.compute_bitrate(),
            keyframe_interval: self.base_keyframe_interval,
            hardware_accel: self.base_hardware_accel,
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::{Quality as SceneQuality, SceneInfo};

    fn base_config() -> EncoderConfig {
        EncoderConfig {
            codec: CodecType::H264,
            width: 1920,
            height: 1080,
            target_fps: 60,
            target_bitrate_bps: 4_000_000,
            keyframe_interval: 120,
            hardware_accel: false,
        }
    }

    fn make_scene_info(scene_type: SceneType) -> SceneInfo {
        SceneInfo {
            scene_type,
            motion_level: 0.1,
            text_region_ratio: 0.05,
            suggested_fps: 30,
            suggested_quality: SceneQuality::Medium,
        }
    }

    #[test]
    fn initial_config_matches_base() {
        let ctrl = DefaultAdaptiveController::new(&base_config());
        let config = ctrl.current_config();
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
    }

    // -- Acceptance criterion: adaptive_downgrade --

    #[test]
    fn adaptive_downgrade_bandwidth_drops_to_500kbps() {
        let mut ctrl = DefaultAdaptiveController::new(&base_config());

        // Simulate bandwidth dropping to 500 kbps.
        ctrl.on_bandwidth_update(500_000);

        let config = ctrl.current_config();
        assert_eq!(
            config.width, 1280,
            "should downgrade to 720p width, got {}",
            config.width
        );
        assert_eq!(
            config.height, 720,
            "should downgrade to 720p height, got {}",
            config.height
        );
        assert_eq!(ctrl.current_tier(), 0);
    }

    #[test]
    fn adaptive_downgrade_bandwidth_drops_to_1mbps() {
        let mut ctrl = DefaultAdaptiveController::new(&base_config());
        ctrl.on_bandwidth_update(1_000_000);

        let config = ctrl.current_config();
        // 1 Mbps → 720p (tier 0), since threshold for 720p is 1 Mbps.
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
    }

    // -- Acceptance criterion: adaptive_recover --

    #[test]
    fn adaptive_recover_to_1080p_within_3_steps() {
        let mut ctrl = DefaultAdaptiveController::new(&base_config());

        // Drop to 720p.
        ctrl.on_bandwidth_update(500_000);
        assert_eq!(ctrl.current_tier(), 0);
        assert_eq!(ctrl.current_config().width, 1280);

        // Bandwidth recovers to 8 Mbps — step up each call.
        ctrl.on_bandwidth_update(8_000_000);
        let tier_after_step1 = ctrl.current_tier();
        assert!(
            tier_after_step1 > 0,
            "should step up from 720p, tier = {tier_after_step1}"
        );

        ctrl.on_bandwidth_update(8_000_000);
        let tier_after_step2 = ctrl.current_tier();

        ctrl.on_bandwidth_update(8_000_000);
        let tier_after_step3 = ctrl.current_tier();

        // Within 3 steps, should be back at 1080p (tier 2).
        assert!(
            tier_after_step3 >= 2,
            "should reach 1080p within 3 steps, tier = {tier_after_step3}"
        );
        assert_eq!(ctrl.current_config().width, 1920);
        assert_eq!(ctrl.current_config().height, 1080);

        // Verify progression.
        assert!(tier_after_step1 <= tier_after_step2);
        assert!(tier_after_step2 <= tier_after_step3);
    }

    #[test]
    fn adaptive_does_not_exceed_max_tier() {
        let mut ctrl = DefaultAdaptiveController::new(&base_config());
        // Already at max tier.
        ctrl.on_bandwidth_update(100_000_000);
        assert_eq!(ctrl.current_tier(), 2);
        ctrl.on_bandwidth_update(100_000_000);
        assert_eq!(ctrl.current_tier(), 2);
    }

    // -- Scene change tests --

    #[test]
    fn scene_change_updates_scene_type() {
        let mut ctrl = DefaultAdaptiveController::new(&base_config());
        ctrl.on_scene_change(&make_scene_info(SceneType::StaticText));
        let config = ctrl.current_config();
        // Static text → low FPS.
        assert!(config.target_fps <= 10, "got fps {}", config.target_fps);
    }

    #[test]
    fn scene_change_video_increases_fps() {
        let mut ctrl = DefaultAdaptiveController::new(&base_config());
        ctrl.on_scene_change(&make_scene_info(SceneType::FullMotion));
        let config = ctrl.current_config();
        assert!(
            config.target_fps >= 30,
            "FullMotion should have high fps, got {}",
            config.target_fps
        );
    }

    // -- Latency tests --

    #[test]
    fn high_latency_triggers_downgrade() {
        let mut ctrl = DefaultAdaptiveController::new(&base_config());
        assert_eq!(ctrl.current_tier(), 2); // 1080p

        ctrl.on_latency_update(250);
        assert!(
            ctrl.current_tier() < 2,
            "high latency should downgrade"
        );
    }

    #[test]
    fn low_latency_no_downgrade() {
        let mut ctrl = DefaultAdaptiveController::new(&base_config());
        ctrl.on_latency_update(10);
        assert_eq!(ctrl.current_tier(), 2); // Still 1080p.
    }

    // -- Network quality classification --

    #[test]
    fn quality_classification() {
        assert_eq!(
            DefaultAdaptiveController::classify_quality(10, 8_000_000),
            NetworkQuality::Excellent
        );
        assert_eq!(
            DefaultAdaptiveController::classify_quality(50, 2_000_000),
            NetworkQuality::Good
        );
        assert_eq!(
            DefaultAdaptiveController::classify_quality(150, 1_000_000),
            NetworkQuality::Fair
        );
        assert_eq!(
            DefaultAdaptiveController::classify_quality(300, 200_000),
            NetworkQuality::Poor
        );
    }

    // -- Resolution tier lookup --

    #[test]
    fn tier_for_resolution() {
        assert_eq!(DefaultAdaptiveController::tier_for_resolution(1280, 720), 0);
        assert_eq!(DefaultAdaptiveController::tier_for_resolution(1600, 900), 1);
        assert_eq!(DefaultAdaptiveController::tier_for_resolution(1920, 1080), 2);
        assert_eq!(DefaultAdaptiveController::tier_for_resolution(3840, 2160), 2); // capped at max
    }
}
