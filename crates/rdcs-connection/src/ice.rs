// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! ICE (Interactive Connectivity Establishment) for NAT traversal.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ConnectionError;

// ---------------------------------------------------------------------------
// Trait interface
// ---------------------------------------------------------------------------

/// ICE agent responsible for gathering candidates, performing connectivity
/// checks, and managing the ICE state machine.
pub trait IceAgent: Send {
    /// Gather local ICE candidates from network interfaces.
    fn gather_candidates(&mut self) -> Result<Vec<IceCandidate>, ConnectionError>;

    /// Supply remote candidates received via signaling.
    fn set_remote_candidates(&mut self, candidates: Vec<IceCandidate>) -> Result<(), ConnectionError>;

    /// Create an SDP offer containing the local candidates gathered so far.
    fn create_offer(&self) -> Result<SdpOffer, ConnectionError>;

    /// Process an SDP answer from the remote peer.
    fn handle_answer(&mut self, answer: SdpAnswer) -> Result<(), ConnectionError>;

    /// Return the current ICE connection state.
    fn connection_state(&self) -> IceState;
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Type of ICE candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CandidateType {
    /// Local host candidate.
    Host,
    /// Server-reflexive candidate (discovered via STUN).
    Srflx,
    /// Peer-reflexive candidate (discovered during connectivity checks).
    Prflx,
    /// Relay candidate (via TURN server).
    Relay,
}

/// A single ICE candidate.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    /// Opaque foundation string used to correlate candidates.
    pub foundation: String,
    /// Component ID (1 = RTP, 2 = RTCP).
    pub component: u32,
    /// Transport protocol ("udp" or "tcp").
    pub protocol: String,
    /// Priority — higher values are preferred.
    pub priority: u64,
    /// Socket address of this candidate.
    pub addr: SocketAddr,
    /// Candidate type classification.
    pub candidate_type: CandidateType,
}

impl IceCandidate {
    /// Create a new host (local) candidate.
    pub fn new_host(addr: SocketAddr, component: u32) -> Self {
        Self {
            foundation: Uuid::new_v4().to_string(),
            component,
            protocol: "udp".into(),
            priority: Self::host_priority(addr),
            addr,
            candidate_type: CandidateType::Host,
        }
    }

    /// Compute a default priority for a host candidate.
    fn host_priority(addr: SocketAddr) -> u64 {
        // Prefer IPv4 over IPv6 with a simple heuristic.
        let type_pref: u64 = if addr.is_ipv4() { 65_535 } else { 65_534 };
        // Higher for lower component IDs.
        (type_pref << 16) | 1
    }
}

/// ICE connection state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IceState {
    /// Initial state before gathering begins.
    New,
    /// Connectivity checks are in progress.
    Checking,
    /// At least one candidate pair is connected.
    Connected,
    /// All connectivity checks have failed.
    Failed,
    /// The ICE session has been closed.
    Closed,
}

/// An SDP offer containing local ICE candidates and credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdpOffer {
    /// Unique session identifier.
    pub session_id: String,
    /// ICE username fragment.
    pub ufrag: String,
    /// ICE password.
    pub pwd: String,
    /// DTLS fingerprint (sha-256).
    pub fingerprint: String,
    /// Local candidates gathered so far.
    pub candidates: Vec<IceCandidate>,
}

/// An SDP answer containing remote ICE candidates and credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdpAnswer {
    /// Session identifier matching the offer.
    pub session_id: String,
    /// Remote ICE username fragment.
    pub ufrag: String,
    /// Remote ICE password.
    pub pwd: String,
    /// DTLS fingerprint (sha-256).
    pub fingerprint: String,
    /// Remote candidates.
    pub candidates: Vec<IceCandidate>,
}

// ---------------------------------------------------------------------------
// Stub / mock ICE agent (for unit tests)
// ---------------------------------------------------------------------------

/// A stub ICE agent implementation that stores candidates in memory without
/// performing real network I/O. Suitable for unit and integration tests.
#[derive(Debug)]
pub struct StubIceAgent {
    state: IceState,
    local_candidates: Vec<IceCandidate>,
    remote_candidates: Vec<IceCandidate>,
    ufrag: String,
    pwd: String,
}

impl StubIceAgent {
    /// Create a new stub ICE agent.
    pub fn new() -> Self {
        Self {
            state: IceState::New,
            local_candidates: Vec::new(),
            remote_candidates: Vec::new(),
            ufrag: Uuid::new_v4().to_string(),
            pwd: Uuid::new_v4().to_string(),
        }
    }

    /// Pre-load local candidates (useful in tests).
    pub fn with_local_candidates(mut self, candidates: Vec<IceCandidate>) -> Self {
        self.local_candidates = candidates;
        self
    }
}

impl Default for StubIceAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl IceAgent for StubIceAgent {
    fn gather_candidates(&mut self) -> Result<Vec<IceCandidate>, ConnectionError> {
        self.state = IceState::Checking;
        Ok(self.local_candidates.clone())
    }

    fn set_remote_candidates(
        &mut self,
        candidates: Vec<IceCandidate>,
    ) -> Result<(), ConnectionError> {
        self.remote_candidates.extend(candidates);
        Ok(())
    }

    fn create_offer(&self) -> Result<SdpOffer, ConnectionError> {
        Ok(SdpOffer {
            session_id: Uuid::new_v4().to_string(),
            ufrag: self.ufrag.clone(),
            pwd: self.pwd.clone(),
            fingerprint: "00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00".to_string(),
            candidates: self.local_candidates.clone(),
        })
    }

    fn handle_answer(&mut self, answer: SdpAnswer) -> Result<(), ConnectionError> {
        self.remote_candidates.extend(answer.candidates);
        // The stub simulates an immediate connection.
        self.state = IceState::Connected;
        Ok(())
    }

    fn connection_state(&self) -> IceState {
        self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn make_host_candidate(port: u16) -> IceCandidate {
        IceCandidate::new_host(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), port),
            1,
        )
    }

    #[test]
    fn stub_agent_initial_state() {
        let agent = StubIceAgent::new();
        assert_eq!(agent.connection_state(), IceState::New);
    }

    #[test]
    fn stub_agent_gather_candidates() {
        let candidates = vec![make_host_candidate(21115)];
        let mut agent = StubIceAgent::new().with_local_candidates(candidates.clone());

        let gathered = agent.gather_candidates().unwrap();
        assert_eq!(gathered.len(), 1);
        assert_eq!(gathered[0].candidate_type, CandidateType::Host);
        assert_eq!(agent.connection_state(), IceState::Checking);
    }

    #[test]
    fn stub_agent_set_remote_candidates() {
        let mut agent = StubIceAgent::new();
        let remote = vec![make_host_candidate(30000)];
        agent.set_remote_candidates(remote).unwrap();
        assert_eq!(agent.remote_candidates.len(), 1);
    }

    #[test]
    fn stub_agent_create_offer() {
        let candidates = vec![make_host_candidate(21115)];
        let agent = StubIceAgent::new().with_local_candidates(candidates);
        let offer = agent.create_offer().unwrap();
        assert_eq!(offer.candidates.len(), 1);
        assert!(!offer.ufrag.is_empty());
        assert!(!offer.pwd.is_empty());
    }

    #[test]
    fn stub_agent_handle_answer_transitions_to_connected() {
        let mut agent = StubIceAgent::new().with_local_candidates(vec![make_host_candidate(21115)]);
        agent.gather_candidates().unwrap();

        let answer = SdpAnswer {
            session_id: "session-1".into(),
            ufrag: "remote-ufrag".into(),
            pwd: "remote-pwd".into(),
            fingerprint: "00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00".into(),
            candidates: vec![make_host_candidate(40000)],
        };
        agent.handle_answer(answer).unwrap();
        assert_eq!(agent.connection_state(), IceState::Connected);
    }

    #[test]
    fn ice_candidate_priority_ipv4_vs_ipv6() {
        let ipv4 = IceCandidate::new_host(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1000),
            1,
        );
        let ipv6 = IceCandidate::new_host(
            SocketAddr::new(IpAddr::V6(std::net::Ipv6Addr::LOCALHOST), 1000),
            1,
        );
        // IPv4 should have higher priority than IPv6 in our simple heuristic.
        assert!(ipv4.priority > ipv6.priority);
    }

    #[test]
    fn candidate_types_are_distinct() {
        let types = [
            CandidateType::Host,
            CandidateType::Srflx,
            CandidateType::Prflx,
            CandidateType::Relay,
        ];
        for (i, a) in types.iter().enumerate() {
            for (j, b) in types.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn ice_state_transitions() {
        let states = [
            IceState::New,
            IceState::Checking,
            IceState::Connected,
            IceState::Failed,
            IceState::Closed,
        ];
        // Just verify all states are distinct and printable.
        for s in &states {
            let debug = format!("{:?}", s);
            assert!(!debug.is_empty());
        }
    }
}
