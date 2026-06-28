// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! NAT traversal testing and optimization framework.
//!
//! This module provides comprehensive NAT traversal testing capabilities:
//! - Different NAT types simulation (Full Cone, Restricted Cone, Port Restricted, Symmetric)
//! - STUN/TURN server integration testing
//! - ICE candidate gathering and prioritization
//! - P2P connection success rate measurement
//! - Relay fallback verification
//!
//! ## NAT Types
//! - **Full Cone NAT**: Most permissive, easiest to traverse
//! - **Restricted Cone NAT**: Requires prior outbound packet
//! - **Port Restricted Cone NAT**: Requires matching source port
//! - **Symmetric NAT**: Most restrictive, typically requires relay
//!
//! ## Test Strategy
//! 1. Test all NAT type combinations (4x4 = 16 scenarios)
//! 2. Measure success rate for each combination
//! 3. Verify relay fallback when P2P fails
//! 4. Optimize ICE candidate gathering and prioritization

pub mod nat_detector;

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// NAT types according to RFC 3489 and RFC 5389
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NatType {
    /// Open Internet (no NAT)
    None,
    /// Full Cone NAT - Once an internal address is mapped to an external address,
    /// any external host can send packets to the internal host
    FullCone,
    /// Restricted Cone NAT - External host must have received a packet first
    RestrictedCone,
    /// Port Restricted Cone NAT - External host must match both IP and port
    PortRestrictedCone,
    /// Symmetric NAT - Different external port for each destination
    Symmetric,
}

impl NatType {
    /// Returns the difficulty level of traversal (0 = easiest, 4 = hardest)
    pub fn traversal_difficulty(&self) -> u8 {
        match self {
            NatType::None => 0,
            NatType::FullCone => 1,
            NatType::RestrictedCone => 2,
            NatType::PortRestrictedCone => 3,
            NatType::Symmetric => 4,
        }
    }

    /// Returns whether direct P2P is likely possible
    pub fn can_direct_p2p(&self, remote: &NatType) -> bool {
        match (self, remote) {
            (NatType::None, _) | (_, NatType::None) => true,
            (NatType::FullCone, _) | (_, NatType::FullCone) => true,
            (NatType::RestrictedCone, NatType::RestrictedCone) => true,
            (NatType::RestrictedCone, NatType::PortRestrictedCone) => true,
            (NatType::PortRestrictedCone, NatType::RestrictedCone) => true,
            (NatType::Symmetric, _) | (_, NatType::Symmetric) => false,
            _ => false,
        }
    }
}

/// ICE candidate types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CandidateType {
    /// Host candidate (local network interface)
    Host,
    /// Server reflexive candidate (via STUN)
    ServerReflexive,
    /// Relay candidate (via TURN)
    Relay,
}

/// ICE candidate with priority
#[derive(Debug, Clone)]
pub struct IceCandidate {
    pub candidate_type: CandidateType,
    pub address: SocketAddr,
    pub priority: u32,
    pub foundation: String,
}

impl IceCandidate {
    pub fn new(candidate_type: CandidateType, address: SocketAddr) -> Self {
        let priority = Self::calculate_priority(candidate_type);
        Self {
            candidate_type,
            address,
            priority,
            foundation: format!("{:?}_{}", candidate_type, address),
        }
    }

    fn calculate_priority(candidate_type: CandidateType) -> u32 {
        // RFC 5245 priority calculation (simplified)
        match candidate_type {
            CandidateType::Host => 126,
            CandidateType::ServerReflexive => 100,
            CandidateType::Relay => 0,
        }
    }
}

/// NAT traversal test result
#[derive(Debug, Clone)]
pub struct TraversalResult {
    pub local_nat: NatType,
    pub remote_nat: NatType,
    pub success: bool,
    pub connection_method: ConnectionMethod,
    pub time_to_connect_ms: u64,
    pub ice_candidates_gathered: usize,
    pub ice_checks_performed: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionMethod {
    DirectP2P,
    StunAssistedP2P,
    RelayFallback,
    Failed,
}

/// NAT traversal simulator
pub struct NatTraversalSimulator {
    stun_server: SocketAddr,
    turn_server: SocketAddr,
    results: Arc<RwLock<Vec<TraversalResult>>>,
}

impl NatTraversalSimulator {
    pub fn new() -> Self {
        Self {
            stun_server: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 3478),
            turn_server: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4)), 3478),
            results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Test NAT traversal between two peers
    pub async fn test_traversal(
        &self,
        local_nat: NatType,
        remote_nat: NatType,
    ) -> TraversalResult {
        info!("Testing NAT traversal: {:?} <-> {:?}", local_nat, remote_nat);

        let start = Instant::now();

        // Phase 1: Gather ICE candidates
        let local_candidates = self.gather_ice_candidates(local_nat).await;
        let remote_candidates = self.gather_ice_candidates(remote_nat).await;

        debug!(
            "Gathered {} local and {} remote candidates",
            local_candidates.len(),
            remote_candidates.len()
        );

        // Phase 2: Perform connectivity checks
        let (success, method, checks) = self
            .perform_connectivity_checks(&local_candidates, &remote_candidates, local_nat, remote_nat)
            .await;

        let elapsed = start.elapsed();

        let result = TraversalResult {
            local_nat,
            remote_nat,
            success,
            connection_method: method,
            time_to_connect_ms: elapsed.as_millis() as u64,
            ice_candidates_gathered: local_candidates.len() + remote_candidates.len(),
            ice_checks_performed: checks,
        };

        info!(
            "Traversal result: {:?}, method: {:?}, time: {}ms",
            if success { "SUCCESS" } else { "FAILED" },
            method,
            result.time_to_connect_ms
        );

        // Store result for analysis
        self.results.write().await.push(result.clone());

        result
    }

    /// Gather ICE candidates for a peer behind given NAT type
    async fn gather_ice_candidates(&self, nat_type: NatType) -> Vec<IceCandidate> {
        let mut candidates = Vec::new();

        // Simulate gathering time
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Always gather host candidates
        candidates.push(IceCandidate::new(
            CandidateType::Host,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100)), 54321),
        ));

        // Gather server reflexive candidates via STUN (except for Symmetric NAT)
        if nat_type != NatType::Symmetric {
            tokio::time::sleep(Duration::from_millis(100)).await;
            candidates.push(IceCandidate::new(
                CandidateType::ServerReflexive,
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)), 54321),
            ));
        }

        // Always gather relay candidates via TURN
        tokio::time::sleep(Duration::from_millis(150)).await;
        candidates.push(IceCandidate::new(
            CandidateType::Relay,
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 1)), 54321),
        ));

        // Sort by priority (highest first)
        candidates.sort_by(|a, b| b.priority.cmp(&a.priority));

        candidates
    }

    /// Perform connectivity checks between candidate pairs
    async fn perform_connectivity_checks(
        &self,
        local_candidates: &[IceCandidate],
        remote_candidates: &[IceCandidate],
        local_nat: NatType,
        remote_nat: NatType,
    ) -> (bool, ConnectionMethod, usize) {
        let mut checks_performed = 0;

        // Try host candidates first (direct P2P)
        for local in local_candidates.iter() {
            if local.candidate_type != CandidateType::Host {
                continue;
            }
            for remote in remote_candidates.iter() {
                if remote.candidate_type != CandidateType::Host {
                    continue;
                }

                checks_performed += 1;
                tokio::time::sleep(Duration::from_millis(20)).await;

                // Direct P2P only works in some NAT combinations
                if local_nat == NatType::None && remote_nat == NatType::None {
                    debug!("Direct P2P successful (no NAT)");
                    return (true, ConnectionMethod::DirectP2P, checks_performed);
                }
            }
        }

        // Try server reflexive candidates (STUN-assisted P2P)
        for local in local_candidates.iter() {
            if local.candidate_type != CandidateType::ServerReflexive {
                continue;
            }
            for remote in remote_candidates.iter() {
                if remote.candidate_type != CandidateType::ServerReflexive {
                    continue;
                }

                checks_performed += 1;
                tokio::time::sleep(Duration::from_millis(30)).await;

                // Check if P2P is possible based on NAT types
                if local_nat.can_direct_p2p(&remote_nat) {
                    debug!("STUN-assisted P2P successful");
                    return (true, ConnectionMethod::StunAssistedP2P, checks_performed);
                }
            }
        }

        // Fall back to relay candidates
        for local in local_candidates.iter() {
            if local.candidate_type != CandidateType::Relay {
                continue;
            }
            for remote in remote_candidates.iter() {
                if remote.candidate_type != CandidateType::Relay {
                    continue;
                }

                checks_performed += 1;
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Relay always works (unless server is down)
                debug!("Relay connection successful");
                return (true, ConnectionMethod::RelayFallback, checks_performed);
            }
        }

        warn!("All connectivity checks failed");
        (false, ConnectionMethod::Failed, checks_performed)
    }

    /// Run comprehensive NAT traversal test matrix
    pub async fn run_test_matrix(&self) -> TestMatrixResult {
        info!("Running comprehensive NAT traversal test matrix...");

        let nat_types = vec![
            NatType::None,
            NatType::FullCone,
            NatType::RestrictedCone,
            NatType::PortRestrictedCone,
            NatType::Symmetric,
        ];

        let mut matrix = TestMatrixResult::new();
        let total_tests = nat_types.len() * nat_types.len();
        let mut completed = 0;

        for local_nat in &nat_types {
            for remote_nat in &nat_types {
                let result = self.test_traversal(*local_nat, *remote_nat).await;
                matrix.add_result(result);

                completed += 1;
                if completed % 5 == 0 {
                    info!("Progress: {}/{} tests completed", completed, total_tests);
                }
            }
        }

        matrix
    }

    /// Get all test results
    pub async fn get_results(&self) -> Vec<TraversalResult> {
        self.results.read().await.clone()
    }
}

impl Default for NatTraversalSimulator {
    fn default() -> Self {
        Self::new()
    }
}

/// Test matrix result with statistics
pub struct TestMatrixResult {
    results: Vec<TraversalResult>,
}

impl TestMatrixResult {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: TraversalResult) {
        self.results.push(result);
    }

    /// Calculate overall success rate
    pub fn success_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let successful = self.results.iter().filter(|r| r.success).count();
        (successful as f64 / self.results.len() as f64) * 100.0
    }

    /// Calculate P2P success rate (excluding relay)
    pub fn p2p_success_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let p2p_successful = self
            .results
            .iter()
            .filter(|r| {
                r.success
                    && (r.connection_method == ConnectionMethod::DirectP2P
                        || r.connection_method == ConnectionMethod::StunAssistedP2P)
            })
            .count();
        (p2p_successful as f64 / self.results.len() as f64) * 100.0
    }

    /// Calculate relay usage rate
    pub fn relay_usage_rate(&self) -> f64 {
        if self.results.is_empty() {
            return 0.0;
        }
        let relay_used = self
            .results
            .iter()
            .filter(|r| r.connection_method == ConnectionMethod::RelayFallback)
            .count();
        (relay_used as f64 / self.results.len() as f64) * 100.0
    }

    /// Get average connection time
    pub fn average_connection_time_ms(&self) -> u64 {
        if self.results.is_empty() {
            return 0;
        }
        let total: u64 = self.results.iter().map(|r| r.time_to_connect_ms).sum();
        total / self.results.len() as u64
    }

    /// Get success rate by NAT type combination
    pub fn success_rate_by_nat_combination(&self) -> HashMap<(NatType, NatType), f64> {
        let mut stats: HashMap<(NatType, NatType), (usize, usize)> = HashMap::new();

        for result in &self.results {
            let key = (result.local_nat, result.remote_nat);
            let entry = stats.entry(key).or_insert((0, 0));
            entry.0 += 1; // total
            if result.success {
                entry.1 += 1; // successful
            }
        }

        stats
            .into_iter()
            .map(|(key, (total, successful))| {
                let rate = (successful as f64 / total as f64) * 100.0;
                (key, rate)
            })
            .collect()
    }

    /// Generate detailed report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("========== NAT Traversal Test Matrix Report ==========\n\n");

        report.push_str(&format!("Total tests: {}\n", self.results.len()));
        report.push_str(&format!(
            "Overall success rate: {:.1}%\n",
            self.success_rate()
        ));
        report.push_str(&format!(
            "P2P success rate: {:.1}%\n",
            self.p2p_success_rate()
        ));
        report.push_str(&format!(
            "Relay usage rate: {:.1}%\n",
            self.relay_usage_rate()
        ));
        report.push_str(&format!(
            "Average connection time: {}ms\n\n",
            self.average_connection_time_ms()
        ));

        report.push_str("Success rate by NAT combination:\n");
        let mut combinations: Vec<_> = self.success_rate_by_nat_combination().into_iter().collect();
        combinations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        for ((local, remote), rate) in combinations {
            report.push_str(&format!(
                "  {:?} <-> {:?}: {:.1}%\n",
                local, remote, rate
            ));
        }

        report.push_str("\n======================================================\n");
        report
    }

    /// Check if PRD requirement (>60% success) is met
    pub fn meets_prd_requirement(&self) -> bool {
        self.success_rate() > 60.0
    }
}

impl Default for TestMatrixResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nat_type_difficulty() {
        assert_eq!(NatType::None.traversal_difficulty(), 0);
        assert_eq!(NatType::FullCone.traversal_difficulty(), 1);
        assert_eq!(NatType::Symmetric.traversal_difficulty(), 4);
    }

    #[test]
    fn test_can_direct_p2p() {
        assert!(NatType::None.can_direct_p2p(&NatType::Symmetric));
        assert!(NatType::FullCone.can_direct_p2p(&NatType::Symmetric));
        assert!(!NatType::Symmetric.can_direct_p2p(&NatType::Symmetric));
    }

    #[test]
    fn test_ice_candidate_priority() {
        let host = IceCandidate::new(
            CandidateType::Host,
            "192.168.1.1:5000".parse().unwrap(),
        );
        let srflx = IceCandidate::new(
            CandidateType::ServerReflexive,
            "203.0.113.1:5000".parse().unwrap(),
        );
        let relay = IceCandidate::new(
            CandidateType::Relay,
            "198.51.100.1:5000".parse().unwrap(),
        );

        assert!(host.priority > srflx.priority);
        assert!(srflx.priority > relay.priority);
    }

    #[tokio::test]
    async fn test_nat_traversal_no_nat() {
        let simulator = NatTraversalSimulator::new();
        let result = simulator.test_traversal(NatType::None, NatType::None).await;

        assert!(result.success);
        assert_eq!(result.connection_method, ConnectionMethod::DirectP2P);
    }

    #[tokio::test]
    async fn test_nat_traversal_full_cone() {
        let simulator = NatTraversalSimulator::new();
        let result = simulator
            .test_traversal(NatType::FullCone, NatType::FullCone)
            .await;

        assert!(result.success);
        assert_eq!(result.connection_method, ConnectionMethod::StunAssistedP2P);
    }

    #[tokio::test]
    async fn test_nat_traversal_symmetric_requires_relay() {
        let simulator = NatTraversalSimulator::new();
        let result = simulator
            .test_traversal(NatType::Symmetric, NatType::Symmetric)
            .await;

        assert!(result.success);
        assert_eq!(result.connection_method, ConnectionMethod::RelayFallback);
    }

    #[tokio::test]
    async fn test_matrix_result_calculations() {
        let mut matrix = TestMatrixResult::new();

        // Add 10 results: 8 successful, 2 failed
        for i in 0..10 {
            matrix.add_result(TraversalResult {
                local_nat: NatType::None,
                remote_nat: NatType::None,
                success: i < 8,
                connection_method: if i < 8 {
                    ConnectionMethod::DirectP2P
                } else {
                    ConnectionMethod::Failed
                },
                time_to_connect_ms: 100,
                ice_candidates_gathered: 6,
                ice_checks_performed: 3,
            });
        }

        assert_eq!(matrix.success_rate(), 80.0);
        assert!(matrix.meets_prd_requirement());
    }
}
