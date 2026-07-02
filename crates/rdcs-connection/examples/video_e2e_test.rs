// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end video streaming over DataChannel.
//!
//! This example demonstrates the complete pipeline:
//! 1. Screen capture (mock)
//! 2. H.264 encoding (OpenH264)
//! 3. ICE P2P connection
//! 4. Frame transmission over DataChannel
//! 5. Frame reassembly
//! 6. H.264 decoding
//! 7. Save decoded frames
//!
//! Usage:
//!   RUST_LOG=info cargo run -p rdcs-connection --example video_e2e_test

use rdcs_codec::platform::{NativeVideoDecoder, NativeVideoEncoder};
use rdcs_codec::types::{VideoCodec, VideoResolution};
use rdcs_connection::{
    FrameHeader, FrameReassembler, IceAgent, RealIceAgent, VideoChannel,
};
use rdcs_platform::{CapturedFrame, PixelFormat};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, Level};

const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const FPS: u32 = 30;
const BITRATE: u32 = 2_000_000; // 2 Mbps
const NUM_FRAMES: usize = 30; // Send 30 frames (1 second at 30fps)

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("========================================");
    info!("End-to-End Video Streaming Test");
    info!("========================================");
    info!("Resolution: {}x{}", WIDTH, HEIGHT);
    info!("FPS: {}", FPS);
    info!("Bitrate: {} Mbps", BITRATE / 1_000_000);
    info!("Frames to send: {}", NUM_FRAMES);
    info!("");

    // Configure ICE servers
    let ice_servers = vec![
        "stun:stun.l.google.com:19302".to_string(),
        "stun:stun1.l.google.com:19302".to_string(),
    ];

    info!("Step 1: Creating ICE agents...");
    let mut peer_a = RealIceAgent::new(ice_servers.clone()).await?;
    let mut peer_b = RealIceAgent::new_with_options(ice_servers, false).await?;
    info!("✅ Agents created");
    info!("");

    info!("Step 2: Establishing ICE connection...");
    establish_connection(&mut peer_a, &mut peer_b).await?;
    info!("✅ ICE connected");
    info!("");

    info!("Step 3: Getting DataChannels...");
    let dc_a = peer_a.get_data_channel()?;
    let dc_b = peer_b.get_data_channel()?;
    wait_for_data_channels_open(&dc_a, &dc_b).await?;
    info!("✅ DataChannels ready");
    info!("");

    info!("Step 4: Creating VideoChannels...");
    let video_tx = VideoChannel::new(dc_a);
    let video_rx = VideoChannel::new(dc_b);
    info!("✅ VideoChannels created");
    info!("");

    info!("Step 5: Initializing encoder...");
    let mut encoder = NativeVideoEncoder::new(
        VideoCodec::H264,
        VideoResolution::Custom(WIDTH, HEIGHT),
        FPS,
        BITRATE,
    )?;
    info!("✅ Encoder ready");
    info!("");

    info!("Step 6: Initializing decoder...");
    let mut decoder = NativeVideoDecoder::new(VideoCodec::H264)?;
    info!("✅ Decoder ready");
    info!("");

    info!("Step 7: Setting up receiver...");
    let received_frames = Arc::new(Mutex::new(Vec::new()));
    let received_frames_clone = received_frames.clone();
    let reassembler = Arc::new(Mutex::new(FrameReassembler::new(10)));
    let mut decoder_shared = Arc::new(Mutex::new(decoder));

    video_rx.on_message({
        let reassembler = reassembler.clone();
        let received_frames = received_frames_clone.clone();
        let decoder = decoder_shared.clone();

        move |chunk| {
            if chunk.len() < 8 {
                info!("⚠️  Chunk too small: {} bytes", chunk.len());
                return;
            }

            match FrameHeader::deserialize(&chunk[..8]) {
                Ok(header) => {
                    let data = chunk[8..].to_vec();

                    let reassembler = reassembler.clone();
                    let received_frames = received_frames.clone();
                    let decoder = decoder.clone();

                    tokio::spawn(async move {
                        if let Some((frame_id, complete_frame, is_keyframe)) =
                            reassembler.lock().await.add_chunk(header, data)
                        {
                            info!(
                                "📥 Frame {} received: {} bytes (keyframe: {})",
                                frame_id,
                                complete_frame.len(),
                                is_keyframe
                            );

                            // Decode frame
                            let decode_start = Instant::now();
                            match decoder.lock().await.decode_to_captured_frame(&complete_frame) {
                                Ok(decoded) => {
                                    let decode_time = decode_start.elapsed();
                                    info!(
                                        "🎬 Frame {} decoded: {}x{} in {:.2}ms",
                                        frame_id,
                                        decoded.width,
                                        decoded.height,
                                        decode_time.as_secs_f64() * 1000.0
                                    );

                                    received_frames.lock().await.push((
                                        frame_id,
                                        complete_frame.len(),
                                        decode_time,
                                    ));
                                }
                                Err(e) => {
                                    info!("❌ Failed to decode frame {}: {}", frame_id, e);
                                }
                            }
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

    info!("Step 8: Generating and sending {} frames...", NUM_FRAMES);
    let send_start = Instant::now();

    for frame_id in 0..NUM_FRAMES as u32 {
        // Generate test frame
        let captured_frame = generate_test_frame(frame_id);

        // Request keyframe every 30 frames
        if frame_id % 30 == 0 {
            encoder.request_keyframe();
        }

        // Encode
        let encode_start = Instant::now();
        let encoded = encoder.encode_captured_frame(&captured_frame)?;
        let encode_time = encode_start.elapsed();

        let is_keyframe = frame_id % 30 == 0;

        info!(
            "📤 Frame {} encoded: {} bytes in {:.2}ms (keyframe: {})",
            frame_id,
            encoded.len(),
            encode_time.as_secs_f64() * 1000.0,
            is_keyframe
        );

        // Send with protocol header
        send_frame_with_header(&video_tx, frame_id, &encoded, is_keyframe).await?;

        // Sleep to simulate frame rate
        tokio::time::sleep(Duration::from_millis(33)).await; // ~30fps
    }

    let total_send_time = send_start.elapsed();
    info!("");
    info!("✅ All frames sent in {:.2}s", total_send_time.as_secs_f64());
    info!("");

    info!("Step 9: Waiting for frames to be received...");
    tokio::time::sleep(Duration::from_secs(3)).await;

    let received = received_frames.lock().await;
    info!("");
    info!("========================================");
    info!("Test Results");
    info!("========================================");
    info!("Frames sent: {}", NUM_FRAMES);
    info!("Frames received: {}", received.len());
    info!("");

    if !received.is_empty() {
        let avg_decode_time: f64 = received
            .iter()
            .map(|(_, _, t)| t.as_secs_f64())
            .sum::<f64>()
            / received.len() as f64;

        info!("Average decode time: {:.2}ms", avg_decode_time * 1000.0);
        info!("");

        info!("Received frames:");
        for (frame_id, size, decode_time) in received.iter().take(10) {
            info!(
                "  - Frame {}: {} bytes, decoded in {:.2}ms",
                frame_id,
                size,
                decode_time.as_secs_f64() * 1000.0
            );
        }

        if received.len() > 10 {
            info!("  ... and {} more", received.len() - 10);
        }
    }

    info!("");

    let success_rate = (received.len() as f64 / NUM_FRAMES as f64) * 100.0;
    if success_rate >= 95.0 {
        info!("✅ Test passed! Success rate: {:.1}%", success_rate);
        info!("========================================");
        Ok(())
    } else {
        info!("⚠️  Test completed with losses. Success rate: {:.1}%", success_rate);
        info!("========================================");
        Err(format!("Success rate below 95%: {:.1}%", success_rate).into())
    }
}

/// Generate a test frame with a pattern.
fn generate_test_frame(frame_id: u32) -> CapturedFrame {
    let size = (WIDTH * HEIGHT * 4) as usize; // BGRA
    let mut data = vec![0u8; size];

    // Fill with a gradient pattern that changes per frame
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let offset = ((y * WIDTH + x) * 4) as usize;
            let r = ((x as f32 / WIDTH as f32) * 255.0) as u8;
            let g = ((y as f32 / HEIGHT as f32) * 255.0) as u8;
            let b = ((frame_id % 255) as f32) as u8;

            data[offset] = b; // B
            data[offset + 1] = g; // G
            data[offset + 2] = r; // R
            data[offset + 3] = 255; // A
        }
    }

    CapturedFrame {
        data: data.into(),
        width: WIDTH,
        height: HEIGHT,
        pixel_format: PixelFormat::Bgra,
        stride: WIDTH * 4,
        display_id: 0,
        timestamp_us: 0,
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

/// Establish ICE connection between two peers.
async fn establish_connection(
    peer_a: &mut RealIceAgent,
    peer_b: &mut RealIceAgent,
) -> Result<(), Box<dyn std::error::Error>> {
    // Peer A (offerer) gathers candidates and creates offer
    peer_a.gather_candidates()?;
    let offer = peer_a.create_offer()?;

    // Peer B (answerer) receives offer BEFORE gathering candidates
    peer_b.set_remote_offer(&offer)?;

    // Now Peer B can gather candidates (this will create answer)
    peer_b.gather_candidates()?;

    // Get Peer B's credentials and create answer
    let (ufrag_b, pwd_b, fingerprint_b) = peer_b.get_local_credentials_with_fingerprint()?;
    let answer = rdcs_connection::ice::SdpAnswer {
        session_id: offer.session_id.clone(),
        ufrag: ufrag_b,
        pwd: pwd_b,
        fingerprint: fingerprint_b,
        candidates: peer_b.gather_candidates()?,
    };

    // Handle answer on A
    peer_a.handle_answer(answer.clone())?;

    // Exchange candidates
    peer_a.set_remote_candidates(answer.candidates)?;
    peer_b.set_remote_candidates(offer.candidates)?;

    // Wait for connection
    let timeout = Duration::from_secs(10);
    let start = tokio::time::Instant::now();

    loop {
        let state_a = peer_a.connection_state();
        let state_b = peer_b.connection_state();

        if state_a == rdcs_connection::ice::IceState::Connected
            && state_b == rdcs_connection::ice::IceState::Connected
        {
            break;
        }

        if state_a == rdcs_connection::ice::IceState::Failed
            || state_b == rdcs_connection::ice::IceState::Failed
        {
            return Err("ICE connection failed".into());
        }

        if start.elapsed() > timeout {
            return Err("ICE connection timeout".into());
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

/// Wait for DataChannels to open.
async fn wait_for_data_channels_open(
    dc_a: &Arc<webrtc::data_channel::RTCDataChannel>,
    dc_b: &Arc<webrtc::data_channel::RTCDataChannel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let timeout = Duration::from_secs(5);
    let start = tokio::time::Instant::now();

    loop {
        let state_a = dc_a.ready_state();
        let state_b = dc_b.ready_state();

        if state_a == webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
            && state_b == webrtc::data_channel::data_channel_state::RTCDataChannelState::Open
        {
            break;
        }

        if start.elapsed() > timeout {
            return Err(format!(
                "DataChannel open timeout. States: A={:?}, B={:?}",
                state_a, state_b
            )
            .into());
        }

        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
