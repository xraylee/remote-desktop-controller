//! rdcs-ffi: C-compatible FFI layer bridging Flutter/Dart to the Rust core engine.
//!
//! All public functions use C ABI and are exposed via `#[no_mangle] extern "C"`.
//! The FFI is handle-based: every operation takes an opaque `EngineHandle` pointer
//! returned by `rdcs_engine_create`. Multiple engines can coexist (though the
//! typical use case is a single instance).
//!
//! # Safety contract
//!
//! All `extern "C"` functions validate their pointer arguments internally
//! (null checks, bounds checks). The `unsafe` keyword is not used on the
//! function signatures because Dart FFI callers cannot express `unsafe`
//! blocks — safety is enforced on the Rust side.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_char, c_int, CStr, CString};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::runtime::Runtime;

// Import engine types for integration.
use rdcs_crypto::CryptoSession;
use rdcs_platform::{
    CaptureConfig, ClipboardProvider, InputInjector, ScreenCapture, SystemNotify,
};

mod input_handler;
mod video_handler;

// ---------------------------------------------------------------------------
// Error codes (returned as negative i32 from extern "C" functions)
// ---------------------------------------------------------------------------

pub const RDCS_OK: c_int = 0;
pub const RDCS_ERR_NULL_HANDLE: c_int = -1;
pub const RDCS_ERR_INVALID_ARG: c_int = -2;
pub const RDCS_ERR_INTERNAL: c_int = -3;
pub const RDCS_ERR_NOT_INITIALIZED: c_int = -4;
pub const RDCS_ERR_ALREADY_EXISTS: c_int = -5;

// ---------------------------------------------------------------------------
// Event IDs (passed to registered callbacks)
// ---------------------------------------------------------------------------

pub const EVENT_CONNECTION_REQUEST: u32 = 1;
pub const EVENT_CONNECTION_ESTABLISHED: u32 = 2;
pub const EVENT_CONNECTION_LOST: u32 = 3;
pub const EVENT_CONNECTION_RESTORED: u32 = 4;
pub const EVENT_FRAME_READY: u32 = 5;
pub const EVENT_INPUT_RECEIVED: u32 = 6;
pub const EVENT_FILE_TRANSFER_PROGRESS: u32 = 7;
pub const EVENT_FILE_TRANSFER_COMPLETE: u32 = 8;
pub const EVENT_CHAT_MESSAGE: u32 = 9;
pub const EVENT_QUALITY_CHANGED: u32 = 10;
pub const EVENT_NEARBY_DEVICE_FOUND: u32 = 11;
pub const EVENT_NEARBY_DEVICE_LOST: u32 = 12;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Bundles all platform-specific trait implementations into a single struct.
///
/// This allows the engine to use screen capture, input injection, system
/// notifications, and clipboard access through a unified interface. Each field
/// holds a trait object so that real platform implementations (macOS, Windows)
/// can be swapped in without changing the FFI layer.
pub struct PlatformBundle {
    /// Screen capture implementation.
    pub capture: Box<dyn ScreenCapture>,
    /// Input injection implementation.
    pub input: Box<dyn InputInjector>,
    /// System notification implementation.
    pub notify: Box<dyn SystemNotify>,
    /// Clipboard provider implementation.
    pub clipboard: Box<dyn ClipboardProvider>,
}

/// Opaque handle to a running RDCS engine instance.
///
/// Created by [`rdcs_engine_create`], destroyed by [`rdcs_engine_destroy`].
/// All other FFI functions require a valid handle.
pub struct EngineHandle {
    /// Tokio runtime for async operations (will be used when core engine is wired).
    _runtime: Runtime,
    /// Placeholder for the actual CoreEngine (to be implemented).
    // engine: Arc<CoreEngine>,
    /// Registered event callbacks.
    callbacks: Arc<Mutex<Vec<CallbackEntry>>>,
    /// Whether the engine has been shut down.
    shutdown: AtomicBool,
    /// Monotonically increasing session counter.
    next_session_id: AtomicU64,
    /// Factory closure that produces a new CryptoSession for each connection.
    crypto_factory: Arc<dyn Fn() -> CryptoSession + Send + Sync>,
    /// Bundle of platform-specific trait implementations.
    platform: Arc<PlatformBundle>,
}

struct CallbackEntry {
    event_id: u32,
    callback: EventCallback,
}

/// Callback function type for receiving events from the engine.
///
/// - `event_id`: one of the `EVENT_*` constants
/// - `payload`: JSON-encoded event data (UTF-8, null-terminated)
/// - `payload_len`: byte length of payload (excluding null terminator)
pub type EventCallback =
    extern "C" fn(event_id: u32, payload: *const c_char, payload_len: usize);

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Safely convert a C string pointer to a Rust String.
fn cstr_to_string(ptr: *const c_char) -> Result<String, c_int> {
    if ptr.is_null() {
        return Err(RDCS_ERR_NULL_HANDLE);
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(|s| s.to_owned())
        .map_err(|_| RDCS_ERR_INVALID_ARG)
}

/// Convert a Rust string to a heap-allocated C string.
/// Caller must free with [`rdcs_free_string`].
fn string_to_cstring(s: &str) -> *mut c_char {
    CString::new(s)
        .unwrap_or_else(|_| CString::new("").unwrap())
        .into_raw()
}

/// Dispatch an event to all registered callbacks matching the event_id.
fn dispatch_event(handle: &EngineHandle, event_id: u32, payload_json: &str) {
    let callbacks = handle.callbacks.lock().unwrap();
    let payload_cstr = CString::new(payload_json).unwrap_or_else(|_| CString::new("").unwrap());
    let payload_ptr = payload_cstr.as_ptr();
    let payload_len = payload_cstr.to_bytes().len();

    for entry in callbacks.iter() {
        if entry.event_id == event_id || entry.event_id == 0 {
            (entry.callback)(event_id, payload_ptr, payload_len);
        }
    }
}

// ---------------------------------------------------------------------------
// FFI Functions (12 functions matching architecture spec Section 2.2)
// ---------------------------------------------------------------------------

/// Create a new engine instance.
///
/// `config_json`: JSON string with engine configuration (server URLs, etc.).
/// Returns an opaque handle, or null on failure.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_engine_create(config_json: *const c_char) -> *mut EngineHandle {
    let _config_str = match cstr_to_string(config_json) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let runtime = match Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return ptr::null_mut(),
    };

    // Platform-specific initialization: use real implementations on macOS,
    // mock implementations elsewhere.
    #[cfg(target_os = "macos")]
    let platform = Arc::new(PlatformBundle {
        capture: Box::new(rdcs_macos::MacOsScreenCapture::new()),
        input: Box::new(rdcs_macos::MacOsInputInjector::new()),
        notify: Box::new(rdcs_macos::MacOsSystemNotify::new()),
        clipboard: Box::new(rdcs_macos::MacOsClipboard::new()),
    });

    #[cfg(not(target_os = "macos"))]
    let platform = Arc::new(PlatformBundle {
        capture: Box::new(rdcs_platform::mock::MockScreenCapture::new()),
        input: Box::new(rdcs_platform::mock::MockInputInjector::new()),
        notify: Box::new(rdcs_platform::mock::MockSystemNotify::new()),
        clipboard: Box::new(rdcs_platform::mock::MockClipboard::new()),
    });

    let session_counter = Arc::new(AtomicU64::new(0));
    let crypto_factory: Arc<dyn Fn() -> CryptoSession + Send + Sync> = {
        let counter = Arc::clone(&session_counter);
        Arc::new(move || {
            let id = counter.fetch_add(1, Ordering::SeqCst);
            CryptoSession::new(id)
        })
    };

    let handle = Box::new(EngineHandle {
        _runtime: runtime,
        callbacks: Arc::new(Mutex::new(Vec::new())),
        shutdown: AtomicBool::new(false),
        next_session_id: AtomicU64::new(1),
        crypto_factory,
        platform,
    });

    Box::into_raw(handle)
}

/// Destroy an engine instance and free all resources.
///
/// After this call, the handle is invalid and must not be used.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_engine_destroy(handle: *mut EngineHandle) {
    if handle.is_null() {
        return;
    }
    let engine = unsafe { Box::from_raw(handle) };
    engine.shutdown.store(true, Ordering::SeqCst);
    // Drop order: shutdown flag set, then runtime drops (cancels tasks),
    // then callbacks dropped.
    drop(engine);
}

/// Start screen capture on the local machine.
///
/// `config_json`: JSON with capture configuration (fps, resolution, etc.).
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_start_capture(
    handle: *mut EngineHandle,
    config_json: *const c_char,
) -> c_int {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return RDCS_ERR_NULL_HANDLE;
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        return RDCS_ERR_NOT_INITIALIZED;
    }

    let _config_str = match cstr_to_string(config_json) {
        Ok(s) => s,
        Err(code) => return code,
    };

    // Build capture config from the JSON string. For now, use defaults —
    // a future iteration will parse fps/resolution fields from config_json.
    let config = CaptureConfig::default();

    match engine.platform.capture.start(config) {
        Ok(_rx) => {
            dispatch_event(engine, EVENT_FRAME_READY, r#"{"status":"started"}"#);
            RDCS_OK
        }
        Err(e) => {
            let msg = format!(r#"{{"error":"{e}"}}"#);
            dispatch_event(engine, EVENT_FRAME_READY, &msg);
            RDCS_ERR_INTERNAL
        }
    }
}

/// Stop the active screen capture session.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_stop_capture(handle: *mut EngineHandle) -> c_int {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return RDCS_ERR_NULL_HANDLE;
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        return RDCS_ERR_NOT_INITIALIZED;
    }

    // Wire to platform ScreenCapture: stop the active capture session.
    match engine.platform.capture.stop() {
        Ok(()) => RDCS_OK,
        Err(e) => {
            let _ = e; // Log in a future iteration when tracing is wired
            RDCS_ERR_INTERNAL
        }
    }
}

/// Initiate a connection to a remote device.
///
/// `target_code`: 9-digit device code of the remote device.
/// Returns a session ID (positive) on success, or a negative error code.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_connect(
    handle: *mut EngineHandle,
    target_code: *const c_char,
) -> c_int {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return RDCS_ERR_NULL_HANDLE;
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        return RDCS_ERR_NOT_INITIALIZED;
    }
    let _target = match cstr_to_string(target_code) {
        Ok(s) => s,
        Err(code) => return code,
    };

    // Create a CryptoSession for this connection. The session will be used
    // for encryption once the full connection pipeline is wired.
    let _crypto_session = (engine.crypto_factory)();

    // TODO: Wire to ConnectionManager
    let session_id = engine.next_session_id.fetch_add(1, Ordering::SeqCst);
    if session_id > i32::MAX as u64 {
        return RDCS_ERR_INTERNAL;
    }
    dispatch_event(
        engine,
        EVENT_CONNECTION_ESTABLISHED,
        &format!(r#"{{"session_id":{session_id}}}"#),
    );

    // Return session ID as positive i32 (safe for reasonable session counts)
    session_id as c_int
}

/// Disconnect an active session.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_disconnect(handle: *mut EngineHandle, session_id: u64) -> c_int {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return RDCS_ERR_NULL_HANDLE;
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        return RDCS_ERR_NOT_INITIALIZED;
    }

    // TODO: Wire to ConnectionManager
    dispatch_event(
        engine,
        EVENT_CONNECTION_LOST,
        &format!(r#"{{"session_id":{session_id}}}"#),
    );
    RDCS_OK
}

/// Send an input event (mouse or keyboard) to the remote session.
///
/// `event_json`: JSON-encoded input event.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_send_input(
    handle: *mut EngineHandle,
    session_id: u64,
    event_json: *const c_char,
) -> c_int {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return RDCS_ERR_NULL_HANDLE;
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        return RDCS_ERR_NOT_INITIALIZED;
    }
    let event_str = match cstr_to_string(event_json) {
        Ok(s) => s,
        Err(code) => return code,
    };

    // For now, inject input locally to the primary display.
    // TODO: route to correct session when ConnectionManager is wired.
    let _ = session_id;

    // Get the primary display ID
    let display_id = match engine.platform.capture.displays() {
        Ok(displays) => displays
            .into_iter()
            .find(|d| d.is_primary)
            .map(|d| d.id)
            .unwrap_or(0),
        Err(_) => 0,
    };

    // Parse and inject the input event
    match input_handler::handle_input_event(
        engine.platform.input.as_ref(),
        &event_str,
        display_id,
    ) {
        Ok(()) => {
            dispatch_event(
                engine,
                EVENT_INPUT_RECEIVED,
                &format!(r#"{{"session_id":{session_id},"status":"injected"}}"#),
            );
            RDCS_OK
        }
        Err(e) => {
            let error_msg = format!(r#"{{"session_id":{session_id},"error":"{e}"}}"#);
            dispatch_event(engine, EVENT_INPUT_RECEIVED, &error_msg);
            RDCS_ERR_INVALID_ARG
        }
    }
}

/// Initiate a file transfer to the remote session.
///
/// `path`: Local file path to send.
/// `dest`: Destination directory on the remote side.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_send_file(
    handle: *mut EngineHandle,
    session_id: u64,
    path: *const c_char,
    dest: *const c_char,
) -> c_int {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return RDCS_ERR_NULL_HANDLE;
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        return RDCS_ERR_NOT_INITIALIZED;
    }
    let _path = match cstr_to_string(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let _dest = match cstr_to_string(dest) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let _ = session_id;

    // TODO: Wire to TransferManager
    RDCS_OK
}

/// Send a chat message to the remote session.
///
/// `text`: UTF-8 text message.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_send_message(
    handle: *mut EngineHandle,
    session_id: u64,
    text: *const c_char,
) -> c_int {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return RDCS_ERR_NULL_HANDLE;
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        return RDCS_ERR_NOT_INITIALIZED;
    }
    let _text = match cstr_to_string(text) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let _ = session_id;

    // TODO: Wire to chat channel
    RDCS_OK
}

/// Set the quality mode for the active session.
///
/// `mode`: 0 = auto, 1 = clarity priority, 2 = fluidity priority.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_set_quality(
    handle: *mut EngineHandle,
    session_id: u64,
    mode: c_int,
) -> c_int {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return RDCS_ERR_NULL_HANDLE;
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        return RDCS_ERR_NOT_INITIALIZED;
    }
    let _ = session_id;
    let _ = mode;

    // TODO: Wire to AdaptiveController
    RDCS_OK
}

/// Generate a new 4-digit invite code.
///
/// Returns a heap-allocated C string with the code. Caller must free with [`rdcs_free_string`].
/// Returns null on error.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_generate_invite(handle: *mut EngineHandle) -> *mut c_char {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return ptr::null_mut();
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        return ptr::null_mut();
    }

    // TODO: Wire to invite code service (generates 4-digit code, stores in signaling)
    let code = "0000"; // Placeholder
    string_to_cstring(code)
}

/// Register a callback for a specific event type.
///
/// `event_id`: one of the `EVENT_*` constants, or 0 to receive all events.
/// `callback`: function pointer called when the event fires.
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_register_callback(
    handle: *mut EngineHandle,
    event_id: u32,
    callback: EventCallback,
) -> c_int {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        return RDCS_ERR_NULL_HANDLE;
    };

    let mut callbacks = engine.callbacks.lock().unwrap();
    callbacks.push(CallbackEntry {
        event_id,
        callback,
    });

    RDCS_OK
}

/// Free a C string previously returned by [`rdcs_generate_invite`].
#[unsafe(no_mangle)]
pub extern "C" fn rdcs_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Helper: create a test engine, run closure, destroy engine.
    fn with_engine<F: FnOnce(&mut EngineHandle)>(f: F) {
        let config = CString::new("{}").unwrap();
        let handle = rdcs_engine_create(config.as_ptr());
        assert!(!handle.is_null());
        let engine = unsafe { handle.as_mut() }.unwrap();
        f(engine);
        rdcs_engine_destroy(handle);
    }

    #[test]
    fn engine_lifecycle() {
        let config = CString::new("{}").unwrap();
        let handle = rdcs_engine_create(config.as_ptr());
        assert!(!handle.is_null());
        rdcs_engine_destroy(handle);
    }

    #[test]
    fn null_handle_returns_error() {
        let null: *mut EngineHandle = ptr::null_mut();
        assert_eq!(rdcs_stop_capture(null), RDCS_ERR_NULL_HANDLE);
        assert_eq!(
            rdcs_connect(null, ptr::null()),
            RDCS_ERR_NULL_HANDLE
        );
        assert_eq!(rdcs_disconnect(null, 0), RDCS_ERR_NULL_HANDLE);
        assert_eq!(
            rdcs_send_input(null, 0, ptr::null()),
            RDCS_ERR_NULL_HANDLE
        );
        assert_eq!(
            rdcs_send_file(null, 0, ptr::null(), ptr::null()),
            RDCS_ERR_NULL_HANDLE
        );
        assert_eq!(
            rdcs_send_message(null, 0, ptr::null()),
            RDCS_ERR_NULL_HANDLE
        );
        assert_eq!(rdcs_set_quality(null, 0, 0), RDCS_ERR_NULL_HANDLE);
        assert!(rdcs_generate_invite(null).is_null());
        assert_eq!(
            rdcs_register_callback(null, 0, noop_callback),
            RDCS_ERR_NULL_HANDLE
        );
    }

    extern "C" fn noop_callback(_event_id: u32, _payload: *const c_char, _len: usize) {}

    #[test]
    fn register_and_dispatch_callback() {
        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);
        CALL_COUNT.store(0, Ordering::SeqCst);

        extern "C" fn counting_callback(
            _event_id: u32,
            _payload: *const c_char,
            _len: usize,
        ) {
            CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        }

        with_engine(|engine| {
            let handle_ptr = engine as *mut EngineHandle;
            let rc = rdcs_register_callback(
                handle_ptr,
                EVENT_CONNECTION_ESTABLISHED,
                counting_callback,
            );
            assert_eq!(rc, RDCS_OK);

            // Trigger a connect which dispatches EVENT_CONNECTION_ESTABLISHED
            let target = CString::new("123456789").unwrap();
            let session_id = rdcs_connect(handle_ptr, target.as_ptr());
            assert!(session_id > 0);
            assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
        });
    }

    #[test]
    fn generate_invite_returns_string() {
        with_engine(|engine| {
            let handle_ptr = engine as *mut EngineHandle;
            let code = rdcs_generate_invite(handle_ptr);
            assert!(!code.is_null());
            let code_str = unsafe { CStr::from_ptr(code) }.to_str().unwrap();
            assert_eq!(code_str.len(), 4);
            rdcs_free_string(code);
        });
    }

    #[test]
    fn free_null_string_is_safe() {
        rdcs_free_string(ptr::null_mut());
    }

    #[test]
    fn platform_bundle_initialized() {
        with_engine(|engine| {
            // Verify capture is operational via the trait method
            assert!(!engine.platform.capture.is_capturing());
            // Verify displays() works through the trait object
            let displays = engine.platform.capture.displays().unwrap();
            assert!(!displays.is_empty());

            // Verify input injector works through the trait object
            let mouse_event = rdcs_platform::MouseEvent {
                action: rdcs_platform::MouseAction::Move,
                x: 0.0,
                y: 0.0,
                display_id: 1,
            };
            assert!(engine.platform.input.inject_mouse(mouse_event).is_ok());

            // Verify system notify works through the trait object
            assert!(engine
                .platform
                .notify
                .show_notification("test", "test")
                .is_ok());

            // Verify clipboard works through the trait object
            assert_eq!(engine.platform.clipboard.get_text().unwrap(), "");
        });
    }

    #[test]
    fn start_and_stop_capture() {
        with_engine(|engine| {
            let handle_ptr = engine as *mut EngineHandle;
            let config = CString::new("{}").unwrap();

            // Start capture should succeed with mock
            assert_eq!(rdcs_start_capture(handle_ptr, config.as_ptr()), RDCS_OK);
            assert!(engine.platform.capture.is_capturing());

            // Stop capture should succeed with mock
            assert_eq!(rdcs_stop_capture(handle_ptr), RDCS_OK);
            assert!(!engine.platform.capture.is_capturing());
        });
    }

    #[test]
    fn crypto_factory_produces_sessions() {
        with_engine(|engine| {
            let session1 = (engine.crypto_factory)();
            let session2 = (engine.crypto_factory)();
            // Sessions should have distinct IDs
            assert_ne!(session1.session_id(), session2.session_id());
            // New sessions are not ready until handshake completes
            assert!(!session1.is_ready());
            assert!(!session2.is_ready());
        });
    }
}
