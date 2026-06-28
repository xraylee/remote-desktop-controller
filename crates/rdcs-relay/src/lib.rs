// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! RDCS Relay Server — library crate.
//!
//! UDP relay node for forwarding screen-capture traffic
//! when direct P2P connections are unavailable.
//!
//! Provides:
//! - UDP control socket for relay signaling
//! - HTTP health endpoint (GET /health -> "ok")
//! - Graceful shutdown on SIGTERM / SIGINT

pub mod protocol;

pub mod auth;

pub mod session;

pub mod forwarder;

pub mod metrics;

pub mod health;

use std::sync::Arc;

use clap::Parser;
use tokio::net::UdpSocket;
use tokio::signal;
use tokio::sync::RwLock;

/// RDCS Relay — UDP relay node for forwarding screen-capture traffic.
#[derive(Parser, Debug, Clone)]
#[command(name = "rdcs-relay", version, about)]
pub struct RelayConfig {
    /// Address to bind all listeners on.
    #[arg(long, default_value = "0.0.0.0", env = "RELAY_LISTEN_ADDR")]
    pub listen_addr: String,

    /// UDP control-port for relay signaling (STUN-like).
    #[arg(long, default_value_t = 3478, env = "RELAY_CONTROL_PORT")]
    pub control_port: u16,

    /// Minimum port in the relay allocation range.
    #[arg(long, default_value_t = 49152, env = "RELAY_MIN_PORT")]
    pub min_port: u16,

    /// Maximum port in the relay allocation range.
    #[arg(long, default_value_t = 65535, env = "RELAY_MAX_PORT")]
    pub max_port: u16,

    /// HMAC-SHA256 secret used to authenticate Allocate requests.
    #[arg(long, env = "RELAY_HMAC_SECRET")]
    pub hmac_secret: String,

    /// Port for the Prometheus metrics HTTP endpoint.
    #[arg(long, default_value_t = 9090, env = "RELAY_METRICS_PORT")]
    pub metrics_port: u16,

    /// Port for the HTTP health-check endpoint (GET /health).
    #[arg(long, default_value_t = 9091, env = "RELAY_HEALTH_PORT")]
    pub health_port: u16,
}

/// Parse CLI arguments (and env-var fallbacks) into a [`RelayConfig`].
pub fn parse_cli() -> RelayConfig {
    RelayConfig::parse()
}

// ---------------------------------------------------------------------------
// UDP recv loop
// ---------------------------------------------------------------------------

/// Bind the control UDP socket and run the receive loop until `shutdown`
/// resolves.
///
/// The loop owns a [`session::SessionManager`] that tracks active relay
/// sessions and allocated port pairs.  A periodic cleanup tick removes
/// sessions with no activity for 30 seconds.
pub async fn run_udp_loop(
    addr: String,
    port: u16,
    min_port: u16,
    max_port: u16,
    hmac_secret: String,
    relay_metrics: Arc<metrics::RelayMetrics>,
    mut shutdown: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let socket = Arc::new(UdpSocket::bind(format!("{addr}:{port}")).await?);
    tracing::info!("relay control socket listening on {addr}:{port}");

    let mgr = session::SessionManager::new(min_port, max_port, hmac_secret.into_bytes())
        .with_metrics(relay_metrics);
    let sessions = Arc::new(RwLock::new(mgr));

    // Data forwarder — shares the same socket and session manager.
    let fwd = forwarder::DataForwarder::new(socket.clone(), sessions.clone());

    // Cleanup expired sessions every 10 seconds.
    let mut cleanup_interval = tokio::time::interval(std::time::Duration::from_secs(10));
    // The first tick fires immediately; skip it.
    cleanup_interval.tick().await;

    let mut buf = vec![0u8; 65536];
    let sock: &UdpSocket = &socket;
    loop {
        tokio::select! {
            result = sock.recv_from(&mut buf) => {
                let (len, peer) = result?;
                tracing::debug!("received {} bytes from {}", len, peer);
                match protocol::parse_message(&buf[..len]) {
                    Ok((msg, _rest)) => {
                        match msg {
                            protocol::RelayMessage::Allocate { session_id, token } => {
                                let token_str = String::from_utf8_lossy(&token);
                                let mut s = sessions.write().await;
                                match s.allocate(session_id, &token_str, peer) {
                                    Ok((port_a, port_b)) => {
                                        tracing::info!(
                                            session_id,
                                            peer = %peer,
                                            port_a,
                                            port_b,
                                            "allocated relay ports"
                                        );
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            session_id,
                                            peer = %peer,
                                            error = %e,
                                            "allocate failed"
                                        );
                                    }
                                }
                            }
                            protocol::RelayMessage::Release { session_id } => {
                                let mut s = sessions.write().await;
                                match s.release(session_id) {
                                    Ok(()) => {
                                        tracing::info!(
                                            session_id,
                                            "released relay session"
                                        );
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            session_id,
                                            error = %e,
                                            "release failed"
                                        );
                                    }
                                }
                            }
                            protocol::RelayMessage::Keepalive { session_id } => {
                                let mut s = sessions.write().await;
                                if s.keepalive(session_id) {
                                    tracing::debug!(
                                        session_id,
                                        "keepalive OK"
                                    );
                                } else {
                                    tracing::warn!(
                                        session_id,
                                        "keepalive for unknown session"
                                    );
                                }
                            }
                            protocol::RelayMessage::Data { session_id } => {
                                if let Err(e) = fwd.forward_packet(peer, &buf[..len]).await {
                                    tracing::warn!(
                                        session_id,
                                        peer = %peer,
                                        error = %e,
                                        "data forwarding failed"
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(peer = %peer, error = %e, "failed to parse relay packet");
                    }
                }
            }
            _ = cleanup_interval.tick() => {
                let mut s = sessions.write().await;
                let expired = s.cleanup_expired();
                if !expired.is_empty() {
                    tracing::info!(
                        count = expired.len(),
                        sessions = ?expired,
                        "cleaned up expired sessions"
                    );
                }
            }
            _ = shutdown.changed() => {
                tracing::info!("UDP recv loop shutting down");
                break;
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Shutdown signal
// ---------------------------------------------------------------------------

/// Resolves when SIGTERM or SIGINT is received.
pub async fn wait_for_shutdown() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => tracing::info!("received SIGINT, initiating shutdown"),
        _ = terminate => tracing::info!("received SIGTERM, initiating shutdown"),
    }
}
