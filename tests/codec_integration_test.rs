// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! End-to-end integration tests for the video codec pipeline.
//!
//! Tests the complete encode → decode roundtrip with performance validation.

use rdcs_codec::{
    encoder::{EncoderConfig, VideoEncoder, CodecType},
    decoder::{VideoDecoder, DecoderConfig},
    webrtc_encoder::WebRtcEncoder,
    webrtc_decoder::WebRtcDecoder,
};
use rdcs_platform::{CapturedFrame, PixelFormat};

// ---------------------------------------------------------------------------
// Test Helpers
// ---------------------------------------------------------------------------

/// Create a test frame with a checkerboard pattern for visual verification.
fn create_test_frame(width: u32, height: u32, timestamp_us: u64) -> CapturedFrame {
    let stride = width * 4;
    let mut data = vec![0u8; (stride * height) as usize];

    // Create checkerboard pattern (8x8 blocks)
    for y in 0..height {
        for x in 0..width {
            let block_x = x / 8;
            let block_y = y / 8;
            let is_white = (block_x + block_y) % 2 == 0;

            let offset = (y * stride + x * 4) as usize;
            if is_white {
                data[offset] = 255;     // B
                data[offset + 1] = 255; // G
                data[offset + 2] = 255; // R
                data[offset + 3] = 255; // A
            } else {
                data[offset] = 0;       // B
                data[offset + 1] = 0;   // G
                data[offset + 2] = 0;   // R
                data[offset + 3] = 255; // A
            }
        }
    }

    CapturedFrame {
        data,
        width,
        height,
        pixel_format: PixelFormat::Bgra,
        stride,
        display_id: 0,
        timestamp_us,
    }
}

// ---------------------------------------------------------------------------
// Basic Roundtrip Tests
// ---------------------------------------------------------------------------

#[test]
fn test_encode_decode_roundtrip_single_frame() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_codec=trace")
        .with_test_writer()
        .try_init()
        .ok();

    let mut encoder = WebRtcEncoder::new();
    let mut decoder = WebRtcDecoder::new();

    // Configure encoder
    let config = EncoderConfig::full_hd();
    encoder.configure(config).expect("Failed to configure encoder");

    // Configure decoder
    decoder.configure(DecoderConfig {
        codec: CodecType::H264,
        width: 1920,
        height: 1080,
    }).expect("Failed to configure decoder");

    // Create test frame
    let frame = create_test_frame(1920, 1080, 0);
    let original_size = frame.data.len();

    // Encode
    let encoded = encoder.encode(&frame).expect("Failed to encode frame");
    assert!(encoded.is_keyframe, "First frame should be keyframe");
    assert!(encoded.data.len() < original_size, "Encoded size should be smaller");

    // Decode
    let decoded = decoder.decode(&encoded).expect("Failed to decode frame");
    assert_eq!(decoded.width, 1920);
    assert_eq!(decoded.height, 1080);
    assert_eq!(decoded.data.len(), original_size);

    println!("✓ Single frame roundtrip successful");
    println!("  Original: {} bytes", original_size);
    println!("  Encoded:  {} bytes", encoded.data.len());
    println!("  Decoded:  {} bytes", decoded.data.len());
    println!("  Compression ratio: {:.1}:1", original_size as f64 / encoded.data.len() as f64);
}

#[test]
fn test_encode_decode_multiple_frames() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_codec=debug")
        .with_test_writer()
        .try_init()
        .ok();

    let mut encoder = WebRtcEncoder::new();
    let mut decoder = WebRtcDecoder::new();

    // Configure
    let config = EncoderConfig {
        codec: CodecType::H264,
        width: 1920,
        height: 1080,
        target_fps: 60,
        target_bitrate_bps: 4_000_000,
        keyframe_interval: 30,
        hardware_accel: false,
    };
    encoder.configure(config).unwrap();
    decoder.configure(DecoderConfig {
        codec: CodecType::H264,
        width: 1920,
        height: 1080,
    }).unwrap();

    // Encode and decode 60 frames (1 second at 60fps)
    let frame_count = 60;
    let mut keyframe_count = 0;

    for i in 0..frame_count {
        let timestamp = i * 16_667; // 60fps = 16.667ms per frame
        let frame = create_test_frame(1920, 1080, timestamp);

        let encoded = encoder.encode(&frame).expect(&format!("Failed to encode frame {}", i));
        if encoded.is_keyframe {
            keyframe_count += 1;
        }

        let decoded = decoder.decode(&encoded).expect(&format!("Failed to decode frame {}", i));
        assert_eq!(decoded.width, 1920);
        assert_eq!(decoded.height, 1080);
    }

    // Verify keyframe interval (should be 2 keyframes: frame 0 and frame 30)
    assert_eq!(keyframe_count, 2, "Expected 2 keyframes with interval of 30");

    println!("✓ Multiple frames test successful");
    println!("  Frames processed: {}", frame_count);
    println!("  Keyframes: {}", keyframe_count);
}

// ---------------------------------------------------------------------------
// Performance Tests
// ---------------------------------------------------------------------------

#[test]
fn test_encoding_performance_1080p_60fps() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_codec=info")
        .with_test_writer()
        .try_init()
        .ok();

    let mut encoder = WebRtcEncoder::new();
    encoder.configure(EncoderConfig::full_hd()).unwrap();

    // Encode 120 frames (2 seconds at 60fps)
    let frame_count = 120;
    let frame = create_test_frame(1920, 1080, 0);

    let start = std::time::Instant::now();
    for _ in 0..frame_count {
        encoder.encode(&frame).expect("Encoding failed");
    }
    let elapsed = start.elapsed();

    let metrics = encoder.metrics();
    let fps = frame_count as f64 / elapsed.as_secs_f64();

    println!("✓ Encoding performance test");
    println!("  Frames encoded: {}", frame_count);
    println!("  Total time: {:?}", elapsed);
    println!("  Average FPS: {:.1}", fps);
    println!("  Average encode time: {} μs", metrics.avg_encode_time_us);
    println!("  Estimated CPU: {:.1}%", metrics.estimated_cpu_percent());
    println!("  Average frame size: {} KB", metrics.avg_frame_size_bytes / 1024);

    // PRD requirement: CPU < 30% for 1080P/60FPS
    if metrics.meets_prd_requirements() {
        println!("  ✓ Meets PRD requirements (encode time < 5ms)");
    } else {
        println!("  ✗ Does NOT meet PRD requirements");
        println!("    Required: < 5ms, Actual: {} μs", metrics.avg_encode_time_us);
    }

    // Performance should achieve at least 60 FPS
    assert!(fps >= 60.0, "Encoding performance too slow: {:.1} fps (need >= 60)", fps);
}

#[test]
fn test_decoding_performance() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_codec=info")
        .with_test_writer()
        .try_init()
        .ok();

    let mut encoder = WebRtcEncoder::new();
    let mut decoder = WebRtcDecoder::new();

    encoder.configure(EncoderConfig::full_hd()).unwrap();
    decoder.configure(DecoderConfig {
        codec: CodecType::H264,
        width: 1920,
        height: 1080,
    }).unwrap();

    // Pre-encode frames
    let frame = create_test_frame(1920, 1080, 0);
    let mut encoded_frames = Vec::new();
    for _ in 0..120 {
        encoded_frames.push(encoder.encode(&frame).unwrap());
    }

    // Measure decode performance
    let start = std::time::Instant::now();
    for encoded in &encoded_frames {
        decoder.decode(encoded).expect("Decoding failed");
    }
    let elapsed = start.elapsed();

    let metrics = decoder.metrics();
    let fps = encoded_frames.len() as f64 / elapsed.as_secs_f64();

    println!("✓ Decoding performance test");
    println!("  Frames decoded: {}", encoded_frames.len());
    println!("  Total time: {:?}", elapsed);
    println!("  Average FPS: {:.1}", fps);
    println!("  Average decode time: {} μs", metrics.avg_decode_time_us);

    if metrics.meets_prd_requirements() {
        println!("  ✓ Meets PRD requirements (decode time < 3ms)");
    } else {
        println!("  ✗ Does NOT meet PRD requirements");
    }

    // Decoding should be faster than encoding
    assert!(fps >= 60.0, "Decoding performance too slow: {:.1} fps", fps);
}

// ---------------------------------------------------------------------------
// Resolution Tests
// ---------------------------------------------------------------------------

#[test]
fn test_various_resolutions() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_codec=debug")
        .with_test_writer()
        .try_init()
        .ok();

    let resolutions = vec![
        (1280, 720, "720p"),
        (1920, 1080, "1080p"),
        (2560, 1440, "1440p"),
        (3840, 2160, "4K"),
    ];

    for (width, height, name) in resolutions {
        println!("Testing {} ({}x{})...", name, width, height);

        let mut encoder = WebRtcEncoder::new();
        let mut decoder = WebRtcDecoder::new();

        let config = EncoderConfig {
            width,
            height,
            ..EncoderConfig::default()
        };

        encoder.configure(config).unwrap();
        decoder.configure(DecoderConfig {
        codec: CodecType::H264,
        width: 1920,
        height: 1080,
    }).unwrap();

        let frame = create_test_frame(width, height, 0);
        let encoded = encoder.encode(&frame).unwrap();
        let decoded = decoder.decode(&encoded).unwrap();

        assert_eq!(decoded.width, width);
        assert_eq!(decoded.height, height);

        println!("  ✓ {} successful", name);
    }
}

// ---------------------------------------------------------------------------
// Error Handling Tests
// ---------------------------------------------------------------------------

#[test]
fn test_decode_without_configure() {
    let mut decoder = WebRtcDecoder::new();
    let frame = create_test_frame(1920, 1080, 0);

    let mut encoder = WebRtcEncoder::new();
    encoder.configure(EncoderConfig::default()).unwrap();
    let encoded = encoder.encode(&frame).unwrap();

    let result = decoder.decode(&encoded);
    assert!(result.is_err());
}

#[test]
fn test_encode_without_configure() {
    let mut encoder = WebRtcEncoder::new();
    let frame = create_test_frame(1920, 1080, 0);

    let result = encoder.encode(&frame);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Stress Tests
// ---------------------------------------------------------------------------

#[test]
#[ignore] // Run with: cargo test --release -- --ignored
fn test_long_session_1080p_60fps() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_codec=info")
        .with_test_writer()
        .try_init()
        .ok();

    let mut encoder = WebRtcEncoder::new();
    let mut decoder = WebRtcDecoder::new();

    encoder.configure(EncoderConfig::full_hd()).unwrap();
    decoder.configure(DecoderConfig {
        codec: CodecType::H264,
        width: 1920,
        height: 1080,
    }).unwrap();

    // Simulate 5 minutes at 60fps = 18,000 frames
    let frame_count = 18_000;
    let frame = create_test_frame(1920, 1080, 0);

    println!("Starting long session test (5 minutes, 60fps)...");
    let start = std::time::Instant::now();

    for i in 0..frame_count {
        let encoded = encoder.encode(&frame).expect("Encoding failed");
        decoder.decode(&encoded).expect("Decoding failed");

        if (i + 1) % 3600 == 0 {
            let elapsed = start.elapsed();
            let progress = (i + 1) as f64 / frame_count as f64 * 100.0;
            println!("  Progress: {:.1}% ({}/{}) - {:?}", progress, i + 1, frame_count, elapsed);
        }
    }

    let elapsed = start.elapsed();
    let encoder_metrics = encoder.metrics();
    let decoder_metrics = decoder.metrics();

    println!("✓ Long session test completed");
    println!("  Duration: {:?}", elapsed);
    println!("  Frames processed: {}", frame_count);
    println!("  Encoder avg: {} μs", encoder_metrics.avg_encode_time_us);
    println!("  Decoder avg: {} μs", decoder_metrics.avg_decode_time_us);
    println!("  Total encoded: {} MB", encoder_metrics.total_encoded_bytes / 1_000_000);
}
