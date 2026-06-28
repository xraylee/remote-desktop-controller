// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Packet format definitions for the RDCS transport protocol.
//!
//! Wire format (22 bytes header):
//! ```text
//! | Magic (2B) | Version (1B) | Type (1B) | SessionID (8B) | Seq (4B) | Timestamp (4B) | PayloadLen (2B) | Payload |
//! ```

use bytes::{Bytes, BytesMut};

/// Protocol magic bytes for packet identification.
pub const MAGIC: [u8; 2] = [0x52, 0x43]; // "RC"

/// Current protocol version.
pub const VERSION: u8 = 1;

/// Maximum packet payload size in bytes.
pub const MAX_PACKET_SIZE: usize = 1200;

/// Packet header size in bytes.
pub const HEADER_SIZE: usize = 22;

/// Packet type identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PacketType {
    /// Data packet carrying application payload.
    Data = 0x01,
    /// Acknowledgment for received packets.
    Ack = 0x02,
    /// Negative acknowledgment requesting retransmission.
    Nack = 0x03,
    /// Forward error correction repair packet.
    FecRepair = 0x04,
    /// Heartbeat / keep-alive probe.
    Heartbeat = 0x05,
    /// Channel control (open, close, configure).
    Control = 0x06,
}

impl TryFrom<u8> for PacketType {
    type Error = crate::TransportError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::Data),
            0x02 => Ok(Self::Ack),
            0x03 => Ok(Self::Nack),
            0x04 => Ok(Self::FecRepair),
            0x05 => Ok(Self::Heartbeat),
            0x06 => Ok(Self::Control),
            _ => Err(crate::TransportError::PacketError(format!(
                "unknown packet type: {value:#x}"
            ))),
        }
    }
}

/// Packet header containing protocol metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketHeader {
    /// Protocol version (must be [`VERSION`]).
    pub version: u8,
    /// Packet type identifier.
    pub packet_type: PacketType,
    /// Session identifier for the connection.
    pub session_id: u64,
    /// Sequence number within the session.
    pub sequence: u32,
    /// Timestamp in microseconds (for RTT estimation).
    pub timestamp: u32,
    /// Length of the payload in bytes.
    pub payload_len: u16,
}

/// Encode a packet from header and payload into a byte vector.
///
/// The encoded format is:
/// `| Magic(2) | Version(1) | Type(1) | SessionID(8) | Seq(4) | Timestamp(4) | PayloadLen(2) | Payload(N) |`
pub fn encode_packet(header: &PacketHeader, payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(HEADER_SIZE + payload.len());
    buf.extend_from_slice(&MAGIC);
    buf.push(header.version);
    buf.push(header.packet_type as u8);
    buf.extend_from_slice(&header.session_id.to_be_bytes());
    buf.extend_from_slice(&header.sequence.to_be_bytes());
    buf.extend_from_slice(&header.timestamp.to_be_bytes());
    buf.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    buf.extend_from_slice(payload);
    buf
}

/// Decode a packet from raw bytes, returning the header and a payload slice.
///
/// Validates magic bytes, protocol version, and buffer lengths.
pub fn decode_packet(raw: &[u8]) -> Result<(PacketHeader, &[u8]), crate::TransportError> {
    if raw.len() < HEADER_SIZE {
        return Err(crate::TransportError::PacketError(
            "buffer too short for header".into(),
        ));
    }
    if raw[0] != MAGIC[0] || raw[1] != MAGIC[1] {
        return Err(crate::TransportError::PacketError(
            "invalid magic bytes".into(),
        ));
    }
    let version = raw[2];
    if version != VERSION {
        return Err(crate::TransportError::PacketError(format!(
            "unsupported version: {version}"
        )));
    }
    let packet_type = PacketType::try_from(raw[3])?;
    let session_id = u64::from_be_bytes(raw[4..12].try_into().unwrap());
    let sequence = u32::from_be_bytes(raw[12..16].try_into().unwrap());
    let timestamp = u32::from_be_bytes(raw[16..20].try_into().unwrap());
    let payload_len = u16::from_be_bytes(raw[20..22].try_into().unwrap()) as usize;

    if raw.len() < HEADER_SIZE + payload_len {
        return Err(crate::TransportError::PacketError(
            "buffer too short for payload".into(),
        ));
    }

    let header = PacketHeader {
        version,
        packet_type,
        session_id,
        sequence,
        timestamp,
        payload_len: payload_len as u16,
    };
    let payload = &raw[HEADER_SIZE..HEADER_SIZE + payload_len];
    Ok((header, payload))
}

/// A complete transport protocol packet (convenience wrapper around header + payload).
#[derive(Debug, Clone)]
pub struct Packet {
    /// Packet header with protocol metadata.
    pub header: PacketHeader,
    /// Packet payload data.
    pub payload: Bytes,
}

impl Packet {
    /// Create a new data packet.
    pub fn new_data(session_id: u64, sequence: u32, timestamp: u32, payload: Bytes) -> Self {
        let header = PacketHeader {
            version: VERSION,
            packet_type: PacketType::Data,
            session_id,
            sequence,
            timestamp,
            payload_len: payload.len() as u16,
        };
        Self { header, payload }
    }

    /// Create a new packet with the specified type.
    pub fn new(
        packet_type: PacketType,
        session_id: u64,
        sequence: u32,
        payload: Bytes,
    ) -> Self {
        let header = PacketHeader {
            version: VERSION,
            packet_type,
            session_id,
            sequence,
            timestamp: 0,
            payload_len: payload.len() as u16,
        };
        Self { header, payload }
    }

    /// Serialize the packet into a byte buffer.
    pub fn encode(&self) -> BytesMut {
        BytesMut::from(&encode_packet(&self.header, &self.payload)[..])
    }

    /// Deserialize a packet from a byte slice.
    pub fn decode(buf: &[u8]) -> Result<Self, crate::TransportError> {
        let (header, payload_slice) = decode_packet(buf)?;
        Ok(Self {
            header,
            payload: Bytes::copy_from_slice(payload_slice),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_roundtrip() {
        let header = PacketHeader {
            version: VERSION,
            packet_type: PacketType::Data,
            session_id: 0xDEAD_BEEF_CAFE_BABE,
            sequence: 42,
            timestamp: 12345,
            payload_len: 5,
        };
        let payload = b"hello";
        let encoded = encode_packet(&header, payload);
        let (decoded_header, decoded_payload) =
            decode_packet(&encoded).expect("decode should succeed");

        assert_eq!(decoded_header.version, VERSION);
        assert_eq!(decoded_header.packet_type, PacketType::Data);
        assert_eq!(decoded_header.session_id, 0xDEAD_BEEF_CAFE_BABE);
        assert_eq!(decoded_header.sequence, 42);
        assert_eq!(decoded_header.timestamp, 12345);
        assert_eq!(decoded_header.payload_len, 5);
        assert_eq!(decoded_payload, b"hello");
    }

    #[test]
    fn packet_struct_roundtrip() {
        let pkt = Packet::new_data(1, 42, 999, Bytes::from_static(b"hello"));
        let encoded = pkt.encode();
        let decoded = Packet::decode(&encoded).expect("decode should succeed");

        assert_eq!(decoded.header.packet_type, PacketType::Data);
        assert_eq!(decoded.header.session_id, 1);
        assert_eq!(decoded.header.sequence, 42);
        assert_eq!(decoded.header.timestamp, 999);
        assert_eq!(decoded.payload.as_ref(), b"hello");
    }

    #[test]
    fn reject_short_buffer() {
        let result = decode_packet(&[0x52, 0x43]);
        assert!(result.is_err());
    }

    #[test]
    fn reject_bad_magic() {
        let mut buf = vec![0u8; HEADER_SIZE];
        buf[0] = 0xFF;
        buf[1] = 0xFF;
        let result = decode_packet(&buf);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("magic"));
    }

    #[test]
    fn reject_bad_version() {
        let mut buf = vec![0u8; HEADER_SIZE];
        buf[0] = MAGIC[0];
        buf[1] = MAGIC[1];
        buf[2] = 99; // bad version
        let result = decode_packet(&buf);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    #[test]
    fn reject_truncated_payload() {
        let header = PacketHeader {
            version: VERSION,
            packet_type: PacketType::Data,
            session_id: 0,
            sequence: 0,
            timestamp: 0,
            payload_len: 100,
        };
        let mut encoded = encode_packet(&header, &[0u8; 100]);
        encoded.truncate(HEADER_SIZE + 10); // truncate payload
        let result = decode_packet(&encoded);
        assert!(result.is_err());
    }

    #[test]
    fn all_packet_types_roundtrip() {
        let types = [
            PacketType::Data,
            PacketType::Ack,
            PacketType::Nack,
            PacketType::FecRepair,
            PacketType::Heartbeat,
            PacketType::Control,
        ];
        for pt in types {
            let header = PacketHeader {
                version: VERSION,
                packet_type: pt,
                session_id: 1,
                sequence: 0,
                timestamp: 0,
                payload_len: 0,
            };
            let encoded = encode_packet(&header, &[]);
            let (decoded, _) = decode_packet(&encoded).unwrap();
            assert_eq!(decoded.packet_type, pt);
        }
    }

    #[test]
    fn empty_payload_roundtrip() {
        let header = PacketHeader {
            version: VERSION,
            packet_type: PacketType::Heartbeat,
            session_id: 42,
            sequence: 7,
            timestamp: 100,
            payload_len: 0,
        };
        let encoded = encode_packet(&header, &[]);
        assert_eq!(encoded.len(), HEADER_SIZE);
        let (decoded, payload) = decode_packet(&encoded).unwrap();
        assert_eq!(decoded, header);
        assert!(payload.is_empty());
    }

    #[test]
    fn large_payload_roundtrip() {
        let data: Vec<u8> = (0..MAX_PACKET_SIZE).map(|i| (i % 256) as u8).collect();
        let header = PacketHeader {
            version: VERSION,
            packet_type: PacketType::Data,
            session_id: 999,
            sequence: 0,
            timestamp: 0,
            payload_len: data.len() as u16,
        };
        let encoded = encode_packet(&header, &data);
        let (decoded, payload) = decode_packet(&encoded).unwrap();
        assert_eq!(decoded.payload_len as usize, MAX_PACKET_SIZE);
        assert_eq!(payload, data.as_slice());
    }

    #[test]
    fn packet_type_try_from_invalid() {
        assert!(PacketType::try_from(0x00).is_err());
        assert!(PacketType::try_from(0x07).is_err());
        assert!(PacketType::try_from(0xFF).is_err());
    }
}
