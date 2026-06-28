// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Network path selection with L1 → L2 → L3 priority and fallback.

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::ConnectionError;

// ---------------------------------------------------------------------------
// Trait interface
// ---------------------------------------------------------------------------

/// Selects and falls back between network paths with L1 > L2 > L3 priority.
pub trait PathSelector: Send {
    /// Select the highest-priority available path from the candidate set.
    fn select_path(
        &mut self,
        candidates: &[PathCandidate],
    ) -> Result<ConnectionPath, ConnectionError>;

    /// Report that a path has failed. Returns the next fallback path,
    /// or `None` if no further fallback is available.
    fn on_path_failed(&mut self, path: ConnectionPath) -> Option<ConnectionPath>;
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Classification of a network path between two peers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PathType {
    /// L1 — direct connection on the same LAN.
    L1Direct,
    /// L2 — UDP hole-punch over the internet.
    L2Punch,
    /// L3 — relayed through a TURN / relay server.
    L3Relay,
}

/// A candidate path with its type and target address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathCandidate {
    /// Path classification (L1, L2, or L3).
    pub path_type: PathType,
    /// Remote address for this path.
    pub addr: SocketAddr,
    /// Optional measured RTT in milliseconds (lower = better within same tier).
    pub rtt_ms: Option<f64>,
}

/// The selected connection path, wrapping the remote address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionPath {
    /// L1 — direct LAN connection.
    L1Direct(SocketAddr),
    /// L2 — UDP hole-punched internet connection.
    L2Punch(SocketAddr),
    /// L3 — relayed connection.
    L3Relay(SocketAddr),
}

impl ConnectionPath {
    /// Return the path type.
    pub fn path_type(&self) -> PathType {
        match self {
            ConnectionPath::L1Direct(_) => PathType::L1Direct,
            ConnectionPath::L2Punch(_) => PathType::L2Punch,
            ConnectionPath::L3Relay(_) => PathType::L3Relay,
        }
    }

    /// Return the socket address.
    pub fn addr(&self) -> SocketAddr {
        match self {
            ConnectionPath::L1Direct(a)
            | ConnectionPath::L2Punch(a)
            | ConnectionPath::L3Relay(a) => *a,
        }
    }
}

// ---------------------------------------------------------------------------
// Priority-based path selector
// ---------------------------------------------------------------------------

/// Concrete [`PathSelector`] that implements the L1 → L2 → L3 priority chain.
///
/// When multiple candidates of the same tier are available, the one with the
/// lowest RTT is preferred. If no RTT is provided, the first candidate wins.
#[derive(Debug)]
pub struct PriorityPathSelector {
    /// Snapshot of candidates from the last `select_path` call, used for
    /// fallback in `on_path_failed`.
    candidates: Vec<PathCandidate>,
    /// Set of path types that have already been tried and failed.
    failed_tiers: Vec<PathType>,
}

impl PriorityPathSelector {
    /// Create a new path selector.
    pub fn new() -> Self {
        Self {
            candidates: Vec::new(),
            failed_tiers: Vec::new(),
        }
    }

    /// Pick the best candidate for a given tier, preferring lower RTT.
    fn best_for_tier(candidates: &[PathCandidate], tier: PathType) -> Option<ConnectionPath> {
        let mut best: Option<&PathCandidate> = None;
        for c in candidates {
            if c.path_type != tier {
                continue;
            }
            match best {
                None => best = Some(c),
                Some(prev) => {
                    // Prefer lower RTT; treat None as infinity.
                    let dominated = match (c.rtt_ms, prev.rtt_ms) {
                        (Some(new_rtt), Some(prev_rtt)) => new_rtt < prev_rtt,
                        (Some(_), None) => true,
                        _ => false,
                    };
                    if dominated {
                        best = Some(c);
                    }
                }
            }
        }
        best.map(|c| match tier {
            PathType::L1Direct => ConnectionPath::L1Direct(c.addr),
            PathType::L2Punch => ConnectionPath::L2Punch(c.addr),
            PathType::L3Relay => ConnectionPath::L3Relay(c.addr),
        })
    }
}

impl Default for PriorityPathSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl PathSelector for PriorityPathSelector {
    fn select_path(
        &mut self,
        candidates: &[PathCandidate],
    ) -> Result<ConnectionPath, ConnectionError> {
        // Store candidates for future fallback.
        self.candidates = candidates.to_vec();
        self.failed_tiers.clear();

        // Try L1 → L2 → L3 in order.
        for tier in [PathType::L1Direct, PathType::L2Punch, PathType::L3Relay] {
            if let Some(path) = Self::best_for_tier(candidates, tier) {
                return Ok(path);
            }
        }

        Err(ConnectionError::NoViablePath)
    }

    fn on_path_failed(&mut self, path: ConnectionPath) -> Option<ConnectionPath> {
        let failed = path.path_type();
        if !self.failed_tiers.contains(&failed) {
            self.failed_tiers.push(failed);
        }

        // Try the next tier in the priority chain, skipping failed ones.
        for tier in [PathType::L1Direct, PathType::L2Punch, PathType::L3Relay] {
            if self.failed_tiers.contains(&tier) {
                continue;
            }
            if let Some(fallback) = Self::best_for_tier(&self.candidates, tier) {
                return Some(fallback);
            }
        }

        None
    }
}

// ---------------------------------------------------------------------------
// Quality metrics (supplementary, for scoring when needed)
// ---------------------------------------------------------------------------

/// Quality metrics for a network path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathMetrics {
    /// Path type.
    pub path_type: PathType,
    /// Measured round-trip time in milliseconds.
    pub rtt_ms: f64,
    /// Estimated available bandwidth in bits per second.
    pub bandwidth_bps: u64,
    /// Packet loss rate (0.0 to 1.0).
    pub loss_rate: f64,
    /// Path score (higher is better). Computed from rtt, bandwidth, loss.
    pub score: f64,
}

impl PathMetrics {
    /// Compute a quality score from the path metrics.
    pub fn compute_score(&mut self) {
        let rtt_score = (1000.0 / (self.rtt_ms + 1.0)).min(100.0);
        let bw_score = (self.bandwidth_bps as f64 / 1_000_000.0).min(100.0);
        let loss_penalty = (1.0 - self.loss_rate) * 100.0;
        self.score = (rtt_score + bw_score + loss_penalty) / 3.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn addr(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), port)
    }

    fn l1(port: u16) -> PathCandidate {
        PathCandidate {
            path_type: PathType::L1Direct,
            addr: addr(port),
            rtt_ms: Some(1.0),
        }
    }

    fn l2(port: u16) -> PathCandidate {
        PathCandidate {
            path_type: PathType::L2Punch,
            addr: addr(port),
            rtt_ms: Some(30.0),
        }
    }

    fn l3(port: u16) -> PathCandidate {
        PathCandidate {
            path_type: PathType::L3Relay,
            addr: addr(port),
            rtt_ms: Some(80.0),
        }
    }

    // --- select_path tests ---

    #[test]
    fn select_l1_when_available() {
        let mut sel = PriorityPathSelector::new();
        let candidates = vec![l3(3000), l1(1000), l2(2000)];
        let path = sel.select_path(&candidates).unwrap();
        assert_eq!(path, ConnectionPath::L1Direct(addr(1000)));
    }

    #[test]
    fn select_l2_when_l1_unavailable() {
        let mut sel = PriorityPathSelector::new();
        let candidates = vec![l3(3000), l2(2000)];
        let path = sel.select_path(&candidates).unwrap();
        assert_eq!(path, ConnectionPath::L2Punch(addr(2000)));
    }

    #[test]
    fn select_l3_when_only_relay_available() {
        let mut sel = PriorityPathSelector::new();
        let candidates = vec![l3(3000)];
        let path = sel.select_path(&candidates).unwrap();
        assert_eq!(path, ConnectionPath::L3Relay(addr(3000)));
    }

    #[test]
    fn no_viable_path_when_empty() {
        let mut sel = PriorityPathSelector::new();
        let result = sel.select_path(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn prefer_lower_rtt_within_same_tier() {
        let mut sel = PriorityPathSelector::new();
        let candidates = vec![
            PathCandidate {
                path_type: PathType::L1Direct,
                addr: addr(1000),
                rtt_ms: Some(5.0),
            },
            PathCandidate {
                path_type: PathType::L1Direct,
                addr: addr(1001),
                rtt_ms: Some(1.0),
            },
        ];
        let path = sel.select_path(&candidates).unwrap();
        assert_eq!(path, ConnectionPath::L1Direct(addr(1001)));
    }

    // --- on_path_failed (fallback chain) tests ---

    #[test]
    fn fallback_l1_to_l2() {
        let mut sel = PriorityPathSelector::new();
        let candidates = vec![l1(1000), l2(2000), l3(3000)];
        let selected = sel.select_path(&candidates).unwrap();
        assert_eq!(selected.path_type(), PathType::L1Direct);

        let fallback = sel.on_path_failed(selected).unwrap();
        assert_eq!(fallback, ConnectionPath::L2Punch(addr(2000)));
    }

    #[test]
    fn fallback_l2_to_l3() {
        let mut sel = PriorityPathSelector::new();
        let candidates = vec![l1(1000), l2(2000), l3(3000)];
        let _ = sel.select_path(&candidates).unwrap();

        // Fail L1 → L2
        let l2_path = sel.on_path_failed(ConnectionPath::L1Direct(addr(1000))).unwrap();
        assert_eq!(l2_path.path_type(), PathType::L2Punch);

        // Fail L2 → L3
        let l3_path = sel.on_path_failed(l2_path).unwrap();
        assert_eq!(l3_path, ConnectionPath::L3Relay(addr(3000)));
    }

    #[test]
    fn no_fallback_after_l3_fails() {
        let mut sel = PriorityPathSelector::new();
        let candidates = vec![l3(3000)];
        let selected = sel.select_path(&candidates).unwrap();
        assert_eq!(selected.path_type(), PathType::L3Relay);

        let fallback = sel.on_path_failed(selected);
        assert!(fallback.is_none());
    }

    #[test]
    fn full_fallback_chain_l1_l2_l3_then_none() {
        let mut sel = PriorityPathSelector::new();
        let candidates = vec![l1(1000), l2(2000), l3(3000)];
        let p = sel.select_path(&candidates).unwrap();
        assert_eq!(p.path_type(), PathType::L1Direct);

        let p = sel.on_path_failed(p).unwrap();
        assert_eq!(p.path_type(), PathType::L2Punch);

        let p = sel.on_path_failed(p).unwrap();
        assert_eq!(p.path_type(), PathType::L3Relay);

        assert!(sel.on_path_failed(p).is_none());
    }

    // --- ConnectionPath helpers ---

    #[test]
    fn connection_path_addr() {
        let path = ConnectionPath::L2Punch(addr(5555));
        assert_eq!(path.addr(), addr(5555));
    }

    #[test]
    fn connection_path_type() {
        assert_eq!(ConnectionPath::L1Direct(addr(1)).path_type(), PathType::L1Direct);
        assert_eq!(ConnectionPath::L2Punch(addr(2)).path_type(), PathType::L2Punch);
        assert_eq!(ConnectionPath::L3Relay(addr(3)).path_type(), PathType::L3Relay);
    }

    // --- PathMetrics scoring ---

    #[test]
    fn path_metrics_scoring() {
        let mut m = PathMetrics {
            path_type: PathType::L1Direct,
            rtt_ms: 2.0,
            bandwidth_bps: 100_000_000,
            loss_rate: 0.0,
            score: 0.0,
        };
        m.compute_score();
        assert!(m.score > 0.0);
    }
}
