// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! RDCS relay binary protocol: message types, parser, and encoders.
//!
//! # Packet layout (16-byte header)
//!
//! ```text
//! Offset  Size  Field
//! 0..4    4     magic   — must equal [`MAGIC`] (b"RDCS")
//! 4       1     version — protocol version (currently 1)
//! 5       1     msg_type — 0x01=Allocate, 0x02=Release, 0x03=Keepalive, 0x04=Data
//! 6..8    2     reserved — must be zero
//! 8..16   8     session_id — little-endian u64
//! ```
//!
//! **Allocate** is followed by `[16..18] token_len (u16 LE) + token bytes`.
//! **Data** is followed by the raw payload bytes (rest of the datagram).

/// Magic bytes identifying an RDCS relay packet: ASCII `RDCS`.
pub const MAGIC: [u8; 4] = [0x52, 0x44, 0x43, 0x53];

/// Current protocol version.
pub const VERSION: u8 = 1;

/// Maximum allowed token length (bytes). Guards against malformed headers
/// that claim an unreasonably large token.
const MAX_TOKEN_LEN: u16 = 1024;

// Message type discriminators
const MSG_ALLOCATE: u8 = 0x01;
const MSG_RELEASE: u8 = 0x02;
const MSG_KEEPALIVE: u8 = 0x03;
const MSG_DATA: u8 = 0x04;

/// Header size in bytes.
pub const HEADER_LEN: usize = 16;

// ---------------------------------------------------------------------------
// Message types
// ---------------------------------------------------------------------------

/// Messages exchanged over the relay UDP control channel.
#[derive(Debug, Clone, PartialEq)]
pub enum RelayMessage {
    /// Request a relay port allocation, authenticated by an HMAC token.
    Allocate { session_id: u64, token: Vec<u8> },
    /// Release a previously allocated relay port.
    Release { session_id: u64 },
    /// Keep the allocation alive (heartbeat).
    Keepalive { session_id: u64 },
    /// Forward raw screen-capture data through the relay.
    /// The payload is everything after the 16-byte header.
    Data { session_id: u64 },
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur while parsing a relay packet.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ProtocolError {
    #[error("invalid magic bytes")]
    InvalidMagic,

    #[error("unknown message type: {0:#x}")]
    UnknownType(u8),

    #[error("packet too short: expected at least {expected}, got {actual}")]
    TooShort { expected: usize, actual: usize },

    #[error("invalid token length: {0}")]
    InvalidTokenLength(u16),
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// Parse a relay packet from `buf`.
///
/// Returns the parsed [`RelayMessage`] and a slice of the remaining bytes
/// (everything after the consumed portion of `buf`).
///
/// For [`RelayMessage::Data`] the "payload" is implicit: it is the slice of
/// bytes between the header and the end of `buf`. The returned remainder is
/// therefore always empty for Data messages.
pub fn parse_message(buf: &[u8]) -> Result<(RelayMessage, &[u8]), ProtocolError> {
    // --- header (16 bytes) ---------------------------------------------------
    if buf.len() < HEADER_LEN {
        return Err(ProtocolError::TooShort {
            expected: HEADER_LEN,
            actual: buf.len(),
        });
    }

    // magic
    if buf[0..4] != MAGIC {
        return Err(ProtocolError::InvalidMagic);
    }

    let msg_type = buf[5];
    let session_id = u64::from_le_bytes(
        buf[8..16]
            .try_into()
            .expect("slice is exactly 8 bytes"),
    );

    match msg_type {
        MSG_ALLOCATE => {
            // Need 16 (header) + 2 (token_len) = 18 bytes minimum.
            const MIN_ALLOCATE: usize = HEADER_LEN + 2;
            if buf.len() < MIN_ALLOCATE {
                return Err(ProtocolError::TooShort {
                    expected: MIN_ALLOCATE,
                    actual: buf.len(),
                });
            }
            let token_len = u16::from_le_bytes(
                buf[16..18]
                    .try_into()
                    .expect("slice is exactly 2 bytes"),
            );
            if token_len > MAX_TOKEN_LEN {
                return Err(ProtocolError::InvalidTokenLength(token_len));
            }
            let total = MIN_ALLOCATE + usize::from(token_len);
            if buf.len() < total {
                return Err(ProtocolError::TooShort {
                    expected: total,
                    actual: buf.len(),
                });
            }
            let token = buf[MIN_ALLOCATE..total].to_vec();
            Ok((RelayMessage::Allocate { session_id, token }, &buf[total..]))
        }

        MSG_RELEASE => Ok((RelayMessage::Release { session_id }, &buf[HEADER_LEN..])),

        MSG_KEEPALIVE => Ok((RelayMessage::Keepalive { session_id }, &buf[HEADER_LEN..])),

        MSG_DATA => {
            // Payload is everything after the header; no further length field.
            Ok((RelayMessage::Data { session_id }, &buf[buf.len()..]))
        }

        other => Err(ProtocolError::UnknownType(other)),
    }
}

// ---------------------------------------------------------------------------
// Encoders
// ---------------------------------------------------------------------------

/// Build the common 16-byte header.
fn build_header(msg_type: u8, session_id: u64) -> [u8; HEADER_LEN] {
    let mut hdr = [0u8; HEADER_LEN];
    hdr[0..4].copy_from_slice(&MAGIC);
    hdr[4] = VERSION;
    hdr[5] = msg_type;
    // bytes 6..8 are reserved (already zero)
    hdr[8..16].copy_from_slice(&session_id.to_le_bytes());
    hdr
}

/// Encode an [`Allocate`](RelayMessage::Allocate) packet.
pub fn encode_allocate(session_id: u64, token: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + 2 + token.len());
    out.extend_from_slice(&build_header(MSG_ALLOCATE, session_id));
    out.extend_from_slice(&(token.len() as u16).to_le_bytes());
    out.extend_from_slice(token);
    out
}

/// Encode a [`Release`](RelayMessage::Release) packet.
pub fn encode_release(session_id: u64) -> Vec<u8> {
    build_header(MSG_RELEASE, session_id).to_vec()
}

/// Encode a [`Keepalive`](RelayMessage::Keepalive) packet.
pub fn encode_keepalive(session_id: u64) -> Vec<u8> {
    build_header(MSG_KEEPALIVE, session_id).to_vec()
}

/// Encode a [`Data`](RelayMessage::Data) packet.
pub fn encode_data(session_id: u64, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(&build_header(MSG_DATA, session_id));
    out.extend_from_slice(payload);
    out
}

// ---------------------------------------------------------------------------
// Display helper for logging
// ---------------------------------------------------------------------------

impl RelayMessage {
    /// Short human-readable tag for the message variant (useful in log lines).
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Allocate { .. } => "ALLOCATE",
            Self::Release { .. } => "RELEASE",
            Self::Keepalive { .. } => "KEEPALIVE",
            Self::Data { .. } => "DATA",
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- test_parse_allocate --------------------------------------------------

    #[test]
    fn test_parse_allocate() {
        let session_id = 0xDEAD_BEEF_CAFE_BABEu64;
        let token = b"hmac-token-12345";
        let pkt = encode_allocate(session_id, token);

        let (msg, rest) = parse_message(&pkt).expect("parse should succeed");
        assert!(rest.is_empty(), "no bytes should remain after allocate");
        assert_eq!(
            msg,
            RelayMessage::Allocate {
                session_id,
                token: token.to_vec(),
            }
        );
    }

    #[test]
    fn test_parse_allocate_empty_token() {
        let pkt = encode_allocate(42, &[]);
        let (msg, rest) = parse_message(&pkt).expect("parse should succeed");
        assert!(rest.is_empty());
        assert_eq!(
            msg,
            RelayMessage::Allocate {
                session_id: 42,
                token: vec![],
            }
        );
    }

    #[test]
    fn test_parse_allocate_with_trailing_bytes() {
        // Extra bytes after the allocate message should be returned as remainder.
        let mut pkt = encode_allocate(1, b"tok");
        pkt.extend_from_slice(b"EXTRA");
        let (msg, rest) = parse_message(&pkt).expect("parse should succeed");
        assert_eq!(rest, b"EXTRA");
        assert_eq!(
            msg,
            RelayMessage::Allocate {
                session_id: 1,
                token: b"tok".to_vec(),
            }
        );
    }

    // -- test_parse_keepalive -------------------------------------------------

    #[test]
    fn test_parse_keepalive() {
        let session_id = 0x0123_4567_89AB_CDEFu64;
        let pkt = encode_keepalive(session_id);

        let (msg, rest) = parse_message(&pkt).expect("parse should succeed");
        assert!(rest.is_empty());
        assert_eq!(msg, RelayMessage::Keepalive { session_id });
    }

    #[test]
    fn test_parse_keepalive_rejects_short_packet() {
        // Only 10 bytes — not enough for a 16-byte header.
        let short = [0x52, 0x44, 0x43, 0x53, 0x01, 0x03, 0x00, 0x00, 0x00, 0x00];
        let err = parse_message(&short).expect_err("short packet should fail");
        assert_eq!(
            err,
            ProtocolError::TooShort {
                expected: HEADER_LEN,
                actual: 10,
            }
        );
    }

    #[test]
    fn test_parse_keepalive_with_trailing_bytes() {
        let mut pkt = encode_keepalive(7);
        pkt.extend_from_slice(b"\x00\x00"); // 2 extra bytes
        let (msg, rest) = parse_message(&pkt).expect("parse should succeed");
        assert_eq!(rest.len(), 2);
        assert_eq!(msg, RelayMessage::Keepalive { session_id: 7 });
    }

    // -- test_parse_data ------------------------------------------------------

    #[test]
    fn test_parse_data() {
        let session_id = 999u64;
        let payload = b"screen-capture-frame-data";
        let pkt = encode_data(session_id, payload);

        let (msg, _rest) = parse_message(&pkt).expect("parse should succeed");
        assert_eq!(msg, RelayMessage::Data { session_id });
        // Payload bytes live in pkt[HEADER_LEN..]
        assert_eq!(&pkt[HEADER_LEN..], payload);
    }

    #[test]
    fn test_parse_data_empty_payload() {
        let pkt = encode_data(1, &[]);
        let (msg, rest) = parse_message(&pkt).expect("parse should succeed");
        assert!(rest.is_empty());
        assert_eq!(msg, RelayMessage::Data { session_id: 1 });
    }

    #[test]
    fn test_parse_data_large_payload() {
        let payload = vec![0xABu8; 60_000];
        let pkt = encode_data(42, &payload);
        let (msg, _rest) = parse_message(&pkt).expect("parse should succeed");
        assert_eq!(msg, RelayMessage::Data { session_id: 42 });
        assert_eq!(&pkt[HEADER_LEN..], &payload[..]);
    }

    // -- test_encode_roundtrip ------------------------------------------------

    #[test]
    fn test_encode_roundtrip_allocate() {
        let original = RelayMessage::Allocate {
            session_id: 0xFFFF_FFFF_0000_0001u64,
            token: b"round-trip-token".to_vec(),
        };
        let pkt = match &original {
            RelayMessage::Allocate { session_id, token } => encode_allocate(*session_id, token),
            _ => unreachable!(),
        };
        let (parsed, _) = parse_message(&pkt).expect("roundtrip parse should succeed");
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_encode_roundtrip_release() {
        let original = RelayMessage::Release { session_id: 12345 };
        let pkt = encode_release(original.session_id());
        let (parsed, _) = parse_message(&pkt).expect("roundtrip parse should succeed");
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_encode_roundtrip_keepalive() {
        let original = RelayMessage::Keepalive { session_id: 67890 };
        let pkt = encode_keepalive(original.session_id());
        let (parsed, _) = parse_message(&pkt).expect("roundtrip parse should succeed");
        assert_eq!(parsed, original);
    }

    #[test]
    fn test_encode_roundtrip_data() {
        let session_id = 55u64;
        let payload = b"hello world";
        let pkt = encode_data(session_id, payload);
        let (parsed, _) = parse_message(&pkt).expect("roundtrip parse should succeed");
        assert_eq!(parsed, RelayMessage::Data { session_id });
        // Verify payload is recoverable from the raw packet.
        assert_eq!(&pkt[HEADER_LEN..], payload);
    }

    // -- test_invalid_magic ---------------------------------------------------

    #[test]
    fn test_invalid_magic() {
        let mut pkt = encode_keepalive(1);
        // Corrupt the magic bytes.
        pkt[0] = 0xFF;
        pkt[1] = 0xFF;
        let err = parse_message(&pkt).expect_err("bad magic should fail");
        assert_eq!(err, ProtocolError::InvalidMagic);
    }

    #[test]
    fn test_invalid_magic_all_zeros() {
        let pkt = [0u8; HEADER_LEN];
        let err = parse_message(&pkt).expect_err("zero magic should fail");
        assert_eq!(err, ProtocolError::InvalidMagic);
    }

    #[test]
    fn test_empty_buffer() {
        let err = parse_message(&[]).expect_err("empty buf should fail");
        assert_eq!(
            err,
            ProtocolError::TooShort {
                expected: HEADER_LEN,
                actual: 0,
            }
        );
    }

    // -- unknown message type -------------------------------------------------

    #[test]
    fn test_unknown_message_type() {
        let mut pkt = encode_keepalive(1);
        // Set msg_type to an undefined value.
        pkt[5] = 0xFF;
        let err = parse_message(&pkt).expect_err("unknown type should fail");
        assert_eq!(err, ProtocolError::UnknownType(0xFF));
    }

    // -- invalid token length -------------------------------------------------

    #[test]
    fn test_invalid_token_length_exceeds_max() {
        let mut pkt = Vec::with_capacity(HEADER_LEN + 2);
        pkt.extend_from_slice(&build_header(MSG_ALLOCATE, 1));
        // Claim a token_len larger than MAX_TOKEN_LEN.
        pkt.extend_from_slice(&(MAX_TOKEN_LEN + 1).to_le_bytes());
        let err = parse_message(&pkt).expect_err("oversized token should fail");
        assert_eq!(err, ProtocolError::InvalidTokenLength(MAX_TOKEN_LEN + 1));
    }

    #[test]
    fn test_allocate_truncated_token() {
        // Header + token_len claims 10 bytes but only 5 are present.
        let mut pkt = Vec::with_capacity(HEADER_LEN + 2 + 5);
        pkt.extend_from_slice(&build_header(MSG_ALLOCATE, 1));
        pkt.extend_from_slice(&10u16.to_le_bytes()); // claims 10 bytes
        pkt.extend_from_slice(&[0xAA; 5]);           // only 5 bytes of token
        let err = parse_message(&pkt).expect_err("truncated token should fail");
        assert_eq!(
            err,
            ProtocolError::TooShort {
                expected: HEADER_LEN + 2 + 10,
                actual: HEADER_LEN + 2 + 5,
            }
        );
    }

    // -- header field encoding ------------------------------------------------

    #[test]
    fn test_header_version_and_reserved() {
        let pkt = encode_release(1);
        assert_eq!(pkt[4], VERSION, "version byte should be VERSION");
        assert_eq!(pkt[6], 0, "reserved byte 6 should be zero");
        assert_eq!(pkt[7], 0, "reserved byte 7 should be zero");
    }

    #[test]
    fn test_session_id_endianness() {
        // session_id should be stored little-endian.
        let pkt = encode_release(0x0102_0304_0506_0708u64);
        let raw = &pkt[8..16];
        assert_eq!(raw, &[0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    }

    // -- type_name helper -----------------------------------------------------

    #[test]
    fn test_type_name() {
        assert_eq!(
            RelayMessage::Allocate { session_id: 0, token: vec![] }.type_name(),
            "ALLOCATE"
        );
        assert_eq!(
            RelayMessage::Release { session_id: 0 }.type_name(),
            "RELEASE"
        );
        assert_eq!(
            RelayMessage::Keepalive { session_id: 0 }.type_name(),
            "KEEPALIVE"
        );
        assert_eq!(
            RelayMessage::Data { session_id: 0 }.type_name(),
            "DATA"
        );
    }

    // -- ProtocolError Display ------------------------------------------------

    #[test]
    fn test_error_display() {
        let e = ProtocolError::InvalidMagic;
        assert_eq!(e.to_string(), "invalid magic bytes");

        let e = ProtocolError::UnknownType(0xAB);
        assert_eq!(e.to_string(), "unknown message type: 0xab");

        let e = ProtocolError::TooShort { expected: 16, actual: 5 };
        assert_eq!(
            e.to_string(),
            "packet too short: expected at least 16, got 5"
        );

        let e = ProtocolError::InvalidTokenLength(9999);
        assert_eq!(e.to_string(), "invalid token length: 9999");
    }

    // -- Helper trait used in roundtrip tests ---------------------------------

    /// Convenience accessor used only in tests to extract session_id from any
    /// variant without a full match.
    trait SessionIdExt {
        fn session_id(&self) -> u64;
    }

    impl SessionIdExt for RelayMessage {
        fn session_id(&self) -> u64 {
            match self {
                Self::Allocate { session_id, .. }
                | Self::Release { session_id }
                | Self::Keepalive { session_id }
                | Self::Data { session_id } => *session_id,
            }
        }
    }
}
