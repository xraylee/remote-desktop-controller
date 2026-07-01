// Copyright 2024 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! macOS clipboard access using NSPasteboard via objc.
//!
//! On macOS, uses `NSPasteboard.generalPasteboard` for get/set operations
//! and polls `changeCount` for clipboard change detection. On non-macOS
//! platforms, all methods return `PlatformError::NotSupported`.

use std::sync::mpsc;

use rdcs_platform::{ClipboardContent, ClipboardEvent, ClipboardProvider, PlatformError};

/// macOS clipboard provider using NSPasteboard.
///
/// Provides read/write access to the system clipboard (text only) and
/// a polling-based clipboard watcher that detects changes via
/// `NSPasteboard.changeCount`.
///
/// On non-macOS platforms, all methods return `NotSupported`.
#[derive(Debug)]
pub struct MacOsClipboard;

impl MacOsClipboard {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOsClipboard {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// macOS implementation using NSPasteboard via objc
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
#[allow(unexpected_cfgs)]
mod macos_impl {
    use super::*;
    use objc::runtime::{Class, Object};
    use objc::{msg_send, sel, sel_impl};
    use std::ffi::CStr;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    /// Polling interval for clipboard change detection.
    const CLIPBOARD_POLL_INTERVAL: Duration = Duration::from_millis(500);

    /// The UTI string for plain text on the pasteboard.
    const PLAIN_TEXT_UTI: &[u8] = b"public.utf8-plain-text\0";

    /// Create an NSString from a null-terminated byte slice (UTF-8).
    /// Returns an autoreleased NSString pointer.
    ///
    /// # Safety
    /// The input must be null-terminated valid UTF-8.
    unsafe fn make_nsstring(s: &[u8]) -> Result<*mut Object, PlatformError> {
        let cls = Class::get("NSString")
            .ok_or_else(|| PlatformError::NotSupported("NSString class not available".into()))?;
        let obj: *mut Object = msg_send![cls, alloc];
        let obj: *mut Object = msg_send![obj, initWithUTF8String: s.as_ptr()];
        // Autorelease so it's managed by the current pool.
        let _: () = msg_send![obj, autorelease];
        Ok(obj)
    }

    /// Convert an NSString pointer to a Rust String.
    ///
    /// # Safety
    /// `ns_string` must be a valid NSString pointer or null.
    unsafe fn nsstring_to_string(ns_string: *mut Object) -> Option<String> {
        if ns_string.is_null() {
            return None;
        }
        let c_str: *const std::os::raw::c_char = msg_send![ns_string, UTF8String];
        if c_str.is_null() {
            return None;
        }
        Some(CStr::from_ptr(c_str).to_string_lossy().into_owned())
    }

    /// Get the general pasteboard singleton.
    unsafe fn general_pasteboard() -> Result<*mut Object, PlatformError> {
        let cls = Class::get("NSPasteboard").ok_or_else(|| {
            PlatformError::ApiError("NSPasteboard class not found".into())
        })?;
        let pasteboard: *mut Object = msg_send![cls, generalPasteboard];
        if pasteboard.is_null() {
            Err(PlatformError::ApiError(
                "generalPasteboard returned null".into(),
            ))
        } else {
            Ok(pasteboard)
        }
    }

    /// Create a new autorelease pool. Returns the pool object.
    unsafe fn create_autorelease_pool() -> Result<*mut Object, PlatformError> {
        let cls = Class::get("NSAutoreleasePool").ok_or_else(|| {
            PlatformError::ApiError("NSAutoreleasePool not found".into())
        })?;
        let pool: *mut Object = msg_send![cls, new];
        if pool.is_null() {
            Err(PlatformError::ApiError("failed to create autorelease pool".into()))
        } else {
            Ok(pool)
        }
    }

    /// Drain an autorelease pool.
    unsafe fn drain_autorelease_pool(pool: *mut Object) {
        if !pool.is_null() {
            let _: () = msg_send![pool, drain];
        }
    }

    /// Read text from the general pasteboard.
    fn pasteboard_get_text() -> Result<String, PlatformError> {
        unsafe {
            let pool = create_autorelease_pool()?;
            let result = (|| {
                let pasteboard = general_pasteboard()?;
                let type_str = make_nsstring(PLAIN_TEXT_UTI)?;
                let text: *mut Object =
                    msg_send![pasteboard, stringForType: type_str];
                Ok(nsstring_to_string(text).unwrap_or_default())
            })();
            drain_autorelease_pool(pool);
            result
        }
    }

    /// Write text to the general pasteboard.
    fn pasteboard_set_text(text: &str) -> Result<(), PlatformError> {
        unsafe {
            let pool = create_autorelease_pool()?;
            let result = (|| {
                let pasteboard = general_pasteboard()?;
                let _: () = msg_send![pasteboard, clearContents];

                // Create the value NSString.
                let text_bytes: Vec<u8> =
                    text.bytes().chain(std::iter::once(0)).collect();
                let value = make_nsstring(&text_bytes)?;

                // Create the type NSString.
                let type_str = make_nsstring(PLAIN_TEXT_UTI)?;

                let success: bool =
                    msg_send![pasteboard, setString: value forType: type_str];
                if !success {
                    return Err(PlatformError::ApiError(
                        "NSPasteboard.setString failed".into(),
                    ));
                }
                Ok(())
            })();
            drain_autorelease_pool(pool);
            result
        }
    }

    /// Get the current changeCount from the pasteboard.
    fn pasteboard_change_count() -> Result<i64, PlatformError> {
        unsafe {
            let pool = create_autorelease_pool()?;
            let result = (|| {
                let pasteboard = general_pasteboard()?;
                let count: i64 = msg_send![pasteboard, changeCount];
                Ok(count)
            })();
            drain_autorelease_pool(pool);
            result
        }
    }

    impl ClipboardProvider for MacOsClipboard {
        fn get_text(&self) -> Result<String, PlatformError> {
            pasteboard_get_text()
        }

        fn set_text(&self, text: &str) -> Result<(), PlatformError> {
            pasteboard_set_text(text)
        }

        fn watch(&self) -> Result<mpsc::Receiver<ClipboardEvent>, PlatformError> {
            let (tx, rx) = mpsc::channel();
            let running = Arc::new(AtomicBool::new(true));
            let thread_running = running.clone();

            thread::Builder::new()
                .name("rdcs-clipboard-watch".into())
                .spawn(move || {
                    let start_time = Instant::now();
                    let mut last_count: i64 =
                        pasteboard_change_count().unwrap_or(0);

                    while thread_running.load(Ordering::SeqCst) {
                        thread::sleep(CLIPBOARD_POLL_INTERVAL);

                        match pasteboard_change_count() {
                            Ok(count) if count != last_count => {
                                last_count = count;
                                if let Ok(text) = pasteboard_get_text() {
                                    let event = ClipboardEvent {
                                        content: ClipboardContent::Text(text),
                                        timestamp_us: start_time
                                            .elapsed()
                                            .as_micros()
                                            as u64,
                                    };
                                    if tx.send(event).is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(_) | Ok(_) => {}
                        }
                    }
                })
                .map_err(|e| {
                    PlatformError::ApiError(format!(
                        "failed to spawn clipboard watcher: {e}"
                    ))
                })?;

            Ok(rx)
        }
    }
}

// ---------------------------------------------------------------------------
// Non-macOS fallback: all methods return NotSupported
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "macos"))]
impl ClipboardProvider for MacOsClipboard {
    fn get_text(&self) -> Result<String, PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS clipboard requires target_os = \"macos\"".into(),
        ))
    }

    fn set_text(&self, _text: &str) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS clipboard requires target_os = \"macos\"".into(),
        ))
    }

    fn watch(&self) -> Result<mpsc::Receiver<ClipboardEvent>, PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS clipboard watch requires target_os = \"macos\"".into(),
        ))
    }
}
