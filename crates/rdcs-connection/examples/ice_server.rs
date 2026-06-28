// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! ICE Server - acts as the offerer in ICE negotiation.
//!
//! Usage:
//!   cargo run -p rdcs-connection --example ice_server
//!
//! This will:
//! 1. Create an ICE agent
//! 2. Generate an Offer
//! 3. Print the Offer as JSON
//! 4. Wait for Answer input (paste JSON and press Ctrl+D)
//! 5. Establish P2P connection

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
    info!("ICE Server (Offerer)");
    info!("========================================");
    info!("");

    // Configure ICE servers
    let ice_servers = vec![
        "stun:stun.l.google.com:19302".to_string(),
        "stun:stun1.l.google.com:19302".to_string(),
    ];

    info!("Step 1: Creating ICE agent...");
    let mut agent = RealIceAgent::new(ice_servers).await?;
    info!("✅ ICE agent created");
    info!("");

    info!("Step 2: Creating offer and gathering candidates...");
    let offer = agent.create_offer()?;
    info!("✅ Offer created with {} candidates", offer.candidates.len());
    info!("");

    // Convert to JSON
    let offer_msg = OfferMessage {
        session_id: offer.session_id.clone(),
        ufrag: offer.ufrag.clone(),
        pwd: offer.pwd.clone(),
        fingerprint: offer.fingerprint.clone(),
        candidates: offer
            .candidates
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
    info!("📋 OFFER (copy this to the client)");
    info!("========================================");
    println!("{}", serde_json::to_string_pretty(&offer_msg)?);
    info!("========================================");
    info!("");

    info!("Step 3: Waiting for ANSWER...");
    info!("Paste the Answer JSON and press Ctrl+D (Linux/Mac) or Ctrl+Z Enter (Windows):");
    info!("");

    // Read answer from stdin
    let mut answer_json = String::new();
    io::stdin().read_to_string(&mut answer_json)?;

    let answer_msg: AnswerMessage = serde_json::from_str(&answer_json)?;
    info!("✅ Answer received with {} candidates", answer_msg.candidates.len());
    info!("");

    // Convert back to IceCandidate
    let answer = rdcs_connection::ice::SdpAnswer {
        session_id: answer_msg.session_id,
        ufrag: answer_msg.ufrag,
        pwd: answer_msg.pwd,
        fingerprint: answer_msg.fingerprint,
        candidates: answer_msg
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

    info!("Step 4: Processing answer...");
    agent.handle_answer(answer)?;
    info!("✅ Answer processed");
    info!("");

    info!("Step 5: Waiting for ICE connection...");
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
