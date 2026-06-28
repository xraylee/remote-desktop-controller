// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! RDCS Relay — binary entry point.

use std::sync::Arc;

use tracing_subscriber::EnvFilter;

use rdcs_relay::metrics;
use rdcs_relay::{parse_cli, run_udp_loop, wait_for_shutdown};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = parse_cli();

    tracing::info!(
        "rdcs-relay starting: control={}:{}, port_range={}-{}, health_port={}, metrics_port={}",
        config.listen_addr,
        config.control_port,
        config.min_port,
        config.max_port,
        config.health_port,
        config.metrics_port,
    );

    // Shared metrics collector.
    let relay_metrics = Arc::new(metrics::RelayMetrics::new());

    // Maximum concurrent sessions derived from the port range.
    // Each session allocates a pair of ports, so capacity = range / 2.
    let max_capacity =
        (config.max_port as u64).saturating_sub(config.min_port as u64).div_ceil(2);

    // Shared shutdown signal: `true` means "shut down now".
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    let health_handle = tokio::spawn(rdcs_relay::health::run_health_server(
        config.listen_addr.clone(),
        config.health_port,
        max_capacity,
        Arc::clone(&relay_metrics),
        shutdown_rx.clone(),
    ));

    let udp_handle = tokio::spawn(run_udp_loop(
        config.listen_addr.clone(),
        config.control_port,
        config.min_port,
        config.max_port,
        config.hmac_secret,
        relay_metrics,
        shutdown_rx,
    ));

    // Block until a termination signal arrives.
    wait_for_shutdown().await;

    // Signal all tasks to stop, then wait for them.
    let _ = shutdown_tx.send(true);
    let _ = health_handle.await;
    let _ = udp_handle.await;

    tracing::info!("rdcs-relay shut down cleanly");
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use clap::Parser;
    use rdcs_relay::RelayConfig;

    #[test]
    fn cli_defaults() {
        let cfg = RelayConfig::try_parse_from(["rdcs-relay", "--hmac-secret", "test-secret"])
            .expect("parse with defaults should succeed");
        assert_eq!(cfg.listen_addr, "0.0.0.0");
        assert_eq!(cfg.control_port, 3478);
        assert_eq!(cfg.min_port, 49152);
        assert_eq!(cfg.max_port, 65535);
        assert_eq!(cfg.metrics_port, 9090);
        assert_eq!(cfg.health_port, 9091);
        assert_eq!(cfg.hmac_secret, "test-secret");
    }

    #[test]
    fn cli_overrides() {
        let cfg = RelayConfig::try_parse_from([
            "rdcs-relay",
            "--listen-addr",
            "127.0.0.1",
            "--control-port",
            "5000",
            "--min-port",
            "50000",
            "--max-port",
            "51000",
            "--hmac-secret",
            "my-secret",
            "--metrics-port",
            "8080",
            "--health-port",
            "8081",
        ])
        .expect("parse with overrides should succeed");
        assert_eq!(cfg.listen_addr, "127.0.0.1");
        assert_eq!(cfg.control_port, 5000);
        assert_eq!(cfg.min_port, 50000);
        assert_eq!(cfg.max_port, 51000);
        assert_eq!(cfg.hmac_secret, "my-secret");
        assert_eq!(cfg.metrics_port, 8080);
        assert_eq!(cfg.health_port, 8081);
    }

    #[test]
    fn cli_missing_hmac_secret_errors() {
        // Temporarily clear RELAY_HMAC_SECRET so there is no env fallback.
        let prev = std::env::var("RELAY_HMAC_SECRET").ok();
        std::env::remove_var("RELAY_HMAC_SECRET");

        let result = RelayConfig::try_parse_from(["rdcs-relay"]);
        assert!(
            result.is_err(),
            "should fail when --hmac-secret is missing and RELAY_HMAC_SECRET is unset"
        );

        // Restore previous value to avoid affecting other tests.
        if let Some(v) = prev {
            std::env::set_var("RELAY_HMAC_SECRET", v);
        }
    }

    #[test]
    fn cli_port_range_validation() {
        let cfg = RelayConfig::try_parse_from([
            "rdcs-relay",
            "--hmac-secret",
            "s",
            "--min-port",
            "10000",
            "--max-port",
            "20000",
        ])
        .expect("parse with port range should succeed");
        assert!(cfg.min_port < cfg.max_port);
        assert_eq!(cfg.min_port, 10000);
        assert_eq!(cfg.max_port, 20000);
    }
}
