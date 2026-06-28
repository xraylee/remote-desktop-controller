// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! rdcs-connection: Connection management for the RDCS remote desktop system.
//!
//! Handles peer discovery (mDNS), NAT traversal (ICE), network path selection,
//! heartbeat monitoring, and automatic reconnection.

pub mod heartbeat;
pub mod ice;
pub mod mdns;
pub mod path;
pub mod reconnect;
pub mod real_ice_agent;
pub mod video_channel;
pub mod frame_reassembler;

use thiserror::Error;

/// Connection-layer errors.
#[derive(Debug, Error)]
pub enum ConnectionError {
    /// No viable network path was found.
    #[error("no viable path")]
    NoViablePath,

    /// The remote peer is unreachable.
    #[error("peer unreachable: {0}")]
    PeerUnreachable(String),

    /// Connection timed out.
    #[error("connection timed out")]
    TimedOut,

    /// Operation timeout.
    #[error("timeout")]
    Timeout,

    /// The connection was reset by the peer.
    #[error("connection reset")]
    Reset,

    /// ICE gathering or connectivity check failed.
    #[error("ice error: {0}")]
    IceError(String),

    /// An I/O error occurred.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience type alias for results in this crate.
pub type Result<T> = std::result::Result<T, ConnectionError>;

// Re-export primary trait interfaces for ergonomic use.
pub use heartbeat::HeartbeatManager;
pub use ice::{IceAgent, StubIceAgent};
pub use mdns::MdnsDiscovery;
pub use path::PathSelector;
pub use real_ice_agent::RealIceAgent;
pub use reconnect::ReconnectStrategy;
pub use video_channel::VideoChannel;
pub use frame_reassembler::{FrameReassembler, FrameHeader, FrameError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = ConnectionError::NoViablePath;
        assert_eq!(err.to_string(), "no viable path");
    }

    #[test]
    fn error_peer_unreachable() {
        let err = ConnectionError::PeerUnreachable("192.168.1.1".into());
        assert!(err.to_string().contains("192.168.1.1"));
    }

    #[test]
    fn error_timed_out() {
        let err = ConnectionError::TimedOut;
        assert_eq!(err.to_string(), "connection timed out");
    }

    #[test]
    fn error_reset() {
        let err = ConnectionError::Reset;
        assert_eq!(err.to_string(), "connection reset");
    }

    #[test]
    fn error_ice() {
        let err = ConnectionError::IceError("STUN timeout".into());
        assert!(err.to_string().contains("STUN timeout"));
    }
}
