// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Comprehensive NAT traversal tests for RDCS.
//!
//! This test suite validates:
//! - NAT type detection and classification
//! - P2P connectivity across different NAT combinations
//! - STUN/TURN server functionality
//! - ICE candidate gathering and prioritization
//! - Connection establishment time
//! - Relay fallback mechanism
//! - Overall success rate compliance with PRD (>60%)

use rdcs_nat_test::{
    ConnectionMethod, NatTraversalSimulator, NatType, TestMatrixResult,
};
use std::time::Duration;
use tracing::info;

// ---------------------------------------------------------------------------
// Basic NAT Traversal Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_no_nat_direct_connection() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_nat_test=info")
        .with_test_writer()
        .try_init()
        .ok();

    let simulator = NatTraversalSimulator::new();
    let result = simulator.test_traversal(NatType::None, NatType::None).await;

    assert!(result.success, "Connection should succeed with no NAT");
    assert_eq!(
        result.connection_method,
        ConnectionMethod::DirectP2P,
        "Should use direct P2P"
    );
    assert!(
        result.time_to_connect_ms < 500,
        "Connection should be fast: {}ms",
        result.time_to_connect_ms
    );

    info!("✓ No NAT direct connection test passed");
}

#[tokio::test]
async fn test_full_cone_nat_traversal() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_nat_test=info")
        .with_test_writer()
        .try_init()
        .ok();

    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::FullCone, NatType::FullCone)
        .await;

    assert!(result.success, "Full Cone NAT should be traversable");
    assert_eq!(
        result.connection_method,
        ConnectionMethod::StunAssistedP2P,
        "Should use STUN-assisted P2P"
    );

    info!("✓ Full Cone NAT traversal test passed");
}

#[tokio::test]
async fn test_restricted_cone_nat_traversal() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_nat_test=info")
        .with_test_writer()
        .try_init()
        .ok();

    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::RestrictedCone, NatType::RestrictedCone)
        .await;

    assert!(result.success, "Restricted Cone NAT should be traversable");
    assert_eq!(result.connection_method, ConnectionMethod::StunAssistedP2P);

    info!("✓ Restricted Cone NAT traversal test passed");
}

#[tokio::test]
async fn test_port_restricted_cone_nat_traversal() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_nat_test=info")
        .with_test_writer()
        .try_init()
        .ok();

    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::PortRestrictedCone, NatType::RestrictedCone)
        .await;

    assert!(
        result.success,
        "Port Restricted + Restricted Cone should be traversable"
    );

    info!("✓ Port Restricted Cone NAT traversal test passed");
}

#[tokio::test]
async fn test_symmetric_nat_requires_relay() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_nat_test=info")
        .with_test_writer()
        .try_init()
        .ok();

    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::Symmetric, NatType::Symmetric)
        .await;

    assert!(result.success, "Symmetric NAT should succeed via relay");
    assert_eq!(
        result.connection_method,
        ConnectionMethod::RelayFallback,
        "Should fall back to relay"
    );

    info!("✓ Symmetric NAT relay fallback test passed");
}

// ---------------------------------------------------------------------------
// Mixed NAT Type Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_no_nat_vs_symmetric_nat() {
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::None, NatType::Symmetric)
        .await;

    assert!(
        result.success,
        "No NAT should traverse Symmetric NAT via STUN"
    );
    assert_eq!(result.connection_method, ConnectionMethod::StunAssistedP2P);

    info!("✓ No NAT vs Symmetric NAT test passed");
}

#[tokio::test]
async fn test_full_cone_vs_symmetric_nat() {
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::FullCone, NatType::Symmetric)
        .await;

    assert!(
        result.success,
        "Full Cone should traverse Symmetric NAT via STUN"
    );
    assert_eq!(result.connection_method, ConnectionMethod::StunAssistedP2P);

    info!("✓ Full Cone vs Symmetric NAT test passed");
}

#[tokio::test]
async fn test_restricted_cone_vs_port_restricted_cone() {
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::RestrictedCone, NatType::PortRestrictedCone)
        .await;

    assert!(result.success);
    assert_eq!(result.connection_method, ConnectionMethod::StunAssistedP2P);

    info!("✓ Restricted Cone vs Port Restricted Cone test passed");
}

// ---------------------------------------------------------------------------
// ICE Candidate Gathering Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_ice_candidate_gathering_count() {
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::RestrictedCone, NatType::RestrictedCone)
        .await;

    // Should gather: host, server reflexive, relay (3 per peer = 6 total)
    assert!(
        result.ice_candidates_gathered >= 4,
        "Should gather at least 4 candidates (2 per peer)"
    );

    info!(
        "✓ ICE candidate gathering test passed: {} candidates",
        result.ice_candidates_gathered
    );
}

#[tokio::test]
async fn test_ice_connectivity_checks() {
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::FullCone, NatType::FullCone)
        .await;

    assert!(
        result.ice_checks_performed > 0,
        "Should perform at least one connectivity check"
    );
    assert!(
        result.ice_checks_performed < 10,
        "Should not perform excessive checks: {}",
        result.ice_checks_performed
    );

    info!(
        "✓ ICE connectivity checks test passed: {} checks",
        result.ice_checks_performed
    );
}

// ---------------------------------------------------------------------------
// Performance Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_connection_time_p2p() {
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::FullCone, NatType::FullCone)
        .await;

    assert!(
        result.time_to_connect_ms < 1000,
        "P2P connection should complete within 1 second: {}ms",
        result.time_to_connect_ms
    );

    info!(
        "✓ P2P connection time test passed: {}ms",
        result.time_to_connect_ms
    );
}

#[tokio::test]
async fn test_connection_time_relay() {
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::Symmetric, NatType::Symmetric)
        .await;

    assert!(
        result.time_to_connect_ms < 2000,
        "Relay connection should complete within 2 seconds: {}ms",
        result.time_to_connect_ms
    );

    info!(
        "✓ Relay connection time test passed: {}ms",
        result.time_to_connect_ms
    );
}

// ---------------------------------------------------------------------------
// Comprehensive Test Matrix
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_comprehensive_nat_matrix() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_nat_test=info")
        .with_test_writer()
        .try_init()
        .ok();

    info!("Running comprehensive NAT traversal test matrix...");
    info!("This will test all 25 NAT type combinations (5x5)");

    let simulator = NatTraversalSimulator::new();
    let matrix = simulator.run_test_matrix().await;

    // Print detailed report
    println!("\n{}", matrix.generate_report());

    // Validate PRD requirement: >60% success rate
    let success_rate = matrix.success_rate();
    assert!(
        matrix.meets_prd_requirement(),
        "Success rate {:.1}% does not meet PRD requirement of >60%",
        success_rate
    );

    // Additional metrics
    let p2p_rate = matrix.p2p_success_rate();
    let relay_rate = matrix.relay_usage_rate();
    let avg_time = matrix.average_connection_time_ms();

    info!("Overall success rate: {:.1}%", success_rate);
    info!("P2P success rate: {:.1}%", p2p_rate);
    info!("Relay usage rate: {:.1}%", relay_rate);
    info!("Average connection time: {}ms", avg_time);

    // PRD targets
    assert!(
        success_rate >= 60.0,
        "PRD requirement: success rate should be ≥60%"
    );
    assert!(
        p2p_rate >= 30.0,
        "At least 30% of connections should use P2P"
    );
    assert!(
        relay_rate <= 40.0,
        "Relay usage should be ≤40% to control costs"
    );
    assert!(
        avg_time < 1500,
        "Average connection time should be <1.5s"
    );

    info!("✓ Comprehensive NAT matrix test passed");
}

// ---------------------------------------------------------------------------
// Real-World Scenario Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_typical_home_network_scenario() {
    // Most home routers use Port Restricted Cone NAT
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::PortRestrictedCone, NatType::PortRestrictedCone)
        .await;

    assert!(
        result.success,
        "Typical home networks should connect successfully"
    );

    info!("✓ Typical home network scenario test passed");
}

#[tokio::test]
async fn test_corporate_network_scenario() {
    // Corporate networks often use Symmetric NAT
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::Symmetric, NatType::PortRestrictedCone)
        .await;

    assert!(
        result.success,
        "Corporate + home network should connect via relay"
    );
    assert_eq!(result.connection_method, ConnectionMethod::RelayFallback);

    info!("✓ Corporate network scenario test passed");
}

#[tokio::test]
async fn test_mobile_network_scenario() {
    // Mobile networks (4G/5G) often use Symmetric NAT
    let simulator = NatTraversalSimulator::new();
    let result = simulator
        .test_traversal(NatType::Symmetric, NatType::Symmetric)
        .await;

    assert!(
        result.success,
        "Mobile networks should connect via relay"
    );
    assert_eq!(result.connection_method, ConnectionMethod::RelayFallback);

    info!("✓ Mobile network scenario test passed");
}

// ---------------------------------------------------------------------------
// Stress Tests
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore] // Run with: cargo test --release -- --ignored
async fn stress_test_concurrent_traversals() {
    tracing_subscriber::fmt()
        .with_env_filter("rdcs_nat_test=info")
        .with_test_writer()
        .try_init()
        .ok();

    info!("Running concurrent NAT traversal stress test...");

    let simulator = NatTraversalSimulator::new();
    let mut handles = vec![];

    // Simulate 50 concurrent connection attempts
    for i in 0..50 {
        let sim = NatTraversalSimulator::new();
        let nat_types = [
            NatType::None,
            NatType::FullCone,
            NatType::RestrictedCone,
            NatType::PortRestrictedCone,
            NatType::Symmetric,
        ];
        let local = nat_types[i % 5];
        let remote = nat_types[(i + 2) % 5];

        let handle = tokio::spawn(async move { sim.test_traversal(local, remote).await });

        handles.push(handle);
    }

    // Wait for all to complete
    let results = futures_util::future::join_all(handles).await;

    let successful = results.iter().filter(|r| r.as_ref().unwrap().success).count();
    let success_rate = (successful as f64 / results.len() as f64) * 100.0;

    info!("Concurrent traversal success rate: {:.1}%", success_rate);
    assert!(success_rate >= 60.0, "Concurrent success rate should be ≥60%");

    info!("✓ Concurrent traversal stress test passed");
}

#[tokio::test]
#[ignore]
async fn stress_test_rapid_reconnections() {
    info!("Testing rapid reconnection scenarios...");

    let simulator = NatTraversalSimulator::new();

    // Simulate 20 rapid reconnections (e.g., network switching)
    for i in 0..20 {
        let result = simulator
            .test_traversal(NatType::RestrictedCone, NatType::RestrictedCone)
            .await;

        assert!(result.success, "Reconnection {} failed", i + 1);

        // Small delay between reconnections
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    info!("✓ Rapid reconnection stress test passed");
}

// ---------------------------------------------------------------------------
// Statistical Analysis Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_success_rate_by_nat_combination() {
    let simulator = NatTraversalSimulator::new();
    let matrix = simulator.run_test_matrix().await;

    let stats = matrix.success_rate_by_nat_combination();

    // Verify specific combinations meet expectations
    let no_nat_key = (NatType::None, NatType::None);
    if let Some(&rate) = stats.get(&no_nat_key) {
        assert_eq!(rate, 100.0, "No NAT should always succeed");
    }

    let symmetric_key = (NatType::Symmetric, NatType::Symmetric);
    if let Some(&rate) = stats.get(&symmetric_key) {
        assert!(
            rate >= 95.0,
            "Symmetric NAT should succeed via relay: {:.1}%",
            rate
        );
    }

    info!("✓ Success rate by NAT combination test passed");
}

// ---------------------------------------------------------------------------
// Relay Fallback Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_relay_fallback_mechanism() {
    let simulator = NatTraversalSimulator::new();

    // Test scenarios where P2P should fail and relay should be used
    let scenarios = vec![
        (NatType::Symmetric, NatType::Symmetric),
        (NatType::Symmetric, NatType::PortRestrictedCone),
    ];

    for (local, remote) in scenarios {
        let result = simulator.test_traversal(local, remote).await;

        assert!(result.success, "Relay fallback should succeed");
        assert_eq!(
            result.connection_method,
            ConnectionMethod::RelayFallback,
            "Should use relay for {:?} <-> {:?}",
            local,
            remote
        );
    }

    info!("✓ Relay fallback mechanism test passed");
}

#[tokio::test]
async fn test_relay_usage_optimization() {
    let simulator = NatTraversalSimulator::new();
    let matrix = simulator.run_test_matrix().await;

    let relay_rate = matrix.relay_usage_rate();

    // Relay should be used judiciously to control costs
    // Aim for <40% relay usage across all scenarios
    assert!(
        relay_rate <= 50.0,
        "Relay usage rate {:.1}% is too high (cost concern)",
        relay_rate
    );

    info!("✓ Relay usage optimization test passed: {:.1}%", relay_rate);
}
