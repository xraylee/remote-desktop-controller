// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Checksum computation for file transfer integrity verification.
//!
//! Uses SHA-256 (via the `sha2` crate) for both whole-file and streaming
//! checksum computation.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::TransferError;

/// Compute the SHA-256 checksum of a file on disk.
///
/// Reads the file in 64 KB chunks to keep memory usage bounded.
pub fn compute_sha256(path: &Path) -> Result<[u8; 32], TransferError> {
    if !path.exists() {
        return Err(TransferError::FileNotFound(path.display().to_string()));
    }

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; crate::CHUNK_SIZE];

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    Ok(hash)
}

/// Compute the SHA-256 hash of an in-memory byte slice.
pub fn compute_chunk_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Streaming SHA-256 checksum calculator.
///
/// Feed data incrementatively with `update()`, then call `finalize()`
/// to obtain the final hash.
#[derive(Debug)]
pub struct ChecksumCalculator {
    hasher: Sha256,
    bytes_processed: u64,
}

impl ChecksumCalculator {
    /// Create a new checksum calculator.
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
            bytes_processed: 0,
        }
    }

    /// Feed data into the checksum computation.
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
        self.bytes_processed += data.len() as u64;
    }

    /// Finalize and return the checksum.
    pub fn finalize(self) -> [u8; 32] {
        let result = self.hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Return the number of bytes processed so far.
    pub fn bytes_processed(&self) -> u64 {
        self.bytes_processed
    }
}

impl Default for ChecksumCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Verify that data matches the expected checksum.
pub fn verify(data: &[u8], expected: &[u8; 32]) -> bool {
    let actual = compute_chunk_hash(data);
    actual == *expected
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn checksum_deterministic() {
        let hash1 = compute_chunk_hash(b"hello world");
        let hash2 = compute_chunk_hash(b"hello world");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn checksum_different_data() {
        let hash1 = compute_chunk_hash(b"hello");
        let hash2 = compute_chunk_hash(b"world");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn verify_matching_data() {
        let data = b"test data";
        let expected = compute_chunk_hash(data);
        assert!(verify(data, &expected));
    }

    #[test]
    fn verify_mismatching_data() {
        let data = b"test data";
        let expected = compute_chunk_hash(b"other data");
        assert!(!verify(data, &expected));
    }

    #[test]
    fn streaming_calculator() {
        let mut calc = ChecksumCalculator::new();
        calc.update(b"hello ");
        calc.update(b"world");
        let hash = calc.finalize();

        let expected = compute_chunk_hash(b"hello world");
        assert_eq!(hash, expected);
    }

    #[test]
    fn streaming_bytes_processed() {
        let mut calc = ChecksumCalculator::new();
        calc.update(b"hello");
        assert_eq!(calc.bytes_processed(), 5);
        calc.update(b" world");
        assert_eq!(calc.bytes_processed(), 11);
    }

    #[test]
    fn file_checksum_matches_chunk_hash() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.bin");

        let data: Vec<u8> = (0..255).collect();
        let mut file = File::create(&path).unwrap();
        file.write_all(&data).unwrap();
        drop(file);

        let file_hash = compute_sha256(&path).unwrap();
        let direct_hash = compute_chunk_hash(&data);
        assert_eq!(file_hash, direct_hash);
    }

    #[test]
    fn file_checksum_not_found() {
        let result = compute_sha256(Path::new("/nonexistent/file.bin"));
        assert!(result.is_err());
        match result.unwrap_err() {
            TransferError::FileNotFound(_) => {}
            other => panic!("expected FileNotFound, got: {:?}", other),
        }
    }

    #[test]
    fn empty_data_checksum() {
        let hash = compute_chunk_hash(b"");
        // SHA-256 of empty input is a well-known constant
        assert_eq!(hash.len(), 32);
        // Just verify it's deterministic
        assert_eq!(hash, compute_chunk_hash(b""));
    }
}
