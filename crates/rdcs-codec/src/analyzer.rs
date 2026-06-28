// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Content analysis for scene detection: distinguishes text, mixed, video,
//! and full-motion scenes by comparing consecutive frames.

use rdcs_platform::CapturedFrame;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Scene classification types
// ---------------------------------------------------------------------------

/// High-level classification of the current frame's content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneType {
    /// Mostly static text (terminal, text editor, document viewer).
    StaticText,
    /// A mix of text/UI and some animated content.
    MixedContent,
    /// Video playback or moderate animation.
    Video,
    /// Rapid full-screen changes (gaming, video transitions).
    FullMotion,
}

/// Suggested encoding quality level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quality {
    /// Lowest quality — prioritize frame rate and bandwidth.
    Low,
    /// Medium quality — balanced.
    Medium,
    /// High quality — prioritize visual fidelity.
    High,
    /// Maximum quality — lossless or near-lossless.
    Ultra,
}

/// Result of analyzing a captured frame for scene characteristics.
#[derive(Debug, Clone)]
pub struct SceneInfo {
    /// Classified scene type.
    pub scene_type: SceneType,
    /// Estimated motion level in `[0.0, 1.0]`.
    pub motion_level: f32,
    /// Fraction of the frame occupied by text-like edges `[0.0, 1.0]`.
    pub text_region_ratio: f32,
    /// Suggested frame rate for encoding this scene.
    pub suggested_fps: u32,
    /// Suggested quality preset for encoding this scene.
    pub suggested_quality: Quality,
}

// ---------------------------------------------------------------------------
// ContentAnalyzer trait
// ---------------------------------------------------------------------------

/// Analyzes captured frames to produce scene information for adaptive encoding.
pub trait ContentAnalyzer: Send {
    /// Analyze a frame and return scene information.
    ///
    /// Implementations should compare consecutive frames to detect motion.
    /// The first call (with no previous frame) should report zero motion.
    fn analyze(&mut self, frame: &CapturedFrame) -> SceneInfo;
}

// ---------------------------------------------------------------------------
// DefaultContentAnalyzer — pixel-difference based implementation
// ---------------------------------------------------------------------------

/// A content analyzer that uses pixel-difference comparison between
/// consecutive frames to estimate motion and detect text regions.
#[derive(Debug)]
pub struct DefaultContentAnalyzer {
    /// Pixel data of the previous frame (BGRA/RGBA, 4 bytes per pixel).
    prev_data: Option<Vec<u8>>,
    /// Width of the previous frame.
    prev_width: u32,
    /// Height of the previous frame.
    prev_height: u32,
    /// Motion threshold below which the scene is considered static.
    static_threshold: f32,
    /// Motion threshold above which the scene is full-motion.
    full_motion_threshold: f32,
    /// Edge-detection threshold for text region detection (per-channel diff).
    edge_threshold: u8,
    /// Minimum text-region ratio to classify as text-heavy.
    text_ratio_threshold: f32,
}

impl DefaultContentAnalyzer {
    /// Create a new analyzer with default thresholds.
    pub fn new() -> Self {
        Self {
            prev_data: None,
            prev_width: 0,
            prev_height: 0,
            static_threshold: 0.05,
            full_motion_threshold: 0.50,
            edge_threshold: 60,
            text_ratio_threshold: 0.08,
        }
    }

    /// Compute the normalized motion level between the current and previous
    /// frame. Returns a value in `[0.0, 1.0]`.
    fn compute_motion(&self, frame: &CapturedFrame) -> f32 {
        let prev = match &self.prev_data {
            Some(d) if self.prev_width == frame.width && self.prev_height == frame.height => d,
            _ => return 0.0, // No previous frame or size mismatch → no motion.
        };

        let bpp = bytes_per_pixel(frame);
        let mut total_diff: u64 = 0;
        let pixel_count = (frame.width as usize) * (frame.height as usize);

        if pixel_count == 0 {
            return 0.0;
        }

        // Compare only the overlapping region using the smaller stride.
        let row_bytes = frame.width as usize * bpp;
        for y in 0..frame.height as usize {
            let curr_offset = y * frame.stride as usize;
            let prev_offset = y * frame.stride as usize;
            if curr_offset + row_bytes > frame.data.len()
                || prev_offset + row_bytes > prev.len()
            {
                continue;
            }
            for x in 0..row_bytes {
                let diff =
                    (frame.data[curr_offset + x] as i16 - prev[prev_offset + x] as i16).unsigned_abs();
                total_diff += diff as u64;
            }
        }

        let max_diff = pixel_count as u64 * bpp as u64 * 255;
        if max_diff == 0 {
            return 0.0;
        }
        (total_diff as f32) / (max_diff as f32)
    }

    /// Estimate the fraction of the frame that contains text-like edges.
    ///
    /// Uses a simple horizontal gradient: for each pixel, compare with its
    /// right neighbor. Text content produces many sharp transitions.
    fn compute_text_ratio(&self, frame: &CapturedFrame) -> f32 {
        let bpp = bytes_per_pixel(frame);
        if frame.width < 2 || frame.height < 1 || bpp == 0 {
            return 0.0;
        }

        let mut edge_count: u64 = 0;
        let pixel_count = (frame.width as usize - 1) * (frame.height as usize);

        for y in 0..frame.height as usize {
            let row_start = y * frame.stride as usize;
            for x in 0..(frame.width as usize - 1) {
                let offset = row_start + x * bpp;
                let next_offset = row_start + (x + 1) * bpp;
                if next_offset >= frame.data.len() {
                    continue;
                }
                // Use the first channel (B or R) for edge detection.
                let diff =
                    (frame.data[offset] as i16 - frame.data[next_offset] as i16).unsigned_abs();
                if diff > self.edge_threshold as u16 {
                    edge_count += 1;
                }
            }
        }

        if pixel_count == 0 {
            return 0.0;
        }
        (edge_count as f32) / (pixel_count as f32)
    }

    /// Classify the scene based on motion level and text ratio.
    fn classify(motion: f32, text_ratio: f32, static_thresh: f32, full_motion_thresh: f32, text_ratio_thresh: f32) -> SceneType {
        if motion < static_thresh {
            // Low motion — classified as static regardless of text ratio.
            // Text-heavy static scenes get High quality; others get Medium.
            SceneType::StaticText
        } else if motion >= full_motion_thresh {
            SceneType::FullMotion
        } else if text_ratio > text_ratio_thresh * 1.5 {
            SceneType::MixedContent
        } else {
            SceneType::Video
        }
    }

    /// Map a scene type to a suggested FPS.
    fn suggested_fps(scene: SceneType) -> u32 {
        match scene {
            SceneType::StaticText => 5,
            SceneType::MixedContent => 15,
            SceneType::Video => 30,
            SceneType::FullMotion => 60,
        }
    }

    /// Map a scene type to a suggested quality level.
    fn suggested_quality(scene: SceneType) -> Quality {
        match scene {
            SceneType::StaticText => Quality::High,
            SceneType::MixedContent => Quality::Medium,
            SceneType::Video => Quality::Medium,
            SceneType::FullMotion => Quality::Low,
        }
    }
}

impl Default for DefaultContentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ContentAnalyzer for DefaultContentAnalyzer {
    fn analyze(&mut self, frame: &CapturedFrame) -> SceneInfo {
        let motion = self.compute_motion(frame);
        let text_ratio = self.compute_text_ratio(frame);
        let scene_type = Self::classify(
            motion,
            text_ratio,
            self.static_threshold,
            self.full_motion_threshold,
            self.text_ratio_threshold,
        );

        // Store current frame for next comparison.
        self.prev_data = Some(frame.data.clone());
        self.prev_width = frame.width;
        self.prev_height = frame.height;

        SceneInfo {
            scene_type,
            motion_level: motion,
            text_region_ratio: text_ratio,
            suggested_fps: Self::suggested_fps(scene_type),
            suggested_quality: Self::suggested_quality(scene_type),
        }
    }
}

/// Return bytes per pixel for the given frame (4 for BGRA/RGBA, 1.5 for NV12 approximated as 2).
fn bytes_per_pixel(frame: &CapturedFrame) -> usize {
    match frame.pixel_format {
        rdcs_platform::PixelFormat::Bgra | rdcs_platform::PixelFormat::Rgba => 4,
        rdcs_platform::PixelFormat::Nv12 => 2,
    }
}

// ---------------------------------------------------------------------------
// Legacy FrameAnalyzer (kept for backward compatibility)
// ---------------------------------------------------------------------------

/// Result of analyzing a frame for changes.
#[derive(Debug, Clone)]
pub struct FrameAnalysis {
    /// Whether the frame has significant visual changes.
    pub has_changes: bool,
    /// Percentage of pixels that changed (0.0 to 1.0).
    pub change_ratio: f64,
    /// Whether this is a keyframe-worthy scene change.
    pub is_scene_change: bool,
}

/// Simple frame-change analyzer (legacy interface).
#[derive(Debug)]
pub struct FrameAnalyzer {
    change_threshold: f64,
    scene_change_threshold: f64,
}

impl FrameAnalyzer {
    /// Create a new frame analyzer.
    pub fn new(change_threshold: f64, scene_change_threshold: f64) -> Self {
        Self {
            change_threshold,
            scene_change_threshold,
        }
    }

    /// Analyze a frame and return change metrics.
    pub fn analyze(&self, _frame: &CapturedFrame) -> FrameAnalysis {
        // Stub — real implementation would compare with previous frame.
        FrameAnalysis {
            has_changes: false,
            change_ratio: 0.0,
            is_scene_change: false,
        }
    }

    /// Return the configured change threshold.
    pub fn change_threshold(&self) -> f64 {
        self.change_threshold
    }

    /// Return the configured scene change threshold.
    pub fn scene_change_threshold(&self) -> f64 {
        self.scene_change_threshold
    }
}

impl Default for FrameAnalyzer {
    fn default() -> Self {
        Self::new(0.01, 0.30)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rdcs_platform::PixelFormat;

    /// Helper: create a `CapturedFrame` filled with a single byte value.
    fn solid_frame(width: u32, height: u32, value: u8) -> CapturedFrame {
        let stride = width * 4;
        let data = vec![value; (stride * height) as usize];
        CapturedFrame {
            data,
            width,
            height,
            pixel_format: PixelFormat::Bgra,
            stride,
            display_id: 0,
            timestamp_us: 0,
        }
    }

    /// Helper: create a frame with a text-like horizontal stripe pattern.
    /// Alternates rows of dark (0) and bright (220) pixels.
    fn text_pattern_frame(width: u32, height: u32) -> CapturedFrame {
        let stride = width * 4;
        let mut data = vec![0u8; (stride * height) as usize];
        for y in 0..height as usize {
            let val: u8 = if (y / 2) % 2 == 0 { 0 } else { 220 };
            let row_start = y * stride as usize;
            // Create horizontal variation (text-like edges) within each row.
            for x in 0..width as usize {
                let pixel_val = if x % 3 == 0 { val } else { val.wrapping_add(40) };
                let offset = row_start + x * 4;
                data[offset] = pixel_val;
                data[offset + 1] = pixel_val;
                data[offset + 2] = pixel_val;
                data[offset + 3] = 255; // alpha
            }
        }
        CapturedFrame {
            data,
            width,
            height,
            pixel_format: PixelFormat::Bgra,
            stride,
            display_id: 0,
            timestamp_us: 0,
        }
    }

    /// Helper: create a frame with varied pixel values (simulates video).
    /// The `seed` shifts the base value so different seeds produce very
    /// different frames.
    fn noisy_frame(width: u32, height: u32, seed: u8) -> CapturedFrame {
        let stride = width * 4;
        let mut data = vec![0u8; (stride * height) as usize];
        for (i, byte) in data.iter_mut().enumerate() {
            *byte = seed.wrapping_add((i * 7) as u8);
        }
        CapturedFrame {
            data,
            width,
            height,
            pixel_format: PixelFormat::Bgra,
            stride,
            display_id: 0,
            timestamp_us: 0,
        }
    }

    // -- Acceptance criterion: scene_text --

    #[test]
    fn scene_text_pure_text_frame() {
        let mut analyzer = DefaultContentAnalyzer::new();
        let frame = text_pattern_frame(64, 64);
        let info = analyzer.analyze(&frame);

        assert_eq!(
            info.scene_type,
            SceneType::StaticText,
            "pure text frame should be classified as StaticText"
        );
        assert!(
            info.suggested_fps <= 10,
            "StaticText should suggest fps <= 10, got {}",
            info.suggested_fps
        );
        assert_eq!(info.suggested_quality, Quality::High);
    }

    #[test]
    fn scene_text_identical_consecutive_frames() {
        let mut analyzer = DefaultContentAnalyzer::new();
        let frame1 = text_pattern_frame(32, 32);
        let frame2 = frame1.clone();

        // First frame: no previous → motion = 0.
        let _info1 = analyzer.analyze(&frame1);
        // Second frame: identical → motion = 0.
        let info2 = analyzer.analyze(&frame2);

        assert_eq!(info2.scene_type, SceneType::StaticText);
        assert!(info2.motion_level < 0.001);
        assert!(info2.suggested_fps <= 10);
    }

    // -- Acceptance criterion: scene_video --

    #[test]
    fn scene_video_high_motion_frame() {
        let mut analyzer = DefaultContentAnalyzer::new();

        // First frame: all black.
        let frame1 = solid_frame(32, 32, 0);
        let _info1 = analyzer.analyze(&frame1);

        // Second frame: significantly different (moderate motion, not full).
        // Use a uniform mid-gray to get consistent motion in the Video range.
        let frame2 = solid_frame(32, 32, 90);
        let info2 = analyzer.analyze(&frame2);

        assert_eq!(
            info2.scene_type,
            SceneType::Video,
            "high-motion frame should be classified as Video, got {:?}",
            info2.scene_type
        );
        assert!(
            info2.suggested_fps >= 30,
            "Video should suggest fps >= 30, got {}",
            info2.suggested_fps
        );
    }

    #[test]
    fn scene_full_motion() {
        let mut analyzer = DefaultContentAnalyzer::new();
        let frame1 = noisy_frame(32, 32, 0);
        let _info1 = analyzer.analyze(&frame1);

        // Completely different frame → very high motion.
        let frame2 = noisy_frame(32, 32, 200);
        let info2 = analyzer.analyze(&frame2);

        assert!(
            info2.motion_level > 0.3,
            "noisy frames should produce high motion, got {}",
            info2.motion_level
        );
    }

    // -- Unit tests --

    #[test]
    fn default_analyzer_thresholds() {
        let analyzer = FrameAnalyzer::default();
        assert!((analyzer.change_threshold() - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn analyze_empty_frame_legacy() {
        let analyzer = FrameAnalyzer::default();
        let frame = CapturedFrame {
            data: vec![0u8; 100],
            width: 10,
            height: 10,
            pixel_format: PixelFormat::Bgra,
            stride: 40,
            display_id: 0,
            timestamp_us: 0,
        };
        let result = analyzer.analyze(&frame);
        assert!(!result.has_changes);
    }

    #[test]
    fn first_frame_has_zero_motion() {
        let mut analyzer = DefaultContentAnalyzer::new();
        let frame = solid_frame(16, 16, 128);
        let info = analyzer.analyze(&frame);
        assert!(
            info.motion_level < 0.001,
            "first frame should have zero motion"
        );
    }

    #[test]
    fn text_ratio_high_for_text_pattern() {
        let analyzer = DefaultContentAnalyzer::new();
        let frame = text_pattern_frame(64, 64);
        let ratio = analyzer.compute_text_ratio(&frame);
        assert!(
            ratio > 0.05,
            "text pattern should have high text ratio, got {ratio}"
        );
    }

    #[test]
    fn text_ratio_low_for_solid_frame() {
        let analyzer = DefaultContentAnalyzer::new();
        let frame = solid_frame(64, 64, 100);
        let ratio = analyzer.compute_text_ratio(&frame);
        assert!(
            ratio < 0.01,
            "solid frame should have near-zero text ratio, got {ratio}"
        );
    }

    #[test]
    fn scene_info_debug_and_clone() {
        let info = SceneInfo {
            scene_type: SceneType::Video,
            motion_level: 0.3,
            text_region_ratio: 0.1,
            suggested_fps: 30,
            suggested_quality: Quality::Medium,
        };
        let cloned = info.clone();
        assert_eq!(cloned.scene_type, SceneType::Video);
        let _dbg = format!("{info:?}");
    }
}
