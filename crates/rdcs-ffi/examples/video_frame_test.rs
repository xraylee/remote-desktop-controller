//! Example: Test video frame delivery through FFI layer.
//!
//! This example demonstrates simulated video frame flow:
//! 1. Generate test H.264 frames (or use stub encoder)
//! 2. Pass through FFI video handler
//! 3. Receive decoded frames via EVENT_FRAME_READY callback
//!
//! Run with: cargo run --example video_frame_test

use rdcs_ffi::{
    rdcs_engine_create, rdcs_engine_destroy, rdcs_register_callback, rdcs_start_capture,
    EventCallback, EVENT_FRAME_READY, RDCS_OK,
};
use std::ffi::{c_char, CString};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);

extern "C" fn frame_callback(event_id: u32, payload: *const c_char, payload_len: usize) {
    let count = FRAME_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

    let payload_str = if !payload.is_null() && payload_len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(payload as *const u8, payload_len) };
        String::from_utf8_lossy(slice).to_string()
    } else {
        String::from("(empty)")
    };

    // Parse JSON to extract frame info (don't print full base64 data)
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&payload_str) {
        let width = json["width"].as_u64().unwrap_or(0);
        let height = json["height"].as_u64().unwrap_or(0);
        let format = json["format"].as_str().unwrap_or("unknown");
        let data_len = json["data"].as_str().map(|s| s.len()).unwrap_or(0);

        println!(
            "📥 Frame {}: {}x{} {} ({} bytes base64)",
            count, width, height, format, data_len
        );
    } else {
        println!("📥 Event {}: {}", event_id, payload_str);
    }
}

fn main() {
    println!("🚀 Video Frame Delivery Test\n");
    println!("This test demonstrates video frame flow through the FFI layer.");
    println!("NOTE: This is a stub test - real video requires WebRTC DataChannel.\n");

    // 1. Create engine
    let config = CString::new("{}").unwrap();
    let handle = rdcs_engine_create(config.as_ptr());
    if handle.is_null() {
        eprintln!("❌ Failed to create engine");
        return;
    }
    println!("✅ Engine created");

    // 2. Register callback for frame events
    let rc = rdcs_register_callback(handle, EVENT_FRAME_READY, frame_callback);
    if rc != RDCS_OK {
        eprintln!("❌ Failed to register callback");
        rdcs_engine_destroy(handle);
        return;
    }
    println!("✅ Callback registered\n");

    // 3. Start capture (will use mock capture)
    let capture_config = CString::new(r#"{"fps":30,"width":1280,"height":720}"#).unwrap();
    let rc = rdcs_start_capture(handle, capture_config.as_ptr());
    if rc != RDCS_OK {
        eprintln!("❌ Failed to start capture");
        rdcs_engine_destroy(handle);
        return;
    }
    println!("✅ Capture started");
    println!("⏳ Waiting for frames...\n");

    // 4. Simulate receiving frames
    // In a real scenario, frames would arrive from:
    // - Local capture → encode → send
    // - Remote receive → decode → display
    //
    // For this test, we just wait and show that the infrastructure is ready.
    thread::sleep(Duration::from_secs(3));

    let total_frames = FRAME_COUNT.load(Ordering::SeqCst);

    println!("\n📊 Summary:");
    println!("  Total frames received: {}", total_frames);

    if total_frames == 0 {
        println!("\n💡 Note: No frames received in stub mode.");
        println!("   Real video frames will flow once WebRTC is connected.");
        println!("   Expected flow:");
        println!("     1. Screen capture (rdcs-macos)");
        println!("     2. H.264 encode (VideoToolbox)");
        println!("     3. WebRTC DataChannel send");
        println!("     4. WebRTC DataChannel receive");
        println!("     5. H.264 decode (OpenH264) ← video_handler.rs");
        println!("     6. EVENT_FRAME_READY → Flutter");
        println!("     7. Flutter: Image.memory() render");
    }

    // 5. Clean up
    rdcs_engine_destroy(handle);
    println!("\n✅ Engine destroyed");

    println!("\n🎯 Next Steps:");
    println!("   1. Complete Flutter video renderer widget");
    println!("   2. Wire WebRTC DataChannel frame receiver");
    println!("   3. Connect video_handler to DataChannel");
    println!("   4. Test end-to-end video flow");
}
