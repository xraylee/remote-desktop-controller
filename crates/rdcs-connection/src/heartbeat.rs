// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Heartbeat mechanism for connection liveness monitoring.

use std::net::SocketAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::ConnectionError;

// ---------------------------------------------------------------------------
// Trait interface
// ---------------------------------------------------------------------------

/// Manages heartbeat probes for a connection and reports liveness.
pub trait HeartbeatManager: Send {
    /// Begin sending heartbeat probes to the given peer.
    fn start(&self, peer: SocketAddr) -> Result<(), ConnectionError>;

    /// Returns `true` if the peer has responded within the TTL window.
    fn is_alive(&self) -> bool;

    /// Return the most recently measured round-trip time in milliseconds,
    /// or `None` if no measurement is available.
    fn last_rtt(&self) -> Option<u32>;
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the heartbeat monitor.
#[derive(Debug, Clone)]
pub struct HeartbeatConfig {
    /// Interval between heartbeat probes.
    pub interval: Duration,
    /// Time-to-live: if no heartbeat response is received within this
    /// duration the peer is considered dead. **Default: 30 s.**
    pub ttl: Duration,
    /// Timeout for a single heartbeat response.
    pub response_timeout: Duration,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5),
            ttl: Duration::from_secs(30),
            response_timeout: Duration::from_secs(5),
        }
    }
}

// ---------------------------------------------------------------------------
// Low-level heartbeat monitor (stateful, not thread-safe on its own)
// ---------------------------------------------------------------------------

/// Tracks heartbeat state for a connection (single-threaded building block).
#[derive(Debug)]
pub struct HeartbeatMonitor {
    config: HeartbeatConfig,
    last_pong: Option<Instant>,
    last_rtt_ms: Option<u32>,
}

impl HeartbeatMonitor {
    /// Create a new monitor with the given configuration.
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            config,
            last_pong: None,
            last_rtt_ms: None,
        }
    }

    /// Record a successful pong response with the measured RTT.
    pub fn on_pong(&mut self, rtt_ms: u32) {
        self.last_pong = Some(Instant::now());
        self.last_rtt_ms = Some(rtt_ms);
    }

    /// Check whether the peer is still considered alive based on the TTL.
    pub fn is_alive(&self) -> bool {
        match self.last_pong {
            Some(t) => t.elapsed() < self.config.ttl,
            None => false,
        }
    }

    /// Return the last measured RTT.
    pub fn last_rtt(&self) -> Option<u32> {
        self.last_rtt_ms
    }

    /// Return the heartbeat interval.
    pub fn interval(&self) -> Duration {
        self.config.interval
    }

    /// Return the TTL.
    pub fn ttl(&self) -> Duration {
        self.config.ttl
    }

    /// Manually set the last-pong instant (useful for testing).
    #[cfg(test)]
    fn set_last_pong(&mut self, instant: Instant) {
        self.last_pong = Some(instant);
    }
}

// ---------------------------------------------------------------------------
// Thread-safe HeartbeatManager implementation
// ---------------------------------------------------------------------------

/// Thread-safe [`HeartbeatManager`] implementation that wraps a
/// [`HeartbeatMonitor`] with a mutex.
#[derive(Debug)]
pub struct HeartbeatManagerImpl {
    inner: Mutex<HeartbeatInner>,
}

#[derive(Debug)]
struct HeartbeatInner {
    monitor: HeartbeatMonitor,
    peer: Option<SocketAddr>,
}

impl HeartbeatManagerImpl {
    /// Create a new heartbeat manager with the given configuration.
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            inner: Mutex::new(HeartbeatInner {
                monitor: HeartbeatMonitor::new(config),
                peer: None,
            }),
        }
    }

    /// Record a successful pong with the measured RTT (in ms).
    pub fn on_pong(&self, rtt_ms: u32) {
        let mut inner = self.inner.lock().unwrap();
        inner.monitor.on_pong(rtt_ms);
    }

    /// Return the currently tracked peer address.
    pub fn peer(&self) -> Option<SocketAddr> {
        self.inner.lock().unwrap().peer
    }

    /// Manually set the "last pong" instant for testing.
    #[cfg(test)]
    pub fn set_last_pong_for_test(&self, instant: Instant) {
        let mut inner = self.inner.lock().unwrap();
        inner.monitor.set_last_pong(instant);
    }
}

impl HeartbeatManager for HeartbeatManagerImpl {
    fn start(&self, peer: SocketAddr) -> Result<(), ConnectionError> {
        let mut inner = self.inner.lock().unwrap();
        inner.peer = Some(peer);
        // Treat start as the initial "pong" so the peer is alive immediately.
        inner.monitor.on_pong(0);
        Ok(())
    }

    fn is_alive(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.monitor.is_alive()
    }

    fn last_rtt(&self) -> Option<u32> {
        let inner = self.inner.lock().unwrap();
        inner.monitor.last_rtt()
    }
}

// ---------------------------------------------------------------------------
// Mock implementation
// ---------------------------------------------------------------------------

/// A mock heartbeat manager for tests where you want full control over
/// liveness state without real timing.
#[derive(Debug)]
pub struct MockHeartbeatManager {
    alive: Mutex<bool>,
    rtt: Mutex<Option<u32>>,
}

impl MockHeartbeatManager {
    /// Create a mock that reports as alive with no RTT.
    pub fn new() -> Self {
        Self {
            alive: Mutex::new(true),
            rtt: Mutex::new(None),
        }
    }

    /// Override the alive state.
    pub fn set_alive(&self, alive: bool) {
        *self.alive.lock().unwrap() = alive;
    }

    /// Override the RTT value.
    pub fn set_rtt(&self, rtt_ms: Option<u32>) {
        *self.rtt.lock().unwrap() = rtt_ms;
    }
}

impl Default for MockHeartbeatManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HeartbeatManager for MockHeartbeatManager {
    fn start(&self, _peer: SocketAddr) -> Result<(), ConnectionError> {
        self.set_alive(true);
        Ok(())
    }

    fn is_alive(&self) -> bool {
        *self.alive.lock().unwrap()
    }

    fn last_rtt(&self) -> Option<u32> {
        *self.rtt.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 21115)
    }

    // --- HeartbeatMonitor tests ---

    #[test]
    fn monitor_not_alive_before_any_pong() {
        let monitor = HeartbeatMonitor::new(HeartbeatConfig::default());
        assert!(!monitor.is_alive());
    }

    #[test]
    fn monitor_alive_after_pong() {
        let mut monitor = HeartbeatMonitor::new(HeartbeatConfig::default());
        monitor.on_pong(10);
        assert!(monitor.is_alive());
        assert_eq!(monitor.last_rtt(), Some(10));
    }

    #[test]
    fn monitor_dead_after_ttl_expires() {
        let config = HeartbeatConfig {
            ttl: Duration::from_secs(30),
            ..Default::default()
        };
        let mut monitor = HeartbeatMonitor::new(config);
        monitor.on_pong(5);
        assert!(monitor.is_alive());

        // Simulate a pong that happened 31 seconds ago.
        monitor.set_last_pong(Instant::now() - Duration::from_secs(31));
        assert!(!monitor.is_alive());
    }

    #[test]
    fn monitor_alive_within_ttl() {
        let config = HeartbeatConfig {
            ttl: Duration::from_secs(30),
            ..Default::default()
        };
        let mut monitor = HeartbeatMonitor::new(config);
        monitor.on_pong(5);

        // Simulate a pong that happened 29 seconds ago (within TTL).
        monitor.set_last_pong(Instant::now() - Duration::from_secs(29));
        assert!(monitor.is_alive());
    }

    #[test]
    fn monitor_exactly_at_ttl_boundary() {
        let config = HeartbeatConfig {
            ttl: Duration::from_secs(30),
            ..Default::default()
        };
        let mut monitor = HeartbeatMonitor::new(config);
        monitor.on_pong(5);

        // Exactly 30s should be NOT alive (elapsed >= ttl).
        monitor.set_last_pong(Instant::now() - Duration::from_secs(30));
        assert!(!monitor.is_alive());
    }

    // --- HeartbeatManagerImpl tests ---

    #[test]
    fn manager_start_makes_peer_alive() {
        let mgr = HeartbeatManagerImpl::new(HeartbeatConfig::default());
        assert!(!mgr.is_alive()); // no pong yet

        mgr.start(test_addr()).unwrap();
        assert!(mgr.is_alive());
        assert_eq!(mgr.peer(), Some(test_addr()));
    }

    #[test]
    fn manager_on_pong_updates_rtt() {
        let mgr = HeartbeatManagerImpl::new(HeartbeatConfig::default());
        mgr.start(test_addr()).unwrap();
        mgr.on_pong(42);
        assert_eq!(mgr.last_rtt(), Some(42));
    }

    #[test]
    fn manager_30s_ttl_detection() {
        let config = HeartbeatConfig {
            ttl: Duration::from_secs(30),
            ..Default::default()
        };
        let mgr = HeartbeatManagerImpl::new(config);
        mgr.start(test_addr()).unwrap();
        assert!(mgr.is_alive());

        // Simulate the last pong being 31 seconds ago.
        mgr.set_last_pong_for_test(Instant::now() - Duration::from_secs(31));
        assert!(!mgr.is_alive());
    }

    #[test]
    fn manager_pong_resets_ttl() {
        let config = HeartbeatConfig {
            ttl: Duration::from_secs(30),
            ..Default::default()
        };
        let mgr = HeartbeatManagerImpl::new(config);
        mgr.start(test_addr()).unwrap();

        // Make it almost dead.
        mgr.set_last_pong_for_test(Instant::now() - Duration::from_secs(29));
        assert!(mgr.is_alive());

        // Fresh pong brings it back.
        mgr.on_pong(10);
        assert!(mgr.is_alive());
        assert_eq!(mgr.last_rtt(), Some(10));
    }

    // --- MockHeartbeatManager tests ---

    #[test]
    fn mock_alive_control() {
        let mock = MockHeartbeatManager::new();
        assert!(mock.is_alive());

        mock.set_alive(false);
        assert!(!mock.is_alive());

        mock.start(test_addr()).unwrap();
        assert!(mock.is_alive());
    }

    #[test]
    fn mock_rtt_control() {
        let mock = MockHeartbeatManager::new();
        assert_eq!(mock.last_rtt(), None);

        mock.set_rtt(Some(100));
        assert_eq!(mock.last_rtt(), Some(100));
    }
}
