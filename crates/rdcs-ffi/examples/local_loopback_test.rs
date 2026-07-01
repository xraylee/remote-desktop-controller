// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Local loopback test for video capture → encode → decode → Flutter display.
//!
//! This example demonstrates the complete video pipeline:
//! 1. Screen capture (macOS ScreenCapture API)
//! 2. Hardware encoding (VideoToolbox H.264)
//! 3. Software decoding (OpenH264)
//! 4. Event dispatch to Flutter (EVENT_FRAME_READY)
//!
//! Run with:
//! ```bash
//! cargo run --package rdcs-ffi --example local_loopback_test
//! ```

use std::ffi::CString;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

// Import FFI functions directly from the library
use rdcs_ffi::{
    rdcs_engine_create, rdcs_engine_destroy, rdcs_register_callback, rdcs_start_capture,
    rdcs_stop_capture, EVENT_FRAME_READY,
};

// Global frame counter for testing
static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);

/// Callback function for receiving frame events.
extern "C" fn frame_callback(event_id: u32, payload: *const i8, payload_len: usize) {
    if event_id == EVENT_FRAME_READY {
        let payload_str = unsafe {
            let slice = std::slice::from_raw_parts(payload as *const u8, payload_len);
            String::from_utf8_lossy(slice)
        };

        // Parse frame info
        if payload_str.contains("width") {
            let count = FRAME_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
            if count % 30 == 0 {
                println!("✅ Received {} frames", count);
                println!("   Frame payload: {}", payload_str);
            }
        } else if payload_str.contains("started") {
            println!("🚀 Video capture started");
        } else if payload_str.contains("error") {
            eprintln!("❌ Error: {}", payload_str);
        }
    }
}

fn main() {
    println!("=== RDCS FFI Local Loopback Test ===\n");

    // Step 1: Create engine
    let config = CString::new("{}").unwrap();
    let engine = unsafe { rdcs_engine_create(config.as_ptr()) };
    if engine.is_null() {
        eprintln!("❌ Failed to create engine");
        return;
    }
    println!("✅ Engine created\n");

    // Step 2: Register callback
    let result = unsafe { rdcs_register_callback(engine, EVENT_FRAME_READY, frame_callback) };
    if result != 0 {
        eprintln!("❌ Failed to register callback: {}", result);
        unsafe { rdcs_engine_destroy(engine) };
        return;
    }
    println!("✅ Frame callback registered\n");

    // Step 3: Start capture
    let capture_config = CString::new(r#"{"fps":30,"width":1920,"height":1080}"#).unwrap();
    let result = unsafe { rdcs_start_capture(engine, capture_config.as_ptr()) };
    if result != 0 {
        eprintln!("❌ Failed to start capture: {}", result);
        unsafe { rdcs_engine_destroy(engine) };
        return;
    }
    println!("✅ Capture started\n");

    // Step 4: Run for 5 seconds
    println!("🎬 Capturing video for 5 seconds...\n");
    for i in 1..=5 {
        thread::sleep(Duration::from_secs(1));
        let frames = FRAME_COUNT.load(Ordering::SeqCst);
        println!("  [{}s] {} frames received (~{} FPS)", i, frames, frames / i);
    }

    // Step 5: Stop capture
    println!("\n🛑 Stopping capture...");
    let result = unsafe { rdcs_stop_capture(engine) };
    if result != 0 {
        eprintln!("❌ Failed to stop capture: {}", result);
    } else {
        println!("✅ Capture stopped");
    }

    thread::sleep(Duration::from_millis(500)); // Allow cleanup

    // Step 6: Destroy engine
    println!("🧹 Cleaning up...");
    unsafe { rdcs_engine_destroy(engine) };
    println!("✅ Engine destroyed\n");

    // Summary
    let total_frames = FRAME_COUNT.load(Ordering::SeqCst);
    println!("=== Test Summary ===");
    println!("Total frames: {}", total_frames);
    println!("Average FPS: ~{}", total_frames / 5);
    println!("Expected: ~30 FPS (150 frames in 5 seconds)");

    if total_frames >= 120 {
        println!("\n✅ SUCCESS: Video pipeline working!");
    } else if total_frames > 0 {
        println!("\n⚠️  PARTIAL: Pipeline working but low FPS");
    } else {
        println!("\n❌ FAILED: No frames received");
    }
}
