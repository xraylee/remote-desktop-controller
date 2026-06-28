// Copyright 2024 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

// Suppress `unexpected_cfgs` warnings from the objc crate's `msg_send!` macro
// expansion, which internally checks for `cfg(cargo-clippy)`.
#![allow(unexpected_cfgs)]

//! rdcs-macos: macOS platform implementations for the RDCS remote desktop system.
//!
//! Provides macOS-specific implementations of the platform abstraction traits
//! defined in `rdcs-platform`: screen capture (core-graphics / ScreenCaptureKit),
//! input injection (CGEvent), clipboard (NSPasteboard), and permission checking.
//!
//! ## Modules
//!
//! - [`capture`] — Screen capture via `CGDisplayCreateImage`
//! - [`input`] — Input injection via `CGEventPost`
//! - [`clipboard`] — Clipboard access via `NSPasteboard`
//! - [`permissions`] — Permission checking (`CGPreflightScreenCaptureAccess`, `AXIsProcessTrusted`)
//!
//! ## Non-macOS Fallback
//!
//! When compiled on a non-macOS target, all trait methods return
//! `PlatformError::NotSupported`. This allows the crate to compile and tests
//! to pass on any platform.

use std::sync::mpsc;

use rdcs_platform::{
    AudioCapture, AudioChunk, AudioConfig, AudioDeviceInfo, ClipboardProvider, InputInjector,
    PlatformError, ScreenCapture, SystemNotify, SystemSound, TrayStatus,
};

pub mod capture;
pub mod clipboard;
pub mod input;
pub mod permissions;

// Re-export concrete types for direct use.
pub use capture::MacOsScreenCapture;
pub use clipboard::MacOsClipboard;
pub use input::MacOsInputInjector;
pub use permissions::{
    check_accessibility_permission, check_screen_recording_permission,
    request_screen_recording_permission,
};

// ---------------------------------------------------------------------------
// Audio capture (CoreAudio) — stub, real implementation deferred
// ---------------------------------------------------------------------------

/// macOS audio capture using CoreAudio APIs.
///
/// Currently a stub returning `NotSupported`. A future implementation will use
/// the CoreAudio `AudioDeviceCreateIOProcID` API for system audio capture.
#[derive(Debug)]
pub struct MacOsAudioCapture;

impl MacOsAudioCapture {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOsAudioCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioCapture for MacOsAudioCapture {
    fn start(
        &self,
        _config: AudioConfig,
    ) -> Result<mpsc::Receiver<AudioChunk>, PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS audio capture not yet implemented".into(),
        ))
    }

    fn stop(&self) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS audio capture not yet implemented".into(),
        ))
    }

    fn devices(&self) -> Result<Vec<AudioDeviceInfo>, PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS audio device enumeration not yet implemented".into(),
        ))
    }
}

// ---------------------------------------------------------------------------
// System notifications — stub, real implementation deferred
// ---------------------------------------------------------------------------

/// macOS system notification manager.
///
/// Currently a stub returning `NotSupported`. A future implementation will use
/// `UNUserNotificationCenter` for desktop notifications and `NSSound` for
/// audio feedback.
#[derive(Debug)]
pub struct MacOsSystemNotify;

impl MacOsSystemNotify {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOsSystemNotify {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemNotify for MacOsSystemNotify {
    fn show_notification(&self, _title: &str, _body: &str) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS notifications not yet implemented".into(),
        ))
    }

    fn set_tray_status(&self, _status: TrayStatus) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS tray status not yet implemented".into(),
        ))
    }

    fn play_sound(&self, _sound: SystemSound) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS system sounds not yet implemented".into(),
        ))
    }
}

// ---------------------------------------------------------------------------
// Platform factory
// ---------------------------------------------------------------------------

/// Factory for creating macOS platform implementations.
///
/// Each method returns a boxed trait object ready for use via the
/// `rdcs-platform` abstraction layer.
pub struct MacPlatform;

impl MacPlatform {
    /// Create a screen capture instance (core-graphics / ScreenCaptureKit).
    pub fn create_capture() -> Box<dyn ScreenCapture> {
        Box::new(MacOsScreenCapture::new())
    }

    /// Create an input injector (CGEvent-based).
    pub fn create_input() -> Box<dyn InputInjector> {
        Box::new(MacOsInputInjector::new())
    }

    /// Create an audio capture instance (CoreAudio — stub).
    pub fn create_audio() -> Box<dyn AudioCapture> {
        Box::new(MacOsAudioCapture::new())
    }

    /// Create a system notification manager (stub).
    pub fn create_notify() -> Box<dyn SystemNotify> {
        Box::new(MacOsSystemNotify::new())
    }

    /// Create a clipboard provider (NSPasteboard).
    pub fn create_clipboard() -> Box<dyn ClipboardProvider> {
        Box::new(MacOsClipboard::new())
    }
}

// ---------------------------------------------------------------------------
// Tests — designed to pass on any platform
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// All macOS struct types can be constructed without panic.
    #[test]
    fn macos_types_constructible() {
        let _screen = MacOsScreenCapture::new();
        let _input = MacOsInputInjector::new();
        let _audio = MacOsAudioCapture::new();
        let _notify = MacOsSystemNotify::new();
        let _clipboard = MacOsClipboard::new();
    }

    /// Permission check functions return without panic.
    #[test]
    fn permission_checks_callable() {
        // On non-macOS these return false / Err(NotSupported) gracefully.
        let _screen = check_screen_recording_permission();
        let _accessibility = check_accessibility_permission();
        // request_screen_recording_permission should not panic.
        let _request = request_screen_recording_permission();
    }

    /// The platform factory produces valid trait objects.
    #[test]
    fn platform_factory_works() {
        let _capture: Box<dyn ScreenCapture> = MacPlatform::create_capture();
        let _input: Box<dyn InputInjector> = MacPlatform::create_input();
        let _audio: Box<dyn AudioCapture> = MacPlatform::create_audio();
        let _notify: Box<dyn SystemNotify> = MacPlatform::create_notify();
        let _clipboard: Box<dyn ClipboardProvider> = MacPlatform::create_clipboard();
    }

    /// Non-macOS fallback methods return NotSupported gracefully.
    /// Only tested on non-macOS platforms where all methods are stubs.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_fallback_returns_not_supported() {
        let capture = MacOsScreenCapture::new();
        let config = rdcs_platform::CaptureConfig::default();
        assert!(capture.start(config).is_err());
        assert!(capture.stop().is_err());
        assert!(capture.displays().is_err());

        let input = MacOsInputInjector::new();
        let mouse = rdcs_platform::MouseEvent {
            action: rdcs_platform::MouseAction::Move,
            x: 0.0,
            y: 0.0,
            display_id: 0,
        };
        assert!(input.inject_mouse(mouse).is_err());

        let key = rdcs_platform::KeyEvent {
            key_code: 0,
            pressed: true,
            modifiers: rdcs_platform::KeyModifiers::default(),
        };
        assert!(input.inject_key(key).is_err());

        let scroll = rdcs_platform::ScrollEvent {
            delta_x: 0.0,
            delta_y: 0.0,
            is_precise: false,
        };
        assert!(input.inject_scroll(scroll).is_err());

        let clipboard = MacOsClipboard::new();
        assert!(clipboard.get_text().is_err());
        assert!(clipboard.set_text("test").is_err());
        assert!(clipboard.watch().is_err());

        let audio = MacOsAudioCapture::new();
        let audio_config = rdcs_platform::AudioConfig::default();
        assert!(audio.start(audio_config).is_err());
        assert!(audio.stop().is_err());
        assert!(audio.devices().is_err());

        let notify = MacOsSystemNotify::new();
        assert!(notify.show_notification("title", "body").is_err());
        assert!(notify
            .set_tray_status(rdcs_platform::TrayStatus::Idle)
            .is_err());
        assert!(notify
            .play_sound(rdcs_platform::SystemSound::Connected)
            .is_err());
    }

    /// Screen capture `is_capturing` returns false by default.
    #[test]
    fn screen_capture_initial_state() {
        let capture = MacOsScreenCapture::new();
        assert!(!capture.is_capturing());
    }

    /// Permission functions on non-macOS return expected fallback values.
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn non_macos_permission_fallbacks() {
        assert!(!check_screen_recording_permission());
        assert!(!check_accessibility_permission());
        assert!(request_screen_recording_permission().is_err());
    }
}
