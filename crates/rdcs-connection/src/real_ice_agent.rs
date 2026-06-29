// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Real ICE agent implementation using webrtc-rs library.

use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_candidate::{RTCIceCandidate, RTCIceCandidateInit};
use webrtc::ice_transport::ice_candidate_type::RTCIceCandidateType;
use webrtc::ice_transport::ice_connection_state::RTCIceConnectionState;
use webrtc::ice_transport::ice_gathering_state::RTCIceGatheringState;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::signaling_state::RTCSignalingState;
use webrtc::peer_connection::RTCPeerConnection;

use crate::ice::{
    CandidateType, IceAgent, IceCandidate, IceState, SdpAnswer, SdpOffer,
};
use crate::ConnectionError;

/// Real ICE agent implementation using WebRTC.
pub struct RealIceAgent {
    peer_connection: Arc<RTCPeerConnection>,
    data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
    local_candidates: Arc<Mutex<Vec<IceCandidate>>>,
    state: Arc<Mutex<IceState>>,
    ice_connection_state: Arc<Mutex<IceState>>,
    session_id: String,
    #[allow(dead_code)]
    ufrag: String,
    #[allow(dead_code)]
    pwd: String,
}

impl RealIceAgent {
    /// Create a new real ICE agent with the given ICE servers.
    pub async fn new(ice_servers: Vec<String>) -> Result<Self, ConnectionError> {
        Self::new_with_options(ice_servers, true).await
    }

    /// Create a new real ICE agent with options.
    ///
    /// # Arguments
    /// * `ice_servers` - List of STUN/TURN server URLs
    /// * `create_data_channel` - If true, creates a DataChannel (for offerer). If false, waits for remote DataChannel (for answerer).
    pub async fn new_with_options(
        ice_servers: Vec<String>,
        create_data_channel: bool,
    ) -> Result<Self, ConnectionError> {
        info!("Creating real ICE agent with {} servers (create_dc: {})", ice_servers.len(), create_data_channel);

        // Create a MediaEngine (not used for data-only connections, but required by webrtc-rs)
        let mut media_engine = MediaEngine::default();

        // Create an InterceptorRegistry
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut media_engine)
            .map_err(|e| ConnectionError::IceError(format!("Failed to register interceptors: {}", e)))?;

        // Create the API object
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .with_interceptor_registry(registry)
            .build();

        // Configure ICE servers
        let rtc_ice_servers: Vec<RTCIceServer> = ice_servers
            .into_iter()
            .map(|url| RTCIceServer {
                urls: vec![url],
                ..Default::default()
            })
            .collect();

        let config = RTCConfiguration {
            ice_servers: rtc_ice_servers,
            ..Default::default()
        };

        // Create PeerConnection
        let peer_connection = Arc::new(
            api.new_peer_connection(config)
                .await
                .map_err(|e| ConnectionError::IceError(format!("Failed to create peer connection: {}", e)))?,
        );

        let local_candidates = Arc::new(Mutex::new(Vec::new()));
        let state = Arc::new(Mutex::new(IceState::New));
        let ice_connection_state = Arc::new(Mutex::new(IceState::New));
        let data_channel = Arc::new(Mutex::new(None));

        // Set up ICE candidate handler
        let local_candidates_clone = local_candidates.clone();
        peer_connection
            .on_ice_candidate(Box::new(move |candidate: Option<RTCIceCandidate>| {
                let local_candidates = local_candidates_clone.clone();
                Box::pin(async move {
                    if let Some(c) = candidate {
                        debug!("ICE candidate gathered: {:?}", c);
                        let ice_candidate = convert_webrtc_candidate(&c);
                        local_candidates.lock().await.push(ice_candidate);
                    } else {
                        debug!("ICE gathering complete");
                    }
                })
            }));

        // Set up ICE connection state change handler (separate from peer connection state)
        let ice_state_clone = ice_connection_state.clone();
        peer_connection
            .on_ice_connection_state_change(Box::new(move |s: RTCIceConnectionState| {
                let ice_state = ice_state_clone.clone();
                Box::pin(async move {
                    info!("ICE connection state changed: {:?}", s);
                    let new_state = match s {
                        RTCIceConnectionState::New => IceState::New,
                        RTCIceConnectionState::Checking => IceState::Checking,
                        RTCIceConnectionState::Connected => IceState::Connected,
                        RTCIceConnectionState::Completed => IceState::Connected,
                        RTCIceConnectionState::Failed => IceState::Failed,
                        RTCIceConnectionState::Disconnected => IceState::Failed,
                        RTCIceConnectionState::Closed => IceState::Closed,
                        _ => IceState::New,
                    };
                    *ice_state.lock().await = new_state;
                })
            }));

        // Set up connection state change handler (for debugging)
        let state_clone = state.clone();
        peer_connection
            .on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
                let state = state_clone.clone();
                Box::pin(async move {
                    info!("PeerConnection state changed: {:?}", s);
                    let new_state = match s {
                        RTCPeerConnectionState::New => IceState::New,
                        RTCPeerConnectionState::Connecting => IceState::Checking,
                        RTCPeerConnectionState::Connected => IceState::Connected,
                        RTCPeerConnectionState::Failed => IceState::Failed,
                        RTCPeerConnectionState::Disconnected => IceState::Failed,
                        RTCPeerConnectionState::Closed => IceState::Closed,
                        _ => IceState::New,
                    };
                    *state.lock().await = new_state;
                })
            }));

        // Set up on_data_channel handler for answerer side
        // When the remote peer creates a DataChannel, this handler will be called
        let data_channel_clone = data_channel.clone();
        peer_connection.on_data_channel(Box::new(move |dc| {
            let data_channel = data_channel_clone.clone();
            Box::pin(async move {
                info!("📨 Received DataChannel from remote peer: {}", dc.label());

                // Replace the local DataChannel with the one from remote
                let mut dc_lock = data_channel.lock().await;
                if dc_lock.is_none() {
                    info!("Setting up DataChannel from remote offer");
                    *dc_lock = Some(dc);
                } else {
                    info!("DataChannel already exists (we are the offerer)");
                }
            })
        }));

        // Create a data channel to trigger ICE candidate gathering
        // Note: Only the offerer's DataChannel will be used for actual communication
        if create_data_channel {
            let dc = peer_connection
                .create_data_channel("rdcs-control", None)
                .await
                .map_err(|e| ConnectionError::IceError(format!("Failed to create data channel: {}", e)))?;

            info!("Created data channel: {}", dc.label());

            // Store the data channel for later use (dc is already Arc<RTCDataChannel>)
            *data_channel.lock().await = Some(dc);
        } else {
            info!("Waiting for remote peer to create DataChannel (answerer mode)");
        }

        Ok(Self {
            peer_connection,
            data_channel,
            local_candidates,
            state,
            ice_connection_state,
            session_id: uuid::Uuid::new_v4().to_string(),
            ufrag: uuid::Uuid::new_v4().to_string(),
            pwd: uuid::Uuid::new_v4().to_string(),
        })
    }

    /// Set remote offer (for answerer side).
    /// This must be called before set_remote_candidates on the answerer.
    pub fn set_remote_offer(&self, offer: &SdpOffer) -> Result<(), ConnectionError> {
        let peer_connection = self.peer_connection.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Format offer as SDP
                let sdp = format_sdp_offer(offer);

                let remote_desc = webrtc::peer_connection::sdp::session_description::RTCSessionDescription::offer(sdp)
                    .map_err(|e| ConnectionError::IceError(format!("Failed to create offer: {}", e)))?;

                peer_connection
                    .set_remote_description(remote_desc)
                    .await
                    .map_err(|e| {
                        ConnectionError::IceError(format!("Failed to set remote description: {}", e))
                    })?;

                debug!("Set remote offer");
                Ok(())
            })
        })
    }

    /// Get local ICE credentials from the current local description.
    pub fn get_local_credentials(&self) -> Result<(String, String), ConnectionError> {
        let peer_connection = self.peer_connection.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let local_desc = peer_connection.local_description().await;

                if let Some(desc) = local_desc {
                    extract_ice_credentials(&desc.sdp)
                } else {
                    Err(ConnectionError::IceError("No local description set".to_string()))
                }
            })
        })
    }

    /// Get local ICE credentials and fingerprint from the current local description.
    pub fn get_local_credentials_with_fingerprint(&self) -> Result<(String, String, String), ConnectionError> {
        let peer_connection = self.peer_connection.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let local_desc = peer_connection.local_description().await;

                if let Some(desc) = local_desc {
                    let (ufrag, pwd) = extract_ice_credentials(&desc.sdp)?;
                    let fingerprint = extract_fingerprint(&desc.sdp)?;
                    Ok((ufrag, pwd, fingerprint))
                } else {
                    Err(ConnectionError::IceError("No local description set".to_string()))
                }
            })
        })
    }

    /// Get the DataChannel for video transmission.
    pub fn get_data_channel(&self) -> Result<Arc<RTCDataChannel>, ConnectionError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Some(dc) = self.data_channel.lock().await.as_ref() {
                    Ok(dc.clone())
                } else {
                    Err(ConnectionError::IceError("DataChannel not ready".to_string()))
                }
            })
        })
    }

    /// Wait for ICE gathering to complete.
    #[allow(dead_code)]
    async fn wait_for_gathering(&self) -> Result<(), ConnectionError> {
        // Wait for gathering state to become complete
        let timeout = tokio::time::Duration::from_secs(10);
        let start = tokio::time::Instant::now();

        loop {
            let gathering_state = self.peer_connection.ice_gathering_state();

            if gathering_state == RTCIceGatheringState::Complete {
                debug!("ICE gathering complete");
                return Ok(());
            }

            if start.elapsed() > timeout {
                warn!("ICE gathering timeout");
                return Err(ConnectionError::Timeout);
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

impl IceAgent for RealIceAgent {
    fn gather_candidates(&mut self) -> Result<Vec<IceCandidate>, ConnectionError> {
        let local_candidates = self.local_candidates.clone();
        let state = self.state.clone();
        let peer_connection = self.peer_connection.clone();

        // Use block_in_place for better async context handling
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                *state.lock().await = IceState::Checking;

                let signaling_state = peer_connection.signaling_state();
                debug!("Current signaling state: {:?}", signaling_state);

                // If we haven't set local description yet, create offer/answer
                if signaling_state == RTCSignalingState::Stable {
                    // We're the offerer, create an offer
                    debug!("Creating offer to trigger ICE gathering");
                    let offer = peer_connection
                        .create_offer(None)
                        .await
                        .map_err(|e| ConnectionError::IceError(format!("Failed to create offer: {}", e)))?;

                    peer_connection
                        .set_local_description(offer)
                        .await
                        .map_err(|e| {
                            ConnectionError::IceError(format!("Failed to set local description: {}", e))
                        })?;
                } else if signaling_state == RTCSignalingState::HaveRemoteOffer {
                    // We're the answerer, create an answer
                    debug!("Creating answer to trigger ICE gathering");
                    let answer = peer_connection
                        .create_answer(None)
                        .await
                        .map_err(|e| ConnectionError::IceError(format!("Failed to create answer: {}", e)))?;

                    peer_connection
                        .set_local_description(answer)
                        .await
                        .map_err(|e| {
                            ConnectionError::IceError(format!("Failed to set local description: {}", e))
                        })?;
                }

                debug!("Local description set, waiting for candidates");

                // Wait for gathering to complete
                let timeout = tokio::time::Duration::from_secs(10);
                let start = tokio::time::Instant::now();

                loop {
                    let gathering_state = peer_connection.ice_gathering_state();
                    debug!("ICE gathering state: {:?}", gathering_state);

                    if gathering_state == RTCIceGatheringState::Complete {
                        debug!("ICE gathering complete");
                        break;
                    }

                    if start.elapsed() > timeout {
                        warn!("ICE gathering timeout");
                        return Err(ConnectionError::Timeout);
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }

                // Return gathered candidates
                let candidates = local_candidates.lock().await.clone();
                info!("Gathered {} candidates", candidates.len());
                Ok(candidates)
            })
        })
    }

    fn set_remote_candidates(
        &mut self,
        candidates: Vec<IceCandidate>,
    ) -> Result<(), ConnectionError> {
        let peer_connection = self.peer_connection.clone();
        let candidates_count = candidates.len();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                for candidate in &candidates {
                    let webrtc_candidate = convert_to_webrtc_candidate(candidate);

                    peer_connection
                        .add_ice_candidate(webrtc_candidate)
                        .await
                        .map_err(|e| {
                            ConnectionError::IceError(format!("Failed to add ICE candidate: {}", e))
                        })?;
                }

                debug!("Added {} remote candidates", candidates_count);
                Ok(())
            })
        })
    }

    fn create_offer(&self) -> Result<SdpOffer, ConnectionError> {
        let peer_connection = self.peer_connection.clone();
        let local_candidates = self.local_candidates.clone();
        let session_id = self.session_id.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create offer
                let offer = peer_connection
                    .create_offer(None)
                    .await
                    .map_err(|e| ConnectionError::IceError(format!("Failed to create offer: {}", e)))?;

                // Set local description
                peer_connection
                    .set_local_description(offer.clone())
                    .await
                    .map_err(|e| {
                        ConnectionError::IceError(format!("Failed to set local description: {}", e))
                    })?;

                // Wait for gathering
                let timeout = tokio::time::Duration::from_secs(10);
                let start = tokio::time::Instant::now();

                loop {
                    let gathering_state = peer_connection.ice_gathering_state();

                    if gathering_state == RTCIceGatheringState::Complete {
                        debug!("ICE gathering complete");
                        break;
                    }

                    if start.elapsed() > timeout {
                        warn!("ICE gathering timeout");
                        return Err(ConnectionError::Timeout);
                    }

                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }

                let candidates = local_candidates.lock().await.clone();

                // Extract real ICE credentials and fingerprint from the SDP
                let (ufrag, pwd) = extract_ice_credentials(&offer.sdp)?;
                let fingerprint = extract_fingerprint(&offer.sdp)?;

                Ok(SdpOffer {
                    session_id,
                    ufrag,
                    pwd,
                    fingerprint,
                    candidates,
                })
            })
        })
    }

    fn handle_answer(&mut self, answer: SdpAnswer) -> Result<(), ConnectionError> {
        let peer_connection = self.peer_connection.clone();
        let candidates_count = answer.candidates.len();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Convert SdpAnswer to RTCSessionDescription
                // Note: In a real implementation, you'd need to properly format the SDP
                let sdp = format_sdp_answer(&answer);

                let remote_desc = RTCSessionDescription::answer(sdp)
                    .map_err(|e| ConnectionError::IceError(format!("Failed to create answer: {}", e)))?;

                peer_connection
                    .set_remote_description(remote_desc)
                    .await
                    .map_err(|e| {
                        ConnectionError::IceError(format!("Failed to set remote description: {}", e))
                    })?;

                // Add remote candidates
                for candidate in &answer.candidates {
                    let webrtc_candidate = convert_to_webrtc_candidate(candidate);

                    peer_connection
                        .add_ice_candidate(webrtc_candidate)
                        .await
                        .map_err(|e| {
                            ConnectionError::IceError(format!("Failed to add ICE candidate: {}", e))
                        })?;
                }

                debug!("Processed answer with {} candidates", candidates_count);
                Ok(())
            })
        })
    }

    fn connection_state(&self) -> IceState {
        let state = self.ice_connection_state.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                *state.lock().await
            })
        })
    }
}

/// Extract ICE credentials from SDP string.
fn extract_ice_credentials(sdp: &str) -> Result<(String, String), ConnectionError> {
    let mut ufrag = None;
    let mut pwd = None;

    for line in sdp.lines() {
        if line.starts_with("a=ice-ufrag:") {
            ufrag = Some(line.trim_start_matches("a=ice-ufrag:").trim().to_string());
        } else if line.starts_with("a=ice-pwd:") {
            pwd = Some(line.trim_start_matches("a=ice-pwd:").trim().to_string());
        }

        if ufrag.is_some() && pwd.is_some() {
            break;
        }
    }

    match (ufrag, pwd) {
        (Some(u), Some(p)) => Ok((u, p)),
        _ => Err(ConnectionError::IceError("Failed to extract ICE credentials from SDP".to_string())),
    }
}

/// Extract DTLS fingerprint from SDP string.
fn extract_fingerprint(sdp: &str) -> Result<String, ConnectionError> {
    for line in sdp.lines() {
        if line.starts_with("a=fingerprint:") {
            // Format: a=fingerprint:sha-256 XX:XX:XX:...
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                return Ok(parts[1].trim().to_string());
            }
        }
    }
    Err(ConnectionError::IceError("Failed to extract fingerprint from SDP".to_string()))
}

/// Convert webrtc-rs ICE candidate to our IceCandidate type.
fn convert_webrtc_candidate(candidate: &RTCIceCandidate) -> IceCandidate {
    let candidate_type = match candidate.typ {
        RTCIceCandidateType::Host => CandidateType::Host,
        RTCIceCandidateType::Srflx => CandidateType::Srflx,
        RTCIceCandidateType::Prflx => CandidateType::Prflx,
        RTCIceCandidateType::Relay => CandidateType::Relay,
        _ => CandidateType::Host,
    };

    // Parse address
    let addr = format!("{}:{}", candidate.address, candidate.port)
        .parse()
        .unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap());

    IceCandidate {
        foundation: candidate.foundation.clone(),
        component: candidate.component as u32,
        protocol: candidate.protocol.to_string(),
        priority: candidate.priority as u64,
        addr,
        candidate_type,
    }
}

/// Convert our IceCandidate to webrtc-rs format.
fn convert_to_webrtc_candidate(candidate: &IceCandidate) -> RTCIceCandidateInit {
    let typ = match candidate.candidate_type {
        CandidateType::Host => "host",
        CandidateType::Srflx => "srflx",
        CandidateType::Prflx => "prflx",
        CandidateType::Relay => "relay",
    };

    // Format candidate string according to ICE spec
    let candidate_str = format!(
        "candidate:{} {} {} {} {} {} typ {}",
        candidate.foundation,
        candidate.component,
        candidate.protocol,
        candidate.priority,
        candidate.addr.ip(),
        candidate.addr.port(),
        typ
    );

    RTCIceCandidateInit {
        candidate: candidate_str,
        ..Default::default()
    }
}

/// Format SDP offer from our SdpOffer structure.
fn format_sdp_offer(offer: &SdpOffer) -> String {
    let mut sdp = String::new();

    // Generate numeric session ID from hash of the UUID
    let session_id_numeric = offer.session_id.bytes().map(|b| b as u64).sum::<u64>();

    sdp.push_str("v=0\r\n");
    sdp.push_str(&format!("o=- {} 2 IN IP4 0.0.0.0\r\n", session_id_numeric));
    sdp.push_str("s=-\r\n");
    sdp.push_str("t=0 0\r\n");
    sdp.push_str(&format!("a=ice-ufrag:{}\r\n", offer.ufrag));
    sdp.push_str(&format!("a=ice-pwd:{}\r\n", offer.pwd));
    sdp.push_str(&format!("a=fingerprint:sha-256 {}\r\n", offer.fingerprint));

    // Add media line (data channel)
    sdp.push_str("m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n");
    sdp.push_str("c=IN IP4 0.0.0.0\r\n");
    sdp.push_str("a=setup:actpass\r\n");
    sdp.push_str("a=mid:0\r\n");
    sdp.push_str("a=sctp-port:5000\r\n");

    // Add candidates
    for candidate in &offer.candidates {
        let typ = match candidate.candidate_type {
            CandidateType::Host => "host",
            CandidateType::Srflx => "srflx",
            CandidateType::Prflx => "prflx",
            CandidateType::Relay => "relay",
        };

        sdp.push_str(&format!(
            "a=candidate:{} {} {} {} {} {} typ {}\r\n",
            candidate.foundation,
            candidate.component,
            candidate.protocol.to_uppercase(),
            candidate.priority,
            candidate.addr.ip(),
            candidate.addr.port(),
            typ
        ));
    }

    sdp
}

/// Format SDP answer from our SdpAnswer structure.
///
/// This is a simplified version. In production, use a proper SDP library.
fn format_sdp_answer(answer: &SdpAnswer) -> String {
    let mut sdp = String::new();

    // Generate numeric session ID from hash of the UUID
    let session_id_numeric = answer.session_id.bytes().map(|b| b as u64).sum::<u64>();

    sdp.push_str("v=0\r\n");
    sdp.push_str(&format!("o=- {} 2 IN IP4 0.0.0.0\r\n", session_id_numeric));
    sdp.push_str("s=-\r\n");
    sdp.push_str("t=0 0\r\n");
    sdp.push_str(&format!("a=ice-ufrag:{}\r\n", answer.ufrag));
    sdp.push_str(&format!("a=ice-pwd:{}\r\n", answer.pwd));
    sdp.push_str(&format!("a=fingerprint:sha-256 {}\r\n", answer.fingerprint));

    // Add media line (data channel)
    sdp.push_str("m=application 9 UDP/DTLS/SCTP webrtc-datachannel\r\n");
    sdp.push_str("c=IN IP4 0.0.0.0\r\n");
    sdp.push_str("a=setup:actpass\r\n");
    sdp.push_str("a=mid:0\r\n");
    sdp.push_str("a=sctp-port:5000\r\n");

    // Add candidates
    for candidate in &answer.candidates {
        let typ = match candidate.candidate_type {
            CandidateType::Host => "host",
            CandidateType::Srflx => "srflx",
            CandidateType::Prflx => "prflx",
            CandidateType::Relay => "relay",
        };

        sdp.push_str(&format!(
            "a=candidate:{} {} {} {} {} {} typ {}\r\n",
            candidate.foundation,
            candidate.component,
            candidate.protocol.to_uppercase(),
            candidate.priority,
            candidate.addr.ip(),
            candidate.addr.port(),
            typ
        ));
    }

    sdp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_create_real_ice_agent() {
        let ice_servers = vec![
            "stun:stun.l.google.com:19302".to_string(),
        ];

        let agent = RealIceAgent::new(ice_servers).await;
        assert!(agent.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore] // Requires network access to STUN server
    async fn test_gather_candidates() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init()
            .ok();

        let ice_servers = vec![
            "stun:stun.l.google.com:19302".to_string(),
        ];

        let mut agent = RealIceAgent::new(ice_servers).await.unwrap();

        // Gather candidates (this will actually contact STUN server)
        let candidates = agent.gather_candidates();

        // Should have at least host candidates
        match &candidates {
            Ok(cands) => {
                println!("Gathered {} candidates", cands.len());
                for c in cands {
                    println!("  {:?}: {}", c.candidate_type, c.addr);
                }
            }
            Err(e) => {
                println!("Failed to gather candidates: {}", e);
            }
        }

        assert!(candidates.is_ok());
        let candidates = candidates.unwrap();
        assert!(!candidates.is_empty());
    }
}
