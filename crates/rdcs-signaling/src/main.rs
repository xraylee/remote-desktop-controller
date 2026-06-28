// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! RDCS Signaling Server binary entry point.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::signal;
use tracing_subscriber::EnvFilter;

use rdcs_signaling::config::AppConfig;
use rdcs_signaling::handlers::relay::RelayNode;
use rdcs_signaling::{handlers, redis, scaling, ws};
use rdcs_signaling::router_with_state;

/// Install the `tracing` subscriber, respecting the `RDCS_LOG_LEVEL` env var.
fn init_tracing(config: &AppConfig) {
    let filter = EnvFilter::try_new(&config.log_level)
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
}

/// Resolve a shutdown signal from either SIGINT (Ctrl-C) or SIGTERM.
///
/// On non-Unix platforms only SIGINT is available.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl-C handler");
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
        _ = ctrl_c => tracing::info!("received SIGINT, shutting down"),
        _ = terminate => tracing::info!("received SIGTERM, shutting down"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env().map_err(|e| anyhow::anyhow!("{e}"))?;
    init_tracing(&config);

    tracing::info!("rdcs-signaling server starting");

    let redis_pool = redis::create_pool(&config.redis_url).await?;
    tracing::info!("connected to redis at {}", config.redis_url);

    // Create a separate Redis client for keyspace notification pub/sub.
    let redis_client = ::redis::Client::open(config.redis_url.as_str())
        .map_err(|e| anyhow::anyhow!("failed to create Redis pub/sub client: {e}"))?;

    let relay_nodes: Vec<RelayNode> = config
        .relay_nodes
        .iter()
        .map(|n| RelayNode::new(n.addr.clone(), n.port, n.region.clone(), n.max_sessions))
        .collect();
    tracing::info!("loaded {} relay node(s)", relay_nodes.len());

    let state = ws::AppState {
        session_manager: ws::SessionManager::new(),
        redis_pool: Some(redis_pool.clone()),
        relay_nodes,
        hmac_secret: config.relay_hmac_secret.as_bytes().to_vec(),
    };

    // Spawn the keyspace notification subscription task for offline detection.
    let session_mgr_for_offline = state.session_manager.clone();
    let pool_for_offline = redis_pool.clone();
    tokio::spawn(async move {
        handlers::offline::subscribe_keyspace_notifications(
            &redis_client,
            pool_for_offline,
            session_mgr_for_offline,
        )
        .await;
    });

    // Spawn the pub/sub bridge for cross-instance messaging.
    let redis_scaling_client = ::redis::Client::open(config.redis_url.as_str())
        .map_err(|e| anyhow::anyhow!("failed to create Redis scaling client: {e}"))?;
    let bridge = scaling::PubSubBridge::new(
        redis_pool.clone(),
        redis_scaling_client,
        Arc::new(state.session_manager.clone()),
    );
    tokio::spawn(async move {
        if let Err(err) = bridge.run().await {
            tracing::error!(%err, "pub/sub bridge stopped");
        }
    });

    let app = router_with_state(state);
    let addr: SocketAddr = config
        .bind_addr
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid bind address `{}`: {}", config.bind_addr, e))?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("signaling server listening on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("signaling server stopped");
    Ok(())
}
