//! Input event handling for FFI layer.
//!
//! Parses JSON input events from Flutter/Dart and converts them to
//! platform-agnostic input events that can be injected via InputInjector.

use rdcs_platform::{
    InputInjector, KeyEvent, KeyModifiers, MouseAction, MouseButton, MouseEvent, PlatformError,
    ScrollEvent,
};
use serde::{Deserialize, Serialize};

/// Top-level input event wrapper received from Flutter.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputEventJson {
    Mouse(MouseEventJson),
    Keyboard(KeyboardEventJson),
    Scroll(ScrollEventJson),
}

/// Mouse event from Flutter.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MouseEventJson {
    /// Action: "move", "click", "double_click", "press", "release"
    pub action: String,
    /// X coordinate in screen space
    pub x: f64,
    /// Y coordinate in screen space
    pub y: f64,
    /// Optional button: "left", "right", "middle"
    #[serde(default)]
    pub button: Option<String>,
}

/// Keyboard event from Flutter.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyboardEventJson {
    /// Key code (USB HID usage code or platform virtual key)
    pub key_code: u32,
    /// "press" or "release"
    pub action: String,
    /// Modifier flags
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub control: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub meta: bool,
}

/// Scroll event from Flutter.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScrollEventJson {
    /// Horizontal scroll delta
    pub delta_x: f64,
    /// Vertical scroll delta
    pub delta_y: f64,
    /// Whether the values are precise pixel deltas
    #[serde(default)]
    pub is_precise: bool,
}

/// Parse JSON input event and inject it using the provided InputInjector.
pub fn handle_input_event(
    injector: &dyn InputInjector,
    event_json: &str,
    display_id: u64,
) -> Result<(), PlatformError> {
    let event: InputEventJson = serde_json::from_str(event_json)
        .map_err(|e| PlatformError::ApiError(format!("failed to parse input JSON: {e}")))?;

    match event {
        InputEventJson::Mouse(mouse) => handle_mouse_event(injector, mouse, display_id),
        InputEventJson::Keyboard(keyboard) => handle_keyboard_event(injector, keyboard),
        InputEventJson::Scroll(scroll) => handle_scroll_event(injector, scroll),
    }
}

fn handle_mouse_event(
    injector: &dyn InputInjector,
    event: MouseEventJson,
    display_id: u64,
) -> Result<(), PlatformError> {
    let action = match event.action.as_str() {
        "move" => MouseAction::Move,
        "click" => {
            let button = parse_mouse_button(event.button.as_deref())?;
            // For click, send press then release
            let press_event = MouseEvent {
                action: MouseAction::Press(button),
                x: event.x,
                y: event.y,
                display_id,
            };
            injector.inject_mouse(press_event)?;

            let release_event = MouseEvent {
                action: MouseAction::Release(button),
                x: event.x,
                y: event.y,
                display_id,
            };
            return injector.inject_mouse(release_event);
        }
        "double_click" => {
            let button = parse_mouse_button(event.button.as_deref())?;
            MouseAction::DoubleClick(button)
        }
        "press" => {
            let button = parse_mouse_button(event.button.as_deref())?;
            MouseAction::Press(button)
        }
        "release" => {
            let button = parse_mouse_button(event.button.as_deref())?;
            MouseAction::Release(button)
        }
        _ => {
            return Err(PlatformError::ApiError(format!(
                "unknown mouse action: {}",
                event.action
            )))
        }
    };

    let mouse_event = MouseEvent {
        action,
        x: event.x,
        y: event.y,
        display_id,
    };

    injector.inject_mouse(mouse_event)
}

fn parse_mouse_button(button: Option<&str>) -> Result<MouseButton, PlatformError> {
    match button {
        None | Some("left") => Ok(MouseButton::Left),
        Some("right") => Ok(MouseButton::Right),
        Some("middle") => Ok(MouseButton::Middle),
        Some(other) => Err(PlatformError::ApiError(format!(
            "unknown mouse button: {other}"
        ))),
    }
}

fn handle_keyboard_event(
    injector: &dyn InputInjector,
    event: KeyboardEventJson,
) -> Result<(), PlatformError> {
    let pressed = match event.action.as_str() {
        "press" => true,
        "release" => false,
        _ => {
            return Err(PlatformError::ApiError(format!(
                "unknown keyboard action: {}",
                event.action
            )))
        }
    };

    let key_event = KeyEvent {
        key_code: event.key_code,
        pressed,
        modifiers: KeyModifiers {
            shift: event.shift,
            control: event.control,
            alt: event.alt,
            meta: event.meta,
        },
    };

    injector.inject_key(key_event)
}

fn handle_scroll_event(
    injector: &dyn InputInjector,
    event: ScrollEventJson,
) -> Result<(), PlatformError> {
    let scroll_event = ScrollEvent {
        delta_x: event.delta_x,
        delta_y: event.delta_y,
        is_precise: event.is_precise,
    };

    injector.inject_scroll(scroll_event)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdcs_platform::mock::MockInputInjector;

    #[test]
    fn parse_mouse_move() {
        let json = r#"{"type":"mouse","action":"move","x":100.0,"y":200.0}"#;
        let injector = MockInputInjector::new();
        assert!(handle_input_event(&injector, json, 1).is_ok());
    }

    #[test]
    fn parse_mouse_click() {
        let json = r#"{"type":"mouse","action":"click","x":50.0,"y":75.0,"button":"left"}"#;
        let injector = MockInputInjector::new();
        assert!(handle_input_event(&injector, json, 1).is_ok());
    }

    #[test]
    fn parse_keyboard_press() {
        let json = r#"{"type":"keyboard","key_code":4,"action":"press","shift":true}"#;
        let injector = MockInputInjector::new();
        assert!(handle_input_event(&injector, json, 1).is_ok());
    }

    #[test]
    fn parse_scroll() {
        let json = r#"{"type":"scroll","delta_x":0.0,"delta_y":10.0,"is_precise":true}"#;
        let injector = MockInputInjector::new();
        assert!(handle_input_event(&injector, json, 1).is_ok());
    }

    #[test]
    fn invalid_json_returns_error() {
        let json = r#"{"invalid"}"#;
        let injector = MockInputInjector::new();
        assert!(handle_input_event(&injector, json, 1).is_err());
    }

    #[test]
    fn unknown_mouse_action_returns_error() {
        let json = r#"{"type":"mouse","action":"unknown","x":0.0,"y":0.0}"#;
        let injector = MockInputInjector::new();
        assert!(handle_input_event(&injector, json, 1).is_err());
    }
}
