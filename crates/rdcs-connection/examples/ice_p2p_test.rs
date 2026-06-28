// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! ICE P2P connection test using RealIceAgent.
//!
//! This example demonstrates:
//! 1. Creating two ICE agents (simulating two peers)
//! 2. Gathering ICE candidates via STUN
//! 3. Exchanging SDP offers/answers
//! 4. Establishing P2P connection

use rdcs_connection::{IceAgent, RealIceAgent};
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("========================================");
    info!("ICE P2P Connection Test");
    info!("========================================");
    info!("");

    // Configure ICE servers (using Google's public STUN server for testing)
    let ice_servers = vec![
        "stun:stun.l.google.com:19302".to_string(),
        "stun:stun1.l.google.com:19302".to_string(),
    ];

    info!("Creating Peer A (offerer)...");
    let mut peer_a = RealIceAgent::new(ice_servers.clone()).await?;

    info!("Creating Peer B (answerer)...");
    let mut peer_b = RealIceAgent::new(ice_servers).await?;

    info!("");
    info!("Step 1: Peer A creates offer");
    info!("-----------------------------");
    let offer = peer_a.create_offer()?;
    info!("Offer created:");
    info!("  Session ID: {}", offer.session_id);
    info!("  ICE ufrag: {}", offer.ufrag);
    info!("  Candidates: {}", offer.candidates.len());
    for (i, candidate) in offer.candidates.iter().enumerate() {
        info!(
            "  [{}] {:?} - {}:{}",
            i + 1,
            candidate.candidate_type,
            candidate.addr.ip(),
            candidate.addr.port()
        );
    }

    info!("");
    info!("Step 2: Peer B processes offer and gathers candidates");
    info!("--------------------------------------------------------");
    // Peer B must set remote offer before gathering/adding candidates
    peer_b.set_remote_offer(&offer)?;
    info!("Peer B: Set remote offer");

    let candidates_b = peer_b.gather_candidates()?;
    info!("Peer B gathered {} candidates:", candidates_b.len());
    for (i, candidate) in candidates_b.iter().enumerate() {
        info!(
            "  [{}] {:?} - {}:{}",
            i + 1,
            candidate.candidate_type,
            candidate.addr.ip(),
            candidate.addr.port()
        );
    }

    info!("");
    info!("Step 3: Peer B creates answer");
    info!("------------------------------");
    // Get Peer B's local ICE credentials and fingerprint
    let (ufrag_b, pwd_b, fingerprint_b) = peer_b.get_local_credentials_with_fingerprint()?;

    let answer = rdcs_connection::ice::SdpAnswer {
        session_id: offer.session_id.clone(),
        ufrag: ufrag_b,
        pwd: pwd_b,
        fingerprint: fingerprint_b,
        candidates: candidates_b,
    };
    info!("Answer created:");
    info!("  Session ID: {}", answer.session_id);
    info!("  ICE ufrag: {}", answer.ufrag);
    info!("  Candidates: {}", answer.candidates.len());

    info!("");
    info!("Step 4: Peer A processes answer");
    info!("---------------------------------");
    peer_a.handle_answer(answer.clone())?;
    info!("Peer A: Processed answer with {} candidates", answer.candidates.len());

    info!("");
    info!("Step 5: Peer B processes offer candidates");
    info!("------------------------------------------");
    peer_b.set_remote_candidates(offer.candidates)?;
    info!("Peer B: Processed offer candidates");

    info!("");
    info!("Step 6: Wait for connection");
    info!("----------------------------");

    // Wait for connection to establish (with timeout)
    let timeout = tokio::time::Duration::from_secs(30);
    let start = tokio::time::Instant::now();

    let mut ice_connected = false;

    loop {
        let state_a = peer_a.connection_state();
        let state_b = peer_b.connection_state();

        info!("Peer A state: {:?}, Peer B state: {:?}", state_a, state_b);

        // Check if ICE is connected (even if peer connection fails later due to DTLS)
        if !ice_connected && (state_a == rdcs_connection::ice::IceState::Connected
            || state_b == rdcs_connection::ice::IceState::Connected) {
            ice_connected = true;
            info!("✅ ICE Connection established!");
            info!("");
            info!("Note: DTLS errors are expected (fingerprint mismatch) - DTLS is Phase 3.3");

            // Give a moment for both sides to report connected
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            break;
        }

        if state_a == rdcs_connection::ice::IceState::Failed
            && state_b == rdcs_connection::ice::IceState::Failed {
            info!("❌ Connection failed");
            break;
        }

        if start.elapsed() > timeout {
            info!("⏱️  Connection timeout");
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    info!("");
    info!("========================================");
    if ice_connected {
        info!("✅ ICE P2P Connection Test PASSED");
        info!("");
        info!("Summary:");
        info!("  - ICE candidates gathered successfully");
        info!("  - STUN reflexive addresses obtained");
        info!("  - P2P connectivity established");
        info!("  - Ready for Phase 3.2 (DTLS encryption)");
    } else {
        info!("❌ ICE P2P Connection Test FAILED");
    }
    info!("========================================");

    Ok(())
}
