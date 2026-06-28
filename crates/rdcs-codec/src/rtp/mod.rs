// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! RTP/SRTP integration layer for video codec transport.
//!
//! This module provides H.264 RTP packetization (RFC 6184) and SRTP encryption
//! for secure video transport over WebRTC.
//!
//! # Architecture
//!
//! ```text
//! Encoder → H.264 NAL units → RTP Packetizer → SRTP → Network
//!                                                ↓
//! Decoder ← H.264 NAL units ← RTP Depacketizer ← SRTP ← Network
//! ```
//!
//! # Usage
//!
//! ## Packetization (Sender)
//!
//! ```no_run
//! use rdcs_codec::rtp::{H264Packetizer, PacketizerConfig};
//!
//! let config = PacketizerConfig {
//!     mtu: 1200,
//!     clock_rate: 90000,
//!     ssrc: 0x12345678,
//! };
//!
//! let mut packetizer = H264Packetizer::new(config);
//!
//! // Encode and packetize
//! let h264_data = encoder.encode(&frame)?;
//! let rtp_packets = packetizer.packetize(&h264_data)?;
//!
//! // Send packets
//! for packet in rtp_packets {
//!     send_to_network(packet.as_bytes())?;
//! }
//! ```
//!
//! ## Depacketization (Receiver)
//!
//! ```no_run
//! use rdcs_codec::rtp::H264Depacketizer;
//!
//! let mut depacketizer = H264Depacketizer::new();
//!
//! // Receive RTP packets
//! while let Some(packet) = receive_from_network()? {
//!     if let Some(h264_data) = depacketizer.depacketize(&packet)? {
//!         // Complete frame received, decode it
//!         let frame = decoder.decode(&h264_data)?;
//!         render(frame);
//!     }
//! }
//! ```

pub mod depacketizer;
pub mod packetizer;
pub mod srtp;

pub use depacketizer::{H264Depacketizer, DepacketizerStats};
pub use packetizer::{H264Packetizer, PacketizerConfig, PacketizerStats};
pub use srtp::{SrtpContext, SrtpConfig};

use thiserror::Error;

/// RTP module errors.
#[derive(Debug, Error)]
pub enum RtpError {
    /// Invalid RTP packet format.
    #[error("invalid RTP packet: {0}")]
    InvalidPacket(String),

    /// NAL unit parsing error.
    #[error("NAL unit error: {0}")]
    NalUnitError(String),

    /// Packet is too large for configured MTU.
    #[error("packet too large: {size} bytes exceeds MTU {mtu}")]
    PacketTooLarge { size: usize, mtu: usize },

    /// SRTP encryption/decryption error.
    #[error("SRTP error: {0}")]
    SrtpError(String),

    /// Sequence number gap detected (potential packet loss).
    #[error("sequence gap: expected {expected}, got {actual}")]
    SequenceGap { expected: u16, actual: u16 },

    /// Fragmentation error.
    #[error("fragmentation error: {0}")]
    FragmentationError(String),

    /// WebRTC library error.
    #[error("webrtc error: {0}")]
    WebRtcError(#[from] webrtc::Error),
}

/// Result type for RTP operations.
pub type Result<T> = std::result::Result<T, RtpError>;

/// RTP packet header (12 bytes fixed header).
#[derive(Debug, Clone, Copy)]
pub struct RtpHeader {
    /// Version (V): 2 bits, must be 2 for RTP.
    pub version: u8,
    /// Padding (P): 1 bit, indicates padding bytes at the end.
    pub padding: bool,
    /// Extension (X): 1 bit, indicates header extension.
    pub extension: bool,
    /// CSRC count (CC): 4 bits, number of CSRC identifiers.
    pub csrc_count: u8,
    /// Marker (M): 1 bit, marks significant events (e.g., end of frame).
    pub marker: bool,
    /// Payload type (PT): 7 bits, codec identifier.
    pub payload_type: u8,
    /// Sequence number: 16 bits, increments by 1 for each packet.
    pub sequence_number: u16,
    /// Timestamp: 32 bits, sampling instant (90kHz for video).
    pub timestamp: u32,
    /// SSRC: 32 bits, synchronization source identifier.
    pub ssrc: u32,
}

impl RtpHeader {
    /// Size of the fixed RTP header in bytes.
    pub const SIZE: usize = 12;

    /// Parse RTP header from raw bytes.
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < Self::SIZE {
            return Err(RtpError::InvalidPacket(format!(
                "packet too short: {} bytes, expected at least {}",
                data.len(),
                Self::SIZE
            )));
        }

        let version = (data[0] >> 6) & 0x03;
        if version != 2 {
            return Err(RtpError::InvalidPacket(format!(
                "invalid RTP version: {}, expected 2",
                version
            )));
        }

        Ok(Self {
            version,
            padding: (data[0] & 0x20) != 0,
            extension: (data[0] & 0x10) != 0,
            csrc_count: data[0] & 0x0F,
            marker: (data[1] & 0x80) != 0,
            payload_type: data[1] & 0x7F,
            sequence_number: u16::from_be_bytes([data[2], data[3]]),
            timestamp: u32::from_be_bytes([data[4], data[5], data[6], data[7]]),
            ssrc: u32::from_be_bytes([data[8], data[9], data[10], data[11]]),
        })
    }

    /// Serialize RTP header to bytes.
    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];

        buf[0] = (self.version << 6)
            | (if self.padding { 0x20 } else { 0 })
            | (if self.extension { 0x10 } else { 0 })
            | self.csrc_count;

        buf[1] = (if self.marker { 0x80 } else { 0 }) | self.payload_type;

        buf[2..4].copy_from_slice(&self.sequence_number.to_be_bytes());
        buf[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[8..12].copy_from_slice(&self.ssrc.to_be_bytes());

        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtp_header_roundtrip() {
        let header = RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker: true,
            payload_type: 96,
            sequence_number: 12345,
            timestamp: 1234567890,
            ssrc: 0x12345678,
        };

        let bytes = header.to_bytes();
        let parsed = RtpHeader::from_bytes(&bytes).unwrap();

        assert_eq!(parsed.version, header.version);
        assert_eq!(parsed.marker, header.marker);
        assert_eq!(parsed.payload_type, header.payload_type);
        assert_eq!(parsed.sequence_number, header.sequence_number);
        assert_eq!(parsed.timestamp, header.timestamp);
        assert_eq!(parsed.ssrc, header.ssrc);
    }

    #[test]
    fn test_invalid_version() {
        let mut bytes = [0u8; 12];
        bytes[0] = 0x00; // version = 0

        let result = RtpHeader::from_bytes(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid RTP version"));
    }

    #[test]
    fn test_packet_too_short() {
        let bytes = [0u8; 5];
        let result = RtpHeader::from_bytes(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("packet too short"));
    }
}
