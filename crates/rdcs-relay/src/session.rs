// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Relay session tracking and UDP port-pool management.
//!
//! When an ALLOCATE message arrives with a valid HMAC token the
//! [`SessionManager`] allocates a pair of UDP ports from [`PortPool`],
//! creates a [`RelaySession`], and returns the ports so the relay loop
//! can bind and forward traffic.

#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use crate::auth;
use crate::metrics::RelayMetrics;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced by session / port operations.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("invalid token: {0}")]
    InvalidToken(String),

    #[error("session {0} already exists")]
    SessionExists(u64),

    #[error("session {0} not found")]
    SessionNotFound(u64),

    #[error("no available port pairs")]
    NoPortsAvailable,
}

// ---------------------------------------------------------------------------
// PeerInfo
// ---------------------------------------------------------------------------

/// Information about one peer in a relay session.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// The peer's socket address (IP + source port seen by the relay).
    pub addr: SocketAddr,
    /// The UDP port allocated on the relay for this peer.
    pub port: u16,
}

// ---------------------------------------------------------------------------
// RelaySession
// ---------------------------------------------------------------------------

/// A relay session mapping two peers through a port pair.
#[derive(Debug)]
pub struct RelaySession {
    /// Unique session identifier (matches the token / protocol session_id).
    pub session_id: u64,
    /// First peer (the allocator).
    pub peer_a: PeerInfo,
    /// Second peer (placeholder — filled when peer B joins).
    pub peer_b: PeerInfo,
    /// When the session was created.
    pub created_at: Instant,
    /// Last time activity was observed (allocate, keepalive, data).
    pub last_activity: Instant,
    /// Running total of bytes forwarded through this session.
    pub bytes_forwarded: u64,
}

// ---------------------------------------------------------------------------
// PortPool
// ---------------------------------------------------------------------------

/// Allocates and tracks UDP port pairs from a configured range.
pub struct PortPool {
    min_port: u16,
    max_port: u16,
    allocated: HashSet<u16>,
}

impl PortPool {
    /// Create a pool that allocates ports in `[min_port, max_port]`.
    pub fn new(min_port: u16, max_port: u16) -> Self {
        Self {
            min_port,
            max_port,
            allocated: HashSet::new(),
        }
    }

    /// Allocate two consecutive (even, odd) ports.
    ///
    /// Returns `(port_a, port_b)` where `port_a` is even and `port_b = port_a + 1`.
    /// Returns [`SessionError::NoPortsAvailable`] when the pool is exhausted.
    pub fn allocate_pair(&mut self) -> Result<(u16, u16), SessionError> {
        // Iterate aligned even-port pairs.
        let start = if self.min_port % 2 == 0 {
            self.min_port
        } else {
            self.min_port + 1
        };

        let mut port = start;
        while port < self.max_port {
            if !self.allocated.contains(&port) && !self.allocated.contains(&(port + 1)) {
                self.allocated.insert(port);
                self.allocated.insert(port + 1);
                return Ok((port, port + 1));
            }
            port += 2;
        }

        Err(SessionError::NoPortsAvailable)
    }

    /// Return two ports to the pool.
    pub fn release_pair(&mut self, port_a: u16, port_b: u16) {
        self.allocated.remove(&port_a);
        self.allocated.remove(&port_b);
    }

    /// Number of ports still available (not yet allocated).
    pub fn available_count(&self) -> usize {
        let total = (self.max_port as usize).saturating_sub(self.min_port as usize) + 1;
        total.saturating_sub(self.allocated.len())
    }
}

// ---------------------------------------------------------------------------
// SessionManager
// ---------------------------------------------------------------------------

/// Manages all active relay sessions and their port allocations.
pub struct SessionManager {
    sessions: HashMap<u64, RelaySession>,
    port_pool: PortPool,
    hmac_secret: Vec<u8>,
    /// Optional metrics collector. When set, session operations update
    /// the corresponding counters automatically.
    metrics: Option<Arc<RelayMetrics>>,
}

impl SessionManager {
    /// Create a new manager.
    ///
    /// * `min_port` / `max_port` — UDP port range for relay allocations.
    /// * `hmac_secret` — shared secret used to verify allocation tokens.
    pub fn new(min_port: u16, max_port: u16, hmac_secret: Vec<u8>) -> Self {
        Self {
            sessions: HashMap::new(),
            port_pool: PortPool::new(min_port, max_port),
            hmac_secret,
            metrics: None,
        }
    }

    /// Attach a [`RelayMetrics`] collector so that session operations
    /// automatically update counters.
    pub fn with_metrics(mut self, metrics: Arc<RelayMetrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    /// Handle an ALLOCATE request.
    ///
    /// 1. Verify the HMAC token.
    /// 2. Reject if the session already exists.
    /// 3. Allocate a port pair.
    /// 4. Create and store the [`RelaySession`].
    ///
    /// Returns the allocated `(port_a, port_b)`.
    pub fn allocate(
        &mut self,
        session_id: u64,
        token: &str,
        peer_a_addr: SocketAddr,
    ) -> Result<(u16, u16), SessionError> {
        // 1. Verify token.
        let payload = auth::verify_token(token, &self.hmac_secret)
            .map_err(|e| SessionError::InvalidToken(e.to_string()))?;

        // Cross-check: the token's session_id must match the requested one.
        if payload.session_id != session_id {
            return Err(SessionError::InvalidToken(format!(
                "token session_id {} does not match requested {}",
                payload.session_id, session_id
            )));
        }

        // 2. Reject duplicates.
        if self.sessions.contains_key(&session_id) {
            return Err(SessionError::SessionExists(session_id));
        }

        // 3. Allocate ports.
        let (port_a, port_b) = self.port_pool.allocate_pair()?;

        // 4. Create session.
        let now = Instant::now();
        let session = RelaySession {
            session_id,
            peer_a: PeerInfo {
                addr: peer_a_addr,
                port: port_a,
            },
            peer_b: PeerInfo {
                // peer_b is unassigned until it connects; use unspecified.
                addr: SocketAddr::from(([0, 0, 0, 0], 0)),
                port: port_b,
            },
            created_at: now,
            last_activity: now,
            bytes_forwarded: 0,
        };

        self.sessions.insert(session_id, session);

        // Update metrics.
        if let Some(ref m) = self.metrics {
            m.inc_session();
            m.inc_slot_alloc();
        }

        Ok((port_a, port_b))
    }

    /// Handle a RELEASE request — tear down the session and free its ports.
    pub fn release(&mut self, session_id: u64) -> Result<(), SessionError> {
        let session = self
            .sessions
            .remove(&session_id)
            .ok_or(SessionError::SessionNotFound(session_id))?;

        self.port_pool
            .release_pair(session.peer_a.port, session.peer_b.port);

        // Update metrics.
        if let Some(ref m) = self.metrics {
            m.dec_session();
            m.inc_slot_reclaim();
        }

        Ok(())
    }

    /// Register or update peer B's address for an existing session.
    ///
    /// Returns `true` if the session exists and was updated, `false` otherwise.
    pub fn set_peer_b(&mut self, session_id: u64, addr: SocketAddr) -> bool {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.peer_b.addr = addr;
            session.last_activity = Instant::now();
            true
        } else {
            false
        }
    }

    /// Resolve the forwarding target for a DATA packet and update the
    /// session's byte counter.
    ///
    /// Given a `session_id` and the source address of the sender, returns
    /// the *other* peer's address (the forwarding destination).
    ///
    /// Returns:
    /// - `Ok(SocketAddr)` when the session exists and the sender matches
    ///   one of the two peers.
    /// - `Err((session_id, None))` when the session does not exist.
    /// - `Err((session_id, Some(src)))` when neither peer matches `src`.
    pub fn forward_target(
        &mut self,
        session_id: u64,
        src_addr: SocketAddr,
    ) -> Result<SocketAddr, (u64, Option<SocketAddr>)> {
        let session = match self.sessions.get_mut(&session_id) {
            Some(s) => s,
            None => return Err((session_id, None)),
        };

        let target = if src_addr == session.peer_a.addr {
            session.peer_b.addr
        } else if src_addr == session.peer_b.addr {
            session.peer_a.addr
        } else {
            return Err((session_id, Some(src_addr)));
        };

        session.bytes_forwarded += 1;
        Ok(target)
    }

    /// Handle a KEEPALIVE — reset the activity timestamp.
    ///
    /// Returns `true` if the session exists and was touched, `false` otherwise.
    pub fn keepalive(&mut self, session_id: u64) -> bool {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.last_activity = Instant::now();
            true
        } else {
            false
        }
    }

    /// Look up a session by ID.
    pub fn get(&self, session_id: u64) -> Option<&RelaySession> {
        self.sessions.get(&session_id)
    }

    /// Remove sessions with no activity for longer than `timeout`.
    ///
    /// Returns the IDs of the sessions that were removed.
    pub fn cleanup_expired(&mut self) -> Vec<u64> {
        let timeout = std::time::Duration::from_secs(30);
        let now = Instant::now();

        let expired: Vec<u64> = self
            .sessions
            .iter()
            .filter(|(_, s)| now.duration_since(s.last_activity) > timeout)
            .map(|(id, _)| *id)
            .collect();

        for id in &expired {
            if let Some(session) = self.sessions.remove(id) {
                self.port_pool
                    .release_pair(session.peer_a.port, session.peer_b.port);
                // Update metrics.
                if let Some(ref m) = self.metrics {
                    m.dec_session();
                    m.inc_slot_reclaim();
                }
            }
        }

        expired
    }

    /// Number of currently active sessions.
    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }

    /// Number of ports still available in the pool (not yet allocated).
    pub fn available_ports(&self) -> usize {
        self.port_pool.available_count()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{generate_token, TokenPayload};
    use std::net::{Ipv4Addr, SocketAddrV4};

    const SECRET: &[u8] = b"test-session-secret";

    /// Helper: generate a valid token for the given session_id.
    fn valid_token(session_id: u64) -> String {
        let payload = TokenPayload {
            session_id,
            relay_addr: "127.0.0.1:3478".into(),
            nonce: session_id, // unique per session_id for simplicity
            expires_at: 4_102_444_800, // year 2100
        };
        generate_token(&payload, SECRET)
    }

    /// Helper: dummy peer address.
    fn peer_addr(port: u16) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(192, 168, 1, 100), port))
    }

    /// Helper: create a SessionManager with a small port range for testing.
    fn test_manager(min: u16, max: u16) -> SessionManager {
        SessionManager::new(min, max, SECRET.to_vec())
    }

    // -- PortPool tests -------------------------------------------------------

    #[test]
    fn port_pool_allocate_pair() {
        let mut pool = PortPool::new(50000, 50009);
        let (a, b) = pool.allocate_pair().unwrap();
        assert_eq!(a, 50000);
        assert_eq!(b, 50001);
        assert_eq!(pool.available_count(), 8);
    }

    #[test]
    fn port_pool_allocate_multiple_pairs() {
        let mut pool = PortPool::new(50000, 50007);
        let (a1, b1) = pool.allocate_pair().unwrap();
        let (a2, b2) = pool.allocate_pair().unwrap();
        let (a3, b3) = pool.allocate_pair().unwrap();
        let (a4, b4) = pool.allocate_pair().unwrap();

        assert_eq!((a1, b1), (50000, 50001));
        assert_eq!((a2, b2), (50002, 50003));
        assert_eq!((a3, b3), (50004, 50005));
        assert_eq!((a4, b4), (50006, 50007));
        assert_eq!(pool.available_count(), 0);
    }

    #[test]
    fn port_pool_exhaustion() {
        let mut pool = PortPool::new(50000, 50001); // only 1 pair
        let _ = pool.allocate_pair().unwrap();
        let err = pool.allocate_pair().unwrap_err();
        assert!(
            matches!(err, SessionError::NoPortsAvailable),
            "expected NoPortsAvailable, got {err:?}"
        );
    }

    #[test]
    fn port_pool_release_and_reuse() {
        let mut pool = PortPool::new(50000, 50001);
        let (a, b) = pool.allocate_pair().unwrap();
        assert_eq!(pool.available_count(), 0);

        pool.release_pair(a, b);
        assert_eq!(pool.available_count(), 2);

        // Should be able to allocate again.
        let (a2, b2) = pool.allocate_pair().unwrap();
        assert_eq!((a2, b2), (50000, 50001));
    }

    #[test]
    fn port_pool_available_count() {
        let pool = PortPool::new(50000, 50009);
        assert_eq!(pool.available_count(), 10);
    }

    #[test]
    fn port_pool_odd_min() {
        // When min_port is odd, allocation starts at the next even port.
        let mut pool = PortPool::new(50001, 50010);
        let (a, b) = pool.allocate_pair().unwrap();
        assert_eq!(a, 50002);
        assert_eq!(b, 50003);
    }

    // -- Acceptance: allocate_creates_session ---------------------------------

    #[test]
    fn allocate_creates_session() {
        let mut mgr = test_manager(50000, 50099);
        let token = valid_token(1);
        let addr = peer_addr(12345);

        let (port_a, port_b) = mgr.allocate(1, &token, addr).unwrap();

        assert!(port_a >= 50000 && port_b <= 50099);
        assert_eq!(port_b, port_a + 1);
        assert_eq!(mgr.active_count(), 1);

        let session = mgr.get(1).unwrap();
        assert_eq!(session.session_id, 1);
        assert_eq!(session.peer_a.addr, addr);
        assert_eq!(session.peer_a.port, port_a);
        assert_eq!(session.peer_b.port, port_b);
        assert_eq!(session.bytes_forwarded, 0);
    }

    // -- Acceptance: allocate_rejects_invalid_token ---------------------------

    #[test]
    fn allocate_rejects_invalid_token() {
        let mut mgr = test_manager(50000, 50099);

        let err = mgr
            .allocate(1, "totally-invalid-token", peer_addr(1000))
            .unwrap_err();
        assert!(
            matches!(err, SessionError::InvalidToken(_)),
            "expected InvalidToken, got {err:?}"
        );
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn allocate_rejects_wrong_secret() {
        let mut mgr = test_manager(50000, 50099);
        let payload = TokenPayload {
            session_id: 1,
            relay_addr: "127.0.0.1:3478".into(),
            nonce: 1,
            expires_at: 4_102_444_800,
        };
        let token = generate_token(&payload, b"wrong-secret");

        let err = mgr.allocate(1, &token, peer_addr(1000)).unwrap_err();
        assert!(
            matches!(err, SessionError::InvalidToken(_)),
            "expected InvalidToken, got {err:?}"
        );
    }

    #[test]
    fn allocate_rejects_mismatched_session_id() {
        let mut mgr = test_manager(50000, 50099);
        let token = valid_token(42); // token says session_id = 42

        let err = mgr.allocate(99, &token, peer_addr(1000)).unwrap_err();
        assert!(
            matches!(err, SessionError::InvalidToken(_)),
            "expected InvalidToken for mismatched session_id, got {err:?}"
        );
    }

    #[test]
    fn allocate_rejects_duplicate_session() {
        let mut mgr = test_manager(50000, 50099);
        let token = valid_token(1);
        mgr.allocate(1, &token, peer_addr(1000)).unwrap();

        // Second allocate with the same session_id must fail.
        let token2 = valid_token(1);
        let err = mgr.allocate(1, &token2, peer_addr(2000)).unwrap_err();
        assert!(
            matches!(err, SessionError::SessionExists(1)),
            "expected SessionExists(1), got {err:?}"
        );
    }

    // -- Acceptance: release_frees_ports --------------------------------------

    #[test]
    fn release_frees_ports() {
        let mut mgr = test_manager(50000, 50001); // only 1 pair available
        let token = valid_token(1);
        let (port_a, port_b) = mgr.allocate(1, &token, peer_addr(1000)).unwrap();

        // Pool is now exhausted.
        assert_eq!(mgr.active_count(), 1);

        // Release.
        mgr.release(1).unwrap();
        assert_eq!(mgr.active_count(), 0);
        assert!(mgr.get(1).is_none());

        // Ports should be available again — allocate a new session.
        let token2 = valid_token(2);
        let (pa2, pb2) = mgr.allocate(2, &token2, peer_addr(2000)).unwrap();
        assert_eq!((pa2, pb2), (port_a, port_b));
    }

    #[test]
    fn release_nonexistent_session() {
        let mut mgr = test_manager(50000, 50099);
        let err = mgr.release(999).unwrap_err();
        assert!(
            matches!(err, SessionError::SessionNotFound(999)),
            "expected SessionNotFound(999), got {err:?}"
        );
    }

    // -- Acceptance: port_exhaustion ------------------------------------------

    #[test]
    fn port_exhaustion() {
        let mut mgr = test_manager(50000, 50003); // 2 pairs: (50000,50001), (50002,50003)

        let t1 = valid_token(1);
        let t2 = valid_token(2);
        mgr.allocate(1, &t1, peer_addr(1000)).unwrap();
        mgr.allocate(2, &t2, peer_addr(2000)).unwrap();

        // All ports used.
        let t3 = valid_token(3);
        let err = mgr.allocate(3, &t3, peer_addr(3000)).unwrap_err();
        assert!(
            matches!(err, SessionError::NoPortsAvailable),
            "expected NoPortsAvailable, got {err:?}"
        );

        // Release one and try again.
        mgr.release(1).unwrap();
        let t4 = valid_token(4);
        let result = mgr.allocate(4, &t4, peer_addr(4000));
        assert!(result.is_ok(), "allocation should succeed after release");
    }

    // -- Acceptance: keepalive_resets_timeout ---------------------------------

    #[test]
    fn keepalive_resets_timeout() {
        let mut mgr = test_manager(50000, 50099);
        let token = valid_token(1);
        mgr.allocate(1, &token, peer_addr(1000)).unwrap();

        // First keepalive should succeed.
        assert!(mgr.keepalive(1), "keepalive for existing session must return true");

        // Unknown session returns false.
        assert!(!mgr.keepalive(999), "keepalive for unknown session must return false");
    }

    #[test]
    fn keepalive_updates_last_activity() {
        let mut mgr = test_manager(50000, 50099);
        let token = valid_token(1);
        mgr.allocate(1, &token, peer_addr(1000)).unwrap();

        let activity_before = mgr.get(1).unwrap().last_activity;

        // Small delay so Instant::now() advances.
        std::thread::sleep(std::time::Duration::from_millis(5));

        assert!(mgr.keepalive(1));
        let activity_after = mgr.get(1).unwrap().last_activity;

        assert!(
            activity_after > activity_before,
            "keepalive should advance last_activity"
        );
    }

    // -- Acceptance: cleanup_removes_expired ----------------------------------

    #[test]
    fn cleanup_removes_expired() {
        let mut mgr = test_manager(50000, 50099);

        // Allocate two sessions.
        let t1 = valid_token(1);
        let t2 = valid_token(2);
        mgr.allocate(1, &t1, peer_addr(1000)).unwrap();
        mgr.allocate(2, &t2, peer_addr(2000)).unwrap();
        assert_eq!(mgr.active_count(), 2);

        // Manually backdate session 1's last_activity to 60s ago.
        let backdate = Instant::now() - std::time::Duration::from_secs(60);
        mgr.sessions.get_mut(&1).unwrap().last_activity = backdate;

        let removed = mgr.cleanup_expired();
        assert_eq!(removed, vec![1], "only session 1 should be expired");
        assert_eq!(mgr.active_count(), 1);
        assert!(mgr.get(1).is_none());
        assert!(mgr.get(2).is_some());
    }

    #[test]
    fn cleanup_releases_ports_of_expired_sessions() {
        let mut mgr = test_manager(50000, 50001); // 1 pair
        let token = valid_token(1);
        let (port_a, port_b) = mgr.allocate(1, &token, peer_addr(1000)).unwrap();

        // Backdate to trigger cleanup.
        let backdate = Instant::now() - std::time::Duration::from_secs(60);
        mgr.sessions.get_mut(&1).unwrap().last_activity = backdate;

        let removed = mgr.cleanup_expired();
        assert_eq!(removed.len(), 1);

        // Ports should be free now.
        let token2 = valid_token(2);
        let (pa2, pb2) = mgr.allocate(2, &token2, peer_addr(2000)).unwrap();
        assert_eq!((pa2, pb2), (port_a, port_b));
    }

    #[test]
    fn cleanup_no_expired_sessions() {
        let mut mgr = test_manager(50000, 50099);
        let token = valid_token(1);
        mgr.allocate(1, &token, peer_addr(1000)).unwrap();

        // Session was just created — should not be cleaned up.
        let removed = mgr.cleanup_expired();
        assert!(removed.is_empty());
        assert_eq!(mgr.active_count(), 1);
    }

    // -- SessionError Display -------------------------------------------------

    #[test]
    fn session_error_display() {
        assert_eq!(
            SessionError::InvalidToken("bad".into()).to_string(),
            "invalid token: bad"
        );
        assert_eq!(
            SessionError::SessionExists(42).to_string(),
            "session 42 already exists"
        );
        assert_eq!(
            SessionError::SessionNotFound(7).to_string(),
            "session 7 not found"
        );
        assert_eq!(
            SessionError::NoPortsAvailable.to_string(),
            "no available port pairs"
        );
    }

    // -- SessionManager basics ------------------------------------------------

    #[test]
    fn manager_active_count_starts_at_zero() {
        let mgr = test_manager(50000, 50099);
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn manager_get_nonexistent() {
        let mgr = test_manager(50000, 50099);
        assert!(mgr.get(1).is_none());
    }

    #[test]
    fn manager_multiple_sessions() {
        let mut mgr = test_manager(50000, 50099);
        for i in 1..=5 {
            let token = valid_token(i);
            mgr.allocate(i, &token, peer_addr(1000 + i as u16)).unwrap();
        }
        assert_eq!(mgr.active_count(), 5);

        for i in 1..=5 {
            assert!(mgr.get(i).is_some());
        }
    }

    // -- Acceptance: session_metrics_integrated --------------------------------

    #[test]
    fn allocate_updates_metrics() {
        use crate::metrics::RelayMetrics;
        use std::sync::Arc;

        let metrics = Arc::new(RelayMetrics::new());
        let mut mgr = SessionManager::new(50000, 50099, SECRET.to_vec())
            .with_metrics(Arc::clone(&metrics));

        let token = valid_token(1);
        mgr.allocate(1, &token, peer_addr(1000)).unwrap();

        let snap = metrics.snapshot();
        assert_eq!(snap.active_sessions, 1, "allocate should inc_session");
        assert_eq!(snap.slots_allocated, 1, "allocate should inc_slot_alloc");
    }

    #[test]
    fn release_updates_metrics() {
        use crate::metrics::RelayMetrics;
        use std::sync::Arc;

        let metrics = Arc::new(RelayMetrics::new());
        let mut mgr = SessionManager::new(50000, 50099, SECRET.to_vec())
            .with_metrics(Arc::clone(&metrics));

        let token = valid_token(1);
        mgr.allocate(1, &token, peer_addr(1000)).unwrap();
        assert_eq!(metrics.snapshot().active_sessions, 1);

        mgr.release(1).unwrap();

        let snap = metrics.snapshot();
        assert_eq!(snap.active_sessions, 0, "release should dec_session");
        assert_eq!(snap.slots_reclaimed, 1, "release should inc_slot_reclaim");
        assert_eq!(snap.slots_allocated, 1, "slots_allocated unchanged after release");
    }

    #[test]
    fn cleanup_updates_metrics() {
        use crate::metrics::RelayMetrics;
        use std::sync::Arc;

        let metrics = Arc::new(RelayMetrics::new());
        let mut mgr = SessionManager::new(50000, 50099, SECRET.to_vec())
            .with_metrics(Arc::clone(&metrics));

        let t1 = valid_token(1);
        let t2 = valid_token(2);
        mgr.allocate(1, &t1, peer_addr(1000)).unwrap();
        mgr.allocate(2, &t2, peer_addr(2000)).unwrap();
        assert_eq!(metrics.snapshot().active_sessions, 2);

        // Backdate session 1 to trigger cleanup.
        let backdate = Instant::now() - std::time::Duration::from_secs(60);
        mgr.sessions.get_mut(&1).unwrap().last_activity = backdate;

        let removed = mgr.cleanup_expired();
        assert_eq!(removed.len(), 1);

        let snap = metrics.snapshot();
        assert_eq!(snap.active_sessions, 1, "cleanup should dec_session for expired");
        assert_eq!(snap.slots_reclaimed, 1, "cleanup should inc_slot_reclaim for expired");
    }

    #[test]
    fn metrics_not_required() {
        // SessionManager without metrics should work fine (no panics).
        let mut mgr = test_manager(50000, 50099);
        let token = valid_token(1);
        mgr.allocate(1, &token, peer_addr(1000)).unwrap();
        mgr.release(1).unwrap();
        assert_eq!(mgr.active_count(), 0);
    }

    #[test]
    fn multiple_alloc_release_cycles() {
        use crate::metrics::RelayMetrics;
        use std::sync::Arc;

        let metrics = Arc::new(RelayMetrics::new());
        let mut mgr = SessionManager::new(50000, 50099, SECRET.to_vec())
            .with_metrics(Arc::clone(&metrics));

        for i in 1..=3 {
            let token = valid_token(i);
            mgr.allocate(i, &token, peer_addr(1000 + i as u16)).unwrap();
        }
        assert_eq!(metrics.snapshot().active_sessions, 3);
        assert_eq!(metrics.snapshot().slots_allocated, 3);

        mgr.release(1).unwrap();
        mgr.release(2).unwrap();
        assert_eq!(metrics.snapshot().active_sessions, 1);
        assert_eq!(metrics.snapshot().slots_reclaimed, 2);

        // Allocate one more.
        let token4 = valid_token(4);
        mgr.allocate(4, &token4, peer_addr(4000)).unwrap();
        assert_eq!(metrics.snapshot().active_sessions, 2);
        assert_eq!(metrics.snapshot().slots_allocated, 4);
    }
}
