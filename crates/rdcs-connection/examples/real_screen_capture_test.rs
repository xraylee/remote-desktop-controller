// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Real screen capture + hardware encoder test.
//!
//! This test captures the actual screen using CGDisplayStream (macOS),
//! encodes with VideoToolbox hardware encoder, and measures performance.
//!
//! Usage:
//!   RUST_LOG=info cargo run -p rdcs-connection --example real_screen_capture_test

use rdcs_codec::platform::NativeVideoEncoder;
use rdcs_codec::types::{VideoCodec, VideoResolution};
use rdcs_macos::MacOsScreenCapture;
use rdcs_platform::{CaptureConfig, ScreenCapture};
use std::time::Instant;
use tracing::{info, Level};

const FPS: u32 = 30;
const BITRATE: u32 = 2_000_000; // 2 Mbps
const TEST_DURATION_SECS: u64 = 3; // Capture for 3 seconds

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("========================================");
    info!("Real Screen Capture + Hardware Encoder Test");
    info!("========================================");
    info!("FPS: {}", FPS);
    info!("Bitrate: {} Mbps", BITRATE / 1_000_000);
    info!("Duration: {} seconds", TEST_DURATION_SECS);
    info!("Encoder: VideoToolbox (Hardware)");
    info!("");

    // Create screen capture
    info!("Initializing screen capture...");
    let capture = MacOsScreenCapture::new();

    let config = CaptureConfig {
        fps: FPS,
        pixel_format: rdcs_platform::PixelFormat::Rgba,
        max_width: None,
        max_height: None,
        capture_cursor: true,
    };

    let rx = capture.start(config)?;
    info!("✅ Screen capture started");
    info!("");

    // Wait for first frame to determine resolution
    info!("Waiting for first frame...");
    let first_frame = rx.recv()?;
    let width = first_frame.width;
    let height = first_frame.height;
    info!("✅ First frame received: {}x{}", width, height);
    info!("");

    // Initialize encoder with actual resolution
    info!("Initializing hardware encoder...");
    let mut encoder = NativeVideoEncoder::new(
        VideoCodec::H264,
        VideoResolution::Custom(width, height),
        FPS,
        BITRATE,
    )?;
    info!("✅ Encoder ready");
    info!("");

    // Encode frames
    info!("Encoding real screen frames...");
    let mut frame_count = 0usize;
    let mut total_encode_time = 0.0f64;
    let mut total_bytes = 0usize;
    let start_time = Instant::now();

    // Encode first frame
    let encode_start = Instant::now();
    encoder.request_keyframe();
    let encoded = encoder.encode_captured_frame(&first_frame)?;
    let encode_time = encode_start.elapsed().as_secs_f64() * 1000.0;

    frame_count += 1;
    total_encode_time += encode_time;
    total_bytes += encoded.len();

    info!("  Frame 0: {} bytes in {:.2}ms (keyframe)", encoded.len(), encode_time);

    // Encode subsequent frames
    while start_time.elapsed().as_secs() < TEST_DURATION_SECS {
        match rx.recv_timeout(std::time::Duration::from_millis(100)) {
            Ok(frame) => {
                let encode_start = Instant::now();

                // Request keyframe every 30 frames
                if frame_count % 30 == 0 {
                    encoder.request_keyframe();
                }

                let encoded = encoder.encode_captured_frame(&frame)?;
                let encode_time = encode_start.elapsed().as_secs_f64() * 1000.0;

                total_encode_time += encode_time;
                total_bytes += encoded.len();

                if frame_count < 5 || frame_count % 30 == 0 {
                    let is_keyframe = frame_count % 30 == 0;
                    info!(
                        "  Frame {}: {} bytes in {:.2}ms{}",
                        frame_count,
                        encoded.len(),
                        encode_time,
                        if is_keyframe { " (keyframe)" } else { "" }
                    );
                }

                frame_count += 1;
            }
            Err(_) => {
                // Timeout, continue
                continue;
            }
        }
    }

    // Stop capture
    capture.stop()?;
    info!("");
    info!("✅ Capture stopped");
    info!("");

    // Print statistics
    info!("========================================");
    info!("Results");
    info!("========================================");
    info!("Frames captured: {}", frame_count);
    info!("Total bytes: {} KB", total_bytes / 1024);

    if frame_count > 0 {
        let avg_encode_ms = total_encode_time / frame_count as f64;
        let actual_fps = frame_count as f64 / start_time.elapsed().as_secs_f64();
        let avg_bitrate = (total_bytes as f64 * 8.0) / start_time.elapsed().as_secs_f64() / 1_000_000.0;

        info!("Average encode time: {:.2}ms", avg_encode_ms);
        info!("Actual FPS: {:.1}", actual_fps);
        info!("Average bitrate: {:.2} Mbps", avg_bitrate);
        info!("");

        // Compare with target
        if avg_encode_ms < 33.0 {
            info!("✅ Encode time meets 30fps requirement");
        } else {
            info!("⚠️  Encode time exceeds 30fps requirement");
        }

        if actual_fps >= 25.0 {
            info!("✅ FPS meets requirement");
        } else {
            info!("⚠️  FPS below target");
        }
    }

    info!("========================================");

    Ok(())
}
