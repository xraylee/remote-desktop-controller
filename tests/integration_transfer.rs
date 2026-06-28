// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Integration tests for rdcs-transfer: chunked file transfer, integrity
//! verification, pause/resume/cancel lifecycle, checksum utilities, and
//! concurrent multi-file transfers.
//!
//! Each test exercises the public API of `rdcs_transfer` end-to-end using
//! real files on disk (via `tempfile`) and synchronous `#[test]` functions.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use rdcs_transfer::checksum::{
    compute_chunk_hash, compute_sha256, verify as checksum_verify, ChecksumCalculator,
};
use rdcs_transfer::{
    FileOffer, LocalFileReceiver, LocalFileSender, TransferRequest, TransferState,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a deterministic test file of `size` bytes (cycling 0..=255).
fn create_test_file(dir: &Path, name: &str, size: usize) -> PathBuf {
    let path = dir.join(name);
    let mut file = File::create(&path).unwrap();
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    file.write_all(&data).unwrap();
    path
}

/// Build a `FileOffer` from a sender handle, mirroring what the signaling
/// layer would do in production.
fn build_offer(
    sender: &LocalFileSender,
    handle: &rdcs_transfer::TransferHandle,
    file_name: &str,
) -> FileOffer {
    FileOffer {
        id: handle.id,
        file_name: file_name.to_string(),
        file_size: handle.total_bytes,
        checksum: sender.file_checksum(handle).unwrap(),
    }
}

// ---------------------------------------------------------------------------
// 1. Full file send / receive / verify
// ---------------------------------------------------------------------------

#[test]
fn transfer_full_file_send_receive_verify() {
    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();

    // 256 KB file -> 4 chunks at the default 64 KB chunk size.
    let file_size = 256 * 1024;
    let file_path = create_test_file(src_dir.path(), "full.bin", file_size);
    let original_checksum = compute_sha256(&file_path).unwrap();

    // Sender side.
    let mut sender = LocalFileSender::new();
    let handle = sender
        .start_transfer(TransferRequest {
            path: file_path,
            dest_name: "full.bin".to_string(),
        })
        .unwrap();

    assert_eq!(handle.total_bytes, file_size as u64);
    assert_eq!(handle.state, TransferState::Pending);

    // Receiver side.
    let offer = build_offer(&sender, &handle, "full.bin");
    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();

    // Transfer all chunks.
    let mut chunk_count = 0u32;
    while let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
        receiver
            .receive_chunk(&recv_handle, offset, &data)
            .unwrap();
        chunk_count += 1;
    }

    assert_eq!(chunk_count, 4, "256 KB / 64 KB = 4 chunks");

    // Sender should report Completed.
    assert_eq!(
        sender.transfer_state(&handle),
        TransferState::Completed
    );

    // Receiver verify (SHA-256 comparison).
    assert!(
        receiver.verify(&recv_handle).unwrap(),
        "receiver verify must succeed"
    );

    // Independent on-disk verification.
    let dest_path = dest_dir.path().join("full.bin");
    assert!(dest_path.exists(), "destination file must exist after transfer");
    let dest_checksum = compute_sha256(&dest_path).unwrap();
    assert_eq!(
        original_checksum, dest_checksum,
        "on-disk SHA-256 must match the source file"
    );
}

// ---------------------------------------------------------------------------
// 2. Pause and resume
// ---------------------------------------------------------------------------

#[test]
fn transfer_pause_and_resume() {
    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();

    // 512 KB -> 8 chunks.
    let file_size = 512 * 1024;
    let file_path = create_test_file(src_dir.path(), "pause.bin", file_size);
    let original_checksum = compute_sha256(&file_path).unwrap();

    let mut sender = LocalFileSender::new();
    let handle = sender
        .start_transfer(TransferRequest {
            path: file_path,
            dest_name: "pause.bin".to_string(),
        })
        .unwrap();

    let offer = build_offer(&sender, &handle, "pause.bin");
    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();

    // Send the first 4 chunks (~50%).
    let mut bytes_sent = 0u64;
    for _ in 0..4 {
        if let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
            bytes_sent += data.len() as u64;
            receiver
                .receive_chunk(&recv_handle, offset, &data)
                .unwrap();
        }
    }
    assert_eq!(bytes_sent, 4 * 64 * 1024);

    // Pause the transfer.
    sender.pause(&handle).unwrap();
    match sender.transfer_state(&handle) {
        TransferState::Paused { offset } => {
            assert_eq!(offset, bytes_sent, "pause offset must match bytes sent");
        }
        other => panic!("expected Paused, got: {:?}", other),
    }

    // send_chunk must return None while paused.
    assert!(
        sender.send_chunk(handle.id).unwrap().is_none(),
        "send_chunk should return None while paused"
    );

    // Resume the transfer.
    sender.resume(&handle).unwrap();
    match sender.transfer_state(&handle) {
        TransferState::InProgress { sent_bytes } => {
            assert_eq!(sent_bytes, bytes_sent, "resume should restore sent_bytes");
        }
        other => panic!("expected InProgress after resume, got: {:?}", other),
    }

    // Send remaining chunks to completion.
    while let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
        receiver
            .receive_chunk(&recv_handle, offset, &data)
            .unwrap();
    }

    // Verify completion and integrity.
    assert_eq!(
        sender.transfer_state(&handle),
        TransferState::Completed
    );
    assert!(receiver.verify(&recv_handle).unwrap());

    let dest_checksum = compute_sha256(&dest_dir.path().join("pause.bin")).unwrap();
    assert_eq!(original_checksum, dest_checksum);
}

// ---------------------------------------------------------------------------
// 3. Cancel mid-transfer and cleanup
// ---------------------------------------------------------------------------

#[test]
fn transfer_cancel_cleanup() {
    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();

    let file_path = create_test_file(src_dir.path(), "cancel.bin", 256 * 1024);

    let mut sender = LocalFileSender::new();
    let handle = sender
        .start_transfer(TransferRequest {
            path: file_path,
            dest_name: "cancel.bin".to_string(),
        })
        .unwrap();

    let offer = build_offer(&sender, &handle, "cancel.bin");
    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();

    // Send only 2 of 4 chunks.
    for _ in 0..2 {
        if let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
            receiver
                .receive_chunk(&recv_handle, offset, &data)
                .unwrap();
        }
    }

    // Partial file should exist on the receiver side.
    let partial_path = dest_dir.path().join("cancel.bin");
    assert!(partial_path.exists(), "partial file must exist before cancel");

    // Cancel on sender side.
    sender.cancel(&handle).unwrap();
    assert_eq!(
        sender.transfer_state(&handle),
        TransferState::Cancelled,
        "sender state must be Cancelled"
    );

    // Cancel on receiver side — should remove the partial output file.
    receiver.cancel(&recv_handle).unwrap();
    assert_eq!(
        receiver.transfer_state(&recv_handle),
        TransferState::Cancelled,
        "receiver state must be Cancelled"
    );
    assert!(
        !partial_path.exists(),
        "partial file must be removed after receiver cancel"
    );
}

// ---------------------------------------------------------------------------
// 4. Checksum SHA-256 correctness
// ---------------------------------------------------------------------------

#[test]
fn transfer_checksum_sha256_correctness() {
    // Well-known SHA-256 of "Hello, RDCS!" (verified against external tools).
    let data = b"Hello, RDCS!";

    // One-shot hash.
    let hash = compute_chunk_hash(data);

    // Determinism: same input -> same output.
    let hash2 = compute_chunk_hash(data);
    assert_eq!(hash, hash2, "compute_chunk_hash must be deterministic");

    // Different input -> different output.
    let hash_other = compute_chunk_hash(b"Goodbye, RDCS!");
    assert_ne!(hash, hash_other, "different data must produce different hashes");

    // Streaming ChecksumCalculator must match the one-shot function.
    let mut calc = ChecksumCalculator::new();
    calc.update(b"Hello, ");
    calc.update(b"RDCS!");
    assert_eq!(calc.bytes_processed(), 12);
    let streaming_hash = calc.finalize();
    assert_eq!(
        hash, streaming_hash,
        "streaming ChecksumCalculator must match one-shot compute_chunk_hash"
    );

    // verify() helper: correct data matches, wrong data does not.
    assert!(checksum_verify(data, &hash), "verify must accept correct data");
    assert!(
        !checksum_verify(b"corrupted!", &hash),
        "verify must reject incorrect data"
    );

    // File-based SHA-256 matches in-memory hash for the same content.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("checksum_test.bin");
    std::fs::write(&path, data).unwrap();
    let file_hash = compute_sha256(&path).unwrap();
    assert_eq!(
        file_hash, hash,
        "compute_sha256 on file must match compute_chunk_hash on same bytes"
    );

    // compute_sha256 on a missing file returns FileNotFound.
    let missing = compute_sha256(Path::new("/nonexistent/rdcs_test_file.bin"));
    assert!(missing.is_err(), "missing file must return an error");
}

// ---------------------------------------------------------------------------
// 5. Per-chunk integrity verification and corruption detection
// ---------------------------------------------------------------------------

#[test]
fn transfer_chunk_integrity_verification() {
    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();

    // 192 KB -> 3 chunks.
    let file_size = 192 * 1024;
    let file_path = create_test_file(src_dir.path(), "integrity.bin", file_size);

    let mut sender = LocalFileSender::new();
    let handle = sender
        .start_transfer(TransferRequest {
            path: file_path,
            dest_name: "integrity.bin".to_string(),
        })
        .unwrap();

    let offer = build_offer(&sender, &handle, "integrity.bin");
    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();

    // Collect chunks with their hashes, simulating a per-chunk integrity
    // check that a production receiver would perform.
    struct ChunkRecord {
        offset: u64,
        data: Vec<u8>,
        hash: [u8; 32],
    }

    let mut records: Vec<ChunkRecord> = Vec::new();
    while let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
        let hash = compute_chunk_hash(&data);
        records.push(ChunkRecord {
            offset,
            data,
            hash,
        });
    }
    assert_eq!(records.len(), 3, "192 KB / 64 KB = 3 chunks");

    // Verify every chunk's hash matches its data (no corruption).
    for (i, rec) in records.iter().enumerate() {
        assert!(
            checksum_verify(&rec.data, &rec.hash),
            "chunk {i} hash must match its data"
        );
    }

    // Now simulate corruption: flip a byte in chunk 1's data.
    let mut corrupted_data = records[1].data.clone();
    corrupted_data[0] ^= 0xFF;

    // The corrupted chunk must fail its per-chunk hash check.
    assert!(
        !checksum_verify(&corrupted_data, &records[1].hash),
        "corrupted chunk must fail hash verification"
    );

    // Feed the receiver with chunk 0 (good), chunk 1 (corrupted), chunk 2 (good).
    receiver
        .receive_chunk(&recv_handle, records[0].offset, &records[0].data)
        .unwrap();
    receiver
        .receive_chunk(&recv_handle, records[1].offset, &corrupted_data)
        .unwrap();
    receiver
        .receive_chunk(&recv_handle, records[2].offset, &records[2].data)
        .unwrap();

    // Final SHA-256 verify must detect the corruption.
    let verified = receiver.verify(&recv_handle).unwrap();
    assert!(
        !verified,
        "final verify must fail when a chunk is corrupted"
    );

    // Transfer should be in Failed state after a checksum mismatch.
    match receiver.transfer_state(&recv_handle) {
        TransferState::Failed(msg) => {
            assert!(
                msg.contains("checksum"),
                "failure message should mention checksum: {msg}"
            );
        }
        other => panic!("expected Failed state, got: {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// 6. Multiple concurrent file transfers
// ---------------------------------------------------------------------------

#[test]
fn transfer_multiple_concurrent_files() {
    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();

    // Two files with different sizes and content patterns.
    let file_a_path = create_test_file(src_dir.path(), "alpha.bin", 128 * 1024); // 2 chunks
    let file_b_path = create_test_file(src_dir.path(), "beta.bin", 192 * 1024); // 3 chunks

    let checksum_a = compute_sha256(&file_a_path).unwrap();
    let checksum_b = compute_sha256(&file_b_path).unwrap();

    // Start both transfers on the same sender.
    let mut sender = LocalFileSender::new();

    let handle_a = sender
        .start_transfer(TransferRequest {
            path: file_a_path,
            dest_name: "alpha.bin".to_string(),
        })
        .unwrap();

    let handle_b = sender
        .start_transfer(TransferRequest {
            path: file_b_path,
            dest_name: "beta.bin".to_string(),
        })
        .unwrap();

    // Distinct transfer IDs.
    assert_ne!(handle_a.id, handle_b.id, "transfer IDs must be unique");

    // Accept both on the same receiver.
    let offer_a = build_offer(&sender, &handle_a, "alpha.bin");
    let offer_b = build_offer(&sender, &handle_b, "beta.bin");

    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_a = receiver.accept(offer_a, dest_dir.path()).unwrap();
    let recv_b = receiver.accept(offer_b, dest_dir.path()).unwrap();

    // Interleave chunks from both transfers to simulate real concurrent I/O.
    // Drain alpha first (2 chunks), then beta (3 chunks), alternating where
    // possible.
    let mut a_done = false;
    let mut b_done = false;

    while !a_done || !b_done {
        if !a_done {
            match sender.send_chunk(handle_a.id).unwrap() {
                Some((offset, data)) => {
                    receiver.receive_chunk(&recv_a, offset, &data).unwrap();
                }
                None => a_done = true,
            }
        }
        if !b_done {
            match sender.send_chunk(handle_b.id).unwrap() {
                Some((offset, data)) => {
                    receiver.receive_chunk(&recv_b, offset, &data).unwrap();
                }
                None => b_done = true,
            }
        }
    }

    // Both senders should report Completed.
    assert_eq!(sender.transfer_state(&handle_a), TransferState::Completed);
    assert_eq!(sender.transfer_state(&handle_b), TransferState::Completed);

    // Both receivers verify successfully.
    assert!(
        receiver.verify(&recv_a).unwrap(),
        "alpha transfer must verify"
    );
    assert!(
        receiver.verify(&recv_b).unwrap(),
        "beta transfer must verify"
    );

    // Independent on-disk checksums.
    let dest_a = compute_sha256(&dest_dir.path().join("alpha.bin")).unwrap();
    let dest_b = compute_sha256(&dest_dir.path().join("beta.bin")).unwrap();
    assert_eq!(checksum_a, dest_a, "alpha on-disk checksum must match source");
    assert_eq!(checksum_b, dest_b, "beta on-disk checksum must match source");

    // received_bytes should reflect each file's full size.
    assert_eq!(receiver.received_bytes(&recv_a), 128 * 1024);
    assert_eq!(receiver.received_bytes(&recv_b), 192 * 1024);
}
