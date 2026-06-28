// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! WebSocket signaling module.
//!
//! This module provides:
//! - [`handler::ws_handler`] — axum handler that upgrades `GET /ws` to a
//!   WebSocket connection and runs the per-connection message loop.
//! - [`handler::AppState`] — shared application state (session manager +
//!   optional Redis pool) threaded through handlers via axum's `State`.
//! - [`message::WsMessage`] — the 10 signaling message types plus an error
//!   response, all JSON-serialized with a `"type"` discriminator field.
//! - [`session::SessionManager`] — thread-safe, in-memory registry of active
//!   WebSocket sessions keyed by device code.

pub mod handler;
pub mod message;
pub mod session;

// Re-export the most commonly items at the module level for convenience.
pub use handler::{ws_handler, AppState};
pub use message::WsMessage;
pub use session::SessionManager;
