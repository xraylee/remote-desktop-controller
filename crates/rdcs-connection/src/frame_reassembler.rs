// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Video frame reassembly from chunked DataChannel messages.

use std::collections::HashMap;
use tracing::{debug, warn};

/// Frame header: 8 bytes
///
/// Layout:
/// - frame_id: u32 (4 bytes)
/// - flags: u8 (1 byte)
///   - bit 0: is_keyframe
///   - bit 1-7: reserved
/// - chunk_index: u8 (1 byte)
/// - total_chunks: u8 (1 byte)
/// - reserved: u8 (1 byte)
#[derive(Debug, Clone, Copy)]
pub struct FrameHeader {
    pub frame_id: u32,
    pub is_keyframe: bool,
    pub chunk_index: u8,
    pub total_chunks: u8,
}

impl FrameHeader {
    /// Header size in bytes.
    pub const SIZE: usize = 8;

    /// Serialize header to bytes.
    pub fn serialize(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.frame_id.to_be_bytes());
        buf[4] = if self.is_keyframe { 1 } else { 0 };
        buf[5] = self.chunk_index;
        buf[6] = self.total_chunks;
        buf[7] = 0; // reserved
        buf
    }

    /// Deserialize header from bytes.
    pub fn deserialize(buf: &[u8]) -> Result<Self, FrameError> {
        if buf.len() < 8 {
            return Err(FrameError::InvalidHeader);
        }

        Ok(Self {
            frame_id: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            is_keyframe: buf[4] == 1,
            chunk_index: buf[5],
            total_chunks: buf[6],
        })
    }
}

/// Errors related to frame processing.
#[derive(Debug, Clone)]
pub enum FrameError {
    InvalidHeader,
    InvalidChunkIndex,
    FrameTooBig,
}

impl std::fmt::Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::InvalidHeader => write!(f, "Invalid frame header"),
            FrameError::InvalidChunkIndex => write!(f, "Invalid chunk index"),
            FrameError::FrameTooBig => write!(f, "Frame too big"),
        }
    }
}

impl std::error::Error for FrameError {}

/// Partial frame being reassembled.
struct PartialFrame {
    chunks: Vec<Option<Vec<u8>>>,
    total_chunks: u8,
    received_chunks: u8,
    is_keyframe: bool,
}

/// Reassembles fragmented video frames from DataChannel chunks.
pub struct FrameReassembler {
    pending_frames: HashMap<u32, PartialFrame>,
    max_pending: usize,
}

impl FrameReassembler {
    /// Create a new frame reassembler.
    ///
    /// # Arguments
    /// * `max_pending` - Maximum number of incomplete frames to buffer
    pub fn new(max_pending: usize) -> Self {
        Self {
            pending_frames: HashMap::new(),
            max_pending,
        }
    }

    /// Add a chunk to the reassembler.
    ///
    /// Returns `Some((frame_id, frame_data, is_keyframe))` if the frame is complete.
    pub fn add_chunk(
        &mut self,
        header: FrameHeader,
        data: Vec<u8>,
    ) -> Option<(u32, Vec<u8>, bool)> {
        // Validate chunk index
        if header.chunk_index >= header.total_chunks {
            warn!(
                "Invalid chunk index {}/{} for frame {}",
                header.chunk_index, header.total_chunks, header.frame_id
            );
            return None;
        }

        // Get or create partial frame
        let frame = self.pending_frames
            .entry(header.frame_id)
            .or_insert_with(|| PartialFrame {
                chunks: vec![None; header.total_chunks as usize],
                total_chunks: header.total_chunks,
                received_chunks: 0,
                is_keyframe: header.is_keyframe,
            });

        // Add chunk if not already received
        if frame.chunks[header.chunk_index as usize].is_none() {
            frame.chunks[header.chunk_index as usize] = Some(data);
            frame.received_chunks += 1;

            debug!(
                "Frame {} chunk {}/{} received",
                header.frame_id, frame.received_chunks, frame.total_chunks
            );
        } else {
            debug!(
                "Duplicate chunk {}/{} for frame {} (ignored)",
                header.chunk_index, header.total_chunks, header.frame_id
            );
        }

        // Check if frame is complete
        if frame.received_chunks == frame.total_chunks {
            let complete_frame: Vec<u8> = frame.chunks
                .iter()
                .filter_map(|c| c.as_ref())
                .flat_map(|c| c.iter().copied())
                .collect();

            let is_keyframe = frame.is_keyframe;
            self.pending_frames.remove(&header.frame_id);

            debug!(
                "Frame {} complete: {} bytes (keyframe: {})",
                header.frame_id,
                complete_frame.len(),
                is_keyframe
            );

            return Some((header.frame_id, complete_frame, is_keyframe));
        }

        // Clean up old frames if too many are pending
        if self.pending_frames.len() > self.max_pending {
            if let Some(oldest_id) = self.pending_frames.keys().min().copied() {
                warn!(
                    "Dropping incomplete frame {} (too many pending: {})",
                    oldest_id,
                    self.pending_frames.len()
                );
                self.pending_frames.remove(&oldest_id);
            }
        }

        None
    }

    /// Get the number of pending incomplete frames.
    pub fn pending_count(&self) -> usize {
        self.pending_frames.len()
    }

    /// Clear all pending frames.
    pub fn clear(&mut self) {
        self.pending_frames.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_header_serialization() {
        let header = FrameHeader {
            frame_id: 42,
            is_keyframe: true,
            chunk_index: 2,
            total_chunks: 5,
        };

        let bytes = header.serialize();
        let decoded = FrameHeader::deserialize(&bytes).unwrap();

        assert_eq!(decoded.frame_id, 42);
        assert_eq!(decoded.is_keyframe, true);
        assert_eq!(decoded.chunk_index, 2);
        assert_eq!(decoded.total_chunks, 5);
    }

    #[test]
    fn test_frame_header_size() {
        assert_eq!(FrameHeader::SIZE, 8);
    }

    #[test]
    fn test_single_chunk_frame() {
        let mut reassembler = FrameReassembler::new(10);

        let header = FrameHeader {
            frame_id: 1,
            is_keyframe: true,
            chunk_index: 0,
            total_chunks: 1,
        };

        let data = vec![1, 2, 3, 4, 5];
        let result = reassembler.add_chunk(header, data.clone());

        assert!(result.is_some());
        let (frame_id, frame_data, is_keyframe) = result.unwrap();
        assert_eq!(frame_id, 1);
        assert_eq!(frame_data, data);
        assert_eq!(is_keyframe, true);
    }

    #[test]
    fn test_multi_chunk_frame() {
        let mut reassembler = FrameReassembler::new(10);

        let chunk1 = vec![1, 2, 3];
        let chunk2 = vec![4, 5, 6];
        let chunk3 = vec![7, 8, 9];

        // Add chunks out of order
        let header2 = FrameHeader {
            frame_id: 100,
            is_keyframe: false,
            chunk_index: 1,
            total_chunks: 3,
        };
        assert!(reassembler.add_chunk(header2, chunk2.clone()).is_none());

        let header3 = FrameHeader {
            frame_id: 100,
            is_keyframe: false,
            chunk_index: 2,
            total_chunks: 3,
        };
        assert!(reassembler.add_chunk(header3, chunk3.clone()).is_none());

        // Add final chunk - should complete the frame
        let header1 = FrameHeader {
            frame_id: 100,
            is_keyframe: false,
            chunk_index: 0,
            total_chunks: 3,
        };
        let result = reassembler.add_chunk(header1, chunk1.clone());

        assert!(result.is_some());
        let (frame_id, frame_data, is_keyframe) = result.unwrap();
        assert_eq!(frame_id, 100);
        assert_eq!(frame_data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(is_keyframe, false);
    }

    #[test]
    fn test_duplicate_chunk_ignored() {
        let mut reassembler = FrameReassembler::new(10);

        let header = FrameHeader {
            frame_id: 1,
            is_keyframe: false,
            chunk_index: 0,
            total_chunks: 2,
        };

        let data1 = vec![1, 2, 3];
        let data2 = vec![4, 5, 6];

        // Add same chunk twice
        assert!(reassembler.add_chunk(header, data1.clone()).is_none());
        assert!(reassembler.add_chunk(header, data2.clone()).is_none());

        // Should still have 1 pending frame with 1 chunk
        assert_eq!(reassembler.pending_count(), 1);
    }

    #[test]
    fn test_max_pending_cleanup() {
        let mut reassembler = FrameReassembler::new(2);

        // Add incomplete frames
        for frame_id in 0..5 {
            let header = FrameHeader {
                frame_id,
                is_keyframe: false,
                chunk_index: 0,
                total_chunks: 2,
            };
            reassembler.add_chunk(header, vec![1, 2, 3]);
        }

        // Should only keep max_pending frames
        assert!(reassembler.pending_count() <= 2);
    }

    #[test]
    fn test_clear() {
        let mut reassembler = FrameReassembler::new(10);

        let header = FrameHeader {
            frame_id: 1,
            is_keyframe: false,
            chunk_index: 0,
            total_chunks: 2,
        };

        reassembler.add_chunk(header, vec![1, 2, 3]);
        assert_eq!(reassembler.pending_count(), 1);

        reassembler.clear();
        assert_eq!(reassembler.pending_count(), 0);
    }
}
