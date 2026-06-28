// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Selective NACK (Negative Acknowledgment) retransmission manager.
//!
//! Tracks missing sequence numbers detected via gaps in the receive stream
//! and generates NACK requests with bounded retry counts. Once a sequence
//! number exceeds its maximum retry count, it is abandoned (presumed lost
//! beyond recovery at the transport layer).

use std::collections::{BTreeSet, HashMap};

/// Maximum number of tracked missing entries to prevent memory exhaustion.
const MAX_TRACKED: usize = 1000;

/// Maximum gap range per single `report_gap` call.
const MAX_GAP_RANGE: u32 = 1000;

/// Tracks missing sequence numbers and generates NACK reports.
///
/// Each missing sequence number is tracked with a retry counter. On each
/// call to [`generate_nack_list`](NackTracker::generate_nack_list), the
/// counter is incremented. Once it reaches `max_retries`, the sequence
/// number is removed from tracking (abandoned).
#[derive(Debug)]
pub struct NackTracker {
    /// Sequence numbers detected as missing.
    missing: BTreeSet<u32>,
    /// Maximum number of retransmission requests per sequence.
    max_retries: u8,
    /// Per-sequence retry counts.
    retry_counts: HashMap<u32, u8>,
}

impl NackTracker {
    /// Create a new NACK tracker with the given maximum retry count.
    pub fn new(max_retries: u8) -> Self {
        Self {
            missing: BTreeSet::new(),
            max_retries,
            retry_counts: HashMap::new(),
        }
    }

    /// Report a gap: sequences in `[expected, received)` are marked as missing.
    ///
    /// The gap range is bounded to [`MAX_GAP_RANGE`] (1000) and the total
    /// tracked entries are bounded to [`MAX_TRACKED`] (1000) to prevent
    /// memory exhaustion from pathological inputs.
    pub fn report_gap(&mut self, expected: u32, received: u32) {
        let end = received.min(expected.saturating_add(MAX_GAP_RANGE));
        for seq in expected..end {
            if self.missing.len() >= MAX_TRACKED {
                tracing::warn!(
                    tracked = self.missing.len(),
                    "NACK tracker at capacity, dropping gap entries"
                );
                break;
            }
            self.missing.insert(seq);
        }
    }

    /// Mark a sequence number as received (no longer missing).
    pub fn mark_received(&mut self, seq: u32) {
        self.missing.remove(&seq);
        self.retry_counts.remove(&seq);
    }

    /// Generate a list of sequence numbers to request via NACK.
    ///
    /// Increments the retry counter for each tracked missing sequence.
    /// Sequences that have exceeded `max_retries` are removed from tracking
    /// and excluded from the returned list (abandoned).
    pub fn generate_nack_list(&mut self) -> Vec<u32> {
        let mut nacks = Vec::new();
        let mut exhausted = Vec::new();

        for &seq in &self.missing {
            let count = self.retry_counts.entry(seq).or_insert(0);
            if *count < self.max_retries {
                *count += 1;
                nacks.push(seq);
            } else {
                exhausted.push(seq);
            }
        }

        // Remove exhausted entries
        for seq in exhausted {
            tracing::debug!(seq, retries = self.max_retries, "NACK: abandoning exhausted seq");
            self.missing.remove(&seq);
            self.retry_counts.remove(&seq);
        }

        nacks
    }

    /// Return the number of currently tracked missing sequences.
    pub fn missing_count(&self) -> usize {
        self.missing.len()
    }

    /// Return the configured maximum retry count.
    pub fn max_retries(&self) -> u8 {
        self.max_retries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_missing_packets() {
        let mut tracker = NackTracker::new(3);
        tracker.report_gap(5, 8);
        assert_eq!(tracker.missing_count(), 3);

        let nacks = tracker.generate_nack_list();
        assert_eq!(nacks, vec![5, 6, 7]);

        tracker.mark_received(6);
        assert_eq!(tracker.missing_count(), 2);
    }

    #[test]
    fn exhaust_retries() {
        let mut tracker = NackTracker::new(1);
        tracker.report_gap(0, 1);
        let _ = tracker.generate_nack_list(); // retry 1
        let nacks = tracker.generate_nack_list(); // exhausted
        assert!(nacks.is_empty());
    }

    /// Acceptance test: single packet retransmitted max 3 times then abandoned.
    #[test]
    fn nack_retry_max_three() {
        let mut tracker = NackTracker::new(3);
        tracker.report_gap(5, 6); // packet 5 is missing

        // First 3 calls should include seq 5
        for round in 1..=3 {
            let nacks = tracker.generate_nack_list();
            assert!(
                nacks.contains(&5),
                "round {round}: seq 5 should be in NACK list"
            );
        }

        // 4th call: exhausted, seq 5 abandoned
        let nacks = tracker.generate_nack_list();
        assert!(
            !nacks.contains(&5),
            "seq 5 should be abandoned after 3 retries"
        );
        assert_eq!(
            tracker.missing_count(),
            0,
            "no missing packets should remain"
        );
    }

    #[test]
    fn mark_received_clears_retry_count() {
        let mut tracker = NackTracker::new(3);
        tracker.report_gap(5, 6);

        // Use 2 of 3 retries
        tracker.generate_nack_list();
        tracker.generate_nack_list();

        // Mark as received (arrived via retransmission)
        tracker.mark_received(5);
        assert_eq!(tracker.missing_count(), 0);

        // Report same gap again — should get fresh 3 retries
        tracker.report_gap(5, 6);
        let nacks = tracker.generate_nack_list();
        assert!(nacks.contains(&5));
    }

    #[test]
    fn bounded_gap_prevents_memory_exhaustion() {
        let mut tracker = NackTracker::new(3);
        // Report a massive gap (10000 packets)
        tracker.report_gap(0, 10_000);
        // Should be capped at MAX_TRACKED (1000)
        assert!(tracker.missing_count() <= MAX_TRACKED);
    }

    #[test]
    fn multiple_gaps_tracked_independently() {
        let mut tracker = NackTracker::new(2);
        tracker.report_gap(3, 5); // missing 3, 4
        tracker.report_gap(10, 12); // missing 10, 11

        let nacks = tracker.generate_nack_list();
        assert_eq!(nacks, vec![3, 4, 10, 11]);
        assert_eq!(tracker.missing_count(), 4);

        // Mark one from each gap as received
        tracker.mark_received(4);
        tracker.mark_received(10);
        assert_eq!(tracker.missing_count(), 2);

        let nacks = tracker.generate_nack_list();
        assert_eq!(nacks, vec![3, 11]);
    }
}
