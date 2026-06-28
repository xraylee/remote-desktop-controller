// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! H.264 RTP Depacketizer (RFC 6184).
//!
//! Reassembles H.264 NAL units from RTP packets and outputs Annex B format.
//!
//! # Features
//!
//! - Single NAL Unit Mode depacketization
//! - FU-A fragment reassembly
//! - Sequence number gap detection (packet loss)
//! - Out-of-order packet handling
//! - Annex B output format

use super::{Result, RtpError, RtpHeader};
use crate::rtp::packetizer::NalUnitType;
use bytes::{BufMut, BytesMut};
use std::collections::HashMap;
use tracing::{debug, trace, warn};

/// Maximum age of incomplete fragments (in sequence numbers).
const MAX_FRAGMENT_AGE: u16 = 1000;

/// H.264 RTP Depacketizer.
pub struct H264Depacketizer {
    /// Last received sequence number.
    last_sequence: Option<u16>,
    /// Incomplete FU-A fragments (keyed by timestamp).
    fragments: HashMap<u32, FragmentBuffer>,
    /// Statistics.
    stats: DepacketizerStats,
}

/// Depacketizer statistics.
#[derive(Debug, Clone, Default)]
pub struct DepacketizerStats {
    pub packets_received: u64,
    pub nal_units_assembled: u64,
    pub fragments_reassembled: u64,
    pub keyframes_received: u32,
    pub bytes_received: u64,
    pub packets_lost: u32,
    pub packets_out_of_order: u32,
}

/// Buffer for reassembling fragmented NAL units.
struct FragmentBuffer {
    /// NAL unit header (from first fragment).
    nal_header: u8,
    /// Accumulated payload.
    payload: BytesMut,
    /// Sequence number of first fragment.
    first_sequence: u16,
    /// Expected next sequence number.
    next_sequence: u16,
    /// Whether we've received the last fragment.
    complete: bool,
}

impl H264Depacketizer {
    /// Create a new H.264 RTP depacketizer.
    pub fn new() -> Self {
        Self {
            last_sequence: None,
            fragments: HashMap::new(),
            stats: DepacketizerStats::default(),
        }
    }

    /// Depacketize an RTP packet.
    ///
    /// Returns `Some(Vec<u8>)` when a complete NAL unit (or frame) is ready,
    /// `None` if more packets are needed.
    ///
    /// Output is in Annex B format (start codes + NAL units).
    pub fn depacketize(&mut self, packet: &[u8]) -> Result<Option<Vec<u8>>> {
        let header = RtpHeader::from_bytes(packet)?;
        let payload = &packet[RtpHeader::SIZE..];

        self.stats.packets_received += 1;
        self.stats.bytes_received += packet.len() as u64;

        // Check for sequence gaps (packet loss)
        if let Some(last_seq) = self.last_sequence {
            let expected = last_seq.wrapping_add(1);
            if header.sequence_number != expected {
                let gap = header.sequence_number.wrapping_sub(expected);
                if gap < 32768 {
                    // Forward gap (packet loss)
                    self.stats.packets_lost += gap as u32;
                    warn!(
                        "Packet loss detected: expected seq {}, got {} (gap: {})",
                        expected, header.sequence_number, gap
                    );
                } else {
                    // Backward gap (out-of-order or duplicate)
                    self.stats.packets_out_of_order += 1;
                    trace!(
                        "Out-of-order packet: seq {} (expected {})",
                        header.sequence_number,
                        expected
                    );
                }
            }
        }

        self.last_sequence = Some(header.sequence_number);

        // Clean up old incomplete fragments
        self.cleanup_old_fragments(header.sequence_number);

        // Parse NAL unit type from payload
        if payload.is_empty() {
            return Err(RtpError::InvalidPacket("Empty RTP payload".into()));
        }

        let nal_type = NalUnitType::from_header(payload[0]);

        match nal_type {
            Some(NalUnitType::FuA) => {
                // Fragmentation Unit A
                self.handle_fu_a_packet(&header, payload)
            }
            Some(_) | None => {
                // Single NAL Unit Mode
                self.handle_single_nal_packet(&header, payload)
            }
        }
    }

    /// Handle a single NAL unit packet.
    fn handle_single_nal_packet(
        &mut self,
        header: &RtpHeader,
        payload: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        trace!(
            "Single NAL unit: seq={}, ts={}, size={} bytes",
            header.sequence_number,
            header.timestamp,
            payload.len()
        );

        self.stats.nal_units_assembled += 1;

        let nal_type = NalUnitType::from_header(payload[0]);
        if nal_type.map(|t| t.is_keyframe()).unwrap_or(false) {
            self.stats.keyframes_received += 1;
        }

        // Convert to Annex B format: start code + NAL unit
        let mut annex_b = BytesMut::with_capacity(4 + payload.len());
        annex_b.put_slice(&[0, 0, 0, 1]); // Start code
        annex_b.put_slice(payload);

        Ok(Some(annex_b.to_vec()))
    }

    /// Handle a Fragmentation Unit A packet.
    fn handle_fu_a_packet(
        &mut self,
        header: &RtpHeader,
        payload: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        if payload.len() < 2 {
            return Err(RtpError::InvalidPacket(
                "FU-A packet too short".into(),
            ));
        }

        let fu_indicator = payload[0];
        let fu_header = payload[1];
        let fragment_payload = &payload[2..];

        let start = (fu_header & 0x80) != 0;
        let end = (fu_header & 0x40) != 0;
        let nal_type_bits = fu_header & 0x1F;

        trace!(
            "FU-A fragment: seq={}, ts={}, start={}, end={}, type={}, size={} bytes",
            header.sequence_number,
            header.timestamp,
            start,
            end,
            nal_type_bits,
            fragment_payload.len()
        );

        if start {
            // First fragment - create new buffer
            let nal_header = (fu_indicator & 0xE0) | nal_type_bits;

            let mut payload_buf = BytesMut::with_capacity(1500);
            payload_buf.put_slice(fragment_payload);

            let fragment = FragmentBuffer {
                nal_header,
                payload: payload_buf,
                first_sequence: header.sequence_number,
                next_sequence: header.sequence_number.wrapping_add(1),
                complete: end,
            };

            self.fragments.insert(header.timestamp, fragment);

            if end {
                // Single-fragment NAL unit (unusual but valid)
                return self.finalize_fragment(header.timestamp);
            }
        } else if let Some(fragment) = self.fragments.get_mut(&header.timestamp) {
            // Middle or last fragment
            if header.sequence_number != fragment.next_sequence {
                warn!(
                    "Fragment sequence mismatch: expected {}, got {}",
                    fragment.next_sequence, header.sequence_number
                );
                // Discard incomplete fragment
                self.fragments.remove(&header.timestamp);
                return Ok(None);
            }

            fragment.payload.put_slice(fragment_payload);
            fragment.next_sequence = header.sequence_number.wrapping_add(1);

            if end {
                fragment.complete = true;
                return self.finalize_fragment(header.timestamp);
            }
        } else {
            // Middle/end fragment without start - discard
            warn!(
                "Received FU-A fragment without start: seq={}, ts={}",
                header.sequence_number, header.timestamp
            );
            return Ok(None);
        }

        Ok(None)
    }

    /// Finalize a complete fragmented NAL unit.
    fn finalize_fragment(&mut self, timestamp: u32) -> Result<Option<Vec<u8>>> {
        if let Some(fragment) = self.fragments.remove(&timestamp) {
            if !fragment.complete {
                return Ok(None);
            }

            self.stats.nal_units_assembled += 1;
            self.stats.fragments_reassembled += 1;

            let nal_type = NalUnitType::from_header(fragment.nal_header);
            if nal_type.map(|t| t.is_keyframe()).unwrap_or(false) {
                self.stats.keyframes_received += 1;
            }

            debug!(
                "Reassembled fragmented NAL unit: {} bytes",
                fragment.payload.len() + 1
            );

            // Convert to Annex B format: start code + NAL header + payload
            let mut annex_b = BytesMut::with_capacity(4 + 1 + fragment.payload.len());
            annex_b.put_slice(&[0, 0, 0, 1]); // Start code
            annex_b.put_u8(fragment.nal_header);
            annex_b.put_slice(&fragment.payload);

            Ok(Some(annex_b.to_vec()))
        } else {
            Ok(None)
        }
    }

    /// Clean up old incomplete fragments to prevent memory leaks.
    fn cleanup_old_fragments(&mut self, current_sequence: u16) {
        self.fragments.retain(|_ts, fragment| {
            let age = current_sequence.wrapping_sub(fragment.first_sequence);
            age < MAX_FRAGMENT_AGE
        });
    }

    /// Get current depacketizer statistics.
    pub fn stats(&self) -> &DepacketizerStats {
        &self.stats
    }

    /// Reset statistics counters.
    pub fn reset_stats(&mut self) {
        self.stats = DepacketizerStats::default();
    }
}

impl Default for H264Depacketizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtp::packetizer::{H264Packetizer, PacketizerConfig};

    #[test]
    fn test_single_nal_unit_depacketization() {
        let mut depacketizer = H264Depacketizer::new();

        // Create a simple RTP packet with single NAL unit
        let config = PacketizerConfig::default();
        let mut packetizer = H264Packetizer::new(config);

        let mut annex_b = vec![0, 0, 0, 1, 0x65]; // IDR NAL unit
        annex_b.extend_from_slice(&[0xAA; 50]); // Dummy payload

        let packets = packetizer.packetize(&annex_b, 12345).unwrap();
        assert_eq!(packets.len(), 1);

        let result = depacketizer.depacketize(&packets[0]).unwrap();
        assert!(result.is_some());

        let output = result.unwrap();
        assert!(output.starts_with(&[0, 0, 0, 1, 0x65]));
        assert_eq!(depacketizer.stats().nal_units_assembled, 1);
        assert_eq!(depacketizer.stats().keyframes_received, 1);
    }

    #[test]
    fn test_fu_a_reassembly() {
        let mut depacketizer = H264Depacketizer::new();

        // Create fragmented packets
        let config = PacketizerConfig {
            mtu: 100, // Small MTU to force fragmentation
            ..Default::default()
        };
        let mut packetizer = H264Packetizer::new(config);

        let mut annex_b = vec![0, 0, 0, 1, 0x65]; // IDR NAL unit
        annex_b.extend_from_slice(&[0xBB; 200]); // Large payload

        let packets = packetizer.packetize(&annex_b, 12345).unwrap();
        assert!(packets.len() > 1);

        // Depacketize all fragments
        let mut result = None;
        for packet in packets {
            result = depacketizer.depacketize(&packet).unwrap();
        }

        // Should have complete NAL unit after last packet
        assert!(result.is_some());
        let output = result.unwrap();
        assert!(output.starts_with(&[0, 0, 0, 1, 0x65]));
        assert_eq!(depacketizer.stats().fragments_reassembled, 1);
    }

    #[test]
    fn test_packet_loss_detection() {
        let mut depacketizer = H264Depacketizer::new();

        let config = PacketizerConfig::default();
        let mut packetizer = H264Packetizer::new(config);

        // Create two separate NAL units
        let annex_b1 = vec![0, 0, 0, 1, 0x67, 1, 2, 3]; // SPS
        let annex_b2 = vec![0, 0, 0, 1, 0x68, 4, 5]; // PPS

        let packets1 = packetizer.packetize(&annex_b1, 100).unwrap();
        let packets2 = packetizer.packetize(&annex_b2, 200).unwrap();

        // Process first packet
        depacketizer.depacketize(&packets1[0]).unwrap();

        // Skip second packet (simulate loss), process third
        depacketizer.depacketize(&packets2[0]).unwrap();

        // Should detect packet loss
        assert_eq!(depacketizer.stats().packets_lost, 1);
    }

    #[test]
    fn test_multiple_nal_units() {
        let mut depacketizer = H264Depacketizer::new();

        let config = PacketizerConfig::default();
        let mut packetizer = H264Packetizer::new(config);

        // Create Annex B with multiple NAL units
        let mut annex_b = vec![];
        annex_b.extend_from_slice(&[0, 0, 0, 1, 0x67, 1, 2]); // SPS
        annex_b.extend_from_slice(&[0, 0, 0, 1, 0x68, 3]); // PPS
        annex_b.extend_from_slice(&[0, 0, 0, 1, 0x65, 4, 5, 6]); // IDR

        let packets = packetizer.packetize(&annex_b, 12345).unwrap();
        assert_eq!(packets.len(), 3);

        // Depacketize each
        for packet in packets {
            let result = depacketizer.depacketize(&packet).unwrap();
            assert!(result.is_some());
        }

        assert_eq!(depacketizer.stats().nal_units_assembled, 3);
        assert_eq!(depacketizer.stats().keyframes_received, 3);
    }
}
