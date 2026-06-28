// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! H.264 RTP Packetizer (RFC 6184).
//!
//! Converts H.264 Annex B NAL units into RTP packets for network transmission.
//!
//! # Packetization Modes
//!
//! - **Single NAL Unit Mode**: Small NAL units fit in one RTP packet
//! - **Fragmentation Unit Mode (FU-A)**: Large NAL units split across multiple packets
//!
//! # NAL Unit Types
//!
//! - **SPS (7)**: Sequence Parameter Set (codec config)
//! - **PPS (8)**: Picture Parameter Set (codec config)
//! - **IDR (5)**: Instantaneous Decoder Refresh (keyframe)
//! - **Non-IDR (1)**: Inter-frame (P-frame/B-frame)

use super::{Result, RtpError, RtpHeader};
use bytes::{BufMut, BytesMut};
use tracing::{debug, trace, warn};

/// H.264 NAL unit type identifiers (5 bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NalUnitType {
    /// Non-IDR slice (P-frame/B-frame).
    NonIdrSlice = 1,
    /// IDR slice (keyframe).
    IdrSlice = 5,
    /// Sequence Parameter Set.
    Sps = 7,
    /// Picture Parameter Set.
    Pps = 8,
    /// Fragmentation Unit A (for splitting large NAL units).
    FuA = 28,
}

impl NalUnitType {
    /// Parse NAL unit type from header byte.
    pub fn from_header(header: u8) -> Option<Self> {
        match header & 0x1F {
            1 => Some(Self::NonIdrSlice),
            5 => Some(Self::IdrSlice),
            7 => Some(Self::Sps),
            8 => Some(Self::Pps),
            28 => Some(Self::FuA),
            _ => None,
        }
    }

    /// Check if this is a keyframe NAL unit.
    pub fn is_keyframe(self) -> bool {
        matches!(self, Self::IdrSlice | Self::Sps | Self::Pps)
    }
}

/// Packetizer configuration.
#[derive(Debug, Clone)]
pub struct PacketizerConfig {
    /// Maximum Transmission Unit (default: 1200 bytes for typical networks).
    pub mtu: usize,
    /// RTP clock rate (must be 90000 for H.264 per RFC 6184).
    pub clock_rate: u32,
    /// Synchronization Source identifier.
    pub ssrc: u32,
    /// RTP payload type (typically 96-127 for dynamic types).
    pub payload_type: u8,
}

impl Default for PacketizerConfig {
    fn default() -> Self {
        Self {
            mtu: 1200,
            clock_rate: 90000,
            ssrc: rand::random(),
            payload_type: 96,
        }
    }
}

/// H.264 RTP Packetizer.
pub struct H264Packetizer {
    config: PacketizerConfig,
    sequence_number: u16,
    stats: PacketizerStats,
}

/// Packetizer statistics.
#[derive(Debug, Clone, Default)]
pub struct PacketizerStats {
    pub packets_sent: u64,
    pub nal_units_processed: u64,
    pub fragmented_nal_units: u64,
    pub keyframes_sent: u32,
    pub bytes_sent: u64,
}

impl H264Packetizer {
    /// Create a new H.264 RTP packetizer.
    pub fn new(config: PacketizerConfig) -> Self {
        if config.clock_rate != 90000 {
            warn!(
                "Non-standard clock rate {} (RFC 6184 requires 90000)",
                config.clock_rate
            );
        }

        Self {
            config,
            sequence_number: rand::random(),
            stats: PacketizerStats::default(),
        }
    }

    /// Packetize H.264 Annex B data into RTP packets.
    ///
    /// Input must be in Annex B format (NAL units separated by start codes).
    pub fn packetize(&mut self, annex_b_data: &[u8], timestamp: u32) -> Result<Vec<Vec<u8>>> {
        let nal_units = self.parse_annex_b(annex_b_data)?;
        let mut packets = Vec::new();

        for nal_unit in nal_units {
            let nal_type = NalUnitType::from_header(nal_unit[0]);

            trace!(
                "Processing NAL unit: type={:?}, size={} bytes",
                nal_type,
                nal_unit.len()
            );

            self.stats.nal_units_processed += 1;
            if nal_type.map(|t| t.is_keyframe()).unwrap_or(false) {
                self.stats.keyframes_sent += 1;
            }

            let nal_packets = self.packetize_nal_unit(nal_unit, timestamp)?;
            packets.extend(nal_packets);
        }

        debug!(
            "Packetized {} NAL units into {} RTP packets",
            nal_units.len(),
            packets.len()
        );

        Ok(packets)
    }

    /// Parse Annex B format to extract NAL units.
    fn parse_annex_b<'a>(&self, data: &'a [u8]) -> Result<Vec<&'a [u8]>> {
        let mut nal_units = Vec::new();
        let mut start = 0;

        while start < data.len() {
            // Find start code: 0x00 0x00 0x00 0x01 or 0x00 0x00 0x01
            let start_code_len = if data[start..].starts_with(&[0, 0, 0, 1]) {
                4
            } else if data[start..].starts_with(&[0, 0, 1]) {
                3
            } else {
                return Err(RtpError::NalUnitError(format!(
                    "Invalid start code at offset {}",
                    start
                )));
            };

            let nal_start = start + start_code_len;

            // Find next start code or end of data
            let mut nal_end = data.len();
            for i in (nal_start + 1)..data.len() {
                if data[i..].starts_with(&[0, 0, 0, 1]) || data[i..].starts_with(&[0, 0, 1]) {
                    nal_end = i;
                    break;
                }
            }

            if nal_start < nal_end {
                nal_units.push(&data[nal_start..nal_end]);
            }

            start = nal_end;
        }

        if nal_units.is_empty() {
            return Err(RtpError::NalUnitError(
                "No NAL units found in Annex B data".into(),
            ));
        }

        Ok(nal_units)
    }

    /// Packetize a single NAL unit.
    fn packetize_nal_unit(&mut self, nal_unit: &[u8], timestamp: u32) -> Result<Vec<Vec<u8>>> {
        let max_payload = self.config.mtu - RtpHeader::SIZE;

        if nal_unit.len() <= max_payload {
            // Single NAL Unit Mode
            Ok(vec![self.create_single_nal_packet(nal_unit, timestamp, true)?])
        } else {
            // Fragmentation Unit Mode (FU-A)
            self.stats.fragmented_nal_units += 1;
            self.create_fu_a_packets(nal_unit, timestamp, max_payload)
        }
    }

    /// Create a single NAL unit RTP packet.
    fn create_single_nal_packet(
        &mut self,
        nal_unit: &[u8],
        timestamp: u32,
        marker: bool,
    ) -> Result<Vec<u8>> {
        let header = RtpHeader {
            version: 2,
            padding: false,
            extension: false,
            csrc_count: 0,
            marker,
            payload_type: self.config.payload_type,
            sequence_number: self.sequence_number,
            timestamp,
            ssrc: self.config.ssrc,
        };

        self.sequence_number = self.sequence_number.wrapping_add(1);

        let mut packet = BytesMut::with_capacity(RtpHeader::SIZE + nal_unit.len());
        packet.put_slice(&header.to_bytes());
        packet.put_slice(nal_unit);

        self.stats.packets_sent += 1;
        self.stats.bytes_sent += packet.len() as u64;

        Ok(packet.to_vec())
    }

    /// Create Fragmentation Unit A (FU-A) packets for large NAL units.
    fn create_fu_a_packets(
        &mut self,
        nal_unit: &[u8],
        timestamp: u32,
        max_payload: usize,
    ) -> Result<Vec<Vec<u8>>> {
        let nal_header = nal_unit[0];
        let nal_payload = &nal_unit[1..];

        // FU-A header overhead: FU indicator (1 byte) + FU header (1 byte)
        let fu_overhead = 2;
        let max_fragment_size = max_payload - fu_overhead;

        let mut packets = Vec::new();
        let mut offset = 0;

        while offset < nal_payload.len() {
            let remaining = nal_payload.len() - offset;
            let fragment_size = remaining.min(max_fragment_size);
            let is_first = offset == 0;
            let is_last = offset + fragment_size >= nal_payload.len();

            let fu_indicator = (nal_header & 0xE0) | (NalUnitType::FuA as u8);
            let fu_header = ((if is_first { 0x80 } else { 0 })
                | (if is_last { 0x40 } else { 0 })
                | (nal_header & 0x1F));

            let header = RtpHeader {
                version: 2,
                padding: false,
                extension: false,
                csrc_count: 0,
                marker: is_last,
                payload_type: self.config.payload_type,
                sequence_number: self.sequence_number,
                timestamp,
                ssrc: self.config.ssrc,
            };

            self.sequence_number = self.sequence_number.wrapping_add(1);

            let mut packet =
                BytesMut::with_capacity(RtpHeader::SIZE + fu_overhead + fragment_size);
            packet.put_slice(&header.to_bytes());
            packet.put_u8(fu_indicator);
            packet.put_u8(fu_header);
            packet.put_slice(&nal_payload[offset..offset + fragment_size]);

            self.stats.packets_sent += 1;
            self.stats.bytes_sent += packet.len() as u64;

            packets.push(packet.to_vec());
            offset += fragment_size;
        }

        debug!(
            "Fragmented NAL unit ({} bytes) into {} FU-A packets",
            nal_unit.len(),
            packets.len()
        );

        Ok(packets)
    }

    /// Get current packetizer statistics.
    pub fn stats(&self) -> &PacketizerStats {
        &self.stats
    }

    /// Reset statistics counters.
    pub fn reset_stats(&mut self) {
        self.stats = PacketizerStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_nal_unit(nal_type: u8, size: usize) -> Vec<u8> {
        let mut nal = vec![nal_type; size];
        nal[0] = nal_type; // NAL header
        nal
    }

    #[test]
    fn test_single_nal_unit_mode() {
        let config = PacketizerConfig {
            mtu: 1200,
            clock_rate: 90000,
            ssrc: 0x12345678,
            payload_type: 96,
        };
        let mut packetizer = H264Packetizer::new(config);

        // Create small Annex B data
        let mut annex_b = vec![0, 0, 0, 1]; // start code
        annex_b.extend_from_slice(&create_test_nal_unit(0x65, 100)); // IDR slice

        let packets = packetizer.packetize(&annex_b, 12345).unwrap();

        assert_eq!(packets.len(), 1);
        assert_eq!(packetizer.stats().packets_sent, 1);
        assert_eq!(packetizer.stats().keyframes_sent, 1);
    }

    #[test]
    fn test_fu_a_fragmentation() {
        let config = PacketizerConfig {
            mtu: 200, // Small MTU to force fragmentation
            clock_rate: 90000,
            ssrc: 0x12345678,
            payload_type: 96,
        };
        let mut packetizer = H264Packetizer::new(config);

        // Create large Annex B data
        let mut annex_b = vec![0, 0, 0, 1]; // start code
        annex_b.extend_from_slice(&create_test_nal_unit(0x65, 500)); // Large IDR slice

        let packets = packetizer.packetize(&annex_b, 12345).unwrap();

        // Should be fragmented into multiple packets
        assert!(packets.len() > 1);
        assert_eq!(packetizer.stats().fragmented_nal_units, 1);

        // Check FU-A indicator in first packet
        let first_payload = &packets[0][RtpHeader::SIZE..];
        assert_eq!(first_payload[0] & 0x1F, NalUnitType::FuA as u8);
    }

    #[test]
    fn test_multiple_nal_units() {
        let config = PacketizerConfig::default();
        let mut packetizer = H264Packetizer::new(config);

        // Create Annex B with SPS + PPS + IDR
        let mut annex_b = vec![];

        // SPS
        annex_b.extend_from_slice(&[0, 0, 0, 1]);
        annex_b.extend_from_slice(&create_test_nal_unit(0x67, 20));

        // PPS
        annex_b.extend_from_slice(&[0, 0, 0, 1]);
        annex_b.extend_from_slice(&create_test_nal_unit(0x68, 10));

        // IDR
        annex_b.extend_from_slice(&[0, 0, 0, 1]);
        annex_b.extend_from_slice(&create_test_nal_unit(0x65, 100));

        let packets = packetizer.packetize(&annex_b, 12345).unwrap();

        assert_eq!(packets.len(), 3); // One packet per NAL unit
        assert_eq!(packetizer.stats().nal_units_processed, 3);
        assert_eq!(packetizer.stats().keyframes_sent, 3); // SPS, PPS, IDR all count
    }

    #[test]
    fn test_parse_annex_b() {
        let config = PacketizerConfig::default();
        let packetizer = H264Packetizer::new(config);

        let mut data = vec![];
        data.extend_from_slice(&[0, 0, 0, 1, 0x67, 1, 2, 3]); // SPS
        data.extend_from_slice(&[0, 0, 0, 1, 0x68, 4, 5]); // PPS

        let nal_units = packetizer.parse_annex_b(&data).unwrap();

        assert_eq!(nal_units.len(), 2);
        assert_eq!(nal_units[0][0], 0x67);
        assert_eq!(nal_units[1][0], 0x68);
    }
}
