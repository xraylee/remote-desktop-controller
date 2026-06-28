// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Encode and decode pipelines that wire together content analysis,
//! adaptive control, encoding, and decoding.

use rdcs_platform::CapturedFrame;

use crate::adaptive::AdaptiveController;
use crate::analyzer::{ContentAnalyzer, SceneInfo};
use crate::decoder::{DecodedFrame, DecoderConfig, VideoDecoder};
use crate::encoder::{EncodedFrame, EncoderConfig, VideoEncoder};
use crate::Result;

// ---------------------------------------------------------------------------
// Pipeline state
// ---------------------------------------------------------------------------

/// State of an encoding or decoding pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineState {
    /// Pipeline has not been started.
    Uninitialized,
    /// Pipeline is actively processing frames.
    Running,
    /// Pipeline is paused (no active session).
    Paused,
    /// Pipeline has been shut down.
    Stopped,
}

// ---------------------------------------------------------------------------
// EncodePipeline
// ---------------------------------------------------------------------------

/// The encoding pipeline processes captured frames through:
/// 1. Content analysis (scene detection)
/// 2. Adaptive controller updates
/// 3. Video encoding
///
/// The pipeline owns the analyzer, encoder, and adaptive controller
/// via trait objects so that different implementations can be swapped in.
pub struct EncodePipeline {
    analyzer: Box<dyn ContentAnalyzer>,
    encoder: Box<dyn VideoEncoder>,
    adaptive: Box<dyn AdaptiveController>,
    state: PipelineState,
    frames_processed: u64,
    last_encoder_config: Option<EncoderConfig>,
}

impl EncodePipeline {
    /// Create a new encoding pipeline from its components.
    pub fn new(
        analyzer: Box<dyn ContentAnalyzer>,
        encoder: Box<dyn VideoEncoder>,
        adaptive: Box<dyn AdaptiveController>,
    ) -> Self {
        Self {
            analyzer,
            encoder,
            adaptive,
            state: PipelineState::Uninitialized,
            frames_processed: 0,
            last_encoder_config: None,
        }
    }

    /// Start the pipeline and configure the encoder with the adaptive
    /// controller's initial configuration.
    pub fn start(&mut self, config: EncoderConfig) -> Result<()> {
        self.encoder.configure(config.clone())?;
        self.last_encoder_config = Some(config);
        self.state = PipelineState::Running;
        self.frames_processed = 0;
        Ok(())
    }

    /// Pause the pipeline — subsequent `process_frame` calls will be rejected.
    pub fn pause(&mut self) {
        self.state = PipelineState::Paused;
    }

    /// Resume a paused pipeline.
    pub fn resume(&mut self) -> Result<()> {
        if self.state != PipelineState::Paused {
            return Err(crate::CodecError::InvalidConfig(
                "pipeline not paused".into(),
            ));
        }
        self.state = PipelineState::Running;
        Ok(())
    }

    /// Shut down the pipeline.
    pub fn stop(&mut self) {
        self.state = PipelineState::Stopped;
    }

    /// Process a single captured frame through the full encoding pipeline:
    /// analyze → adapt → encode.
    ///
    /// Returns the encoded frame on success.
    pub fn process_frame(&mut self, frame: &CapturedFrame) -> Result<EncodedFrame> {
        if self.state != PipelineState::Running {
            return Err(crate::CodecError::InvalidConfig(format!(
                "pipeline not running (state: {:?})",
                self.state
            )));
        }

        // 1. Analyze the frame.
        let scene_info: SceneInfo = self.analyzer.analyze(frame);

        // 2. Update the adaptive controller.
        self.adaptive.on_scene_change(&scene_info);

        // 3. Reconfigure encoder if adaptive controller changed config.
        let new_config = self.adaptive.current_config();
        if Some(&new_config) != self.last_encoder_config.as_ref() {
            tracing::debug!(
                "Adaptive config changed: {}x{} @ {}fps, reconfiguring encoder",
                new_config.width,
                new_config.height,
                new_config.target_fps
            );
            self.encoder.configure(new_config.clone())?;
            self.last_encoder_config = Some(new_config);
        }

        // 4. Encode the frame.
        let encoded = self.encoder.encode(frame)?;
        self.frames_processed += 1;

        Ok(encoded)
    }

    /// Flush any buffered encoded frames.
    pub fn flush(&mut self) -> Result<Vec<EncodedFrame>> {
        self.encoder.flush()
    }

    /// Return the current pipeline state.
    pub fn state(&self) -> PipelineState {
        self.state
    }

    /// Return the total number of frames processed.
    pub fn frames_processed(&self) -> u64 {
        self.frames_processed
    }

    /// Return a reference to the adaptive controller.
    pub fn adaptive(&self) -> &dyn AdaptiveController {
        self.adaptive.as_ref()
    }

    /// Return a mutable reference to the adaptive controller (for
    /// injecting bandwidth/latency updates from the network layer).
    pub fn adaptive_mut(&mut self) -> &mut dyn AdaptiveController {
        self.adaptive.as_mut()
    }
}

impl std::fmt::Debug for EncodePipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EncodePipeline")
            .field("state", &self.state)
            .field("frames_processed", &self.frames_processed)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// DecodePipeline
// ---------------------------------------------------------------------------

/// The decoding pipeline wraps a video decoder and provides a simple
/// interface for decoding received frames.
pub struct DecodePipeline {
    decoder: Box<dyn VideoDecoder>,
    state: PipelineState,
    frames_decoded: u64,
}

impl DecodePipeline {
    /// Create a new decoding pipeline.
    pub fn new(decoder: Box<dyn VideoDecoder>) -> Self {
        Self {
            decoder,
            state: PipelineState::Uninitialized,
            frames_decoded: 0,
        }
    }

    /// Start the pipeline with the given decoder configuration.
    pub fn start(&mut self, config: DecoderConfig) -> Result<()> {
        self.decoder.configure(config)?;
        self.state = PipelineState::Running;
        self.frames_decoded = 0;
        Ok(())
    }

    /// Decode a single encoded frame.
    pub fn decode(&mut self, frame: &EncodedFrame) -> Result<DecodedFrame> {
        if self.state != PipelineState::Running {
            return Err(crate::CodecError::InvalidConfig(format!(
                "pipeline not running (state: {:?})",
                self.state
            )));
        }
        let decoded = self.decoder.decode(frame)?;
        self.frames_decoded += 1;
        Ok(decoded)
    }

    /// Reset the decoder state (e.g. after a stream interruption).
    pub fn reset(&mut self) -> Result<()> {
        self.decoder.reset()
    }

    /// Stop the pipeline.
    pub fn stop(&mut self) {
        self.state = PipelineState::Stopped;
    }

    /// Return the current pipeline state.
    pub fn state(&self) -> PipelineState {
        self.state
    }

    /// Return the total number of frames decoded.
    pub fn frames_decoded(&self) -> u64 {
        self.frames_decoded
    }
}

impl std::fmt::Debug for DecodePipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DecodePipeline")
            .field("state", &self.state)
            .field("frames_decoded", &self.frames_decoded)
            .finish()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adaptive::DefaultAdaptiveController;
    use crate::analyzer::DefaultContentAnalyzer;
    use crate::decoder::StubDecoder;
    use crate::encoder::{CodecType, StubEncoder};
    use rdcs_platform::PixelFormat;

    fn make_frame(w: u32, h: u32, ts: u64) -> CapturedFrame {
        let stride = w * 4;
        CapturedFrame {
            data: vec![128u8; (stride * h) as usize],
            width: w,
            height: h,
            pixel_format: PixelFormat::Bgra,
            stride,
            display_id: 0,
            timestamp_us: ts,
        }
    }

    fn make_text_frame(w: u32, h: u32) -> CapturedFrame {
        let stride = w * 4;
        let mut data = vec![0u8; (stride * h) as usize];
        for y in 0..h as usize {
            let val: u8 = if (y / 2) % 2 == 0 { 0 } else { 220 };
            let row_start = y * stride as usize;
            for x in 0..w as usize {
                let pixel_val = if x % 3 == 0 { val } else { val.wrapping_add(40) };
                let offset = row_start + x * 4;
                data[offset] = pixel_val;
                data[offset + 1] = pixel_val;
                data[offset + 2] = pixel_val;
                data[offset + 3] = 255;
            }
        }
        CapturedFrame {
            data,
            width: w,
            height: h,
            pixel_format: PixelFormat::Bgra,
            stride,
            display_id: 0,
            timestamp_us: 0,
        }
    }

    fn build_encode_pipeline() -> EncodePipeline {
        let base_config = EncoderConfig::default();
        EncodePipeline::new(
            Box::new(DefaultContentAnalyzer::new()),
            Box::new(StubEncoder::new()),
            Box::new(DefaultAdaptiveController::new(&base_config)),
        )
    }

    fn build_decode_pipeline() -> DecodePipeline {
        DecodePipeline::new(Box::new(StubDecoder::new()))
    }

    // -- Pipeline lifecycle --

    #[test]
    fn encode_pipeline_lifecycle() {
        let mut pipeline = build_encode_pipeline();
        assert_eq!(pipeline.state(), PipelineState::Uninitialized);

        pipeline
            .start(EncoderConfig::default())
            .expect("start should succeed");
        assert_eq!(pipeline.state(), PipelineState::Running);

        pipeline.pause();
        assert_eq!(pipeline.state(), PipelineState::Paused);

        pipeline.resume().expect("resume should succeed");
        assert_eq!(pipeline.state(), PipelineState::Running);

        pipeline.stop();
        assert_eq!(pipeline.state(), PipelineState::Stopped);
    }

    #[test]
    fn encode_pipeline_rejects_frames_when_not_running() {
        let mut pipeline = build_encode_pipeline();
        let frame = make_frame(16, 16, 0);
        assert!(pipeline.process_frame(&frame).is_err());
    }

    #[test]
    fn encode_pipeline_resume_requires_paused_state() {
        let mut pipeline = build_encode_pipeline();
        pipeline.start(EncoderConfig::default()).unwrap();
        // Not paused — resume should fail.
        assert!(pipeline.resume().is_err());
    }

    // -- End-to-end pipeline test --

    #[test]
    fn pipeline_e2e_encode_decode_preserves_dimensions() {
        let width = 64u32;
        let height = 48u32;

        // Build and start encode pipeline.
        let mut encode_pipeline = build_encode_pipeline();
        encode_pipeline
            .start(EncoderConfig::default())
            .unwrap();

        // Build and start decode pipeline.
        let mut decode_pipeline = build_decode_pipeline();
        decode_pipeline
            .start(DecoderConfig {
                codec: CodecType::H264,
                width,
                height,
            })
            .unwrap();

        // Process several frames through the full pipeline.
        for i in 0..5 {
            let frame = make_frame(width, height, i * 16_667);
            let encoded = encode_pipeline.process_frame(&frame).unwrap();
            let decoded = decode_pipeline.decode(&encoded).unwrap();

            assert_eq!(decoded.width, width, "width mismatch at frame {i}");
            assert_eq!(decoded.height, height, "height mismatch at frame {i}");
            assert_eq!(
                decoded.pts_us,
                i * 16_667,
                "timestamp mismatch at frame {i}"
            );
        }

        assert_eq!(encode_pipeline.frames_processed(), 5);
        assert_eq!(decode_pipeline.frames_decoded(), 5);
    }

    // -- Scene-based adaptation in pipeline --

    #[test]
    fn pipeline_text_scene_uses_low_fps() {
        let mut pipeline = build_encode_pipeline();
        pipeline.start(EncoderConfig::default()).unwrap();

        // Feed a text-like frame.
        let frame = make_text_frame(64, 64);
        let _encoded = pipeline.process_frame(&frame).unwrap();

        // The adaptive controller should have received a StaticText scene.
        let config = pipeline.adaptive().current_config();
        assert!(
            config.target_fps <= 10,
            "text scene should use low fps, got {}",
            config.target_fps
        );
    }

    // -- Bandwidth adaptation in pipeline --

    #[test]
    fn pipeline_bandwidth_drop_reduces_resolution() {
        let mut pipeline = build_encode_pipeline();
        pipeline.start(EncoderConfig::default()).unwrap();

        // Initial: 1080p.
        assert_eq!(pipeline.adaptive().current_config().width, 1920);

        // Simulate bandwidth drop.
        pipeline.adaptive_mut().on_bandwidth_update(500_000);

        let config = pipeline.adaptive().current_config();
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
    }

    // -- Flush --

    #[test]
    fn pipeline_flush() {
        let mut pipeline = build_encode_pipeline();
        pipeline.start(EncoderConfig::default()).unwrap();
        let flushed = pipeline.flush().unwrap();
        assert!(flushed.is_empty()); // StubEncoder has no buffered frames.
    }

    // -- Decode pipeline state --

    #[test]
    fn decode_pipeline_lifecycle() {
        let mut pipeline = build_decode_pipeline();
        assert_eq!(pipeline.state(), PipelineState::Uninitialized);

        pipeline
            .start(DecoderConfig {
                codec: CodecType::H264,
                width: 1920,
                height: 1080,
            })
            .unwrap();
        assert_eq!(pipeline.state(), PipelineState::Running);

        pipeline.stop();
        assert_eq!(pipeline.state(), PipelineState::Stopped);
    }

    #[test]
    fn decode_pipeline_rejects_frames_when_not_running() {
        let mut pipeline = build_decode_pipeline();
        let encoded = EncodedFrame {
            data: vec![0u8; 32],
            is_keyframe: true,
            pts_us: 0,
            dts_us: 0,
            codec: CodecType::H264,
            width: 10,
            height: 10,
        };
        assert!(pipeline.decode(&encoded).is_err());
    }

    #[test]
    fn decode_pipeline_reset() {
        let mut pipeline = build_decode_pipeline();
        pipeline
            .start(DecoderConfig {
                codec: CodecType::H264,
                width: 1920,
                height: 1080,
            })
            .unwrap();
        assert!(pipeline.reset().is_ok());
    }

    // -- Debug formatting --

    #[test]
    fn encode_pipeline_debug() {
        let pipeline = build_encode_pipeline();
        let dbg = format!("{pipeline:?}");
        assert!(dbg.contains("EncodePipeline"));
    }

    #[test]
    fn decode_pipeline_debug() {
        let pipeline = build_decode_pipeline();
        let dbg = format!("{pipeline:?}");
        assert!(dbg.contains("DecodePipeline"));
    }
}
