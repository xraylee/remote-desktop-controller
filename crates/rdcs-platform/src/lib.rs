//! rdcs-platform: Platform abstraction traits for the RDCS remote desktop system.
//!
//! Defines OS-agnostic interfaces for screen capture, input injection, audio
//! capture, system notifications, and clipboard access. Platform crates
//! (e.g. `rdcs-macos`, `rdcs-windows`) implement these traits.
//!
//! All traits are object-safe (`dyn Trait`) to allow runtime polymorphism.
//! Start-style methods return `std::sync::mpsc::Receiver` channels so that
//! implementations can spawn background threads internally while keeping
//! the trait interface fully synchronous and object-safe.

use serde::{Deserialize, Serialize};
use std::sync::mpsc;

pub mod mock;

// ---------------------------------------------------------------------------
// Screen capture types
// ---------------------------------------------------------------------------

/// Pixel format of a captured frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelFormat {
    /// 32-bit BGRA (most common on macOS/Windows).
    Bgra,
    /// 32-bit RGBA.
    Rgba,
    /// YUV 4:2:0 planar (used by hardware encoders).
    Nv12,
}

/// Configuration for a screen capture session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureConfig {
    /// Target frame rate in frames per second.
    pub fps: u32,
    /// Desired pixel format for captured frames.
    pub pixel_format: PixelFormat,
    /// Optional maximum resolution width. `None` means native.
    pub max_width: Option<u32>,
    /// Optional maximum resolution height. `None` means native.
    pub max_height: Option<u32>,
    /// Whether to capture the cursor in the frame.
    pub capture_cursor: bool,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            fps: 60,
            pixel_format: PixelFormat::Bgra,
            max_width: None,
            max_height: None,
            capture_cursor: true,
        }
    }
}

/// Information about a single display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    /// Platform-assigned display identifier.
    pub id: u64,
    /// Human-readable display name.
    pub name: String,
    /// Native width in pixels.
    pub width: u32,
    /// Native height in pixels.
    pub height: u32,
    /// Refresh rate in Hz.
    pub refresh_rate: f64,
    /// Scale factor (e.g. 2.0 for Retina displays).
    pub scale_factor: f64,
    /// Whether this is the primary display.
    pub is_primary: bool,
}

/// A single captured video frame.
#[derive(Debug, Clone)]
pub struct CapturedFrame {
    /// Raw pixel data.
    pub data: Vec<u8>,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Pixel format of the data.
    pub pixel_format: PixelFormat,
    /// Bytes per row (stride). May be larger than `width * bpp`.
    pub stride: u32,
    /// Display this frame was captured from.
    pub display_id: u64,
    /// Monotonic timestamp in microseconds.
    pub timestamp_us: u64,
}

// ---------------------------------------------------------------------------
// Input types
// ---------------------------------------------------------------------------

/// Mouse button identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse action types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseAction {
    Move,
    Press(MouseButton),
    Release(MouseButton),
    DoubleClick(MouseButton),
}

/// A mouse input event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    /// The action to perform.
    pub action: MouseAction,
    /// X coordinate in screen space.
    pub x: f64,
    /// Y coordinate in screen space.
    pub y: f64,
    /// Target display for the event.
    pub display_id: u64,
}

/// A keyboard input event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    /// Platform-independent key code (USB HID usage codes).
    pub key_code: u32,
    /// Whether the key is pressed (`true`) or released (`false`).
    pub pressed: bool,
    /// Active modifier flags (shift, ctrl, alt, meta).
    pub modifiers: KeyModifiers,
}

/// Modifier key flags.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

/// A scroll/wheel event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollEvent {
    /// Horizontal scroll delta (positive = right).
    pub delta_x: f64,
    /// Vertical scroll delta (positive = down).
    pub delta_y: f64,
    /// Whether the values represent precise pixel deltas.
    pub is_precise: bool,
}

// ---------------------------------------------------------------------------
// Audio types
// ---------------------------------------------------------------------------

/// Audio sample format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioSampleFormat {
    /// 16-bit signed integer.
    I16,
    /// 32-bit float.
    F32,
}

/// Configuration for an audio capture session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels (1 = mono, 2 = stereo).
    pub channels: u16,
    /// Sample format.
    pub sample_format: AudioSampleFormat,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            channels: 2,
            sample_format: AudioSampleFormat::F32,
        }
    }
}

/// A chunk of captured audio data.
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// Raw audio sample data.
    pub data: Vec<u8>,
    /// Configuration describing the format of `data`.
    pub config: AudioConfig,
    /// Monotonic timestamp in microseconds.
    pub timestamp_us: u64,
}

/// Information about an audio device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    /// Platform-assigned device identifier.
    pub id: String,
    /// Human-readable device name.
    pub name: String,
    /// Whether this is an input device (`true`) or output device (`false`).
    pub is_input: bool,
    /// Whether this is the system default device.
    pub is_default: bool,
}

// ---------------------------------------------------------------------------
// System notification types
// ---------------------------------------------------------------------------

/// Status indicator shown in the system tray.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrayStatus {
    /// Idle, waiting for connection.
    Idle,
    /// Actively connected to a remote session.
    Connected,
    /// An error condition needs attention.
    Error,
    /// Application is updating.
    Updating,
}

/// System sound identifiers for audio feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemSound {
    /// Played when a remote session connects.
    Connected,
    /// Played when a remote session disconnects.
    Disconnected,
    /// Played when an error occurs.
    Error,
    /// Played when a file transfer completes.
    TransferComplete,
}

// ---------------------------------------------------------------------------
// Clipboard types
// ---------------------------------------------------------------------------

/// Content stored on the clipboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipboardContent {
    /// Plain UTF-8 text.
    Text(String),
    /// Raw image data (PNG encoded).
    Image(Vec<u8>),
    /// File path list.
    Files(Vec<String>),
}

/// An event emitted when clipboard content changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEvent {
    /// The new clipboard content.
    pub content: ClipboardContent,
    /// Monotonic timestamp in microseconds.
    pub timestamp_us: u64,
}

// ---------------------------------------------------------------------------
// Platform traits — all object-safe (dyn-compatible)
// ---------------------------------------------------------------------------

/// Captures screen content from one or more displays.
///
/// Implementations spawn background threads internally; the returned
/// `Receiver` delivers frames to the caller.
pub trait ScreenCapture: Send + Sync {
    /// Start capturing frames. Returns a channel receiver for frames.
    fn start(
        &self,
        config: CaptureConfig,
    ) -> Result<mpsc::Receiver<CapturedFrame>, PlatformError>;

    /// Stop the active capture session.
    fn stop(&self) -> Result<(), PlatformError>;

    /// Returns `true` if a capture session is currently active.
    fn is_capturing(&self) -> bool;

    /// List all available displays.
    fn displays(&self) -> Result<Vec<DisplayInfo>, PlatformError>;
}

/// Injects mouse, keyboard, and scroll events into the OS input queue.
pub trait InputInjector: Send + Sync {
    /// Inject a mouse event.
    fn inject_mouse(&self, event: MouseEvent) -> Result<(), PlatformError>;

    /// Inject a keyboard event.
    fn inject_key(&self, event: KeyEvent) -> Result<(), PlatformError>;

    /// Inject a scroll event.
    fn inject_scroll(&self, event: ScrollEvent) -> Result<(), PlatformError>;
}

/// Captures system or application audio.
pub trait AudioCapture: Send + Sync {
    /// Start capturing audio. Returns a channel receiver for audio chunks.
    fn start(&self, config: AudioConfig) -> Result<mpsc::Receiver<AudioChunk>, PlatformError>;

    /// Stop the active audio capture session.
    fn stop(&self) -> Result<(), PlatformError>;

    /// List available audio devices.
    fn devices(&self) -> Result<Vec<AudioDeviceInfo>, PlatformError>;
}

/// Manages system tray icon, notifications, and audio feedback.
pub trait SystemNotify: Send + Sync {
    /// Show a desktop notification.
    fn show_notification(&self, title: &str, body: &str) -> Result<(), PlatformError>;

    /// Update the system tray status indicator.
    fn set_tray_status(&self, status: TrayStatus) -> Result<(), PlatformError>;

    /// Play a system sound for audio feedback.
    fn play_sound(&self, sound: SystemSound) -> Result<(), PlatformError>;
}

/// Reads from and writes to the system clipboard, with change notifications.
pub trait ClipboardProvider: Send + Sync {
    /// Get the current clipboard text content.
    fn get_text(&self) -> Result<String, PlatformError>;

    /// Set clipboard text content.
    fn set_text(&self, text: &str) -> Result<(), PlatformError>;

    /// Watch for clipboard changes. Returns a channel that receives events.
    fn watch(&self) -> Result<mpsc::Receiver<ClipboardEvent>, PlatformError>;
}

// ---------------------------------------------------------------------------
// Platform error
// ---------------------------------------------------------------------------

/// Errors originating from platform-specific operations.
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    /// The requested operation is not supported on this platform.
    #[error("not supported: {0}")]
    NotSupported(String),

    /// A required system permission was denied (e.g. screen recording).
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// A platform API call failed.
    #[error("platform api error: {0}")]
    ApiError(String),

    /// The device or resource was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// An I/O error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_capture_config() {
        let config = CaptureConfig::default();
        assert_eq!(config.fps, 60);
        assert_eq!(config.pixel_format, PixelFormat::Bgra);
        assert!(config.capture_cursor);
    }

    #[test]
    fn default_audio_config() {
        let config = AudioConfig::default();
        assert_eq!(config.sample_rate, 48_000);
        assert_eq!(config.channels, 2);
    }

    #[test]
    fn platform_error_display() {
        let err = PlatformError::PermissionDenied("screen recording".into());
        assert_eq!(err.to_string(), "permission denied: screen recording");
    }

    /// Verify traits are object-safe by creating trait objects.
    #[test]
    fn traits_are_object_safe() {
        fn _assert_object_safe_capture(_: Box<dyn ScreenCapture>) {}
        fn _assert_object_safe_input(_: Box<dyn InputInjector>) {}
        fn _assert_object_safe_audio(_: Box<dyn AudioCapture>) {}
        fn _assert_object_safe_notify(_: Box<dyn SystemNotify>) {}
        fn _assert_object_safe_clipboard(_: Box<dyn ClipboardProvider>) {}
    }
}
