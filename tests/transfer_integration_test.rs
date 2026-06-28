// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! End-to-end integration tests for file transfer and clipboard synchronization.
//!
//! Tests the complete transfer pipeline with performance validation and stress testing.

use rdcs_transfer::*;
use rdcs_platform::mock::MockClipboard;
use rdcs_platform::{ClipboardContent, ClipboardProvider};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Test Helpers
// ---------------------------------------------------------------------------

fn create_test_file(dir: &Path, name: &str, size: usize) -> PathBuf {
    let path = dir.join(name);
    let mut file = File::create(&path).unwrap();
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    file.write_all(&data).unwrap();
    file.sync_all().unwrap();
    path
}

fn compute_transfer_speed(bytes: u64, duration: Duration) -> f64 {
    let seconds = duration.as_secs_f64();
    if seconds == 0.0 {
        return 0.0;
    }
    (bytes as f64 / seconds) / 1_000_000.0 // MB/s
}

// ---------------------------------------------------------------------------
// File Transfer Performance Tests
// ---------------------------------------------------------------------------

#[test]
fn test_100mb_file_transfer_performance() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_transfer=info")
        .with_test_writer()
        .try_init()
        .ok();

    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();
    let file_size = 100 * 1024 * 1024; // 100MB

    println!("Creating 100MB test file...");
    let file_path = create_test_file(src_dir.path(), "large.bin", file_size);

    let original_checksum = checksum::compute_sha256(&file_path).unwrap();

    // Setup sender and receiver
    let mut sender = LocalFileSender::new();
    let handle = sender.start_transfer(TransferRequest {
        path: file_path.clone(),
        dest_name: "large.bin".to_string(),
    }).unwrap();

    let offer = FileOffer {
        id: handle.id,
        file_name: "large.bin".to_string(),
        file_size: handle.total_bytes,
        checksum: sender.file_checksum(&handle).unwrap(),
    };

    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();

    // Perform transfer with timing
    println!("Starting transfer...");
    let start = Instant::now();
    let mut chunk_count = 0u32;
    let mut last_progress_print = Instant::now();

    while let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
        receiver.receive_chunk(&recv_handle, offset, &data).unwrap();
        chunk_count += 1;

        // Print progress every 500ms
        if last_progress_print.elapsed() > Duration::from_millis(500) {
            let progress = sender.progress(&handle);
            println!("  Progress: {:.1}% ({} MB / {} MB)",
                progress.percentage,
                progress.sent_bytes / 1_000_000,
                progress.total_bytes / 1_000_000);
            last_progress_print = Instant::now();
        }
    }

    let elapsed = start.elapsed();
    let speed = compute_transfer_speed(file_size as u64, elapsed);

    // Verify integrity
    assert!(receiver.verify(&recv_handle).unwrap());
    let dest_path = dest_dir.path().join("large.bin");
    let dest_checksum = checksum::compute_sha256(&dest_path).unwrap();
    assert_eq!(original_checksum, dest_checksum);

    println!("✓ Transfer completed successfully");
    println!("  File size: {} MB", file_size / 1_000_000);
    println!("  Chunks: {}", chunk_count);
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {:.2} MB/s", speed);

    // PRD requirement: > 10 MB/s on local network
    // In-memory test should be much faster
    assert!(speed > 10.0, "Transfer speed {:.2} MB/s is below 10 MB/s threshold", speed);
}

#[test]
fn test_concurrent_file_transfers() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_transfer=debug")
        .with_test_writer()
        .try_init()
        .ok();

    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();

    // Create 5 files of different sizes
    let files = vec![
        ("file1.bin", 1024 * 1024),      // 1MB
        ("file2.bin", 5 * 1024 * 1024),  // 5MB
        ("file3.bin", 10 * 1024 * 1024), // 10MB
        ("file4.bin", 2 * 1024 * 1024),  // 2MB
        ("file5.bin", 3 * 1024 * 1024),  // 3MB
    ];

    println!("Creating test files...");
    let mut test_files = Vec::new();
    for (name, size) in &files {
        let path = create_test_file(src_dir.path(), name, *size);
        let checksum = checksum::compute_sha256(&path).unwrap();
        test_files.push((path, name.to_string(), checksum));
    }

    // Start all transfers
    println!("Starting concurrent transfers...");
    let start = Instant::now();

    let mut sender = LocalFileSender::new();
    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let mut handles = Vec::new();

    for (path, name, _) in &test_files {
        let handle = sender.start_transfer(TransferRequest {
            path: path.clone(),
            dest_name: name.clone(),
        }).unwrap();

        let offer = FileOffer {
            id: handle.id,
            file_name: name.clone(),
            file_size: handle.total_bytes,
            checksum: sender.file_checksum(&handle).unwrap(),
        };

        let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();
        handles.push((handle.id, recv_handle));
    }

    // Transfer chunks round-robin
    let mut active_transfers = handles.len();
    let mut total_chunks = 0u32;

    while active_transfers > 0 {
        let mut completed_this_round = 0;

        for (send_id, recv_handle) in &handles {
            if let Some((offset, data)) = sender.send_chunk(*send_id).unwrap() {
                receiver.receive_chunk(recv_handle, offset, &data).unwrap();
                total_chunks += 1;
            } else {
                // This transfer is complete
                if sender.transfer_state(&TransferHandle {
                    id: *send_id,
                    state: TransferState::Pending,
                    total_bytes: 0
                }) == TransferState::Completed {
                    completed_this_round += 1;
                }
            }
        }

        active_transfers -= completed_this_round;
    }

    let elapsed = start.elapsed();

    // Verify all files
    println!("Verifying transferred files...");
    for (i, (_, _, original_checksum)) in test_files.iter().enumerate() {
        let (_, recv_handle) = &handles[i];
        assert!(receiver.verify(recv_handle).unwrap());

        let dest_path = dest_dir.path().join(&test_files[i].1);
        let dest_checksum = checksum::compute_sha256(&dest_path).unwrap();
        assert_eq!(*original_checksum, dest_checksum);
    }

    let total_bytes: usize = files.iter().map(|(_, size)| size).sum();
    let speed = compute_transfer_speed(total_bytes as u64, elapsed);

    println!("✓ Concurrent transfers completed successfully");
    println!("  Files: {}", files.len());
    println!("  Total size: {} MB", total_bytes / 1_000_000);
    println!("  Total chunks: {}", total_chunks);
    println!("  Time: {:?}", elapsed);
    println!("  Average speed: {:.2} MB/s", speed);
}

#[test]
fn test_pause_resume_reliability() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_transfer=debug")
        .with_test_writer()
        .try_init()
        .ok();

    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();
    let file_size = 10 * 1024 * 1024; // 10MB

    let file_path = create_test_file(src_dir.path(), "pause_test.bin", file_size);
    let original_checksum = checksum::compute_sha256(&file_path).unwrap();

    let mut sender = LocalFileSender::new();
    let handle = sender.start_transfer(TransferRequest {
        path: file_path,
        dest_name: "pause_test.bin".to_string(),
    }).unwrap();

    let offer = FileOffer {
        id: handle.id,
        file_name: "pause_test.bin".to_string(),
        file_size: handle.total_bytes,
        checksum: sender.file_checksum(&handle).unwrap(),
    };

    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();

    // Transfer with multiple pause/resume cycles
    println!("Testing pause/resume cycles...");
    let mut chunk_count = 0u32;
    let mut pause_count = 0u32;

    loop {
        // Send a few chunks
        for _ in 0..3 {
            if let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
                receiver.receive_chunk(&recv_handle, offset, &data).unwrap();
                chunk_count += 1;
            } else {
                // Transfer complete
                break;
            }
        }

        // Check if complete
        if sender.transfer_state(&handle) == TransferState::Completed {
            break;
        }

        // Pause
        sender.pause(&handle).unwrap();
        pause_count += 1;
        println!("  Paused at {:.1}%", sender.progress(&handle).percentage);

        // Simulate some delay
        std::thread::sleep(Duration::from_millis(10));

        // Resume
        sender.resume(&handle).unwrap();
    }

    // Verify
    assert!(receiver.verify(&recv_handle).unwrap());
    let dest_checksum = checksum::compute_sha256(&dest_dir.path().join("pause_test.bin")).unwrap();
    assert_eq!(original_checksum, dest_checksum);

    println!("✓ Pause/resume test completed");
    println!("  Chunks: {}", chunk_count);
    println!("  Pause/resume cycles: {}", pause_count);
}

// ---------------------------------------------------------------------------
// Clipboard Synchronization Tests
// ---------------------------------------------------------------------------

#[test]
fn test_clipboard_text_synchronization() {
    use clipboard_sync::{PollingClipboardSync, ClipboardFilterMode};

    tracing_subscriber::fmt()
        .with_env_filter("rdcs_transfer=trace")
        .with_test_writer()
        .try_init()
        .ok();

    let clipboard = MockClipboard::new();
    clipboard.set_text("initial text").unwrap();

    let sync = PollingClipboardSync::start(
        Box::new(clipboard),
        Duration::from_millis(50),
        ClipboardFilterMode::TextOnly,
    ).unwrap();

    let rx = sync.local_change().unwrap();

    println!("✓ Clipboard sync started");

    // Wait for polling to stabilize
    std::thread::sleep(Duration::from_millis(100));

    // Apply a remote change
    let remote_content = ClipboardContent::Text("remote text".to_string());
    let event = rdcs_platform::ClipboardEvent {
        content: remote_content.clone(),
        timestamp_us: 1000,
    };

    sync.apply_remote(event).unwrap();

    println!("✓ Remote clipboard change applied");

    // Verify sync is still active
    assert!(sync.is_active());

    // Stop sync
    sync.stop().unwrap();
    std::thread::sleep(Duration::from_millis(100));

    assert!(!sync.is_active());

    println!("✓ Clipboard sync stopped cleanly");
}

#[test]
fn test_clipboard_latency() {
    use clipboard_sync::{PollingClipboardSync, ClipboardFilterMode};

    let clipboard = MockClipboard::new();
    let sync = PollingClipboardSync::start(
        Box::new(clipboard),
        Duration::from_millis(10),
        ClipboardFilterMode::TextOnly,
    ).unwrap();

    // Measure time to apply 100 remote changes
    let start = Instant::now();

    for i in 0..100 {
        let event = rdcs_platform::ClipboardEvent {
            content: ClipboardContent::Text(format!("text {}", i)),
            timestamp_us: i * 1000,
        };
        sync.apply_remote(event).unwrap();
    }

    let elapsed = start.elapsed();
    let avg_latency = elapsed.as_micros() / 100;

    println!("✓ Clipboard sync latency test");
    println!("  Operations: 100");
    println!("  Total time: {:?}", elapsed);
    println!("  Average latency: {} μs", avg_latency);

    // PRD requirement: < 500ms per sync
    // Average should be much lower (< 1ms)
    assert!(avg_latency < 1000, "Average latency {} μs is too high", avg_latency);

    sync.stop().unwrap();
}

// ---------------------------------------------------------------------------
// Error Handling Tests
// ---------------------------------------------------------------------------

#[test]
fn test_transfer_checksum_mismatch() {
    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();
    let file_path = create_test_file(src_dir.path(), "corrupt.bin", 64 * 1024);

    let mut sender = LocalFileSender::new();
    let handle = sender.start_transfer(TransferRequest {
        path: file_path,
        dest_name: "corrupt.bin".to_string(),
    }).unwrap();

    // Create offer with wrong checksum
    let offer = FileOffer {
        id: handle.id,
        file_name: "corrupt.bin".to_string(),
        file_size: handle.total_bytes,
        checksum: [0u8; 32], // Wrong checksum
    };

    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();

    // Transfer all data
    while let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
        receiver.receive_chunk(&recv_handle, offset, &data).unwrap();
    }

    // Verification should fail
    assert!(!receiver.verify(&recv_handle).unwrap());

    println!("✓ Checksum mismatch correctly detected");
}

#[test]
fn test_transfer_cancel_cleanup() {
    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();
    let file_path = create_test_file(src_dir.path(), "cancel.bin", 1024 * 1024);

    let mut sender = LocalFileSender::new();
    let handle = sender.start_transfer(TransferRequest {
        path: file_path,
        dest_name: "cancel.bin".to_string(),
    }).unwrap();

    let offer = FileOffer {
        id: handle.id,
        file_name: "cancel.bin".to_string(),
        file_size: handle.total_bytes,
        checksum: sender.file_checksum(&handle).unwrap(),
    };

    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();

    // Transfer one chunk
    if let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
        receiver.receive_chunk(&recv_handle, offset, &data).unwrap();
    }

    let partial_path = dest_dir.path().join("cancel.bin");
    assert!(partial_path.exists());

    // Cancel
    sender.cancel(&handle).unwrap();
    receiver.cancel(&recv_handle).unwrap();

    // Partial file should be removed
    assert!(!partial_path.exists());

    println!("✓ Cancel cleanup successful");
}

// ---------------------------------------------------------------------------
// Stress Tests
// ---------------------------------------------------------------------------

#[test]
#[ignore] // Run with: cargo test --release -- --ignored
fn stress_test_large_file_1gb() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_transfer=info")
        .with_test_writer()
        .try_init()
        .ok();

    let src_dir = tempfile::tempdir().unwrap();
    let dest_dir = tempfile::tempdir().unwrap();
    let file_size = 1024 * 1024 * 1024; // 1GB

    println!("Creating 1GB test file (this may take a while)...");
    let file_path = create_test_file(src_dir.path(), "huge.bin", file_size);

    let original_checksum = checksum::compute_sha256(&file_path).unwrap();

    let mut sender = LocalFileSender::new();
    let handle = sender.start_transfer(TransferRequest {
        path: file_path,
        dest_name: "huge.bin".to_string(),
    }).unwrap();

    let offer = FileOffer {
        id: handle.id,
        file_name: "huge.bin".to_string(),
        file_size: handle.total_bytes,
        checksum: sender.file_checksum(&handle).unwrap(),
    };

    let mut receiver = LocalFileReceiver::new(dest_dir.path());
    let recv_handle = receiver.accept(offer, dest_dir.path()).unwrap();

    println!("Starting 1GB transfer...");
    let start = Instant::now();
    let mut chunk_count = 0u32;
    let mut last_print = Instant::now();

    while let Some((offset, data)) = sender.send_chunk(handle.id).unwrap() {
        receiver.receive_chunk(&recv_handle, offset, &data).unwrap();
        chunk_count += 1;

        if last_print.elapsed() > Duration::from_secs(5) {
            let progress = sender.progress(&handle);
            let elapsed_so_far = start.elapsed();
            let speed = compute_transfer_speed(progress.sent_bytes, elapsed_so_far);
            println!("  {:.1}% - {} MB / {} MB - {:.2} MB/s",
                progress.percentage,
                progress.sent_bytes / 1_000_000,
                progress.total_bytes / 1_000_000,
                speed);
            last_print = Instant::now();
        }
    }

    let elapsed = start.elapsed();
    let speed = compute_transfer_speed(file_size as u64, elapsed);

    assert!(receiver.verify(&recv_handle).unwrap());
    let dest_checksum = checksum::compute_sha256(&dest_dir.path().join("huge.bin")).unwrap();
    assert_eq!(original_checksum, dest_checksum);

    println!("✓ 1GB transfer completed successfully");
    println!("  Chunks: {}", chunk_count);
    println!("  Time: {:?}", elapsed);
    println!("  Speed: {:.2} MB/s", speed);
}
