//! Example: Test input injection through FFI layer.
//!
//! This example demonstrates the complete input flow:
//! 1. Flutter sends JSON input event via FFI
//! 2. FFI layer parses JSON
//! 3. Platform-specific injector injects the event
//!
//! Run with: cargo run --example input_injection_test

use rdcs_ffi::{
    rdcs_engine_create, rdcs_engine_destroy, rdcs_register_callback, rdcs_send_input,
    EventCallback, EVENT_INPUT_RECEIVED, RDCS_OK,
};
use std::ffi::{c_char, CString};
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

static EVENT_COUNT: AtomicU32 = AtomicU32::new(0);

extern "C" fn input_callback(event_id: u32, payload: *const c_char, payload_len: usize) {
    EVENT_COUNT.fetch_add(1, Ordering::SeqCst);

    let payload_str = if !payload.is_null() && payload_len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(payload as *const u8, payload_len) };
        String::from_utf8_lossy(slice).to_string()
    } else {
        String::from("(empty)")
    };

    println!("📥 Event {}: {}", event_id, payload_str);
}

fn main() {
    println!("🚀 Input Injection Test\n");
    println!("This test demonstrates input event handling through the FFI layer.");
    println!("On macOS, events will be injected to the system (requires accessibility permissions).");
    println!("On other platforms, events will be logged by the mock injector.\n");

    // 1. Create engine
    let config = CString::new("{}").unwrap();
    let handle = rdcs_engine_create(config.as_ptr());
    if handle.is_null() {
        eprintln!("❌ Failed to create engine");
        return;
    }
    println!("✅ Engine created");

    // 2. Register callback for input events
    let rc = rdcs_register_callback(handle, EVENT_INPUT_RECEIVED, input_callback);
    if rc != RDCS_OK {
        eprintln!("❌ Failed to register callback");
        rdcs_engine_destroy(handle);
        return;
    }
    println!("✅ Callback registered\n");

    // 3. Test various input events
    let test_cases = vec![
        (
            "Mouse Move",
            r#"{"type":"mouse","action":"move","x":100.0,"y":200.0}"#,
        ),
        (
            "Mouse Click",
            r#"{"type":"mouse","action":"click","x":150.0,"y":250.0,"button":"left"}"#,
        ),
        (
            "Mouse Right Click",
            r#"{"type":"mouse","action":"click","x":200.0,"y":300.0,"button":"right"}"#,
        ),
        (
            "Double Click",
            r#"{"type":"mouse","action":"double_click","x":300.0,"y":400.0}"#,
        ),
        (
            "Keyboard Press 'A'",
            r#"{"type":"keyboard","key_code":4,"action":"press","shift":false}"#,
        ),
        (
            "Keyboard Release 'A'",
            r#"{"type":"keyboard","key_code":4,"action":"release","shift":false}"#,
        ),
        (
            "Keyboard Shift+A",
            r#"{"type":"keyboard","key_code":4,"action":"press","shift":true}"#,
        ),
        (
            "Scroll Down",
            r#"{"type":"scroll","delta_x":0.0,"delta_y":10.0,"is_precise":true}"#,
        ),
    ];

    for (idx, (name, json)) in test_cases.iter().enumerate() {
        println!("Test {}: {}", idx + 1, name);
        let event_json = CString::new(*json).unwrap();
        let rc = rdcs_send_input(handle, 1, event_json.as_ptr());

        if rc == RDCS_OK {
            println!("  ✅ Injected successfully");
        } else {
            println!("  ❌ Injection failed with code {}", rc);
        }

        // Small delay between events
        thread::sleep(Duration::from_millis(100));
    }

    println!("\n📊 Summary:");
    println!("  Total events sent: {}", test_cases.len());
    println!("  Callbacks received: {}", EVENT_COUNT.load(Ordering::SeqCst));

    // 4. Clean up
    rdcs_engine_destroy(handle);
    println!("\n✅ Engine destroyed");
    println!("\n💡 Note: On macOS, check System Preferences > Security & Privacy > Privacy > Accessibility");
    println!("   to grant permission if events were not injected.");
}
