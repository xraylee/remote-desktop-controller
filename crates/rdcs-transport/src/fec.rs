// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Forward Error Correction using XOR-based parity over sub-groups.
//!
//! For every `k` data packets, `p` repair packets are generated. The data
//! packets are divided into `p` equal sub-groups, and each repair packet is
//! the XOR of all data packets in its sub-group. This allows recovery of
//! up to one missing data packet per sub-group (up to `p` total losses).
//!
//! ## Default Configuration
//! - 8 data packets + 2 repair packets per group (8+2)
//! - Sub-group 0: data[0..4], repair[0] = XOR of data[0..4]
//! - Sub-group 1: data[4..8], repair[1] = XOR of data[4..8]
//! - Can recover from up to 2 lost data packets (one per sub-group)

/// Default number of data packets per FEC group.
pub const DEFAULT_DATA_COUNT: usize = 8;

/// Default number of repair/parity packets per FEC group.
pub const DEFAULT_PARITY_COUNT: usize = 2;

/// Legacy constant for backward compatibility (single-repair group size).
pub const DEFAULT_GROUP_SIZE: usize = 4;

/// A FEC encoder that produces repair packets from groups of data packets.
///
/// Data packets are divided into sub-groups of size `data_count / parity_count`.
/// Each repair packet is the XOR of all data packets in its sub-group.
#[derive(Debug)]
pub struct FecEncoder {
    /// Number of data packets per FEC group.
    data_count: usize,
    /// Number of repair packets per FEC group.
    parity_count: usize,
    /// Accumulating data packets for the current group.
    current_group: Vec<Vec<u8>>,
}

impl FecEncoder {
    /// Create a new FEC encoder with the specified data and parity counts.
    ///
    /// # Panics
    /// Panics if `data_count` is 0, `parity_count` is 0, or
    /// `data_count` is not evenly divisible by `parity_count`.
    pub fn new(data_count: usize, parity_count: usize) -> Self {
        assert!(data_count > 0, "data_count must be > 0");
        assert!(parity_count > 0, "parity_count must be > 0");
        assert!(
            data_count.is_multiple_of(parity_count),
            "data_count ({data_count}) must be divisible by parity_count ({parity_count})"
        );
        Self {
            data_count,
            parity_count,
            current_group: Vec::with_capacity(data_count),
        }
    }

    /// Create a new FEC encoder with default 8+2 configuration.
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_DATA_COUNT, DEFAULT_PARITY_COUNT)
    }

    /// Feed a data packet into the encoder.
    ///
    /// Returns a vector of repair packets when the group is complete,
    /// or `None` if more packets are needed.
    pub fn feed(&mut self, data: &[u8]) -> Option<Vec<Vec<u8>>> {
        self.current_group.push(data.to_vec());
        if self.current_group.len() >= self.data_count {
            let repairs = self.compute_repairs();
            self.current_group.clear();
            Some(repairs)
        } else {
            None
        }
    }

    /// Compute repair packets by XORing data packets within each sub-group.
    fn compute_repairs(&self) -> Vec<Vec<u8>> {
        let sub_size = self.data_count / self.parity_count;
        let mut repairs = Vec::with_capacity(self.parity_count);

        for p in 0..self.parity_count {
            let start = p * sub_size;
            let end = start + sub_size;
            let max_len = self.current_group[start..end]
                .iter()
                .map(|pkt| pkt.len())
                .max()
                .unwrap_or(0);

            let mut repair = vec![0u8; max_len];
            for pkt in &self.current_group[start..end] {
                for (i, &byte) in pkt.iter().enumerate() {
                    repair[i] ^= byte;
                }
            }
            repairs.push(repair);
        }

        repairs
    }

    /// Return the configured data packet count per group.
    pub fn group_size(&self) -> usize {
        self.data_count
    }

    /// Return the configured parity packet count per group.
    pub fn parity_count(&self) -> usize {
        self.parity_count
    }

    /// Return the number of data packets currently accumulated.
    pub fn current_count(&self) -> usize {
        self.current_group.len()
    }
}

/// A FEC decoder that recovers missing data packets from available data
/// and repair packets within a group.
#[derive(Debug)]
pub struct FecDecoder {
    /// Number of data packets per FEC group.
    data_count: usize,
    /// Number of repair packets per FEC group.
    parity_count: usize,
}

impl FecDecoder {
    /// Create a new FEC decoder matching the encoder's configuration.
    ///
    /// # Panics
    /// Panics if `data_count` is 0, `parity_count` is 0, or
    /// `data_count` is not evenly divisible by `parity_count`.
    pub fn new(data_count: usize, parity_count: usize) -> Self {
        assert!(data_count > 0, "data_count must be > 0");
        assert!(parity_count > 0, "parity_count must be > 0");
        assert!(
            data_count.is_multiple_of(parity_count),
            "data_count ({data_count}) must be divisible by parity_count ({parity_count})"
        );
        Self {
            data_count,
            parity_count,
        }
    }

    /// Create a new FEC decoder with default 8+2 configuration.
    pub fn with_defaults() -> Self {
        Self::new(DEFAULT_DATA_COUNT, DEFAULT_PARITY_COUNT)
    }

    /// Attempt to recover a full group of data packets.
    ///
    /// # Arguments
    /// - `data`: Sparse array of length `data_count`. `Some(data)` for received
    ///   packets, `None` for missing ones.
    /// - `repairs`: Array of length `parity_count`. `Some(repair)` for received
    ///   repair packets, `None` for missing ones.
    ///
    /// # Returns
    /// - `Ok(Vec<Vec<u8>>)` — the full group of `data_count` data packets
    ///   (all missing packets recovered).
    /// - `Err(String)` — if recovery is not possible (too many missing packets
    ///   in a sub-group, or a required repair packet is missing).
    pub fn decode_group(
        &self,
        data: &[Option<Vec<u8>>],
        repairs: &[Option<Vec<u8>>],
    ) -> Result<Vec<Vec<u8>>, String> {
        if data.len() != self.data_count {
            return Err(format!(
                "expected {} data packets, got {}",
                self.data_count,
                data.len()
            ));
        }
        if repairs.len() != self.parity_count {
            return Err(format!(
                "expected {} repair packets, got {}",
                self.parity_count,
                repairs.len()
            ));
        }

        let sub_size = self.data_count / self.parity_count;
        let mut result: Vec<Option<Vec<u8>>> = data.to_vec();

        for (p, repair_opt) in repairs.iter().enumerate() {
            let start = p * sub_size;
            let end = start + sub_size;

            // Find missing packets in this sub-group
            let missing: Vec<usize> = (start..end)
                .filter(|&i| result[i].is_none())
                .collect();

            match missing.len() {
                0 => continue, // All present in this sub-group
                1 => {
                    // Can recover: XOR all present packets with the repair
                    let repair = repair_opt.as_ref().ok_or_else(|| {
                        format!("repair packet {p} missing but needed for recovery")
                    })?;
                    let missing_idx = missing[0];
                    let max_len = repair.len();

                    // Start with the repair packet
                    let mut recovered = repair.clone();

                    // XOR with all present packets in the sub-group
                    for (offset, item) in result[start..end].iter().enumerate() {
                        let abs_idx = start + offset;
                        if abs_idx != missing_idx {
                            if let Some(ref pkt) = item {
                                for (j, &byte) in pkt.iter().enumerate() {
                                    if j < max_len {
                                        recovered[j] ^= byte;
                                    }
                                }
                            }
                        }
                    }

                    tracing::debug!(
                        sub_group = p,
                        recovered_idx = missing_idx,
                        "FEC: recovered missing packet"
                    );
                    result[missing_idx] = Some(recovered);
                }
                n => {
                    return Err(format!(
                        "sub-group {p} has {n} missing packets (max recoverable per sub-group: 1)"
                    ));
                }
            }
        }

        // Collect all recovered data packets
        result
            .into_iter()
            .enumerate()
            .map(|(i, opt)| opt.ok_or_else(|| format!("packet {i} still missing after recovery")))
            .collect()
    }
}

/// Recover a single missing packet from remaining data packets and a repair packet.
///
/// This is a convenience function for the simple case of a single-repair
/// FEC group (one repair covers all data packets). The recovered packet
/// is computed as `repair XOR remaining[0] XOR remaining[1] XOR ...`.
///
/// For multi-sub-group recovery, use [`FecDecoder::decode_group`] instead.
pub fn recover(remaining: &[Vec<u8>], repair: &[u8]) -> Vec<u8> {
    let max_len = remaining
        .iter()
        .map(|p| p.len())
        .max()
        .unwrap_or(repair.len());
    let mut recovered = vec![0u8; max_len];

    // Start with the repair packet
    for (i, &byte) in repair.iter().enumerate() {
        if i < max_len {
            recovered[i] = byte;
        }
    }

    // XOR with all remaining packets
    for packet in remaining {
        for (i, &byte) in packet.iter().enumerate() {
            if i < recovered.len() {
                recovered[i] ^= byte;
            }
        }
    }

    recovered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fec_repair_generation() {
        // Simple 2+1 configuration
        let mut enc = FecEncoder::new(2, 1);
        assert!(enc.feed(b"aaaa").is_none());
        let repairs = enc.feed(b"bbbb").expect("repair should be generated");
        assert_eq!(repairs.len(), 1);

        // XOR of "aaaa" and "bbbb"
        let expected: Vec<u8> = b"aaaa"
            .iter()
            .zip(b"bbbb".iter())
            .map(|(a, b)| a ^ b)
            .collect();
        assert_eq!(repairs[0], expected);
    }

    #[test]
    fn fec_single_recovery() {
        let p1 = b"aaaa".to_vec();
        let p2 = b"bbbb".to_vec();
        let repair: Vec<u8> = p1.iter().zip(p2.iter()).map(|(a, b)| a ^ b).collect();

        // Lose p1, recover from p2 + repair
        let recovered = recover(&[p2.clone()], &repair);
        assert_eq!(recovered, p1);

        // Lose p2, recover from p1 + repair
        let recovered = recover(&[p1.clone()], &repair);
        assert_eq!(recovered, p2);
    }

    /// Acceptance test: 8+2 group, lose 2 data packets → decode_group fully recovers.
    #[test]
    fn fec_group_recovery_8_plus_2() {
        let mut encoder = FecEncoder::new(8, 2);
        let decoder = FecDecoder::new(8, 2);

        // Create 8 data packets with distinct content
        let data: Vec<Vec<u8>> = (0u8..8).map(|i| vec![i.wrapping_mul(17).wrapping_add(42); 32]).collect();

        // Feed all 8 packets to the encoder
        let mut repairs = None;
        for d in &data {
            if let Some(r) = encoder.feed(d) {
                repairs = Some(r);
            }
        }
        let repairs = repairs.expect("should produce repair packets after 8 data packets");
        assert_eq!(repairs.len(), 2);

        // Verify repair correctness
        // repair[0] = XOR of data[0..4]
        let expected_repair0 = xor_packets(&data[0..4]);
        assert_eq!(repairs[0], expected_repair0);
        // repair[1] = XOR of data[4..8]
        let expected_repair1 = xor_packets(&data[4..8]);
        assert_eq!(repairs[1], expected_repair1);

        // Lose one packet from each sub-group (index 2 and index 5)
        let mut received: Vec<Option<Vec<u8>>> = data.iter().map(|d| Some(d.clone())).collect();
        received[2] = None; // lost from sub-group 0
        received[5] = None; // lost from sub-group 1

        let repair_opts: Vec<Option<Vec<u8>>> = repairs.into_iter().map(Some).collect();

        let recovered = decoder
            .decode_group(&received, &repair_opts)
            .expect("should recover all packets");

        assert_eq!(recovered.len(), 8);
        for (i, pkt) in recovered.iter().enumerate() {
            assert_eq!(pkt, &data[i], "recovered packet {i} does not match original");
        }
    }

    #[test]
    fn fec_group_no_losses() {
        let encoder_count = 4;
        let mut encoder = FecEncoder::new(encoder_count, 2);
        let decoder = FecDecoder::new(encoder_count, 2);

        let data: Vec<Vec<u8>> = (0u8..4).map(|i| vec![i; 8]).collect();
        let mut repairs = None;
        for d in &data {
            if let Some(r) = encoder.feed(d) {
                repairs = Some(r);
            }
        }
        let repairs = repairs.unwrap();

        // No losses — all packets present
        let received: Vec<Option<Vec<u8>>> = data.iter().map(|d| Some(d.clone())).collect();
        let repair_opts: Vec<Option<Vec<u8>>> = repairs.into_iter().map(Some).collect();

        let recovered = decoder.decode_group(&received, &repair_opts).unwrap();
        assert_eq!(recovered, data);
    }

    #[test]
    fn fec_group_too_many_losses_in_subgroup() {
        let mut encoder = FecEncoder::new(8, 2);
        let decoder = FecDecoder::new(8, 2);

        let data: Vec<Vec<u8>> = (0u8..8).map(|i| vec![i; 16]).collect();
        let mut repairs = None;
        for d in &data {
            if let Some(r) = encoder.feed(d) {
                repairs = Some(r);
            }
        }
        let repairs = repairs.unwrap();

        // Lose 2 packets from the SAME sub-group (indices 1 and 2, both in sub-group 0)
        let mut received: Vec<Option<Vec<u8>>> = data.iter().map(|d| Some(d.clone())).collect();
        received[1] = None;
        received[2] = None;

        let repair_opts: Vec<Option<Vec<u8>>> = repairs.into_iter().map(Some).collect();

        let result = decoder.decode_group(&received, &repair_opts);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("2 missing packets"));
    }

    #[test]
    fn fec_group_missing_repair() {
        let mut encoder = FecEncoder::new(4, 2);
        let decoder = FecDecoder::new(4, 2);

        let data: Vec<Vec<u8>> = (0u8..4).map(|i| vec![i; 8]).collect();
        let mut repairs = None;
        for d in &data {
            if let Some(r) = encoder.feed(d) {
                repairs = Some(r);
            }
        }
        let _repairs = repairs.unwrap();

        // Lose one data packet and its corresponding repair
        let mut received: Vec<Option<Vec<u8>>> = data.iter().map(|d| Some(d.clone())).collect();
        received[0] = None;

        // Both repair packets missing
        let repair_opts: Vec<Option<Vec<u8>>> = vec![None, None];

        let result = decoder.decode_group(&received, &repair_opts);
        assert!(result.is_err());
    }

    #[test]
    fn fec_encoder_state_tracking() {
        let mut enc = FecEncoder::new(4, 2);
        assert_eq!(enc.current_count(), 0);
        enc.feed(b"a");
        assert_eq!(enc.current_count(), 1);
        enc.feed(b"b");
        assert_eq!(enc.current_count(), 2);
        enc.feed(b"c");
        assert_eq!(enc.current_count(), 3);
        let _ = enc.feed(b"d"); // completes group
        assert_eq!(enc.current_count(), 0); // reset
    }

    /// Helper: XOR all packets in a slice.
    fn xor_packets(packets: &[Vec<u8>]) -> Vec<u8> {
        let max_len = packets.iter().map(|p| p.len()).max().unwrap_or(0);
        let mut result = vec![0u8; max_len];
        for pkt in packets {
            for (i, &byte) in pkt.iter().enumerate() {
                result[i] ^= byte;
            }
        }
        result
    }
}
