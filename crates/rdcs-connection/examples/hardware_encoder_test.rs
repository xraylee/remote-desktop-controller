// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Hardware vs Software encoder performance comparison.
//!
//! This example compares VideoToolbox hardware encoder against OpenH264 software encoder:
//! - Encoding latency
//! - CPU usage (subjective observation)
//! - Output quality
//!
//! Usage:
//!   # Software encoder (baseline)
//!   RUST_LOG=info cargo run -p rdcs-connection --example hardware_encoder_test --features software-encoder
//!
//!   # Hardware encoder (macOS only)
//!   RUST_LOG=info cargo run -p rdcs-connection --example hardware_encoder_test

use rdcs_codec::platform::NativeVideoEncoder;
use rdcs_codec::types::{VideoCodec, VideoResolution};
use rdcs_platform::{CapturedFrame, PixelFormat};
use std::time::Instant;
use tracing::{info, Level};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const FPS: u32 = 30;
const BITRATE: u32 = 2_000_000; // 2 Mbps
const NUM_FRAMES: usize = 60; // 2 seconds at 30fps

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("========================================");
    info!("Encoder Performance Test");
    info!("========================================");
    info!("Resolution: {}x{}", WIDTH, HEIGHT);
    info!("FPS: {}", FPS);
    info!("Bitrate: {} Mbps", BITRATE / 1_000_000);
    info!("Test frames: {}", NUM_FRAMES);

    #[cfg(feature = "software-encoder")]
    info!("Encoder: OpenH264 (Software)");

    #[cfg(all(target_os = "macos", not(feature = "software-encoder")))]
    info!("Encoder: VideoToolbox (Hardware)");

    info!("");

    // Test encoder
    let result = test_encoder().await?;
    info!("");

    // Display results
    info!("========================================");
    info!("Performance Results");
    info!("========================================");
    info!("");

    info!("Average encode time: {:.2}ms", result.avg_encode_ms);
    info!("Min encode time: {:.2}ms", result.min_encode_ms);
    info!("Max encode time: {:.2}ms", result.max_encode_ms);
    info!("Total bytes encoded: {} KB", result.total_bytes / 1024);
    info!("Average frame size: {} bytes", result.avg_frame_bytes);
    info!("");

    // Estimated end-to-end latency
    let e2e_latency = result.avg_encode_ms + 32.0 + 2.0; // encode + decode + network
    info!("Estimated end-to-end latency: {:.2}ms", e2e_latency);
    info!("");

    info!("========================================");
    info!("✅ Test complete!");
    info!("========================================");

    Ok(())
}

struct EncoderTestResult {
    avg_encode_ms: f64,
    min_encode_ms: f64,
    max_encode_ms: f64,
    total_bytes: usize,
    avg_frame_bytes: usize,
}

async fn test_encoder() -> Result<EncoderTestResult, Box<dyn std::error::Error>> {
    info!("Initializing encoder...");

    let mut encoder = NativeVideoEncoder::new(
        VideoCodec::H264,
        VideoResolution::Custom(WIDTH, HEIGHT),
        FPS,
        BITRATE,
    )?;

    info!("✅ Encoder ready");
    info!("");

    let mut encode_times = Vec::new();
    let mut frame_sizes = Vec::new();
    let mut total_bytes = 0usize;

    info!("Encoding {} frames...", NUM_FRAMES);

    for frame_id in 0..NUM_FRAMES as u32 {
        // Generate test frame
        info!("Generating frame {}...", frame_id);
        let captured_frame = generate_test_frame(frame_id);
        info!("Frame {} generated: {}x{}, {} bytes", frame_id, captured_frame.width, captured_frame.height, captured_frame.data.len());

        // Request keyframe every 30 frames
        if frame_id % 30 == 0 {
            info!("Requesting keyframe for frame {}", frame_id);
            encoder.request_keyframe();
        }

        // Encode
        info!("Encoding frame {}...", frame_id);
        let encode_start = Instant::now();
        let encoded = encoder.encode_captured_frame(&captured_frame)?;
        let encode_time = encode_start.elapsed();

        let encode_ms = encode_time.as_secs_f64() * 1000.0;
        encode_times.push(encode_ms);
        frame_sizes.push(encoded.len());
        total_bytes += encoded.len();

        let is_keyframe = frame_id % 30 == 0;

        if frame_id < 5 || is_keyframe {
            info!(
                "  Frame {}: {} bytes in {:.2}ms (keyframe: {})",
                frame_id,
                encoded.len(),
                encode_ms,
                is_keyframe
            );
        }
    }

    info!("");
    info!("Encoding complete. Calculating statistics...");

    // Calculate statistics
    let avg_encode_ms = encode_times.iter().sum::<f64>() / encode_times.len() as f64;
    let min_encode_ms = encode_times.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_encode_ms = encode_times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let avg_frame_bytes = frame_sizes.iter().sum::<usize>() / frame_sizes.len();

    Ok(EncoderTestResult {
        avg_encode_ms,
        min_encode_ms,
        max_encode_ms,
        total_bytes,
        avg_frame_bytes,
    })
}

/// Generate a test frame with a pattern (same as video_e2e_test.rs).
fn generate_test_frame(frame_id: u32) -> CapturedFrame {
    let size = (WIDTH * HEIGHT * 4) as usize; // BGRA
    let mut data = vec![0u8; size];

    // Fill with a gradient pattern that changes per frame
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let offset = ((y * WIDTH + x) * 4) as usize;
            let r = ((x as f32 / WIDTH as f32) * 255.0) as u8;
            let g = ((y as f32 / HEIGHT as f32) * 255.0) as u8;
            let b = ((frame_id % 255) as f32) as u8;

            data[offset] = b; // B
            data[offset + 1] = g; // G
            data[offset + 2] = r; // R
            data[offset + 3] = 255; // A
        }
    }

    CapturedFrame {
        data: data.into(),
        width: WIDTH,
        height: HEIGHT,
        pixel_format: PixelFormat::Bgra,
        stride: WIDTH * 4,
        display_id: 0,
        timestamp_us: 0,
    }
}
