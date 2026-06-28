// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Device registration, heartbeat, disconnect, connection negotiation,
//! relay node allocation, offline detection, and invite code handlers.
//!
//! Each handler implements a specific phase of the device or session lifecycle:
//!
//! - [`register::handle_register`] — stores device online status in Redis,
//!   adds the device to its team's online set, and broadcasts a
//!   `nearby_update` to same-team devices.
//! - [`heartbeat::handle_heartbeat`] — refreshes the Redis TTL for a
//!   device's online key and detects offline peers.
//! - [`disconnect::handle_disconnect`] — removes the device's online key
//!   and team-set membership, then broadcasts an offline notification.
//! - [`connect`] — connection negotiation: ICE offer/answer relay and session
//!   creation tracking.
//! - [`relay`] — relay node allocation with HMAC token generation when P2P
//!   hole-punching fails.
//! - [`offline`] — offline detection via Redis keyspace notifications;
//!   broadcasts `peer_offline` when device online keys expire.
//! - [`invite`] — invite code generation and consumption for device-to-device
//!   connection without knowing device codes.

pub mod connect;
pub mod disconnect;
pub mod heartbeat;
pub mod invite;
pub mod offline;
pub mod register;
pub mod relay;
