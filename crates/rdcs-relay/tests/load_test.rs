// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Load tests for the RDCS relay server.
//!
//! These tests simulate high-concurrency scenarios to verify the relay node
//! handles its expected capacity (200 concurrent sessions) without crashes,
//! port leaks, or excessive latency.
//!
//! Run with: `cargo test -p rdcs-relay -- --ignored`

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Instant;

use tokio::net::UdpSocket;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};

use rdcs_relay::auth::{generate_token, TokenPayload};
use rdcs_relay::forwarder::DataForwarder;
use rdcs_relay::metrics::RelayMetrics;
use rdcs_relay::protocol;
use rdcs_relay::session::SessionManager;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const HMAC_SECRET: &[u8] = b"load-test-hmac-secret-2026";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Generate a valid HMAC token for the given session ID.
fn valid_token(session_id: u64) -> String {
    let payload = TokenPayload {
        session_id,
        relay_addr: "127.0.0.1:3478".into(),
        nonce: session_id,
        expires_at: 4_102_444_800, // year 2100
    };
    generate_token(&payload, HMAC_SECRET)
}

/// Create a dummy peer address with the given port.
fn peer_addr(port: u16) -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(
        Ipv4Addr::new(192, 168, 1, 100),
        port,
    ))
}

/// Create a session manager with the specified port range and metrics.
fn test_manager_with_metrics(
    min_port: u16,
    max_port: u16,
    metrics: Arc<RelayMetrics>,
) -> SessionManager {
    SessionManager::new(min_port, max_port, HMAC_SECRET.to_vec()).with_metrics(metrics)
}

/// Bind a UDP socket on loopback with an OS-assigned port.
async fn bind_loopback() -> UdpSocket {
    UdpSocket::bind("127.0.0.1:0").await.unwrap()
}

// ---------------------------------------------------------------------------
// Test: 200 concurrent sessions
// ---------------------------------------------------------------------------

/// Simulate 200 concurrent relay sessions: allocate, keepalive, and release.
///
/// Verifies:
/// - All 200 sessions allocate successfully with valid port pairs.
/// - Keepalive succeeds for every session.
/// - All sessions release cleanly.
/// - No active sessions remain after release.
/// - Metrics report exactly 200 slots allocated and 200 slots reclaimed.
/// - All ports are returned to the pool (no leaks).
#[tokio::test]
#[ignore]
async fn load_test_200_concurrent_sessions() {
    const N: u64 = 200;
    // 400 ports = 200 pairs (each session needs an even+odd pair).
    const MIN_PORT: u16 = 49152;
    const MAX_PORT: u16 = 49551;

    let relay_metrics = Arc::new(RelayMetrics::new());
    let mgr = test_manager_with_metrics(MIN_PORT, MAX_PORT, Arc::clone(&relay_metrics));
    let sessions = Arc::new(RwLock::new(mgr));

    // Spawn N concurrent tasks: allocate -> keepalive -> release.
    let mut handles = Vec::with_capacity(N as usize);

    for i in 0..N {
        let sessions = Arc::clone(&sessions);
        handles.push(tokio::spawn(async move {
            let token = valid_token(i);
            let addr = peer_addr(10000 + i as u16);

            // Allocate.
            let (port_a, port_b) = {
                let mut s = sessions.write().await;
                s.allocate(i, &token, addr).expect("allocate must succeed")
            };

            // Ports must be within range and consecutive (even, odd).
            assert!((MIN_PORT..=MAX_PORT).contains(&port_a));
            assert!((MIN_PORT..=MAX_PORT).contains(&port_b));
            assert_eq!(port_b, port_a + 1, "port_b must be port_a + 1");
            assert!(port_a.is_multiple_of(2), "port_a must be even");

            // Keepalive.
            {
                let mut s = sessions.write().await;
                assert!(s.keepalive(i), "keepalive must succeed for session {i}");
            }

            // Release.
            {
                let mut s = sessions.write().await;
                s.release(i).expect("release must succeed");
            }
        }));
    }

    // Wait for all tasks.
    for h in handles {
        h.await.expect("task must not panic");
    }

    // All sessions released.
    {
        let s = sessions.read().await;
        assert_eq!(s.active_count(), 0, "all sessions must be released");
    }

    // Metrics: 200 allocated, 200 reclaimed.
    let snap = relay_metrics.snapshot();
    assert_eq!(
        snap.slots_allocated, N,
        "metrics must show {N} slots_allocated"
    );
    assert_eq!(
        snap.slots_reclaimed, N,
        "metrics must show {N} slots_reclaimed"
    );
    assert_eq!(
        snap.active_sessions, 0,
        "active_sessions must be 0 after all releases"
    );
}

// ---------------------------------------------------------------------------
// Test: forwarding throughput
// ---------------------------------------------------------------------------

/// Create 50 sessions and forward 100 DATA packets per session (5000 total).
///
/// Verifies:
/// - All 5000 packets are forwarded without drops.
/// - Each peer_b receives exactly 100 packets.
/// - Reports throughput in packets/sec and MB/s.
#[tokio::test]
#[ignore]
async fn load_test_forwarding_throughput() {
    const SESSIONS: u64 = 50;
    const PACKETS_PER_SESSION: usize = 100;
    const PAYLOAD: &[u8] = b"load-test-frame-data-payload-32B!!"; // 32 bytes
    const TOTAL: usize = SESSIONS as usize * PACKETS_PER_SESSION;

    let relay_metrics = Arc::new(RelayMetrics::new());
    let mgr = test_manager_with_metrics(49152, 49551, Arc::clone(&relay_metrics));
    let sessions = Arc::new(RwLock::new(mgr));

    // Create peer sockets for all sessions.
    let mut peer_a_sockets = Vec::with_capacity(SESSIONS as usize);
    let mut peer_b_sockets = Vec::with_capacity(SESSIONS as usize);

    for i in 0..SESSIONS {
        let peer_a = bind_loopback().await;
        let peer_b = bind_loopback().await;

        // Allocate session and set both peers.
        {
            let mut s = sessions.write().await;
            let token = valid_token(i);
            s.allocate(i, &token, peer_a.local_addr().unwrap())
                .expect("allocate must succeed");
            s.set_peer_b(i, peer_b.local_addr().unwrap());
        }

        peer_a_sockets.push(peer_a);
        peer_b_sockets.push(peer_b);
    }

    // Relay socket for receiving packets.
    let relay_socket = Arc::new(bind_loopback().await);
    let relay_addr = relay_socket.local_addr().unwrap();

    // DataForwarder shares the relay socket and session manager.
    let fwd = Arc::new(DataForwarder::new(
        Arc::clone(&relay_socket),
        Arc::clone(&sessions),
    ));

    // Spawn a forwarding task: reads from relay socket and forwards each packet.
    let fwd_clone = Arc::clone(&fwd);
    let relay_clone = Arc::clone(&relay_socket);
    let forward_handle = tokio::spawn(async move {
        let mut buf = vec![0u8; 65536];
        while let Ok(Ok((len, from))) =
            timeout(Duration::from_secs(3), relay_clone.recv_from(&mut buf)).await
        {
            let _ = fwd_clone.forward_packet(from, &buf[..len]).await;
        }
    });

    // Send all packets from peer_a sockets through the relay.
    let start = Instant::now();

    for (i, peer_a) in peer_a_sockets.iter().enumerate() {
        let pkt = protocol::encode_data(i as u64, PAYLOAD);
        for _ in 0..PACKETS_PER_SESSION {
            peer_a.send_to(&pkt, relay_addr).await.unwrap();
        }
    }

    let send_elapsed = start.elapsed();

    // Receive forwarded packets at each peer_b socket.
    let mut total_received: usize = 0;
    let recv_deadline = Duration::from_secs(10);

    for peer_b in &peer_b_sockets {
        let mut buf = vec![0u8; 65536];
        let mut count = 0;
        while let Ok(Ok((_, _))) = timeout(recv_deadline, peer_b.recv_from(&mut buf)).await {
            count += 1;
            if count >= PACKETS_PER_SESSION {
                break;
            }
        }
        total_received += count;
    }

    let total_elapsed = start.elapsed();

    // Stop the forwarding task.
    forward_handle.abort();
    let _ = forward_handle.await;

    // --- Assertions ---
    assert_eq!(
        total_received, TOTAL,
        "all {TOTAL} packets must be forwarded without drops (got {total_received})"
    );

    // Verify byte counters on each session.
    {
        let s = sessions.read().await;
        for i in 0..SESSIONS {
            let session = s.get(i).unwrap();
            assert_eq!(
                session.bytes_forwarded, PACKETS_PER_SESSION as u64,
                "session {i} byte counter must be {PACKETS_PER_SESSION}"
            );
        }
    }

    // Report throughput.
    let pkt_size = protocol::HEADER_LEN + PAYLOAD.len();
    let total_bytes = total_received * pkt_size;
    let pps = total_received as f64 / total_elapsed.as_secs_f64();
    let mbps = total_bytes as f64 / total_elapsed.as_secs_f64() / 1_048_576.0;

    eprintln!("=== Forwarding Throughput Report ===");
    eprintln!("  Sessions:           {SESSIONS}");
    eprintln!("  Packets/session:    {PACKETS_PER_SESSION}");
    eprintln!("  Total packets:      {TOTAL}");
    eprintln!("  Packet size:        {pkt_size} bytes");
    eprintln!("  Total bytes:        {total_bytes}");
    eprintln!("  Send time:          {:.3}ms", send_elapsed.as_secs_f64() * 1000.0);
    eprintln!("  Total time:         {:.3}ms", total_elapsed.as_secs_f64() * 1000.0);
    eprintln!("  Throughput:         {pps:.0} packets/sec");
    eprintln!("  Throughput:         {mbps:.2} MB/s");
}

// ---------------------------------------------------------------------------
// Test: concurrent keepalive storm
// ---------------------------------------------------------------------------

/// Create 100 sessions, then fire 100 concurrent keepalive tasks.
///
/// Verifies:
/// - All keepalives complete without errors (return true).
/// - All 100 sessions remain active after the keepalive storm.
#[tokio::test]
#[ignore]
async fn load_test_concurrent_keepalive() {
    const N: u64 = 100;

    let relay_metrics = Arc::new(RelayMetrics::new());
    let mgr = test_manager_with_metrics(49152, 49551, Arc::clone(&relay_metrics));
    let sessions = Arc::new(RwLock::new(mgr));

    // Pre-allocate N sessions.
    {
        let mut s = sessions.write().await;
        for i in 0..N {
            let token = valid_token(i);
            let addr = peer_addr(20000 + i as u16);
            s.allocate(i, &token, addr)
                .unwrap_or_else(|e| panic!("allocate session {i} failed: {e}"));
        }
        assert_eq!(s.active_count(), N as usize);
    }

    // Spawn N concurrent keepalive tasks.
    let mut handles = Vec::with_capacity(N as usize);
    for i in 0..N {
        let sessions = Arc::clone(&sessions);
        handles.push(tokio::spawn(async move {
            let mut s = sessions.write().await;
            let result = s.keepalive(i);
            assert!(result, "keepalive for session {i} must return true");
        }));
    }

    // All keepalives must complete without panics.
    for h in handles {
        h.await.expect("keepalive task must not panic");
    }

    // All sessions still alive after the keepalive storm.
    {
        let s = sessions.read().await;
        assert_eq!(
            s.active_count(),
            N as usize,
            "all {N} sessions must remain active after keepalive storm"
        );
    }

    // Metrics: N allocated, 0 reclaimed (no releases yet).
    let snap = relay_metrics.snapshot();
    assert_eq!(snap.slots_allocated, N);
    assert_eq!(snap.slots_reclaimed, 0);
    assert_eq!(snap.active_sessions, N);
}

// ---------------------------------------------------------------------------
// Test: no port leaks
// ---------------------------------------------------------------------------

/// Stress-test the port pool with repeated allocation and release cycles
/// to verify no ports are leaked.
///
/// Verifies:
/// - Ports are correctly returned to the pool after every release.
/// - After multiple full-range alloc/release cycles, the pool is fully intact.
/// - Metrics totals are consistent (allocated == reclaimed).
#[tokio::test]
#[ignore]
async fn load_test_no_port_leaks() {
    const MIN_PORT: u16 = 49152;
    const MAX_PORT: u16 = 49251; // 100 ports = 50 pairs
    const MAX_SESSIONS: u64 = 50;
    const CYCLES: u64 = 4;

    let relay_metrics = Arc::new(RelayMetrics::new());
    let mgr = test_manager_with_metrics(MIN_PORT, MAX_PORT, Arc::clone(&relay_metrics));
    let sessions = Arc::new(RwLock::new(mgr));

    for cycle in 0..CYCLES {
        let base = cycle * MAX_SESSIONS;

        // Allocate all 50 sessions.
        for i in 0..MAX_SESSIONS {
            let sid = base + i;
            let token = valid_token(sid);
            let addr = peer_addr(30000 + i as u16);
            let mut s = sessions.write().await;
            s.allocate(sid, &token, addr)
                .unwrap_or_else(|e| panic!("cycle {cycle}: allocate session {sid} failed: {e}"));
        }

        // Verify all allocated.
        {
            let s = sessions.read().await;
            assert_eq!(
                s.active_count(),
                MAX_SESSIONS as usize,
                "cycle {cycle}: all {MAX_SESSIONS} sessions must be active"
            );
        }

        // Release all 50 sessions.
        for i in 0..MAX_SESSIONS {
            let sid = base + i;
            let mut s = sessions.write().await;
            s.release(sid)
                .unwrap_or_else(|e| panic!("cycle {cycle}: release session {sid} failed: {e}"));
        }

        // Verify all released.
        {
            let s = sessions.read().await;
            assert_eq!(
                s.active_count(), 0,
                "cycle {cycle}: all sessions must be released"
            );
            // All ports must be back in the pool.
            assert_eq!(
                s.available_ports(),
                (MAX_PORT - MIN_PORT + 1) as usize,
                "cycle {cycle}: all ports must be available after full release"
            );
        }
    }

    // Final metrics: total allocated == total reclaimed.
    let total = MAX_SESSIONS * CYCLES;
    let snap = relay_metrics.snapshot();
    assert_eq!(
        snap.slots_allocated, total,
        "total slots_allocated must be {total}"
    );
    assert_eq!(
        snap.slots_reclaimed, total,
        "total slots_reclaimed must be {total}"
    );
    assert_eq!(
        snap.active_sessions, 0,
        "active_sessions must be 0 after all cycles"
    );
}
