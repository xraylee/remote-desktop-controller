// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Video transmission over DataChannel test.
//!
//! This example demonstrates:
//! 1. Establishing ICE P2P connection
//! 2. Getting the DataChannel
//! 3. Wrapping it with VideoChannel
//! 4. Sending and receiving video frames with chunking
//!
//! Usage:
//!   RUST_LOG=info cargo run -p rdcs-connection --example video_datachannel_test

use rdcs_connection::{
    FrameHeader, FrameReassembler, IceAgent, RealIceAgent, VideoChannel,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("========================================");
    info!("Video DataChannel Transmission Test");
    info!("========================================");
    info!("");

    // Configure ICE servers
    let ice_servers = vec![
        "stun:stun.l.google.com:19302".to_string(),
        "stun:stun1.l.google.com:19302".to_string(),
    ];

    info!("Step 1: Creating ICE agents...");
    let mut peer_a = RealIceAgent::new(ice_servers.clone()).await?;
    let mut peer_b = RealIceAgent::new(ice_servers).await?;
    info!("✅ Agents created");
    info!("");

    info!("Step 2: Gathering candidates for Peer A...");
    let candidates_a = peer_a.gather_candidates()?;
    info!("✅ Peer A gathered {} candidates", candidates_a.len());
    info!("");

    info!("Step 3: Creating offer...");
    let offer = peer_a.create_offer()?;
    info!("✅ Offer created");
    info!("");

    info!("Step 4: Setting remote offer on Peer B...");
    peer_b.set_remote_offer(&offer)?;
    info!("✅ Remote offer set");
    info!("");

    info!("Step 5: Gathering candidates for Peer B...");
    let candidates_b = peer_b.gather_candidates()?;
    info!("✅ Peer B gathered {} candidates", candidates_b.len());
    info!("");

    info!("Step 6: Creating answer...");
    let (ufrag_b, pwd_b, fingerprint_b) = peer_b.get_local_credentials_with_fingerprint()?;
    let answer = rdcs_connection::ice::SdpAnswer {
        session_id: offer.session_id.clone(),
        ufrag: ufrag_b,
        pwd: pwd_b,
        fingerprint: fingerprint_b,
        candidates: candidates_b,
    };
    info!("✅ Answer created");
    info!("");

    info!("Step 7: Handling answer on Peer A...");
    peer_a.handle_answer(answer)?;
    info!("✅ Answer handled");
    info!("");

    info!("Step 8: Adding remote candidates...");
    peer_a.set_remote_candidates(offer.candidates)?;
    info!("✅ Remote candidates added");
    info!("");

    info!("Step 9: Waiting for ICE connection...");
    let timeout = Duration::from_secs(10);
    let start = tokio::time::Instant::now();

    loop {
        let state_a = peer_a.connection_state();
        let state_b = peer_b.connection_state();

        if state_a == rdcs_connection::ice::IceState::Connected
            && state_b == rdcs_connection::ice::IceState::Connected
        {
            info!("✅ ICE connection established!");
            break;
        }

        if state_a == rdcs_connection::ice::IceState::Failed
            || state_b == rdcs_connection::ice::IceState::Failed
        {
            return Err("ICE connection failed".into());
        }

        if start.elapsed() > timeout {
            return Err("Connection timeout".into());
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    info!("");

    info!("Step 10: Getting DataChannels...");
    let dc_a = peer_a.get_data_channel()?;
    let dc_b = peer_b.get_data_channel()?;
    info!("✅ DataChannels ready");
    info!("");

    info!("Step 11: Creating VideoChannels...");
    let video_tx = VideoChannel::new(dc_a);
    let video_rx = VideoChannel::new(dc_b);
    info!("✅ VideoChannels created");
    info!("");

    info!("Step 12: Setting up receiver...");
    let received_frames = Arc::new(Mutex::new(Vec::new()));
    let received_frames_clone = received_frames.clone();
    let reassembler = Arc::new(Mutex::new(FrameReassembler::new(10)));

    video_rx.on_message({
        let reassembler = reassembler.clone();
        let received_frames = received_frames_clone.clone();

        move |chunk| {
            if chunk.len() < 8 {
                info!("⚠️  Received chunk too small: {} bytes", chunk.len());
                return;
            }

            match FrameHeader::deserialize(&chunk[..8]) {
                Ok(header) => {
                    let data = chunk[8..].to_vec();

                    let reassembler = reassembler.clone();
                    let received_frames = received_frames.clone();

                    tokio::spawn(async move {
                        if let Some((frame_id, complete_frame, is_keyframe)) =
                            reassembler.lock().await.add_chunk(header, data)
                        {
                            info!(
                                "📥 Received complete frame {}: {} bytes (keyframe: {})",
                                frame_id,
                                complete_frame.len(),
                                is_keyframe
                            );

                            received_frames.lock().await.push((frame_id, complete_frame.len()));
                        }
                    });
                }
                Err(e) => {
                    info!("⚠️  Failed to parse header: {}", e);
                }
            }
        }
    });

    info!("✅ Receiver configured");
    info!("");

    info!("Step 13: Sending test frames...");

    // Test 1: Small frame (single chunk)
    let small_frame = vec![0xAA; 1024]; // 1KB
    let frame_id = 1;
    send_frame_with_header(&video_tx, frame_id, &small_frame, true).await?;
    info!("📤 Sent frame {} (small, {} bytes)", frame_id, small_frame.len());

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test 2: Medium frame (multiple chunks)
    let medium_frame = vec![0xBB; 32_768]; // 32KB
    let frame_id = 2;
    send_frame_with_header(&video_tx, frame_id, &medium_frame, false).await?;
    info!("📤 Sent frame {} (medium, {} bytes)", frame_id, medium_frame.len());

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Test 3: Large frame (many chunks)
    let large_frame = vec![0xCC; 100_000]; // 100KB
    let frame_id = 3;
    send_frame_with_header(&video_tx, frame_id, &large_frame, false).await?;
    info!("📤 Sent frame {} (large, {} bytes)", frame_id, large_frame.len());

    info!("");
    info!("Step 14: Waiting for frames to be received...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    let received = received_frames.lock().await;
    info!("");
    info!("========================================");
    info!("Test Results");
    info!("========================================");
    info!("Frames sent: 3");
    info!("Frames received: {}", received.len());

    for (frame_id, size) in received.iter() {
        info!("  - Frame {}: {} bytes", frame_id, size);
    }

    if received.len() == 3 {
        info!("");
        info!("✅ All frames received successfully!");
        info!("========================================");
        Ok(())
    } else {
        info!("");
        info!("❌ Some frames were lost!");
        info!("========================================");
        Err("Frame loss detected".into())
    }
}

/// Send a frame with protocol header and chunking.
async fn send_frame_with_header(
    video_tx: &VideoChannel,
    frame_id: u32,
    frame_data: &[u8],
    is_keyframe: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    const CHUNK_SIZE: usize = 16_384 - 8; // Reserve 8 bytes for header

    let total_chunks = ((frame_data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE) as u8;

    for (chunk_index, chunk_data) in frame_data.chunks(CHUNK_SIZE).enumerate() {
        let header = FrameHeader {
            frame_id,
            is_keyframe,
            chunk_index: chunk_index as u8,
            total_chunks,
        };

        let mut message = header.serialize().to_vec();
        message.extend_from_slice(chunk_data);

        video_tx.send_frame(&message).await?;
    }

    Ok(())
}
