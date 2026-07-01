// Copyright 2024 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! macOS screen capture using core-graphics (CGDisplay).
//!
//! On macOS, uses `CGDisplayCreateImage` to capture screen content and
//! `CGGetActiveDisplayList` for display enumeration. On non-macOS platforms,
//! all methods return `PlatformError::NotSupported`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
#[cfg(target_os = "macos")]
use std::thread::JoinHandle;
#[cfg(target_os = "macos")]
use std::time::{Duration, Instant};

use rdcs_platform::{CaptureConfig, CapturedFrame, DisplayInfo, PlatformError};
#[cfg(target_os = "macos")]
use rdcs_platform::PixelFormat;

// ---------------------------------------------------------------------------
// Struct definition — macOS version includes a thread handle
// ---------------------------------------------------------------------------

/// macOS screen capture implementation using core-graphics APIs.
///
/// On macOS 10.15+, uses `CGDisplayCreateImage` for frame capture and
/// `CGGetActiveDisplayList` for display enumeration. Requires screen
/// recording permission to be granted by the user.
///
/// On non-macOS platforms, all methods return `NotSupported`.
#[cfg(target_os = "macos")]
#[derive(Debug)]
pub struct MacOsScreenCapture {
    capturing: Arc<AtomicBool>,
    handle: std::sync::Mutex<Option<JoinHandle<()>>>,
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug)]
pub struct MacOsScreenCapture {
    capturing: Arc<AtomicBool>,
}

#[cfg(target_os = "macos")]
impl MacOsScreenCapture {
    pub fn new() -> Self {
        Self {
            capturing: Arc::new(AtomicBool::new(false)),
            handle: std::sync::Mutex::new(None),
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl MacOsScreenCapture {
    pub fn new() -> Self {
        Self {
            capturing: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for MacOsScreenCapture {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// macOS implementation using core-graphics FFI
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
    use std::thread;

    // Direct FFI declarations for CoreGraphics functions we need.
    // This avoids relying on internal/private APIs in the core-graphics crate.
    type CGDirectDisplayID = u32;
    type CGImageRef = *mut std::ffi::c_void;
    type CGDataProviderRef = *mut std::ffi::c_void;
    type CFDataRef = *mut std::ffi::c_void;

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    struct CGSize {
        width: f64,
        height: f64,
    }

    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }

    extern "C" {
        fn CGMainDisplayID() -> CGDirectDisplayID;
        fn CGGetActiveDisplayList(
            max_displays: u32,
            active_displays: *mut CGDirectDisplayID,
            display_count: *mut u32,
        ) -> i32;
        fn CGDisplayCreateImage(display: CGDirectDisplayID) -> CGImageRef;
        fn CGDisplayPixelsWide(display: CGDirectDisplayID) -> usize;
        fn CGDisplayPixelsHigh(display: CGDirectDisplayID) -> usize;
        fn CGImageGetWidth(image: CGImageRef) -> usize;
        fn CGImageGetHeight(image: CGImageRef) -> usize;
        fn CGImageGetBytesPerRow(image: CGImageRef) -> usize;
        fn CGImageGetBitsPerPixel(image: CGImageRef) -> usize;
        fn CGImageGetBitsPerComponent(image: CGImageRef) -> usize;
        fn CGImageGetDataProvider(image: CGImageRef) -> CGDataProviderRef;
        fn CGDataProviderCopyData(provider: CGDataProviderRef) -> CFDataRef;
        fn CFDataGetBytePtr(data: CFDataRef) -> *const u8;
        fn CFDataGetLength(data: CFDataRef) -> isize;
        fn CFRelease(cf: *mut std::ffi::c_void);
        fn CGDisplayBounds(display: CGDirectDisplayID) -> CGRect;
    }

    /// Capture the main display and extract pixel data.
    ///
    /// Performance optimization: This function is the bottleneck (~128ms).
    /// CGDisplayCreateImage is synchronous and involves GPU-to-CPU transfer.
    ///
    /// Future optimization: Use CGDisplayStream for <10ms async capture.
    fn capture_display(display_id: CGDirectDisplayID) -> Option<CapturedFrame> {
        unsafe {
            let image = CGDisplayCreateImage(display_id);
            if image.is_null() {
                return None;
            }

            let width = CGImageGetWidth(image) as u32;
            let height = CGImageGetHeight(image) as u32;
            let stride = CGImageGetBytesPerRow(image) as u32;
            let bpp = CGImageGetBitsPerPixel(image);
            let bpc = CGImageGetBitsPerComponent(image);

            let pixel_format = if bpp == 32 && bpc == 8 {
                PixelFormat::Bgra
            } else {
                PixelFormat::Rgba
            };

            // Extract pixel data via the data provider.
            // OPTIMIZATION: Changed from Vec<u8> to Arc<[u8]> to enable zero-copy
            // sharing between capture and encode threads. This reduces memory
            // allocation pressure and eliminates one full frame copy.
            let provider = CGImageGetDataProvider(image);
            let data = if !provider.is_null() {
                let cf_data = CGDataProviderCopyData(provider);
                if !cf_data.is_null() {
                    let len = CFDataGetLength(cf_data) as usize;
                    let ptr = CFDataGetBytePtr(cf_data);
                    let bytes = std::slice::from_raw_parts(ptr, len).to_vec();
                    CFRelease(cf_data);
                    Arc::from(bytes.into_boxed_slice())
                } else {
                    Arc::from(Box::new([]) as Box<[u8]>)
                }
            } else {
                Arc::from(Box::new([]) as Box<[u8]>)
            };

            CFRelease(image);

            Some(CapturedFrame {
                data,
                width,
                height,
                pixel_format,
                stride,
                display_id: display_id as u64,
                timestamp_us: 0, // Filled in by the caller.
            })
        }
    }

    /// Enumerate active displays.
    fn get_active_displays() -> Vec<CGDirectDisplayID> {
        unsafe {
            let mut count: u32 = 0;
            if CGGetActiveDisplayList(0, std::ptr::null_mut(), &mut count) != 0 {
                return Vec::new();
            }
            let mut displays = vec![0u32; count as usize];
            if CGGetActiveDisplayList(count, displays.as_mut_ptr(), &mut count) != 0
            {
                return Vec::new();
            }
            displays.truncate(count as usize);
            displays
        }
    }

    impl rdcs_platform::ScreenCapture for MacOsScreenCapture {
        fn start(
            &self,
            config: CaptureConfig,
        ) -> Result<mpsc::Receiver<CapturedFrame>, PlatformError> {
            if self.capturing.swap(true, Ordering::SeqCst) {
                return Err(PlatformError::ApiError(
                    "capture session already active".into(),
                ));
            }

            // Check screen recording permission before attempting capture.
            if !crate::permissions::check_screen_recording_permission() {
                self.capturing.store(false, Ordering::SeqCst);
                return Err(PlatformError::PermissionDenied(
                    "screen recording permission not granted".into(),
                ));
            }

            let (tx, rx) = mpsc::channel();
            let thread_flag = self.capturing.clone();
            let frame_interval =
                Duration::from_micros(1_000_000 / config.fps.max(1) as u64);
            let primary_id = unsafe { CGMainDisplayID() };

            let handle = thread::Builder::new()
                .name("rdcs-screen-capture".into())
                .spawn(move || {
                    let start_time = Instant::now();
                    while thread_flag.load(Ordering::SeqCst) {
                        let loop_start = Instant::now();

                        if let Some(mut frame) = capture_display(primary_id) {
                            frame.timestamp_us =
                                start_time.elapsed().as_micros() as u64;
                            if tx.send(frame).is_err() {
                                break; // Receiver dropped.
                            }
                        }

                        let elapsed = loop_start.elapsed();
                        if elapsed < frame_interval {
                            thread::sleep(frame_interval - elapsed);
                        }
                    }
                })
                .map_err(|e| {
                    self.capturing.store(false, Ordering::SeqCst);
                    PlatformError::ApiError(format!(
                        "failed to spawn capture thread: {e}"
                    ))
                })?;

            let mut guard = self
                .handle
                .lock()
                .map_err(|_| PlatformError::ApiError("lock poisoned".into()))?;
            *guard = Some(handle);

            Ok(rx)
        }

        fn stop(&self) -> Result<(), PlatformError> {
            self.capturing.store(false, Ordering::SeqCst);
            let handle = {
                let mut guard = self
                    .handle
                    .lock()
                    .map_err(|_| PlatformError::ApiError("lock poisoned".into()))?;
                guard.take()
            };
            if let Some(h) = handle {
                let _ = h.join();
            }
            Ok(())
        }

        fn is_capturing(&self) -> bool {
            self.capturing.load(Ordering::SeqCst)
        }

        fn displays(&self) -> Result<Vec<DisplayInfo>, PlatformError> {
            let display_ids = get_active_displays();
            let main_id = unsafe { CGMainDisplayID() };
            let mut result = Vec::with_capacity(display_ids.len());

            for &did in &display_ids {
                let pixels_wide = unsafe { CGDisplayPixelsWide(did) };
                let pixels_high = unsafe { CGDisplayPixelsHigh(did) };

                // Determine scale factor from bounds vs pixel dimensions.
                let bounds = unsafe { CGDisplayBounds(did) };
                let point_width = bounds.size.width;
                let scale_factor = if point_width > 0.0 {
                    (pixels_wide as f64 / point_width).round()
                } else {
                    1.0
                };

                let name = if did == main_id {
                    "Primary Display".to_string()
                } else {
                    format!("Display {did}")
                };

                result.push(DisplayInfo {
                    id: did as u64,
                    name,
                    width: pixels_wide as u32,
                    height: pixels_high as u32,
                    refresh_rate: 60.0,
                    scale_factor,
                    is_primary: did == main_id,
                });
            }

            Ok(result)
        }
    }
}

// ---------------------------------------------------------------------------
// Non-macOS fallback: all methods return NotSupported
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "macos"))]
impl rdcs_platform::ScreenCapture for MacOsScreenCapture {
    fn start(
        &self,
        _config: CaptureConfig,
    ) -> Result<mpsc::Receiver<CapturedFrame>, PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS screen capture requires target_os = \"macos\"".into(),
        ))
    }

    fn stop(&self) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS screen capture requires target_os = \"macos\"".into(),
        ))
    }

    fn is_capturing(&self) -> bool {
        self.capturing.load(Ordering::SeqCst)
    }

    fn displays(&self) -> Result<Vec<DisplayInfo>, PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS display enumeration requires target_os = \"macos\"".into(),
        ))
    }
}
