// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Sequence number management and out-of-order packet buffering.
//!
//! [`SendSequencer`] allocates monotonic sequence numbers for outgoing packets
//! and tracks the send window. [`ReceiveSequencer`] buffers out-of-order
//! packets and reports gaps for NACK generation.

use std::collections::BTreeMap;
use std::ops::Range;

/// Maximum allowed gap size to prevent memory exhaustion.
const MAX_GAP: u32 = 1000;

/// Tracks sent packet sequence numbers and manages the send window.
#[derive(Debug)]
pub struct SendSequencer {
    next_seq: u32,
    window_size: u32,
    base_seq: u32,
}

impl SendSequencer {
    /// Create a new send sequencer with the given window size.
    pub fn new(window_size: u32) -> Self {
        Self {
            next_seq: 0,
            window_size,
            base_seq: 0,
        }
    }

    /// Allocate the next sequence number. Returns `None` if the window is full.
    pub fn next_seq(&mut self) -> Option<u32> {
        if self.next_seq.wrapping_sub(self.base_seq) >= self.window_size {
            return None;
        }
        let seq = self.next_seq;
        self.next_seq = self.next_seq.wrapping_add(1);
        Some(seq)
    }

    /// Advance the base sequence upon acknowledgment.
    pub fn acknowledge(&mut self, seq: u32) {
        if seq.wrapping_sub(self.base_seq) < self.window_size {
            self.base_seq = seq.wrapping_add(1);
        }
    }

    /// Return the current base sequence number.
    pub fn base_seq(&self) -> u32 {
        self.base_seq
    }

    /// Return the next sequence number that would be allocated.
    pub fn next_seq_value(&self) -> u32 {
        self.next_seq
    }
}

/// Buffers out-of-order received packets until gaps are filled.
///
/// Packets arriving ahead of the expected sequence are buffered in a
/// `BTreeMap`. When the expected packet arrives, all contiguous buffered
/// packets are drained and delivered together.
#[derive(Debug)]
pub struct ReceiveSequencer {
    expected: u32,
    buffer: BTreeMap<u32, Vec<u8>>,
    max_buffer: usize,
}

impl ReceiveSequencer {
    /// Create a new receive sequencer with the given maximum buffer size.
    ///
    /// The buffer size is capped at [`MAX_GAP`] (1000) to prevent memory exhaustion.
    pub fn new(max_buffer: usize) -> Self {
        Self {
            expected: 0,
            buffer: BTreeMap::new(),
            max_buffer: max_buffer.min(MAX_GAP as usize),
        }
    }

    /// Insert a received packet. Returns contiguous in-order payloads ready
    /// for delivery to the application.
    ///
    /// If `seq` matches the expected sequence, the payload is delivered
    /// immediately along with any buffered packets that become contiguous.
    /// If `seq` is ahead of expected, the payload is buffered.
    /// If `seq` is behind expected (duplicate), an empty vec is returned.
    pub fn insert(&mut self, seq: u32, data: Vec<u8>) -> Vec<Vec<u8>> {
        if seq == self.expected {
            let mut result = vec![data];
            self.expected = self.expected.wrapping_add(1);
            // Drain any buffered packets that are now in order
            while let Some(buffered) = self.buffer.remove(&self.expected) {
                result.push(buffered);
                self.expected = self.expected.wrapping_add(1);
            }
            result
        } else if seq.wrapping_sub(self.expected) > 0
            && seq.wrapping_sub(self.expected) <= self.max_buffer as u32
        {
            self.buffer.insert(seq, data);
            Vec::new()
        } else {
            // Behind expected (duplicate) or too far ahead; drop
            Vec::new()
        }
    }

    /// Return the next expected sequence number.
    pub fn expected(&self) -> u32 {
        self.expected
    }

    /// Report gaps between the expected sequence and buffered packets.
    ///
    /// Returns a list of ranges `[start..end)` representing missing sequence
    /// numbers. Each range starts at a missing sequence and ends at the next
    /// buffered (received) sequence.
    ///
    /// # Example
    /// If expected=3 and buffer contains {4, 5, 7}, returns `[3..4, 6..7]`.
    pub fn gap_report(&self) -> Vec<Range<u32>> {
        let mut gaps = Vec::new();
        let mut current = self.expected;
        for &seq in self.buffer.keys() {
            if seq > current {
                gaps.push(current..seq);
            }
            current = seq.wrapping_add(1);
        }
        gaps
    }

    /// Return the number of buffered (out-of-order) packets.
    pub fn buffered_count(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_sequencer_basic() {
        let mut seq = SendSequencer::new(3);
        assert_eq!(seq.next_seq(), Some(0));
        assert_eq!(seq.next_seq(), Some(1));
        assert_eq!(seq.next_seq(), Some(2));
        assert_eq!(seq.next_seq(), None); // window full
        seq.acknowledge(0);
        assert_eq!(seq.next_seq(), Some(3));
    }

    #[test]
    fn send_sequencer_acknowledge_advances_base() {
        let mut seq = SendSequencer::new(10);
        for _ in 0..5 {
            seq.next_seq();
        }
        assert_eq!(seq.base_seq(), 0);
        seq.acknowledge(2);
        assert_eq!(seq.base_seq(), 3);
    }

    #[test]
    fn receive_sequencer_in_order() {
        let mut seq = ReceiveSequencer::new(64);
        let out = seq.insert(0, vec![1, 2, 3]);
        assert_eq!(out, vec![vec![1, 2, 3]]);
        assert_eq!(seq.expected(), 1);
    }

    #[test]
    fn receive_sequencer_out_of_order() {
        let mut seq = ReceiveSequencer::new(64);
        // Receive packet 1 before packet 0
        let out = seq.insert(1, vec![2]);
        assert!(out.is_empty());
        // Now receive packet 0
        let out = seq.insert(0, vec![1]);
        assert_eq!(out, vec![vec![1], vec![2]]);
        assert_eq!(seq.expected(), 2);
    }

    #[test]
    fn receive_sequencer_duplicate_dropped() {
        let mut seq = ReceiveSequencer::new(64);
        seq.insert(0, vec![1]);
        // Duplicate of already-delivered packet
        let out = seq.insert(0, vec![1]);
        assert!(out.is_empty());
        assert_eq!(seq.expected(), 1);
    }

    #[test]
    fn sequencer_gap_detection() {
        let mut seq = ReceiveSequencer::new(1000);

        // Receive packets 0, 1, 2 in order (advances expected to 3)
        seq.insert(0, vec![0]);
        seq.insert(1, vec![1]);
        seq.insert(2, vec![2]);
        assert_eq!(seq.expected(), 3);

        // Receive out-of-order: 4, 5, 7 (skipping 3 and 6)
        seq.insert(4, vec![4]);
        seq.insert(5, vec![5]);
        seq.insert(7, vec![7]);

        // Expected gaps: [3..4, 6..7]
        let gaps = seq.gap_report();
        assert_eq!(gaps.len(), 2);
        assert_eq!(gaps[0], 3..4);
        assert_eq!(gaps[1], 6..7);
    }

    #[test]
    fn sequencer_gap_fills_on_delivery() {
        let mut seq = ReceiveSequencer::new(100);

        // Receive 0 (expected → 1), then buffer 2, 3
        seq.insert(0, vec![0]);
        seq.insert(2, vec![2]);
        seq.insert(3, vec![3]);

        // Gap: [1..2]
        let gaps = seq.gap_report();
        assert_eq!(gaps, vec![1..2]);

        // Fill the gap: receive packet 1
        let delivered = seq.insert(1, vec![1]);
        assert_eq!(delivered, vec![vec![1], vec![2], vec![3]]);
        assert_eq!(seq.expected(), 4);

        // No more gaps
        let gaps = seq.gap_report();
        assert!(gaps.is_empty());
    }

    #[test]
    fn sequencer_max_buffer_enforced() {
        let mut seq = ReceiveSequencer::new(5);

        seq.insert(0, vec![0]); // expected → 1

        // Packet too far ahead (seq=10, expected=1, gap=9 > max_buffer=5)
        let out = seq.insert(10, vec![10]);
        assert!(out.is_empty());
        assert_eq!(seq.buffered_count(), 0);

        // Packet within range
        let out = seq.insert(3, vec![3]);
        assert!(out.is_empty());
        assert_eq!(seq.buffered_count(), 1);
    }
}
