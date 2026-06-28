// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Chunked file sender for transferring files to a remote peer.
//!
//! `LocalFileSender` manages one or more outgoing file transfers, reading
//! files in fixed-size chunks and tracking each transfer's state (progress,
//! pause, resume, cancel).

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::checksum;
use crate::{TransferError, TransferHandle, TransferProgress, TransferRequest, TransferState};

/// Default chunk size for file transfer (64 KB).
pub const DEFAULT_CHUNK_SIZE: usize = crate::CHUNK_SIZE;

/// Internal state for a single active send transfer.
struct SendEntry {
    file: File,
    total_bytes: u64,
    offset: u64,
    state: TransferState,
    _name: String,
    _checksum: [u8; 32],
}

impl std::fmt::Debug for SendEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendEntry")
            .field("total_bytes", &self.total_bytes)
            .field("offset", &self.offset)
            .field("state", &self.state)
            .field("name", &self._name)
            .finish()
    }
}

/// Manages outgoing file transfers with chunked reading.
///
/// Supports multiple concurrent transfers, each identified by a unique ID.
/// Transfers can be paused, resumed, and cancelled.
#[derive(Debug)]
pub struct LocalFileSender {
    chunk_size: usize,
    next_id: AtomicU64,
    transfers: HashMap<u64, SendEntry>,
}

impl LocalFileSender {
    /// Create a new file sender with the default chunk size (64 KB).
    pub fn new() -> Self {
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            next_id: AtomicU64::new(1),
            transfers: HashMap::new(),
        }
    }

    /// Create a new file sender with a custom chunk size.
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            next_id: AtomicU64::new(1),
            transfers: HashMap::new(),
        }
    }

    /// Start a new file transfer.
    ///
    /// Opens the file at `request.path`, computes its SHA-256 checksum,
    /// and prepares the transfer for chunk-by-chunk sending.
    pub fn start_transfer(
        &mut self,
        request: TransferRequest,
    ) -> Result<TransferHandle, TransferError> {
        let path = &request.path;

        if !path.exists() {
            return Err(TransferError::FileNotFound(path.display().to_string()));
        }

        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let total_bytes = metadata.len();
        let checksum = checksum::compute_sha256(path)?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let entry = SendEntry {
            file,
            total_bytes,
            offset: 0,
            state: TransferState::Pending,
            _name: request.dest_name,
            _checksum: checksum,
        };

        self.transfers.insert(id, entry);

        Ok(TransferHandle {
            id,
            state: TransferState::Pending,
            total_bytes,
        })
    }

    /// Pause an active transfer.
    ///
    /// The transfer's current byte offset is recorded so it can be resumed
    /// later from the same position.
    pub fn pause(&mut self, handle: &TransferHandle) -> Result<(), TransferError> {
        let entry = self
            .transfers
            .get_mut(&handle.id)
            .ok_or_else(|| TransferError::FileNotFound(format!("transfer {} not found", handle.id)))?;

        match &entry.state {
            TransferState::InProgress { sent_bytes } => {
                entry.state = TransferState::Paused { offset: *sent_bytes };
                Ok(())
            }
            TransferState::Pending => {
                entry.state = TransferState::Paused { offset: 0 };
                Ok(())
            }
            _ => Err(TransferError::Cancelled),
        }
    }

    /// Resume a paused transfer.
    pub fn resume(&mut self, handle: &TransferHandle) -> Result<(), TransferError> {
        let entry = self
            .transfers
            .get_mut(&handle.id)
            .ok_or_else(|| TransferError::FileNotFound(format!("transfer {} not found", handle.id)))?;

        match &entry.state {
            TransferState::Paused { offset } => {
                entry.state = TransferState::InProgress { sent_bytes: *offset };
                Ok(())
            }
            _ => Err(TransferError::Cancelled),
        }
    }

    /// Cancel a transfer.
    ///
    /// The transfer is marked as cancelled and its resources are released.
    pub fn cancel(&mut self, handle: &TransferHandle) -> Result<(), TransferError> {
        let entry = self
            .transfers
            .get_mut(&handle.id)
            .ok_or_else(|| TransferError::FileNotFound(format!("transfer {} not found", handle.id)))?;

        entry.state = TransferState::Cancelled;
        Ok(())
    }

    /// Read and return the next chunk of data for the given transfer.
    ///
    /// Returns `Some((offset, data))` with the byte offset and chunk data,
    /// or `None` if the transfer is complete, paused, cancelled, or failed.
    /// Automatically transitions the transfer state from `Pending` to
    /// `InProgress` on the first call.
    pub fn send_chunk(
        &mut self,
        id: u64,
    ) -> Result<Option<(u64, Vec<u8>)>, TransferError> {
        let entry = self
            .transfers
            .get_mut(&id)
            .ok_or_else(|| TransferError::FileNotFound(format!("transfer {} not found", id)))?;

        // Auto-start pending transfers.
        if entry.state == TransferState::Pending {
            entry.state = TransferState::InProgress { sent_bytes: 0 };
        }

        match &entry.state {
            TransferState::InProgress { sent_bytes } => {
                if *sent_bytes >= entry.total_bytes {
                    entry.state = TransferState::Completed;
                    return Ok(None);
                }

                let remaining = (entry.total_bytes - *sent_bytes) as usize;
                let to_read = std::cmp::min(self.chunk_size, remaining);
                let mut buffer = vec![0u8; to_read];

                entry.file.seek(SeekFrom::Start(*sent_bytes))?;
                entry.file.read_exact(&mut buffer)?;

                let offset = *sent_bytes;
                let new_sent = *sent_bytes + to_read as u64;
                entry.offset = new_sent;
                entry.state = TransferState::InProgress {
                    sent_bytes: new_sent,
                };

                if new_sent >= entry.total_bytes {
                    entry.state = TransferState::Completed;
                }

                Ok(Some((offset, buffer)))
            }
            _ => Ok(None),
        }
    }

    /// Return the current progress for a transfer.
    pub fn progress(&self, handle: &TransferHandle) -> TransferProgress {
        let sent_bytes = match self.transfers.get(&handle.id) {
            Some(entry) => match &entry.state {
                TransferState::InProgress { sent_bytes } => *sent_bytes,
                TransferState::Paused { offset } => *offset,
                TransferState::Completed => handle.total_bytes,
                _ => 0,
            },
            None => 0,
        };

        let percentage = if handle.total_bytes == 0 {
            100.0
        } else {
            (sent_bytes as f32 / handle.total_bytes as f32) * 100.0
        };

        TransferProgress {
            total_bytes: handle.total_bytes,
            sent_bytes,
            percentage,
        }
    }

    /// Return the current state of a transfer.
    pub fn transfer_state(&self, handle: &TransferHandle) -> TransferState {
        self.transfers
            .get(&handle.id)
            .map(|e| e.state.clone())
            .unwrap_or(TransferState::Failed("transfer not found".into()))
    }

    /// Return the configured chunk size.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Get the SHA-256 checksum of a file (for building a `FileOffer`).
    pub fn file_checksum(&self, handle: &TransferHandle) -> Option<[u8; 32]> {
        self.transfers.get(&handle.id).map(|e| e._checksum)
    }

    /// Get the destination name of a transfer.
    pub fn file_name(&self, handle: &TransferHandle) -> Option<&str> {
        self.transfers.get(&handle.id).map(|e| e._name.as_str())
    }
}

impl Default for LocalFileSender {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LocalFileSender {
    fn drop(&mut self) {
        for entry in self.transfers.values_mut() {
            entry.state = TransferState::Cancelled;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;

    fn create_test_file(dir: &Path, name: &str, size: usize) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        file.write_all(&data).unwrap();
        path
    }

    #[test]
    fn default_chunk_size() {
        let sender = LocalFileSender::default();
        assert_eq!(sender.chunk_size(), DEFAULT_CHUNK_SIZE);
    }

    #[test]
    fn custom_chunk_size() {
        let sender = LocalFileSender::with_chunk_size(1024);
        assert_eq!(sender.chunk_size(), 1024);
    }

    #[test]
    fn start_transfer_file_not_found() {
        let mut sender = LocalFileSender::new();
        let request = TransferRequest {
            path: PathBuf::from("/nonexistent/file.bin"),
            dest_name: "file.bin".to_string(),
        };
        let result = sender.start_transfer(request);
        assert!(result.is_err());
    }

    #[test]
    fn start_and_send_single_chunk() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_file(dir.path(), "small.bin", 100);

        let mut sender = LocalFileSender::new();
        let request = TransferRequest {
            path,
            dest_name: "small.bin".to_string(),
        };
        let handle = sender.start_transfer(request).unwrap();
        assert_eq!(handle.state, TransferState::Pending);
        assert_eq!(handle.total_bytes, 100);

        let chunk = sender.send_chunk(handle.id).unwrap();
        assert!(chunk.is_some());
        let (offset, data) = chunk.unwrap();
        assert_eq!(offset, 0);
        assert_eq!(data.len(), 100);

        // Next call should return None (completed).
        let chunk = sender.send_chunk(handle.id).unwrap();
        assert!(chunk.is_none());

        assert_eq!(
            sender.transfer_state(&handle),
            TransferState::Completed
        );
    }

    #[test]
    fn progress_tracking() {
        let dir = tempfile::tempdir().unwrap();
        let path = create_test_file(dir.path(), "prog.bin", 200);

        let mut sender = LocalFileSender::with_chunk_size(100);
        let handle = sender
            .start_transfer(TransferRequest {
                path,
                dest_name: "prog.bin".to_string(),
            })
            .unwrap();

        let prog = sender.progress(&handle);
        assert_eq!(prog.sent_bytes, 0);
        assert!((prog.percentage - 0.0).abs() < f32::EPSILON);

        sender.send_chunk(handle.id).unwrap();
        let prog = sender.progress(&handle);
        assert_eq!(prog.sent_bytes, 100);
        assert!((prog.percentage - 50.0).abs() < f32::EPSILON);

        sender.send_chunk(handle.id).unwrap();
        let prog = sender.progress(&handle);
        assert_eq!(prog.sent_bytes, 200);
        assert!((prog.percentage - 100.0).abs() < f32::EPSILON);
    }
}
