//! Mock implementations of all platform abstraction traits for testing.
//!
//! These mocks allow higher-level crates to test their logic without
//! depending on real platform-specific code (screen capture APIs, OS
//! input queues, etc.).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::{
    AudioCapture, AudioChunk, AudioConfig, AudioDeviceInfo, AudioSampleFormat, CapturedFrame,
    ClipboardContent, ClipboardEvent, ClipboardProvider, DisplayInfo, InputInjector, KeyEvent,
    MouseEvent, PlatformError, ScrollEvent, ScreenCapture, SystemNotify, SystemSound,
    TrayStatus,
};

// ---------------------------------------------------------------------------
// InputEvent — unified enum for recording all input types
// ---------------------------------------------------------------------------

/// A recorded input event (mouse, key, or scroll).
#[derive(Debug, Clone)]
pub enum InputEvent {
    /// A recorded mouse event.
    Mouse(MouseEvent),
    /// A recorded keyboard event.
    Key(KeyEvent),
    /// A recorded scroll event.
    Scroll(ScrollEvent),
}

// ---------------------------------------------------------------------------
// NotificationRecord — records all system-notify operations
// ---------------------------------------------------------------------------

/// A recorded system notification operation.
#[derive(Debug, Clone)]
pub enum NotificationRecord {
    /// A desktop notification was shown.
    Notification { title: String, body: String },
    /// The tray status was updated.
    TrayStatus(TrayStatus),
    /// A system sound was played.
    Sound(SystemSound),
}

// ---------------------------------------------------------------------------
// MockScreenCapture
// ---------------------------------------------------------------------------

/// Mock screen capture that emits pre-loaded frames via a channel.
///
/// # Examples
///
/// ```
/// use rdcs_platform::mock::MockScreenCapture;
/// use rdcs_platform::{CapturedFrame, PixelFormat, CaptureConfig, ScreenCapture};
/// use std::sync::Arc;
///
/// let frame = CapturedFrame {
///     data: Arc::from(vec![0u8; 4].into_boxed_slice()),
///     width: 1,
///     height: 1,
///     pixel_format: PixelFormat::Bgra,
///     stride: 4,
///     display_id: 1,
///     timestamp_us: 0,
/// };
/// let cap = MockScreenCapture::with_frames(vec![frame]);
/// let rx = cap.start(CaptureConfig::default()).unwrap();
/// assert!(cap.is_capturing());
/// let received = rx.recv().unwrap();
/// assert_eq!(received.width, 1);
/// cap.stop().unwrap();
/// assert!(!cap.is_capturing());
/// ```
pub struct MockScreenCapture {
    capturing: AtomicBool,
    frames: Mutex<Vec<CapturedFrame>>,
}

impl MockScreenCapture {
    /// Create a new mock with no pre-loaded frames.
    pub fn new() -> Self {
        Self {
            capturing: AtomicBool::new(false),
            frames: Mutex::new(Vec::new()),
        }
    }

    /// Create a new mock pre-loaded with frames to emit on `start()`.
    pub fn with_frames(frames: Vec<CapturedFrame>) -> Self {
        Self {
            capturing: AtomicBool::new(false),
            frames: Mutex::new(frames),
        }
    }
}

impl Default for MockScreenCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCapture for MockScreenCapture {
    fn start(
        &self,
        _config: crate::CaptureConfig,
    ) -> Result<mpsc::Receiver<CapturedFrame>, PlatformError> {
        self.capturing.store(true, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel();

        let frames = self.frames.lock().unwrap().clone();

        thread::spawn(move || {
            for (i, mut frame) in frames.into_iter().enumerate() {
                frame.timestamp_us = i as u64 * 16_667; // ~60 fps intervals
                if tx.send(frame).is_err() {
                    break;
                }
            }
        });

        Ok(rx)
    }

    fn stop(&self) -> Result<(), PlatformError> {
        self.capturing.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn is_capturing(&self) -> bool {
        self.capturing.load(Ordering::SeqCst)
    }

    fn displays(&self) -> Result<Vec<DisplayInfo>, PlatformError> {
        Ok(vec![DisplayInfo {
            id: 1,
            name: "Mock Display".to_string(),
            width: 1920,
            height: 1080,
            refresh_rate: 60.0,
            scale_factor: 1.0,
            is_primary: true,
        }])
    }
}

// ---------------------------------------------------------------------------
// MockInputInjector
// ---------------------------------------------------------------------------

/// Mock input injector that records all injected events for later assertion.
///
/// # Examples
///
/// ```
/// use rdcs_platform::mock::{MockInputInjector, InputEvent};
/// use rdcs_platform::{InputInjector, MouseEvent, MouseAction, MouseButton};
///
/// let injector = MockInputInjector::new();
/// injector.inject_mouse(MouseEvent {
///     action: MouseAction::Press(MouseButton::Left),
///     x: 100.0,
///     y: 200.0,
///     display_id: 1,
/// }).unwrap();
/// assert_eq!(injector.events().len(), 1);
/// ```
pub struct MockInputInjector {
    events: Mutex<Vec<InputEvent>>,
}

impl MockInputInjector {
    /// Create a new mock input injector with an empty event log.
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    /// Return a clone of all recorded events.
    pub fn events(&self) -> Vec<InputEvent> {
        self.events.lock().unwrap().clone()
    }

    /// Clear all recorded events.
    pub fn clear(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl Default for MockInputInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl InputInjector for MockInputInjector {
    fn inject_mouse(&self, event: MouseEvent) -> Result<(), PlatformError> {
        self.events.lock().unwrap().push(InputEvent::Mouse(event));
        Ok(())
    }

    fn inject_key(&self, event: KeyEvent) -> Result<(), PlatformError> {
        self.events.lock().unwrap().push(InputEvent::Key(event));
        Ok(())
    }

    fn inject_scroll(&self, event: ScrollEvent) -> Result<(), PlatformError> {
        self.events
            .lock()
            .unwrap()
            .push(InputEvent::Scroll(event));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MockAudioCapture
// ---------------------------------------------------------------------------

/// Mock audio capture that emits a single synthetic audio chunk.
pub struct MockAudioCapture {
    capturing: AtomicBool,
}

impl MockAudioCapture {
    /// Create a new mock audio capture.
    pub fn new() -> Self {
        Self {
            capturing: AtomicBool::new(false),
        }
    }

    /// Returns `true` if audio capture is currently active.
    pub fn is_capturing(&self) -> bool {
        self.capturing.load(Ordering::SeqCst)
    }
}

impl Default for MockAudioCapture {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioCapture for MockAudioCapture {
    fn start(&self, config: AudioConfig) -> Result<mpsc::Receiver<AudioChunk>, PlatformError> {
        self.capturing.store(true, Ordering::SeqCst);
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let bytes_per_sample = match config.sample_format {
                AudioSampleFormat::I16 => 2,
                AudioSampleFormat::F32 => 4,
            };
            // Generate 10ms of silence at the requested format.
            let num_samples = (config.sample_rate as usize) / 100;
            let data = vec![0u8; num_samples * config.channels as usize * bytes_per_sample];

            let _ = tx.send(AudioChunk {
                data,
                config,
                timestamp_us: 0,
            });
        });

        Ok(rx)
    }

    fn stop(&self) -> Result<(), PlatformError> {
        self.capturing.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn devices(&self) -> Result<Vec<AudioDeviceInfo>, PlatformError> {
        Ok(vec![
            AudioDeviceInfo {
                id: "mock-output".to_string(),
                name: "Mock Output Device".to_string(),
                is_input: false,
                is_default: true,
            },
            AudioDeviceInfo {
                id: "mock-input".to_string(),
                name: "Mock Input Device".to_string(),
                is_input: true,
                is_default: true,
            },
        ])
    }
}

// ---------------------------------------------------------------------------
// MockSystemNotify
// ---------------------------------------------------------------------------

/// Mock system notifier that records all notification operations.
///
/// # Examples
///
/// ```
/// use rdcs_platform::mock::{MockSystemNotify, NotificationRecord};
/// use rdcs_platform::{SystemNotify, TrayStatus};
///
/// let notify = MockSystemNotify::new();
/// notify.show_notification("Hello", "World").unwrap();
/// notify.set_tray_status(TrayStatus::Connected).unwrap();
/// assert_eq!(notify.records().len(), 2);
/// ```
pub struct MockSystemNotify {
    records: Mutex<Vec<NotificationRecord>>,
}

impl MockSystemNotify {
    /// Create a new mock system notifier with an empty record.
    pub fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
        }
    }

    /// Return a clone of all recorded notification operations.
    pub fn records(&self) -> Vec<NotificationRecord> {
        self.records.lock().unwrap().clone()
    }

    /// Clear all recorded operations.
    pub fn clear(&self) {
        self.records.lock().unwrap().clear();
    }
}

impl Default for MockSystemNotify {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemNotify for MockSystemNotify {
    fn show_notification(&self, title: &str, body: &str) -> Result<(), PlatformError> {
        self.records.lock().unwrap().push(NotificationRecord::Notification {
            title: title.to_string(),
            body: body.to_string(),
        });
        Ok(())
    }

    fn set_tray_status(&self, status: TrayStatus) -> Result<(), PlatformError> {
        self.records
            .lock()
            .unwrap()
            .push(NotificationRecord::TrayStatus(status));
        Ok(())
    }

    fn play_sound(&self, sound: SystemSound) -> Result<(), PlatformError> {
        self.records
            .lock()
            .unwrap()
            .push(NotificationRecord::Sound(sound));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// MockClipboard
// ---------------------------------------------------------------------------

/// Mock clipboard that stores text in memory and supports change watching.
pub struct MockClipboard {
    content: Mutex<String>,
}

impl MockClipboard {
    /// Create a new mock clipboard with empty content.
    pub fn new() -> Self {
        Self {
            content: Mutex::new(String::new()),
        }
    }
}

impl Default for MockClipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardProvider for MockClipboard {
    fn get_text(&self) -> Result<String, PlatformError> {
        Ok(self.content.lock().unwrap().clone())
    }

    fn set_text(&self, text: &str) -> Result<(), PlatformError> {
        *self.content.lock().unwrap() = text.to_string();
        Ok(())
    }

    fn watch(&self) -> Result<mpsc::Receiver<ClipboardEvent>, PlatformError> {
        let (tx, rx) = mpsc::channel();
        let content = Arc::new(Mutex::new(self.content.lock().unwrap().clone()));

        thread::spawn({
            let content = Arc::clone(&content);
            move || {
                let mut last = content.lock().unwrap().clone();
                loop {
                    thread::sleep(Duration::from_millis(50));
                    let current = content.lock().unwrap().clone();
                    if current != last {
                        let _ = tx.send(ClipboardEvent {
                            content: ClipboardContent::Text(current.clone()),
                            timestamp_us: 0,
                        });
                        last = current;
                    }
                }
            }
        });

        Ok(rx)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KeyModifiers, MouseAction, MouseButton, PixelFormat,
    };

    fn sample_frame(width: u32, height: u32) -> CapturedFrame {
        let bpp = 4; // BGRA
        CapturedFrame {
            data: vec![0u8; (width * height * bpp) as usize],
            width,
            height,
            pixel_format: PixelFormat::Bgra,
            stride: width * bpp,
            display_id: 1,
            timestamp_us: 0,
        }
    }

    // -- MockScreenCapture --------------------------------------------------

    #[test]
    fn mock_screen_capture_emits_frames() {
        let frames = vec![
            sample_frame(1920, 1080),
            sample_frame(1920, 1080),
            sample_frame(1920, 1080),
        ];
        let cap = MockScreenCapture::with_frames(frames);
        let rx = cap.start(crate::CaptureConfig::default()).unwrap();

        assert!(cap.is_capturing());

        let received: Vec<CapturedFrame> = rx.iter().collect();
        assert_eq!(received.len(), 3);
        assert_eq!(received[0].width, 1920);
        assert_eq!(received[0].height, 1080);
        // Timestamps should be sequential (~60fps intervals)
        assert_eq!(received[0].timestamp_us, 0);
        assert_eq!(received[1].timestamp_us, 16_667);
        assert_eq!(received[2].timestamp_us, 33_334);
    }

    #[test]
    fn mock_screen_capture_stop() {
        let cap = MockScreenCapture::new();
        assert!(!cap.is_capturing());

        let _rx = cap.start(crate::CaptureConfig::default()).unwrap();
        assert!(cap.is_capturing());

        cap.stop().unwrap();
        assert!(!cap.is_capturing());
    }

    #[test]
    fn mock_screen_capture_displays() {
        let cap = MockScreenCapture::new();
        let displays = cap.displays().unwrap();
        assert_eq!(displays.len(), 1);
        assert_eq!(displays[0].name, "Mock Display");
        assert_eq!(displays[0].width, 1920);
        assert_eq!(displays[0].height, 1080);
        assert!(displays[0].is_primary);
    }

    #[test]
    fn mock_screen_capture_empty_frames() {
        let cap = MockScreenCapture::new();
        let rx = cap.start(crate::CaptureConfig::default()).unwrap();
        // Channel should close immediately since there are no frames.
        let received: Vec<CapturedFrame> = rx.iter().collect();
        assert!(received.is_empty());
    }

    // -- MockInputInjector --------------------------------------------------

    #[test]
    fn mock_input_injector_records_mouse() {
        let injector = MockInputInjector::new();
        let event = MouseEvent {
            action: MouseAction::Press(MouseButton::Left),
            x: 100.0,
            y: 200.0,
            display_id: 1,
        };
        injector.inject_mouse(event).unwrap();

        let events = injector.events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            InputEvent::Mouse(m) => {
                assert_eq!(m.x, 100.0);
                assert_eq!(m.y, 200.0);
            }
            _ => panic!("expected mouse event"),
        }
    }

    #[test]
    fn mock_input_injector_records_key() {
        let injector = MockInputInjector::new();
        let event = KeyEvent {
            key_code: 0x04, // 'a'
            pressed: true,
            modifiers: KeyModifiers::default(),
        };
        injector.inject_key(event).unwrap();

        let events = injector.events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            InputEvent::Key(k) => {
                assert_eq!(k.key_code, 0x04);
                assert!(k.pressed);
            }
            _ => panic!("expected key event"),
        }
    }

    #[test]
    fn mock_input_injector_records_scroll() {
        let injector = MockInputInjector::new();
        let event = ScrollEvent {
            delta_x: 0.0,
            delta_y: -3.0,
            is_precise: false,
        };
        injector.inject_scroll(event).unwrap();

        let events = injector.events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            InputEvent::Scroll(s) => {
                assert_eq!(s.delta_y, -3.0);
            }
            _ => panic!("expected scroll event"),
        }
    }

    #[test]
    fn mock_input_injector_clear() {
        let injector = MockInputInjector::new();
        injector
            .inject_mouse(MouseEvent {
                action: MouseAction::Move,
                x: 0.0,
                y: 0.0,
                display_id: 1,
            })
            .unwrap();
        assert_eq!(injector.events().len(), 1);

        injector.clear();
        assert!(injector.events().is_empty());
    }

    #[test]
    fn mock_input_injector_multiple_events() {
        let injector = MockInputInjector::new();
        injector
            .inject_mouse(MouseEvent {
                action: MouseAction::Move,
                x: 10.0,
                y: 20.0,
                display_id: 1,
            })
            .unwrap();
        injector
            .inject_key(KeyEvent {
                key_code: 0x1E,
                pressed: true,
                modifiers: KeyModifiers {
                    control: true,
                    ..KeyModifiers::default()
                },
            })
            .unwrap();
        injector
            .inject_scroll(ScrollEvent {
                delta_x: 0.0,
                delta_y: 1.0,
                is_precise: true,
            })
            .unwrap();

        let events = injector.events();
        assert_eq!(events.len(), 3);
        assert!(matches!(&events[0], InputEvent::Mouse(_)));
        assert!(matches!(&events[1], InputEvent::Key(_)));
        assert!(matches!(&events[2], InputEvent::Scroll(_)));
    }

    // -- MockAudioCapture ---------------------------------------------------

    #[test]
    fn mock_audio_capture_emits_chunk() {
        let cap = MockAudioCapture::new();
        let config = AudioConfig {
            sample_rate: 48_000,
            channels: 2,
            sample_format: AudioSampleFormat::F32,
        };
        let rx = cap.start(config).unwrap();

        assert!(cap.is_capturing());

        let chunk = rx.recv().unwrap();
        // 48000 / 100 = 480 samples, * 2 channels * 4 bytes = 3840 bytes
        assert_eq!(chunk.data.len(), 3840);
        assert_eq!(chunk.config.sample_rate, 48_000);
        assert_eq!(chunk.timestamp_us, 0);
    }

    #[test]
    fn mock_audio_capture_i16_format() {
        let cap = MockAudioCapture::new();
        let config = AudioConfig {
            sample_rate: 44_100,
            channels: 1,
            sample_format: AudioSampleFormat::I16,
        };
        let rx = cap.start(config).unwrap();

        let chunk = rx.recv().unwrap();
        // 44100 / 100 = 441 samples, * 1 channel * 2 bytes = 882 bytes
        assert_eq!(chunk.data.len(), 882);
    }

    #[test]
    fn mock_audio_capture_stop() {
        let cap = MockAudioCapture::new();
        assert!(!cap.is_capturing());

        let _rx = cap.start(AudioConfig::default()).unwrap();
        assert!(cap.is_capturing());

        cap.stop().unwrap();
        assert!(!cap.is_capturing());
    }

    #[test]
    fn mock_audio_capture_devices() {
        let cap = MockAudioCapture::new();
        let devices = cap.devices().unwrap();
        assert_eq!(devices.len(), 2);
        assert!(!devices[0].is_input);
        assert!(devices[1].is_input);
        assert!(devices[0].is_default);
        assert!(devices[1].is_default);
    }

    // -- MockSystemNotify ---------------------------------------------------

    #[test]
    fn mock_system_notify_records_notification() {
        let notify = MockSystemNotify::new();
        notify.show_notification("Title", "Body").unwrap();

        let records = notify.records();
        assert_eq!(records.len(), 1);
        match &records[0] {
            NotificationRecord::Notification { title, body } => {
                assert_eq!(title, "Title");
                assert_eq!(body, "Body");
            }
            _ => panic!("expected notification record"),
        }
    }

    #[test]
    fn mock_system_notify_records_tray_status() {
        let notify = MockSystemNotify::new();
        notify.set_tray_status(TrayStatus::Connected).unwrap();

        let records = notify.records();
        assert_eq!(records.len(), 1);
        match &records[0] {
            NotificationRecord::TrayStatus(status) => {
                assert_eq!(*status, TrayStatus::Connected);
            }
            _ => panic!("expected tray status record"),
        }
    }

    #[test]
    fn mock_system_notify_records_sound() {
        let notify = MockSystemNotify::new();
        notify.play_sound(SystemSound::Connected).unwrap();

        let records = notify.records();
        assert_eq!(records.len(), 1);
        match &records[0] {
            NotificationRecord::Sound(sound) => {
                assert_eq!(*sound, SystemSound::Connected);
            }
            _ => panic!("expected sound record"),
        }
    }

    #[test]
    fn mock_system_notify_clear() {
        let notify = MockSystemNotify::new();
        notify.show_notification("A", "B").unwrap();
        notify.set_tray_status(TrayStatus::Idle).unwrap();
        assert_eq!(notify.records().len(), 2);

        notify.clear();
        assert!(notify.records().is_empty());
    }

    #[test]
    fn mock_system_notify_multiple_operations() {
        let notify = MockSystemNotify::new();
        notify.show_notification("Connected", "Session started").unwrap();
        notify.set_tray_status(TrayStatus::Connected).unwrap();
        notify.play_sound(SystemSound::Connected).unwrap();
        notify.set_tray_status(TrayStatus::Idle).unwrap();

        let records = notify.records();
        assert_eq!(records.len(), 4);
        assert!(matches!(&records[0], NotificationRecord::Notification { .. }));
        assert!(matches!(&records[1], NotificationRecord::TrayStatus(_)));
        assert!(matches!(&records[2], NotificationRecord::Sound(_)));
        assert!(matches!(&records[3], NotificationRecord::TrayStatus(_)));
    }

    // -- MockClipboard ------------------------------------------------------

    #[test]
    fn mock_clipboard_get_set() {
        let clip = MockClipboard::new();
        assert_eq!(clip.get_text().unwrap(), "");

        clip.set_text("hello world").unwrap();
        assert_eq!(clip.get_text().unwrap(), "hello world");

        clip.set_text("updated").unwrap();
        assert_eq!(clip.get_text().unwrap(), "updated");
    }

    #[test]
    fn mock_clipboard_overwrite() {
        let clip = MockClipboard::new();
        clip.set_text("first").unwrap();
        clip.set_text("second").unwrap();
        assert_eq!(clip.get_text().unwrap(), "second");
    }

    // -- Object safety ------------------------------------------------------

    #[test]
    fn all_mocks_are_object_safe() {
        // Verify all mocks can be used as trait objects (dyn Trait).
        let capture: Box<dyn ScreenCapture> = Box::new(MockScreenCapture::new());
        let input: Box<dyn InputInjector> = Box::new(MockInputInjector::new());
        let audio: Box<dyn AudioCapture> = Box::new(MockAudioCapture::new());
        let notify: Box<dyn SystemNotify> = Box::new(MockSystemNotify::new());
        let clipboard: Box<dyn ClipboardProvider> = Box::new(MockClipboard::new());

        // Verify we can call trait methods through trait objects.
        assert!(!capture.is_capturing());
        assert!(capture.displays().is_ok());

        // InputInjector trait methods return Result — just call one.
        assert!(input
            .inject_mouse(MouseEvent {
                action: MouseAction::Move,
                x: 0.0,
                y: 0.0,
                display_id: 1,
            })
            .is_ok());

        assert!(audio.devices().is_ok());

        assert!(notify.show_notification("test", "test").is_ok());

        assert_eq!(clipboard.get_text().unwrap(), "");
    }
}
