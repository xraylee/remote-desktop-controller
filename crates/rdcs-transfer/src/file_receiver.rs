// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Chunked file receiver that reassembles and verifies transferred files.
//!
//! `LocalFileReceiver` manages one or more incoming file transfers, writing
//! received chunks to disk at the correct byte offsets and verifying the
//! SHA-256 checksum once all data has been received.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::checksum;
use crate::{FileOffer, TransferError, TransferHandle, TransferState};

/// Internal state for a single active receive transfer.
struct ReceiveEntry {
    output_path: PathBuf,
    expected_size: u64,
    expected_checksum: [u8; 32],
    received_bytes: u64,
    state: TransferState,
}

/// Manages incoming file transfers with chunked writing and verification.
///
/// Supports multiple concurrent transfers, each identified by a unique ID.
/// Chunks can arrive at arbitrary offsets (supporting resume after disconnect).
/// Once all bytes are received, the file's SHA-256 checksum is verified.
pub struct LocalFileReceiver {
    transfers: HashMap<u64, ReceiveEntry>,
}

impl LocalFileReceiver {
    /// Create a new file receiver.
    pub fn new(_dest_dir: &Path) -> Self {
        Self {
            transfers: HashMap::new(),
        }
    }

    /// Accept an incoming file offer and prepare to receive chunks.
    ///
    /// Creates a temporary file in the destination directory and returns
    /// a `TransferHandle` for tracking the transfer.
    pub fn accept(
        &mut self,
        offer: FileOffer,
        dest_dir: &Path,
    ) -> Result<TransferHandle, TransferError> {
        fs::create_dir_all(dest_dir)?;

        let output_path = dest_dir.join(&offer.file_name);
        // Create the file to ensure the path is writable.
        let _file = File::create(&output_path)?;

        let id = offer.id;
        let total_bytes = offer.file_size;

        let entry = ReceiveEntry {
            output_path,
            expected_size: offer.file_size,
            expected_checksum: offer.checksum,
            received_bytes: 0,
            state: TransferState::InProgress { sent_bytes: 0 },
        };

        self.transfers.insert(id, entry);

        Ok(TransferHandle {
            id,
            state: TransferState::InProgress { sent_bytes: 0 },
            total_bytes,
        })
    }

    /// Reject an incoming file offer.
    ///
    /// This is a no-op since no resources have been allocated yet.
    pub fn reject(&mut self, _offer: &FileOffer) -> Result<(), TransferError> {
        Ok(())
    }

    /// Receive a chunk of data at the given byte offset.
    ///
    /// The chunk is written to the output file at the specified offset.
    pub fn receive_chunk(
        &mut self,
        handle: &TransferHandle,
        offset: u64,
        data: &[u8],
    ) -> Result<(), TransferError> {
        let entry = self
            .transfers
            .get_mut(&handle.id)
            .ok_or_else(|| TransferError::FileNotFound(format!("transfer {} not found", handle.id)))?;

        match &entry.state {
            TransferState::InProgress { .. } => {}
            TransferState::Cancelled => return Err(TransferError::Cancelled),
            _ => return Err(TransferError::Cancelled),
        }

        // Open the file for writing at the specified offset.
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(&entry.output_path)?;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(data)?;

        entry.received_bytes += data.len() as u64;

        if entry.received_bytes >= entry.expected_size {
            entry.state = TransferState::Completed;
        } else {
            entry.state = TransferState::InProgress {
                sent_bytes: entry.received_bytes,
            };
        }

        Ok(())
    }

    /// Verify the completed transfer by comparing the SHA-256 checksum.
    ///
    /// Returns `Ok(true)` if the checksum matches, `Ok(false)` if it does not.
    /// On mismatch, the transfer is marked as `Failed`.
    pub fn verify(&mut self, handle: &TransferHandle) -> Result<bool, TransferError> {
        let entry = self
            .transfers
            .get_mut(&handle.id)
            .ok_or_else(|| TransferError::FileNotFound(format!("transfer {} not found", handle.id)))?;

        if entry.received_bytes < entry.expected_size {
            return Err(TransferError::FileNotFound(format!(
                "incomplete transfer: {} of {} bytes received",
                entry.received_bytes, entry.expected_size
            )));
        }

        let file_hash = checksum::compute_sha256(&entry.output_path)?;

        if file_hash == entry.expected_checksum {
            entry.state = TransferState::Completed;
            Ok(true)
        } else {
            entry.state = TransferState::Failed("checksum mismatch".into());
            Ok(false)
        }
    }

    /// Cancel a transfer and remove the partial output file.
    pub fn cancel(&mut self, handle: &TransferHandle) -> Result<(), TransferError> {
        if let Some(entry) = self.transfers.get_mut(&handle.id) {
            entry.state = TransferState::Cancelled;
            let path = entry.output_path.clone();
            // Best-effort removal of the partial file.
            let _ = fs::remove_file(&path);
            Ok(())
        } else {
            Err(TransferError::FileNotFound(format!(
                "transfer {} not found",
                handle.id
            )))
        }
    }

    /// Return the current state of a transfer.
    pub fn transfer_state(&self, handle: &TransferHandle) -> TransferState {
        self.transfers
            .get(&handle.id)
            .map(|e| e.state.clone())
            .unwrap_or(TransferState::Failed("transfer not found".into()))
    }

    /// Return the number of bytes received so far for a transfer.
    pub fn received_bytes(&self, handle: &TransferHandle) -> u64 {
        self.transfers
            .get(&handle.id)
            .map(|e| e.received_bytes)
            .unwrap_or(0)
    }

    /// Return the output path for a transfer.
    pub fn output_path(&self, handle: &TransferHandle) -> Option<&Path> {
        self.transfers
            .get(&handle.id)
            .map(|e| e.output_path.as_path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checksum::compute_sha256;
    use std::io::Write;

    fn create_test_file(dir: &Path, name: &str, size: usize) -> PathBuf {
        let path = dir.join(name);
        let mut file = File::create(&path).unwrap();
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        file.write_all(&data).unwrap();
        path
    }

    #[test]
    fn accept_and_receive_complete() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();
        let file_path = create_test_file(src_dir.path(), "test.bin", 100);

        let cs = compute_sha256(&file_path).unwrap();

        let mut receiver = LocalFileReceiver::new(dest_dir.path());
        let offer = FileOffer {
            id: 1,
            file_name: "test.bin".to_string(),
            file_size: 100,
            checksum: cs,
        };

        let handle = receiver.accept(offer, dest_dir.path()).unwrap();

        let data: Vec<u8> = (0..100).map(|i| (i % 256) as u8).collect();
        receiver.receive_chunk(&handle, 0, &data).unwrap();

        assert_eq!(receiver.received_bytes(&handle), 100);
        assert_eq!(
            receiver.transfer_state(&handle),
            TransferState::Completed
        );
    }

    #[test]
    fn verify_checksum_match() {
        let src_dir = tempfile::tempdir().unwrap();
        let dest_dir = tempfile::tempdir().unwrap();
        let file_path = create_test_file(src_dir.path(), "verify.bin", 256);

        let cs = compute_sha256(&file_path).unwrap();

        let mut receiver = LocalFileReceiver::new(dest_dir.path());
        let offer = FileOffer {
            id: 1,
            file_name: "verify.bin".to_string(),
            file_size: 256,
            checksum: cs,
        };

        let handle = receiver.accept(offer, dest_dir.path()).unwrap();

        let data: Vec<u8> = (0..256).map(|i| (i % 256) as u8).collect();
        receiver.receive_chunk(&handle, 0, &data).unwrap();

        assert!(receiver.verify(&handle).unwrap());
    }

    #[test]
    fn verify_checksum_mismatch() {
        let dest_dir = tempfile::tempdir().unwrap();

        let mut receiver = LocalFileReceiver::new(dest_dir.path());
        let offer = FileOffer {
            id: 1,
            file_name: "bad.bin".to_string(),
            file_size: 10,
            checksum: [0u8; 32],
        };

        let handle = receiver.accept(offer, dest_dir.path()).unwrap();
        receiver.receive_chunk(&handle, 0, &[0u8; 10]).unwrap();

        assert!(!receiver.verify(&handle).unwrap());
        match receiver.transfer_state(&handle) {
            TransferState::Failed(_) => {}
            other => panic!("expected Failed, got: {:?}", other),
        }
    }

    #[test]
    fn reject_offer() {
        let dest_dir = tempfile::tempdir().unwrap();
        let mut receiver = LocalFileReceiver::new(dest_dir.path());
        let offer = FileOffer {
            id: 1,
            file_name: "rejected.bin".to_string(),
            file_size: 100,
            checksum: [0u8; 32],
        };
        assert!(receiver.reject(&offer).is_ok());
    }
}
