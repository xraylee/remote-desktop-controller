// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! rdcs-transport: Reliable transport layer over UDP for the RDCS remote desktop system.
//!
//! Implements a custom reliable transport protocol on top of UDP, featuring
//! packet sequencing, selective NACK retransmission, forward error correction
//! (FEC), congestion control, and logical channels.

pub mod channel;
pub mod congestion;
pub mod fec;
pub mod nack;
pub mod packet;
pub mod sequencer;

// Phase 2: 简单 TCP 视频传输
pub mod tcp_video;

// Re-export key types for convenience.
pub use packet::{decode_packet, encode_packet, Packet, PacketHeader, PacketType};

use thiserror::Error;

/// Transport-layer errors.
#[derive(Debug, Error)]
pub enum TransportError {
    /// A packet failed validation (bad checksum, out-of-order beyond window).
    #[error("packet error: {0}")]
    PacketError(String),

    /// Congestion control triggered a send pause.
    #[error("congestion: {0}")]
    Congestion(String),

    /// The transport channel was closed.
    #[error("channel closed")]
    ChannelClosed,

    /// Timed out waiting for acknowledgment.
    #[error("timeout")]
    Timeout,

    /// An I/O error occurred on the underlying UDP socket.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl PartialEq for TransportError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::PacketError(a), Self::PacketError(b)) => a == b,
            (Self::Congestion(a), Self::Congestion(b)) => a == b,
            (Self::ChannelClosed, Self::ChannelClosed) => true,
            (Self::Timeout, Self::Timeout) => true,
            (Self::Io(a), Self::Io(b)) => a.kind() == b.kind(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = TransportError::Timeout;
        assert_eq!(err.to_string(), "timeout");
    }

    #[test]
    fn error_equality() {
        assert_eq!(TransportError::Timeout, TransportError::Timeout);
        assert_eq!(TransportError::ChannelClosed, TransportError::ChannelClosed);
        assert_eq!(
            TransportError::PacketError("bad".into()),
            TransportError::PacketError("bad".into())
        );
        assert_ne!(
            TransportError::PacketError("a".into()),
            TransportError::PacketError("b".into())
        );
        assert_ne!(TransportError::Timeout, TransportError::ChannelClosed);
    }
}
