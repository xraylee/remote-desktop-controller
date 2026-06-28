// Copyright 2024 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! macOS permission checking and requesting.
//!
//! On macOS, uses `CGPreflightScreenCaptureAccess` / `CGRequestScreenCaptureAccess`
//! for screen recording permission and `AXIsProcessTrusted` for accessibility
//! permission. On non-macOS platforms, checks return `false` and requests return
//! `NotSupported`.

use rdcs_platform::PlatformError;

// ---------------------------------------------------------------------------
// macOS implementation using CoreGraphics and ApplicationServices
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;

    // FFI declarations for functions that may not be directly exposed by core-graphics.
    extern "C" {
        /// Returns `true` if the calling process has screen capture access.
        /// Available on macOS 10.15+.
        fn CGPreflightScreenCaptureAccess() -> bool;

        /// Prompts the user to grant screen capture access.
        /// Returns `true` if access was granted. Available on macOS 10.15+.
        fn CGRequestScreenCaptureAccess() -> bool;

        /// Returns `true` if the calling process is trusted for accessibility.
        /// From ApplicationServices / HIServices.
        fn AXIsProcessTrusted() -> bool;
    }

    /// Check if screen recording permission is granted.
    ///
    /// Uses `CGPreflightScreenCaptureAccess()` (macOS 10.15+).
    pub fn check_screen_recording_permission() -> bool {
        unsafe { CGPreflightScreenCaptureAccess() }
    }

    /// Request screen recording permission from the user.
    ///
    /// Uses `CGRequestScreenCaptureAccess()` (macOS 10.15+). This will show
    /// a system prompt asking the user to grant screen recording access.
    pub fn request_screen_recording_permission() -> Result<bool, PlatformError> {
        Ok(unsafe { CGRequestScreenCaptureAccess() })
    }

    /// Check if accessibility permission is granted.
    ///
    /// Uses `AXIsProcessTrusted()` from ApplicationServices. This is required
    /// for input injection (CGEventPost).
    pub fn check_accessibility_permission() -> bool {
        unsafe { AXIsProcessTrusted() }
    }
}

#[cfg(target_os = "macos")]
pub use macos_impl::*;

// ---------------------------------------------------------------------------
// Non-macOS fallback
// ---------------------------------------------------------------------------

/// Check if screen recording permission is granted.
///
/// On non-macOS platforms, always returns `false`.
#[cfg(not(target_os = "macos"))]
pub fn check_screen_recording_permission() -> bool {
    false
}

/// Request screen recording permission from the user.
///
/// On non-macOS platforms, returns `Err(PlatformError::NotSupported)`.
#[cfg(not(target_os = "macos"))]
pub fn request_screen_recording_permission() -> Result<bool, PlatformError> {
    Err(PlatformError::NotSupported(
        "screen recording permission check requires target_os = \"macos\"".into(),
    ))
}

/// Check if accessibility permission is granted.
///
/// On non-macOS platforms, always returns `false`.
#[cfg(not(target_os = "macos"))]
pub fn check_accessibility_permission() -> bool {
    false
}
