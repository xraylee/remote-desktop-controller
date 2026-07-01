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
use tokio::sync::mpsc;

// Import engine types for integration.
use rdcs_codec::platform::NativeVideoEncoder;
use rdcs_codec::types::{VideoCodec, VideoResolution};
use rdcs_crypto::CryptoSession;
use rdcs_platform::{
    CaptureConfig, CapturedFrame, ClipboardProvider, InputInjector, PixelFormat, ScreenCapture,
    SystemNotify,
};

#[cfg(target_os = "macos")]
use rdcs_macos::scaling;

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
    runtime: Runtime,
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
    /// Video frame handler for decoding and dispatching frames.
    video_handler: Arc<video_handler::VideoFrameHandler>,
    /// Shutdown signal for background tasks.
    shutdown_tx: Arc<Mutex<Option<mpsc::Sender<()>>>>,
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
        runtime,
        callbacks: Arc::new(Mutex::new(Vec::new())),
        shutdown: AtomicBool::new(false),
        next_session_id: AtomicU64::new(1),
        crypto_factory,
        platform,
        video_handler: Arc::new(video_handler::VideoFrameHandler::new()),
        shutdown_tx: Arc::new(Mutex::new(None)),
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

    // Signal video loop to stop
    if let Some(tx) = engine.shutdown_tx.lock().unwrap().take() {
        let _ = tx.try_send(());
    }

    // Drop order: shutdown flag set, signal sent, then runtime drops (cancels tasks),
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

    // Start screen capture
    let frame_rx = match engine.platform.capture.start(config) {
        Ok(rx) => rx,
        Err(e) => {
            let msg = format!(r#"{{"error":"{}"}}"#, e);
            dispatch_event(engine, EVENT_FRAME_READY, &msg);
            return RDCS_ERR_INTERNAL;
        }
    };

    // Create shutdown channel
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    *engine.shutdown_tx.lock().unwrap() = Some(shutdown_tx);

    // Create async channel to bridge sync receiver to async task
    let (async_tx, mut async_rx) = mpsc::channel::<CapturedFrame>(10);

    // Get the actual screen resolution from the first frame
    let first_frame = match frame_rx.recv() {
        Ok(frame) => frame,
        Err(_) => {
            let msg = r#"{"error":"Failed to receive first frame"}"#;
            dispatch_event(engine, EVENT_FRAME_READY, msg);
            return RDCS_ERR_INTERNAL;
        }
    };

    let actual_width = first_frame.width;
    let actual_height = first_frame.height;
    println!("📐 Detected screen resolution: {}×{}", actual_width, actual_height);

    // Target resolution for encoding (balance quality vs bandwidth)
    let (target_width, target_height) = if actual_width <= 1920 && actual_height <= 1080 {
        // Already reasonable size, use as-is
        (actual_width, actual_height)
    } else {
        // Scale down to 1080p to save bandwidth
        // Maintain aspect ratio
        let aspect_ratio = actual_width as f32 / actual_height as f32;
        if aspect_ratio > 16.0 / 9.0 {
            // Wider than 16:9, fit to width
            (1920, (1920.0 / aspect_ratio) as u32)
        } else {
            // Taller than 16:9, fit to height
            ((1080.0 * aspect_ratio) as u32, 1080)
        }
    };

    println!("🎯 Target encoding resolution: {}×{}", target_width, target_height);
    println!("📉 Scaling ratio: {:.1}%", (target_width as f32 / actual_width as f32) * 100.0);

    // Determine encoder resolution enum
    let encoder_resolution = if target_width == 1920 && target_height == 1080 {
        VideoResolution::HD1080
    } else if target_width == 1280 && target_height == 720 {
        VideoResolution::HD720
    } else {
        VideoResolution::Custom(target_width, target_height)
    };

    // Spawn blocking thread to receive frames from sync channel
    let first_frame_clone = first_frame.clone();
    std::thread::spawn(move || {
        // Send the first frame we already received
        if async_tx.blocking_send(first_frame_clone).is_err() {
            return;
        }
        // Continue receiving remaining frames
        while let Ok(frame) = frame_rx.recv() {
            if async_tx.blocking_send(frame).is_err() {
                break; // Channel closed
            }
        }
    });

    // Spawn encode+decode loop (async task)
    let video_handler = Arc::clone(&engine.video_handler);
    let engine_ptr = handle as usize; // Store as usize for Send
    let session_id = 1u64; // Mock session ID

    engine.runtime.spawn(async move {
        // Calculate appropriate bitrate for 2 MB/s target
        // 2 MB/s = 16 Mbps total budget
        // Reserve some for audio/control: use 2 Mbps for video
        let target_bitrate = 2_000_000; // 2 Mbps

        println!("🎯 Target bitrate: {} Mbps", target_bitrate / 1_000_000);

        // Create encoder with target resolution and optimized bitrate
        let mut encoder = match NativeVideoEncoder::new(
            VideoCodec::H264,
            encoder_resolution,
            30, // Max FPS, will drop frames if needed
            target_bitrate,
        ) {
            Ok(enc) => enc,
            Err(e) => {
                eprintln!("❌ Failed to create encoder: {}", e);
                return;
            }
        };

        println!("✅ Video encoder created ({}×{} @ {} Mbps)",
                 target_width, target_height, target_bitrate / 1_000_000);

        let need_scaling = target_width != actual_width || target_height != actual_height;

        // Frame skipping for bandwidth optimization
        let mut frame_count = 0u64;
        let mut last_encoded_frame: Option<Arc<[u8]>> = None;
        let frame_skip_threshold = 0.01; // 1% change threshold

        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown_rx.recv() => {
                    println!("🛑 Video loop shutting down");
                    break;
                }

                // Process captured frames
                Some(mut frame) = async_rx.recv() => {
                    frame_count += 1;

                    // Scale frame if needed
                    #[cfg(target_os = "macos")]
                    if need_scaling {
                        let (scaled_data, scaled_stride) = scaling::scale_frame(
                            &frame.data,
                            frame.width,
                            frame.height,
                            frame.stride,
                            target_width,
                            target_height,
                            frame.pixel_format,
                        );

                        // Update frame with scaled data
                        frame.data = scaled_data;
                        frame.width = target_width;
                        frame.height = target_height;
                        frame.stride = scaled_stride;
                    }

                    // Frame skipping: check if frame changed significantly
                    let should_encode = if let Some(ref last_frame) = last_encoded_frame {
                        // Simple pixel difference check (sample 1% of pixels)
                        let sample_size = (frame.data.len() / 100).max(1000);
                        let mut diff_count = 0;

                        for i in (0..sample_size).step_by(4) {
                            let idx = (i * 100).min(frame.data.len().saturating_sub(4));
                            if idx + 3 < frame.data.len() && idx + 3 < last_frame.len() {
                                let diff = (frame.data[idx] as i32 - last_frame[idx] as i32).abs()
                                    + (frame.data[idx + 1] as i32 - last_frame[idx + 1] as i32).abs()
                                    + (frame.data[idx + 2] as i32 - last_frame[idx + 2] as i32).abs();
                                if diff > 30 { // Threshold per pixel
                                    diff_count += 1;
                                }
                            }
                        }

                        let change_ratio = diff_count as f32 / sample_size as f32;
                        change_ratio > frame_skip_threshold
                    } else {
                        true // Always encode first frame
                    };

                    // Skip frame if no significant change
                    if !should_encode {
                        if frame_count % 90 == 0 { // Log every 3 seconds
                            println!("⏭️  Skipping static frames (saving bandwidth)");
                        }
                        continue;
                    }

                    // Encode frame
                    let encoded = match encoder.encode_captured_frame(&frame) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("❌ Encode failed: {}", e);
                            continue;
                        }
                    };

                    // Store frame for next comparison
                    last_encoded_frame = Some(Arc::clone(&frame.data));

                    // Decode and dispatch (local loopback)
                    let engine_ref = unsafe { &*(engine_ptr as *const EngineHandle) };
                    if let Err(e) = video_handler.handle_encoded_frame(
                        engine_ref,
                        &encoded,
                        session_id,
                    ) {
                        eprintln!("❌ Video handler failed: {}", e);
                    }
                }
            }
        }

        // Cleanup
        if let Err(e) = encoder.shutdown() {
            eprintln!("⚠️ Encoder shutdown error: {}", e);
        }
        println!("✅ Video loop terminated");
    });

    dispatch_event(engine, EVENT_FRAME_READY, r#"{"status":"started"}"#);
    RDCS_OK
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

    // Signal the video loop to stop
    if let Some(tx) = engine.shutdown_tx.lock().unwrap().take() {
        let _ = tx.try_send(()); // Best effort
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
        eprintln!("❌ rdcs_generate_invite: null handle");
        return ptr::null_mut();
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        eprintln!("❌ rdcs_generate_invite: engine is shutdown");
        return ptr::null_mut();
    }

    // Generate a simple 4-digit invite code using timestamp + counter
    // to avoid thread_rng() issues in non-standard threads (Dart isolates)
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // Mix timestamp and counter to generate a 4-digit code
    let code = format!("{:04}", (timestamp + counter) % 10000);

    println!("✅ Generated invite code: {}", code);

    // TODO: Long-term - register invite code with signaling server
    // POST /api/invite/generate -> store code with expiration time

    string_to_cstring(&code)
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
