// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Integration test: connection management lifecycle.
//!
//! Exercises the rdcs-connection subsystem end-to-end:
//!   1. ICE offer/answer negotiation (StubIceAgent)
//!   2. Heartbeat liveness detection with TTL expiry
//!   3. mDNS peer discovery and browsing
//!   4. Network path selection with L1 → L2 → L3 fallback
//!   5. Exponential-backoff reconnection strategy
//!   6. Combined discover → connect → monitor → reconnect lifecycle

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;

use rdcs_connection::heartbeat::{HeartbeatConfig, HeartbeatManager, HeartbeatManagerImpl};
use rdcs_connection::ice::{IceAgent, IceCandidate, IceState, SdpAnswer, StubIceAgent};
use rdcs_connection::mdns::{
    DiscoveredPeer, MdnsDiscovery, MdnsEvent, MdnsService, MockMdnsDiscovery,
};
use rdcs_connection::path::{
    PathCandidate, PathSelector, PathType, PriorityPathSelector,
};
use rdcs_connection::reconnect::{ReconnectConfig, ReconnectManager, ReconnectStrategy};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_addr(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), port)
}

fn remote_addr(port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 20)), port)
}

// ---------------------------------------------------------------------------
// 1. ICE offer/answer full flow
// ---------------------------------------------------------------------------

#[test]
fn ice_offer_answer_full_flow() {
    // Create a stub ICE agent pre-loaded with two local host candidates.
    let local_candidates = vec![
        IceCandidate::new_host(test_addr(21115), 1),
        IceCandidate::new_host(test_addr(21116), 1),
    ];
    let mut agent = StubIceAgent::new().with_local_candidates(local_candidates);

    // Initial state must be New before any activity.
    assert_eq!(agent.connection_state(), IceState::New);

    // Step 1: Gather local candidates — transitions to Checking.
    let gathered = agent.gather_candidates().unwrap();
    assert_eq!(gathered.len(), 2, "should gather both pre-loaded candidates");
    assert_eq!(agent.connection_state(), IceState::Checking);

    // Step 2: Create an SDP offer containing the gathered candidates.
    let offer = agent.create_offer().unwrap();
    assert_eq!(offer.candidates.len(), 2);
    assert!(!offer.ufrag.is_empty(), "offer must carry ICE credentials");
    assert!(!offer.pwd.is_empty());
    assert!(!offer.session_id.is_empty());

    // Step 3: Construct and handle an SDP answer from the remote peer.
    let remote_candidates = vec![IceCandidate::new_host(remote_addr(30000), 1)];
    let answer = SdpAnswer {
        session_id: offer.session_id.clone(),
        ufrag: "remote-ufrag".into(),
        pwd: "remote-pwd".into(),
        fingerprint: "test-fingerprint".into(),
        candidates: remote_candidates,
    };
    agent.handle_answer(answer).unwrap();

    // Step 4: After handling the answer, the agent must be Connected.
    assert_eq!(agent.connection_state(), IceState::Connected);
}

// ---------------------------------------------------------------------------
// 2. Heartbeat alive detection and TTL expiry
// ---------------------------------------------------------------------------

#[test]
fn heartbeat_alive_detection_and_ttl_expiry() {
    let config = HeartbeatConfig {
        interval: Duration::from_millis(50),
        ttl: Duration::from_millis(100),
        response_timeout: Duration::from_millis(50),
    };
    let mgr = HeartbeatManagerImpl::new(config);
    let peer: SocketAddr = test_addr(21115);

    // Before start, no peer is tracked — not alive.
    assert!(!mgr.is_alive(), "should not be alive before start");
    assert_eq!(mgr.last_rtt(), None, "no RTT before any pong");

    // Start heartbeat monitoring — implicitly records an initial pong.
    mgr.start(peer).unwrap();
    assert!(mgr.is_alive(), "should be alive immediately after start");
    assert_eq!(mgr.peer(), Some(peer));

    // Simulate a pong response with measured RTT.
    mgr.on_pong(15);
    assert_eq!(mgr.last_rtt(), Some(15));
    assert!(mgr.is_alive(), "should remain alive after fresh pong");

    // Simulate a second pong with improved RTT.
    mgr.on_pong(8);
    assert_eq!(mgr.last_rtt(), Some(8), "RTT should reflect latest pong");

    // Wait for the TTL window to expire (100ms TTL + margin).
    std::thread::sleep(Duration::from_millis(150));
    assert!(
        !mgr.is_alive(),
        "peer should be dead after TTL expires with no new pong"
    );
}

// ---------------------------------------------------------------------------
// 3. mDNS discover and browse peers
// ---------------------------------------------------------------------------

#[test]
fn mdns_discover_and_browse_peers() {
    let discovery = MockMdnsDiscovery::new();

    // Inject two Found events and one Lost event for browsing.
    let svc_alpha = MdnsService {
        name: "alpha-workstation".into(),
        service_type: "_rdcs._tcp.local.".into(),
        port: 21115,
        addr: Some("192.168.1.10".parse().unwrap()),
    };
    let svc_beta = MdnsService {
        name: "beta-laptop".into(),
        service_type: "_rdcs._tcp.local.".into(),
        port: 21116,
        addr: Some("192.168.1.20".parse().unwrap()),
    };
    discovery.inject_events(vec![
        MdnsEvent::Found(svc_alpha.clone()),
        MdnsEvent::Found(svc_beta.clone()),
        MdnsEvent::Lost("alpha-workstation".into()),
    ]);

    // Browse returns a tokio mpsc receiver; the runtime must be active
    // when browse() is called because it spawns a delivery task internally.
    let rt = tokio::runtime::Runtime::new().unwrap();
    let events: Vec<MdnsEvent> = rt.block_on(async {
        let mut rx = discovery.browse().unwrap();
        let mut collected = Vec::new();
        while let Some(event) = rx.recv().await {
            collected.push(event);
        }
        collected
    });

    assert_eq!(events.len(), 3, "should receive all three injected events");

    // Verify first Found event — alpha workstation.
    match &events[0] {
        MdnsEvent::Found(svc) => {
            assert_eq!(svc.name, "alpha-workstation");
            assert_eq!(svc.port, 21115);
            assert_eq!(svc.service_type, "_rdcs._tcp.local.");
            assert_eq!(
                svc.addr,
                Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)))
            );
        }
        other => panic!("expected Found event, got {:?}", other),
    }

    // Verify second Found event — beta laptop.
    match &events[1] {
        MdnsEvent::Found(svc) => {
            assert_eq!(svc.name, "beta-laptop");
            assert_eq!(svc.port, 21116);
        }
        other => panic!("expected Found event, got {:?}", other),
    }

    // Verify Lost event.
    match &events[2] {
        MdnsEvent::Lost(name) => assert_eq!(name, "alpha-workstation"),
        other => panic!("expected Lost event, got {:?}", other),
    }

    // Build DiscoveredPeer from a Found event and verify all fields.
    if let MdnsEvent::Found(svc) = &events[0] {
        let peer = DiscoveredPeer {
            name: svc.name.clone(),
            device_code: "DEV001".into(),
            addresses: vec![SocketAddr::new(svc.addr.unwrap(), svc.port)],
            service_type: svc.service_type.clone(),
        };
        assert_eq!(peer.name, "alpha-workstation");
        assert_eq!(peer.device_code, "DEV001");
        assert_eq!(peer.addresses.len(), 1);
        assert_eq!(
            peer.addresses[0],
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 21115)
        );
        assert_eq!(peer.service_type, "_rdcs._tcp.local.");
    }
}

// ---------------------------------------------------------------------------
// 4. Path selection L1 → L2 → L3 priority and fallback
// ---------------------------------------------------------------------------

#[test]
fn path_selection_l1_l2_l3_priority() {
    let mut selector = PriorityPathSelector::new();

    let l1_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)), 1000);
    let l2_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5)), 2000);
    let l3_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)), 3000);

    let candidates = vec![
        PathCandidate {
            path_type: PathType::L3Relay,
            addr: l3_addr,
            rtt_ms: Some(80.0),
        },
        PathCandidate {
            path_type: PathType::L1Direct,
            addr: l1_addr,
            rtt_ms: Some(1.0),
        },
        PathCandidate {
            path_type: PathType::L2Punch,
            addr: l2_addr,
            rtt_ms: Some(30.0),
        },
    ];

    // Initial selection: L1 direct wins despite being listed second.
    let path = selector.select_path(&candidates).unwrap();
    assert_eq!(path.path_type(), PathType::L1Direct);
    assert_eq!(path.addr(), l1_addr);

    // L1 fails — should fall back to L2.
    let fallback = selector.on_path_failed(path).unwrap();
    assert_eq!(fallback.path_type(), PathType::L2Punch);
    assert_eq!(fallback.addr(), l2_addr);

    // L2 fails — should fall back to L3 relay.
    let fallback = selector.on_path_failed(fallback).unwrap();
    assert_eq!(fallback.path_type(), PathType::L3Relay);
    assert_eq!(fallback.addr(), l3_addr);

    // L3 fails — no further fallback available.
    let fallback = selector.on_path_failed(fallback);
    assert!(
        fallback.is_none(),
        "no fallback should remain after all three tiers fail"
    );
}

// ---------------------------------------------------------------------------
// 5. Reconnect exponential backoff sequence
// ---------------------------------------------------------------------------

#[test]
fn reconnect_exponential_backoff_sequence() {
    let config = ReconnectConfig {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(5),
        backoff_multiplier: 2.0,
        max_attempts: 8,
    };
    let mut mgr = ReconnectManager::new(config);

    // Expected exponential sequence: 100, 200, 400, 800, 1600, 3200, 5000 (capped), 5000.
    let expected_ms = [100u64, 200, 400, 800, 1600, 3200, 5000, 5000];

    for (i, &expected) in expected_ms.iter().enumerate() {
        let delay = mgr
            .next_delay_or_none()
            .unwrap_or_else(|| panic!("attempt {} should not be exhausted", i + 1));
        assert_eq!(
            delay,
            Duration::from_millis(expected),
            "delay mismatch at attempt {}",
            i + 1
        );
    }

    // All 8 attempts consumed — further calls return None.
    assert!(
        mgr.next_delay_or_none().is_none(),
        "should return None after max_attempts exhausted"
    );
    assert!(mgr.exhausted());
    assert_eq!(mgr.attempts_remaining(), 0);

    // Reset restores the initial state.
    mgr.reset();
    assert_eq!(mgr.attempts_remaining(), 8);
    assert_eq!(
        mgr.next_delay_or_none().unwrap(),
        Duration::from_millis(100),
        "first delay after reset should match initial_delay"
    );
}

// ---------------------------------------------------------------------------
// 6. Full connection lifecycle: discover → connect → monitor → reconnect
// ---------------------------------------------------------------------------

#[test]
fn full_connection_lifecycle_discover_connect_reconnect() {
    // === Phase 1: mDNS Discovery ===
    let discovery = MockMdnsDiscovery::new();
    discovery.inject_events(vec![MdnsEvent::Found(MdnsService {
        name: "remote-macbook".into(),
        service_type: "_rdcs._tcp.local.".into(),
        port: 21115,
        addr: Some("192.168.1.50".parse().unwrap()),
    })]);

    let events: Vec<MdnsEvent> = {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut rx = discovery.browse().unwrap();
            let mut collected = Vec::new();
            while let Some(event) = rx.recv().await {
                collected.push(event);
            }
            collected
        })
    };
    assert_eq!(events.len(), 1, "should discover exactly one peer");

    // Extract the discovered service and build a DiscoveredPeer.
    let remote_ip: IpAddr = match &events[0] {
        MdnsEvent::Found(svc) => svc.addr.unwrap(),
        other => panic!("expected Found event, got {:?}", other),
    };
    let remote_socket = SocketAddr::new(remote_ip, 21115);
    let peer = DiscoveredPeer {
        name: "remote-macbook".into(),
        device_code: "MAC001".into(),
        addresses: vec![remote_socket],
        service_type: "_rdcs._tcp.local.".into(),
    };
    assert_eq!(peer.addresses.len(), 1);

    // === Phase 2: ICE Connection ===
    let local_candidate = IceCandidate::new_host(test_addr(21115), 1);
    let mut agent = StubIceAgent::new().with_local_candidates(vec![local_candidate]);

    let gathered = agent.gather_candidates().unwrap();
    assert!(!gathered.is_empty());

    let offer = agent.create_offer().unwrap();
    assert_eq!(offer.candidates.len(), 1);

    let remote_candidate = IceCandidate::new_host(remote_socket, 1);
    let answer = SdpAnswer {
        session_id: offer.session_id.clone(),
        ufrag: "remote-ufrag".into(),
        pwd: "remote-pwd".into(),
        fingerprint: "test-fingerprint".into(),
        candidates: vec![remote_candidate],
    };
    agent.handle_answer(answer).unwrap();
    assert_eq!(
        agent.connection_state(),
        IceState::Connected,
        "ICE should be connected after offer/answer exchange"
    );

    // === Phase 3: Heartbeat Monitoring ===
    let hb_config = HeartbeatConfig {
        interval: Duration::from_millis(20),
        ttl: Duration::from_millis(80),
        response_timeout: Duration::from_millis(20),
    };
    let heartbeat = HeartbeatManagerImpl::new(hb_config);
    heartbeat.start(remote_socket).unwrap();
    assert!(heartbeat.is_alive(), "peer should be alive after heartbeat start");

    heartbeat.on_pong(5);
    assert_eq!(heartbeat.last_rtt(), Some(5));

    // === Phase 4: Simulate Connection Failure ===
    // The remote peer stops responding; TTL expiry signals the failure.
    std::thread::sleep(Duration::from_millis(120));
    assert!(
        !heartbeat.is_alive(),
        "peer should be unreachable after TTL expiry"
    );

    // === Phase 5: Reconnect with Exponential Backoff ===
    let reconnect_config = ReconnectConfig {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(2),
        backoff_multiplier: 2.0,
        max_attempts: 5,
    };
    let mut reconnect = ReconnectManager::new(reconnect_config);

    // Attempt reconnection with backoff — first attempt succeeds.
    let delay = reconnect.next_delay_or_none().unwrap();
    assert_eq!(delay, Duration::from_millis(100));

    // Simulate successful reconnection: reset backoff state.
    reconnect.reset();
    assert_eq!(reconnect.attempts_remaining(), 5);

    // Verify the path selector can still route to the peer.
    let mut selector = PriorityPathSelector::new();
    let candidates = vec![
        PathCandidate {
            path_type: PathType::L1Direct,
            addr: remote_socket,
            rtt_ms: Some(2.0),
        },
        PathCandidate {
            path_type: PathType::L3Relay,
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 1)), 3000),
            rtt_ms: Some(100.0),
        },
    ];
    let path = selector.select_path(&candidates).unwrap();
    assert_eq!(
        path.path_type(),
        PathType::L1Direct,
        "reconnection should prefer direct LAN path"
    );
    assert_eq!(path.addr(), remote_socket);
}
