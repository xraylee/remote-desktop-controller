// Copyright 2024 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! macOS input injection using CGEvent APIs.
//!
//! On macOS, uses `CGEventCreateMouseEvent`, `CGEventCreateKeyboardEvent`, and
//! `CGEventCreateScrollWheelEvent` via the `core-graphics` crate. On non-macOS
//! platforms, all methods return `PlatformError::NotSupported`.

use rdcs_platform::{InputInjector, KeyEvent, MouseEvent, PlatformError, ScrollEvent};

/// macOS input injection using CGEvent APIs.
///
/// Injects mouse, keyboard, and scroll events into the macOS HID event
/// stream via `CGEventPost(kCGHIDEventTap, event)`. Requires accessibility
/// permission to be granted.
///
/// On non-macOS platforms, all methods return `NotSupported`.
#[derive(Debug)]
pub struct MacOsInputInjector;

impl MacOsInputInjector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOsInputInjector {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// macOS implementation using core-graphics CGEvent
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;
    use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventType, CGMouseButton};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use core_graphics::geometry::CGPoint;

    /// Create a shared HID event source for input injection.
    fn create_event_source() -> Result<CGEventSource, PlatformError> {
        CGEventSource::new(CGEventSourceStateID::HIDSystemState).map_err(|_| {
            PlatformError::ApiError("failed to create CGEventSource".into())
        })
    }

    /// Map a mouse action to the corresponding CGEventType and optional button.
    fn to_cg_event_type(
        action: &rdcs_platform::MouseAction,
    ) -> (CGEventType, Option<CGMouseButton>) {
        use rdcs_platform::{MouseAction, MouseButton};
        match action {
            MouseAction::Move => (CGEventType::MouseMoved, None),
            MouseAction::Press(MouseButton::Left) => {
                (CGEventType::LeftMouseDown, Some(CGMouseButton::Left))
            }
            MouseAction::Press(MouseButton::Right) => {
                (CGEventType::RightMouseDown, Some(CGMouseButton::Right))
            }
            MouseAction::Press(MouseButton::Middle) => {
                (CGEventType::OtherMouseDown, Some(CGMouseButton::Center))
            }
            MouseAction::Release(MouseButton::Left) => {
                (CGEventType::LeftMouseUp, Some(CGMouseButton::Left))
            }
            MouseAction::Release(MouseButton::Right) => {
                (CGEventType::RightMouseUp, Some(CGMouseButton::Right))
            }
            MouseAction::Release(MouseButton::Middle) => {
                (CGEventType::OtherMouseUp, Some(CGMouseButton::Center))
            }
            MouseAction::DoubleClick(MouseButton::Left) => {
                (CGEventType::LeftMouseDown, Some(CGMouseButton::Left))
            }
            MouseAction::DoubleClick(MouseButton::Right) => {
                (CGEventType::RightMouseDown, Some(CGMouseButton::Right))
            }
            MouseAction::DoubleClick(MouseButton::Middle) => {
                (CGEventType::OtherMouseDown, Some(CGMouseButton::Center))
            }
        }
    }

    impl InputInjector for MacOsInputInjector {
        fn inject_mouse(&self, event: MouseEvent) -> Result<(), PlatformError> {
            let source = create_event_source()?;
            let point = CGPoint::new(event.x, event.y);
            let (event_type, button) = to_cg_event_type(&event.action);
            let cg_button = button.unwrap_or(CGMouseButton::Left);

            let cg_event = CGEvent::new_mouse_event(source, event_type, point, cg_button)
                .map_err(|_| PlatformError::ApiError("failed to create mouse CGEvent".into()))?;

            cg_event.post(CGEventTapLocation::HID);
            Ok(())
        }

        fn inject_key(&self, event: KeyEvent) -> Result<(), PlatformError> {
            let source = create_event_source()?;

            // Note: key_code uses USB HID usage codes in the platform trait.
            // macOS CGEvent expects virtual key codes. A full implementation
            // would include a HID-to-virtual-key mapping table. For now we
            // pass the raw code truncated to u16, which works for common
            // ASCII-mapped keys.
            let key_code = event.key_code as u16;
            let cg_event = CGEvent::new_keyboard_event(source, key_code, event.pressed)
                .map_err(|_| {
                    PlatformError::ApiError("failed to create keyboard CGEvent".into())
                })?;

            // Set modifier flags if any modifiers are active.
            let mut flags: u64 = 0;
            if event.modifiers.shift {
                flags |= 0x0002_0000; // kCGEventFlagMaskShift
            }
            if event.modifiers.control {
                flags |= 0x0004_0000; // kCGEventFlagMaskControl
            }
            if event.modifiers.alt {
                flags |= 0x0008_0000; // kCGEventFlagMaskAlternate
            }
            if event.modifiers.meta {
                flags |= 0x0010_0000; // kCGEventFlagMaskCommand
            }
            if flags > 0 {
                cg_event.set_flags(
                    core_graphics::event::CGEventFlags::from_bits_truncate(flags),
                );
            }

            cg_event.post(CGEventTapLocation::HID);
            Ok(())
        }

        fn inject_scroll(&self, event: ScrollEvent) -> Result<(), PlatformError> {
            // CGEventCreateScrollWheelEvent and related functions are called
            // via FFI because core-graphics 0.24 does not expose them directly.
            extern "C" {
                fn CGEventSourceCreate(state_id: i32) -> *mut std::ffi::c_void;
                fn CGEventCreateScrollWheelEvent(
                    source: *const std::ffi::c_void,
                    units: u32,
                    wheel_count: u32,
                    wheel1: i32,
                    wheel2: i32,
                    wheel3: i32,
                ) -> *mut std::ffi::c_void;
                fn CGEventPost(tap: u32, event: *mut std::ffi::c_void);
                fn CFRelease(cf: *mut std::ffi::c_void);
            }

            // kCGEventSourceStateHIDSystemState = 1
            // kCGScrollEventUnitPixel = 0, kCGScrollEventUnitLine = 1
            // kCGHIDEventTap = 0
            let unit: u32 = if event.is_precise { 0 } else { 1 };

            unsafe {
                let raw_source = CGEventSourceCreate(1);
                if raw_source.is_null() {
                    return Err(PlatformError::ApiError(
                        "failed to create CGEventSource".into(),
                    ));
                }

                let raw = CGEventCreateScrollWheelEvent(
                    raw_source,
                    unit,
                    2, // Two scroll axes.
                    event.delta_y as i32,
                    event.delta_x as i32,
                    0,
                );

                CFRelease(raw_source);

                if raw.is_null() {
                    return Err(PlatformError::ApiError(
                        "failed to create scroll CGEvent".into(),
                    ));
                }

                CGEventPost(0, raw);
                CFRelease(raw);
            }

            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Non-macOS fallback: all methods return NotSupported
// ---------------------------------------------------------------------------

#[cfg(not(target_os = "macos"))]
impl InputInjector for MacOsInputInjector {
    fn inject_mouse(&self, _event: MouseEvent) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS input injection requires target_os = \"macos\"".into(),
        ))
    }

    fn inject_key(&self, _event: KeyEvent) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS input injection requires target_os = \"macos\"".into(),
        ))
    }

    fn inject_scroll(&self, _event: ScrollEvent) -> Result<(), PlatformError> {
        Err(PlatformError::NotSupported(
            "macOS input injection requires target_os = \"macos\"".into(),
        ))
    }
}
