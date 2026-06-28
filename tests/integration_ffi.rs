// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Integration test: FFI engine lifecycle — create → operate → destroy.
//!
//! Exercises the full FFI surface exposed to Flutter/Dart callers:
//!   rdcs_engine_create → rdcs_register_callback → rdcs_start_capture →
//!   rdcs_connect → rdcs_stop_capture → rdcs_disconnect → rdcs_engine_destroy
//!
//! Validates callback dispatch, error handling, resource cleanup, and
//! the interaction between the FFI layer and the underlying platform/crypto
//! subsystems.

use std::ffi::{c_char, CStr, CString};
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

use rdcs_ffi::{
    rdcs_connect, rdcs_disconnect, rdcs_engine_create, rdcs_engine_destroy,
    rdcs_free_string, rdcs_generate_invite, rdcs_register_callback, rdcs_send_input,
    rdcs_send_message, rdcs_set_quality, rdcs_start_capture, rdcs_stop_capture,
    EngineHandle, EVENT_CONNECTION_ESTABLISHED, EVENT_CONNECTION_LOST, EVENT_FRAME_READY,
    RDCS_OK,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Track callback invocations via atomics (safe for use in extern "C" callbacks).
static CALLBACK_COUNT: AtomicU32 = AtomicU32::new(0);
static LAST_EVENT_ID: AtomicU32 = AtomicU32::new(0);

extern "C" fn test_callback(event_id: u32, _payload: *const c_char, _len: usize) {
    CALLBACK_COUNT.fetch_add(1, Ordering::SeqCst);
    LAST_EVENT_ID.store(event_id, Ordering::SeqCst);
}

extern "C" fn noop_callback(_event_id: u32, _payload: *const c_char, _len: usize) {}

/// Create an engine from a JSON config string. Panics on failure.
fn create_engine() -> (*mut EngineHandle, CString) {
    let config = CString::new("{}").unwrap();
    let handle = rdcs_engine_create(config.as_ptr());
    assert!(!handle.is_null(), "engine creation should succeed");
    (handle, config)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn ffi_engine_lifecycle() {
    CALLBACK_COUNT.store(0, Ordering::SeqCst);
    LAST_EVENT_ID.store(0, Ordering::SeqCst);

    // 1. rdcs_engine_create with config JSON
    let (handle, config) = create_engine();

    // 2. rdcs_register_callback for events
    let rc = rdcs_register_callback(handle, EVENT_FRAME_READY, test_callback);
    assert_eq!(rc, RDCS_OK);

    let rc = rdcs_register_callback(handle, EVENT_CONNECTION_ESTABLISHED, test_callback);
    assert_eq!(rc, RDCS_OK);

    let rc = rdcs_register_callback(handle, EVENT_CONNECTION_LOST, test_callback);
    assert_eq!(rc, RDCS_OK);

    // 3. rdcs_start_capture
    let rc = rdcs_start_capture(handle, config.as_ptr());
    assert_eq!(rc, RDCS_OK, "start_capture should succeed with mock platform");

    // Callback should have fired for EVENT_FRAME_READY
    let count_after_start = CALLBACK_COUNT.load(Ordering::SeqCst);
    assert!(count_after_start >= 1, "should have received at least 1 callback after start_capture");
    assert_eq!(LAST_EVENT_ID.load(Ordering::SeqCst), EVENT_FRAME_READY);

    // 4. rdcs_connect (creates a session)
    let target = CString::new("123456789").unwrap();
    let session_id = rdcs_connect(handle, target.as_ptr());
    assert!(session_id > 0, "connect should return a positive session ID");

    // Callback should have fired for EVENT_CONNECTION_ESTABLISHED
    let count_after_connect = CALLBACK_COUNT.load(Ordering::SeqCst);
    assert!(count_after_connect > count_after_start, "should have received connection callback");
    assert_eq!(LAST_EVENT_ID.load(Ordering::SeqCst), EVENT_CONNECTION_ESTABLISHED);

    // 5. rdcs_disconnect
    let rc = rdcs_disconnect(handle, session_id as u64);
    assert_eq!(rc, RDCS_OK);

    // Callback should have fired for EVENT_CONNECTION_LOST
    let count_after_disconnect = CALLBACK_COUNT.load(Ordering::SeqCst);
    assert!(count_after_disconnect > count_after_connect);
    assert_eq!(LAST_EVENT_ID.load(Ordering::SeqCst), EVENT_CONNECTION_LOST);

    // 6. rdcs_stop_capture
    let rc = rdcs_stop_capture(handle);
    assert_eq!(rc, RDCS_OK);

    // 7. rdcs_engine_destroy
    rdcs_engine_destroy(handle);

    // Total callbacks should be at least 4 (start, connect, disconnect + any extras)
    let total = CALLBACK_COUNT.load(Ordering::SeqCst);
    assert!(total >= 3, "expected at least 3 callbacks, got {total}");
}

#[test]
fn ffi_null_handle_safety() {
    let null: *mut EngineHandle = ptr::null_mut();

    // All operations on null handle should return error codes, not crash
    assert!(rdcs_engine_create(ptr::null()).is_null());
    assert_eq!(rdcs_start_capture(null, ptr::null()), rdcs_ffi::RDCS_ERR_NULL_HANDLE);
    assert_eq!(rdcs_stop_capture(null), rdcs_ffi::RDCS_ERR_NULL_HANDLE);
    assert_eq!(rdcs_connect(null, ptr::null()), rdcs_ffi::RDCS_ERR_NULL_HANDLE);
    assert_eq!(rdcs_disconnect(null, 0), rdcs_ffi::RDCS_ERR_NULL_HANDLE);
    assert_eq!(rdcs_send_input(null, 0, ptr::null()), rdcs_ffi::RDCS_ERR_NULL_HANDLE);
    assert_eq!(rdcs_send_message(null, 0, ptr::null()), rdcs_ffi::RDCS_ERR_NULL_HANDLE);
    assert_eq!(rdcs_set_quality(null, 0, 0), rdcs_ffi::RDCS_ERR_NULL_HANDLE);
    assert!(rdcs_generate_invite(null).is_null());
    assert_eq!(
        rdcs_register_callback(null, 0, noop_callback),
        rdcs_ffi::RDCS_ERR_NULL_HANDLE
    );

    // Destroying null should be a safe no-op
    rdcs_engine_destroy(null);
}

#[test]
fn ffi_send_input_and_message() {
    let (handle, _config) = create_engine();

    // Register a catch-all callback (event_id = 0)
    let rc = rdcs_register_callback(handle, 0, noop_callback);
    assert_eq!(rc, RDCS_OK);

    // Connect to get a session
    let target = CString::new("999888777").unwrap();
    let session_id = rdcs_connect(handle, target.as_ptr());
    assert!(session_id > 0);

    // Send input
    let event_json = CString::new(r#"{"type":"mouse","x":100,"y":200}"#).unwrap();
    let rc = rdcs_send_input(handle, session_id as u64, event_json.as_ptr());
    assert_eq!(rc, RDCS_OK);

    // Send chat message
    let text = CString::new("Hello from integration test").unwrap();
    let rc = rdcs_send_message(handle, session_id as u64, text.as_ptr());
    assert_eq!(rc, RDCS_OK);

    // Set quality mode
    let rc = rdcs_set_quality(handle, session_id as u64, 1);
    assert_eq!(rc, RDCS_OK);

    rdcs_engine_destroy(handle);
}

#[test]
fn ffi_generate_invite_and_free() {
    let (handle, _config) = create_engine();

    let code = rdcs_generate_invite(handle);
    assert!(!code.is_null(), "invite code should not be null");

    let code_str = unsafe { CStr::from_ptr(code) }.to_str().unwrap();
    assert_eq!(code_str.len(), 4, "invite code should be 4 characters");

    rdcs_free_string(code);

    // Freeing null should be safe
    rdcs_free_string(ptr::null_mut());

    rdcs_engine_destroy(handle);
}

#[test]
fn ffi_multiple_callbacks_registered() {
    static CB_A: AtomicU32 = AtomicU32::new(0);
    static CB_B: AtomicU32 = AtomicU32::new(0);

    extern "C" fn callback_a(_event_id: u32, _payload: *const c_char, _len: usize) {
        CB_A.fetch_add(1, Ordering::SeqCst);
    }

    extern "C" fn callback_b(_event_id: u32, _payload: *const c_char, _len: usize) {
        CB_B.fetch_add(1, Ordering::SeqCst);
    }

    CB_A.store(0, Ordering::SeqCst);
    CB_B.store(0, Ordering::SeqCst);

    let (handle, config) = create_engine();

    // Register two callbacks for the same event
    rdcs_register_callback(handle, EVENT_FRAME_READY, callback_a);
    rdcs_register_callback(handle, EVENT_FRAME_READY, callback_b);

    // Start capture triggers EVENT_FRAME_READY
    rdcs_start_capture(handle, config.as_ptr());

    // Both callbacks should have fired
    assert_eq!(CB_A.load(Ordering::SeqCst), 1);
    assert_eq!(CB_B.load(Ordering::SeqCst), 1);

    rdcs_engine_destroy(handle);
}

#[test]
fn ffi_multiple_connect_sessions() {
    let (handle, _config) = create_engine();

    let target1 = CString::new("111222333").unwrap();
    let target2 = CString::new("444555666").unwrap();

    let session1 = rdcs_connect(handle, target1.as_ptr());
    let session2 = rdcs_connect(handle, target2.as_ptr());

    assert!(session1 > 0);
    assert!(session2 > 0);
    assert_ne!(session1, session2, "sessions should have distinct IDs");

    // Disconnect both
    assert_eq!(rdcs_disconnect(handle, session1 as u64), RDCS_OK);
    assert_eq!(rdcs_disconnect(handle, session2 as u64), RDCS_OK);

    rdcs_engine_destroy(handle);
}
