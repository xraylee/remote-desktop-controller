// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! ICE Client - acts as the answerer in ICE negotiation.
//!
//! Usage:
//!   cargo run -p rdcs-connection --example ice_client
//!
//! This will:
//! 1. Wait for Offer input (paste JSON and press Ctrl+D)
//! 2. Create an ICE agent
//! 3. Process the Offer
//! 4. Generate an Answer
//! 5. Print the Answer as JSON
//! 6. Establish P2P connection

use rdcs_connection::{IceAgent, RealIceAgent};
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use tracing::{info, Level};

#[derive(Serialize, Deserialize)]
struct OfferMessage {
    session_id: String,
    ufrag: String,
    pwd: String,
    fingerprint: String,
    candidates: Vec<CandidateJson>,
}

#[derive(Serialize, Deserialize)]
struct AnswerMessage {
    session_id: String,
    ufrag: String,
    pwd: String,
    fingerprint: String,
    candidates: Vec<CandidateJson>,
}

#[derive(Serialize, Deserialize)]
struct CandidateJson {
    foundation: String,
    component: u32,
    protocol: String,
    priority: u64,
    address: String,
    port: u16,
    typ: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("========================================");
    info!("ICE Client (Answerer)");
    info!("========================================");
    info!("");

    info!("Step 1: Waiting for OFFER...");
    info!("Paste the Offer JSON and press Ctrl+D (Linux/Mac) or Ctrl+Z Enter (Windows):");
    info!("");

    // Read offer from stdin
    let mut offer_json = String::new();
    io::stdin().read_to_string(&mut offer_json)?;

    let offer_msg: OfferMessage = serde_json::from_str(&offer_json)?;
    info!("✅ Offer received with {} candidates", offer_msg.candidates.len());
    info!("");

    // Configure ICE servers
    let ice_servers = vec![
        "stun:stun.l.google.com:19302".to_string(),
        "stun:stun1.l.google.com:19302".to_string(),
    ];

    info!("Step 2: Creating ICE agent...");
    let mut agent = RealIceAgent::new(ice_servers).await?;
    info!("✅ ICE agent created");
    info!("");

    // Convert to IceCandidate
    let offer = rdcs_connection::ice::SdpOffer {
        session_id: offer_msg.session_id.clone(),
        ufrag: offer_msg.ufrag,
        pwd: offer_msg.pwd,
        fingerprint: offer_msg.fingerprint,
        candidates: offer_msg
            .candidates
            .iter()
            .map(|c| {
                let typ = match c.typ.as_str() {
                    "Host" => rdcs_connection::ice::CandidateType::Host,
                    "Srflx" => rdcs_connection::ice::CandidateType::Srflx,
                    "Prflx" => rdcs_connection::ice::CandidateType::Prflx,
                    "Relay" => rdcs_connection::ice::CandidateType::Relay,
                    _ => rdcs_connection::ice::CandidateType::Host,
                };

                rdcs_connection::ice::IceCandidate {
                    foundation: c.foundation.clone(),
                    component: c.component,
                    protocol: c.protocol.clone(),
                    priority: c.priority,
                    addr: format!("{}:{}", c.address, c.port).parse().unwrap(),
                    candidate_type: typ,
                }
            })
            .collect(),
    };

    info!("Step 3: Setting remote offer...");
    agent.set_remote_offer(&offer)?;
    info!("✅ Remote offer set");
    info!("");

    info!("Step 4: Gathering candidates...");
    let candidates = agent.gather_candidates()?;
    info!("✅ Gathered {} candidates", candidates.len());
    info!("");

    info!("Step 5: Creating answer...");
    let (ufrag, pwd, fingerprint) = agent.get_local_credentials_with_fingerprint()?;

    let answer_msg = AnswerMessage {
        session_id: offer_msg.session_id,
        ufrag,
        pwd,
        fingerprint,
        candidates: candidates
            .iter()
            .map(|c| CandidateJson {
                foundation: c.foundation.clone(),
                component: c.component,
                protocol: c.protocol.clone(),
                priority: c.priority,
                address: c.addr.ip().to_string(),
                port: c.addr.port(),
                typ: format!("{:?}", c.candidate_type),
            })
            .collect(),
    };

    info!("========================================");
    info!("📋 ANSWER (copy this back to the server)");
    info!("========================================");
    println!("{}", serde_json::to_string_pretty(&answer_msg)?);
    info!("========================================");
    info!("");

    info!("Step 6: Adding remote candidates...");
    agent.set_remote_candidates(offer.candidates)?;
    info!("✅ Remote candidates added");
    info!("");

    info!("Step 7: Waiting for ICE connection...");
    let timeout = tokio::time::Duration::from_secs(30);
    let start = tokio::time::Instant::now();

    loop {
        let state = agent.connection_state();
        info!("ICE state: {:?}", state);

        if state == rdcs_connection::ice::IceState::Connected {
            info!("");
            info!("========================================");
            info!("✅ ICE CONNECTION ESTABLISHED!");
            info!("========================================");
            info!("");
            info!("Connection successful. Press Ctrl+C to exit.");

            // Keep running
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
            break;
        }

        if state == rdcs_connection::ice::IceState::Failed {
            info!("");
            info!("❌ ICE connection failed");
            return Err("Connection failed".into());
        }

        if start.elapsed() > timeout {
            info!("");
            info!("⏱️  Connection timeout");
            return Err("Connection timeout".into());
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Ok(())
}
