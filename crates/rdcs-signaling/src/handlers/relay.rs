// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Relay node allocation handler.
//!
//! When a client sends a `relay_request` (typically because P2P hole-punching
//! failed), this handler:
//!
//! 1. Selects the best relay node — preferring the client's region, then
//!    falling back to the node with the lowest current load.
//! 2. Generates an HMAC-SHA256 token binding the session to the chosen relay.
//! 3. Stores the token in Redis with a short TTL so the relay server can
//!    verify it when the client connects.
//! 4. Sends a `relay_assigned` response with the relay address, port, and
//!    token back to the client.
//! 5. Increments the selected node's session counter.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::sync::mpsc;

use crate::error::AppError;
use crate::redis::keys;
use crate::redis::ttl;
use crate::redis::RedisPool;
use crate::ws::message::WsMessage;

// ---------------------------------------------------------------------------
// RelayNode
// ---------------------------------------------------------------------------

/// In-memory representation of a relay server that can be allocated to clients.
///
/// The `current_sessions` counter is shared (via [`Arc<AtomicU32>`]) so that
/// concurrent allocation requests can safely read and increment it without
/// holding a lock.
#[derive(Clone, Debug)]
pub struct RelayNode {
    /// Public address (hostname or IP) of the relay server.
    pub addr: String,
    /// UDP/TCP port the relay server listens on.
    pub port: u16,
    /// Geographic region identifier (e.g. `"ap-east-1"`).
    pub region: String,
    /// Maximum number of concurrent relay sessions this node supports.
    pub max_sessions: u32,
    /// Current number of active relay sessions (shared atomic counter).
    pub current_sessions: Arc<AtomicU32>,
}

impl RelayNode {
    /// Create a new relay node with zero active sessions.
    pub fn new(addr: String, port: u16, region: String, max_sessions: u32) -> Self {
        Self {
            addr,
            port,
            region,
            max_sessions,
            current_sessions: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Current load as a fraction (0.0 .. 1.0+).
    fn load(&self) -> u32 {
        self.current_sessions.load(Ordering::Relaxed)
    }

    /// Whether this node can accept another session.
    fn has_capacity(&self) -> bool {
        self.load() < self.max_sessions
    }
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// Handle a `relay_request` from a client.
///
/// 1. Select relay node: prefer same region as client, then lowest load.
/// 2. Generate HMAC token: `HMAC-SHA256(session_id || relay_addr || nonce || expires_at)`.
/// 3. Store token in Redis with [`ttl::RELAY_TOKEN_TTL`] (30 s).
/// 4. Send [`WsMessage::RelayAssigned`] response to the client.
/// 5. Increment the node's `current_sessions` counter.
///
/// If no relay nodes are available or all are at capacity, an error message
/// is sent to the client instead.
pub async fn handle_relay_request(
    session_id: &str,
    preferred_region: Option<&str>,
    relay_nodes: &[RelayNode],
    hmac_secret: &[u8],
    redis: Option<&mut RedisPool>,
    tx: &mpsc::Sender<WsMessage>,
) -> Result<(), AppError> {
    // 1. Select the best relay node.
    let node = match select_relay_node(relay_nodes, preferred_region) {
        Some(n) => n,
        None => {
            tracing::warn!(
                session_id = %session_id,
                "no relay nodes available for allocation"
            );
            let _ = tx
                .send(WsMessage::Error {
                    code: "no_relay_available".to_string(),
                    message: "no relay nodes available".to_string(),
                })
                .await;
            return Ok(());
        }
    };

    // 2. Generate HMAC-SHA256 token.
    let nonce = uuid::Uuid::new_v4().to_string();
    let expires_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        + ttl::RELAY_TOKEN_TTL;

    let token_hex = generate_hmac_token(
        hmac_secret,
        session_id,
        &node.addr,
        &nonce,
        expires_at,
    );

    // 3. Store token in Redis (best-effort; skip if no pool).
    if let Some(pool) = redis {
        let key = keys::relay_token_key(&token_hex);
        if let Err(err) =
            ttl::set_with_ttl(pool, &key, session_id, ttl::RELAY_TOKEN_TTL).await
        {
            tracing::warn!(
                %err,
                session_id = %session_id,
                "failed to store relay token in Redis"
            );
        }
    }

    // 4. Send RelayAssigned response to the client.
    tracing::info!(
        session_id = %session_id,
        relay_addr = %node.addr,
        relay_port = node.port,
        region = %node.region,
        "relay node allocated"
    );

    let _ = tx
        .send(WsMessage::RelayAssigned {
            session_id: session_id.to_string(),
            relay_addr: node.addr.clone(),
            relay_port: node.port,
            token: token_hex,
        })
        .await;

    // 5. Increment the node's session counter.
    node.current_sessions.fetch_add(1, Ordering::Relaxed);

    Ok(())
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Select the best relay node for a client.
///
/// Strategy:
/// 1. If `preferred_region` is given, pick the node in that region with the
///    lowest current load (among those with remaining capacity).
/// 2. If no region preference or no nodes in the preferred region, pick the
///    node with the lowest load across **all** regions.
/// 3. Returns `None` if every node is at capacity or the list is empty.
fn select_relay_node<'a>(
    nodes: &'a [RelayNode],
    preferred_region: Option<&str>,
) -> Option<&'a RelayNode> {
    // 1. Try same-region match first.
    if let Some(region) = preferred_region {
        let region_node = nodes
            .iter()
            .filter(|n| n.region == region && n.has_capacity())
            .min_by_key(|n| n.load());
        if region_node.is_some() {
            return region_node;
        }
    }

    // 2. Fall back to lowest load across all available nodes.
    nodes
        .iter()
        .filter(|n| n.has_capacity())
        .min_by_key(|n| n.load())
}

/// Compute `HMAC-SHA256(session_id || relay_addr || nonce || expires_at)`
/// and return the result as a lowercase hex string.
fn generate_hmac_token(
    secret: &[u8],
    session_id: &str,
    relay_addr: &str,
    nonce: &str,
    expires_at: u64,
) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)
        .expect("HMAC can take key of any size");

    mac.update(session_id.as_bytes());
    mac.update(relay_addr.as_bytes());
    mac.update(nonce.as_bytes());
    mac.update(expires_at.to_string().as_bytes());

    let result = mac.finalize();
    bytes_to_hex(&result.into_bytes())
}

/// Encode a byte slice as a lowercase hex string.
fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
    }
    s
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a set of relay nodes for testing.
    fn make_nodes() -> Vec<RelayNode> {
        vec![
            RelayNode::new("relay-ap.example.com".into(), 4443, "ap-east-1".into(), 100),
            RelayNode::new("relay-us.example.com".into(), 4443, "us-west-2".into(), 100),
            RelayNode::new("relay-eu.example.com".into(), 4443, "eu-west-1".into(), 100),
        ]
    }

    // -----------------------------------------------------------------------
    // select_relay_node tests
    // -----------------------------------------------------------------------

    #[test]
    fn select_relay_node_prefers_region() {
        let nodes = make_nodes();
        let selected = select_relay_node(&nodes, Some("us-west-2")).unwrap();
        assert_eq!(selected.region, "us-west-2");
        assert_eq!(selected.addr, "relay-us.example.com");
    }

    #[test]
    fn select_relay_node_prefers_region_ap() {
        let nodes = make_nodes();
        let selected = select_relay_node(&nodes, Some("ap-east-1")).unwrap();
        assert_eq!(selected.region, "ap-east-1");
    }

    #[test]
    fn select_relay_node_falls_back_when_region_not_found() {
        let nodes = make_nodes();
        // Request a region that doesn't exist.
        let selected = select_relay_node(&nodes, Some("mars-1")).unwrap();
        // Should fall back to any available node (lowest load, which is 0 for all).
        assert!(selected.has_capacity());
    }

    #[test]
    fn select_relay_node_no_preference_picks_lowest_load() {
        let nodes = make_nodes();
        // Set different loads.
        nodes[0].current_sessions.store(50, Ordering::Relaxed);
        nodes[1].current_sessions.store(10, Ordering::Relaxed);
        nodes[2].current_sessions.store(30, Ordering::Relaxed);

        let selected = select_relay_node(&nodes, None).unwrap();
        assert_eq!(selected.addr, "relay-us.example.com");
        assert_eq!(selected.load(), 10);
    }

    #[test]
    fn select_relay_node_region_with_load_picks_lowest_in_region() {
        let nodes = vec![
            RelayNode::new("ap1".into(), 4443, "ap-east-1".into(), 100),
            RelayNode::new("ap2".into(), 4443, "ap-east-1".into(), 100),
            RelayNode::new("us1".into(), 4443, "us-west-2".into(), 100),
        ];
        nodes[0].current_sessions.store(80, Ordering::Relaxed);
        nodes[1].current_sessions.store(20, Ordering::Relaxed);

        let selected = select_relay_node(&nodes, Some("ap-east-1")).unwrap();
        assert_eq!(selected.addr, "ap2");
        assert_eq!(selected.load(), 20);
    }

    #[test]
    fn select_relay_node_skips_full_nodes() {
        let nodes = vec![
            RelayNode::new("full".into(), 4443, "ap-east-1".into(), 10),
            RelayNode::new("free".into(), 4443, "ap-east-1".into(), 10),
        ];
        nodes[0].current_sessions.store(10, Ordering::Relaxed); // at capacity
        nodes[1].current_sessions.store(3, Ordering::Relaxed);

        let selected = select_relay_node(&nodes, Some("ap-east-1")).unwrap();
        assert_eq!(selected.addr, "free");
    }

    #[test]
    fn select_relay_node_all_full_returns_none() {
        let nodes = vec![
            RelayNode::new("full1".into(), 4443, "ap-east-1".into(), 5),
            RelayNode::new("full2".into(), 4443, "us-west-2".into(), 5),
        ];
        nodes[0].current_sessions.store(5, Ordering::Relaxed);
        nodes[1].current_sessions.store(5, Ordering::Relaxed);

        assert!(select_relay_node(&nodes, None).is_none());
    }

    #[test]
    fn select_relay_node_empty_list_returns_none() {
        let nodes: Vec<RelayNode> = vec![];
        assert!(select_relay_node(&nodes, None).is_none());
        assert!(select_relay_node(&nodes, Some("ap-east-1")).is_none());
    }

    #[test]
    fn select_relay_node_region_full_falls_back_to_other_region() {
        let nodes = vec![
            RelayNode::new("ap-full".into(), 4443, "ap-east-1".into(), 5),
            RelayNode::new("us-free".into(), 4443, "us-west-2".into(), 100),
        ];
        nodes[0].current_sessions.store(5, Ordering::Relaxed);

        // ap-east-1 is full; should fall back to us-west-2.
        let selected = select_relay_node(&nodes, Some("ap-east-1")).unwrap();
        assert_eq!(selected.addr, "us-free");
    }

    // -----------------------------------------------------------------------
    // HMAC token generation tests
    // -----------------------------------------------------------------------

    #[test]
    fn generate_hmac_token_deterministic() {
        let secret = b"test-secret";
        let token1 = generate_hmac_token(secret, "sess-1", "relay.io", "nonce-1", 1000);
        let token2 = generate_hmac_token(secret, "sess-1", "relay.io", "nonce-1", 1000);
        assert_eq!(token1, token2, "same inputs must produce same HMAC");
    }

    #[test]
    fn generate_hmac_token_different_inputs_differ() {
        let secret = b"test-secret";
        let token1 = generate_hmac_token(secret, "sess-1", "relay.io", "nonce-1", 1000);
        let token2 = generate_hmac_token(secret, "sess-2", "relay.io", "nonce-1", 1000);
        assert_ne!(token1, token2, "different session IDs must produce different tokens");
    }

    #[test]
    fn generate_hmac_token_different_secrets_differ() {
        let token1 = generate_hmac_token(b"secret-a", "sess-1", "relay.io", "nonce-1", 1000);
        let token2 = generate_hmac_token(b"secret-b", "sess-1", "relay.io", "nonce-1", 1000);
        assert_ne!(token1, token2, "different secrets must produce different tokens");
    }

    #[test]
    fn generate_hmac_token_is_64_hex_chars() {
        let token = generate_hmac_token(b"key", "s", "r", "n", 0);
        assert_eq!(token.len(), 64, "SHA-256 HMAC produces 32 bytes = 64 hex chars");
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn bytes_to_hex_correctness() {
        assert_eq!(bytes_to_hex(&[0x00, 0xFF, 0xAB, 0x01]), "00ffab01");
        assert_eq!(bytes_to_hex(&[]), "");
    }

    // -----------------------------------------------------------------------
    // handle_relay_request tests (acceptance criteria)
    // -----------------------------------------------------------------------

    /// AC: relay_request_assigns_node — relay_request returns valid relay address.
    #[tokio::test]
    async fn relay_request_assigns_node() {
        let nodes = make_nodes();
        let (tx, mut rx) = mpsc::channel(8);

        let result = handle_relay_request(
            "sess-001",
            None,
            &nodes,
            b"test-secret",
            None, // no Redis
            &tx,
        )
        .await;
        assert!(result.is_ok());

        let msg = rx.recv().await.expect("should receive RelayAssigned");
        match msg {
            WsMessage::RelayAssigned {
                session_id,
                relay_addr,
                relay_port,
                token,
            } => {
                assert_eq!(session_id, "sess-001");
                assert!(!relay_addr.is_empty(), "relay_addr must not be empty");
                assert!(relay_port > 0, "relay_port must be positive");
                assert!(!token.is_empty(), "token must not be empty");
                assert_eq!(token.len(), 64, "token must be 64 hex chars (SHA-256)");
            }
            other => panic!("expected RelayAssigned, got: {other:?}"),
        }
    }

    /// AC: relay_request_generates_token — HMAC token is generated correctly.
    #[tokio::test]
    async fn relay_request_generates_token() {
        let nodes = make_nodes();
        let (tx, mut rx) = mpsc::channel(8);

        let result = handle_relay_request(
            "sess-tok",
            None,
            &nodes,
            b"hmac-key-123",
            None,
            &tx,
        )
        .await;
        assert!(result.is_ok());

        let msg = rx.recv().await.unwrap();
        match msg {
            WsMessage::RelayAssigned { token, .. } => {
                // Token is a valid 64-char hex string.
                assert_eq!(token.len(), 64);
                assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
            }
            other => panic!("expected RelayAssigned, got: {other:?}"),
        }
    }

    /// AC: relay_request_selects_region — preferred_region is honored.
    #[tokio::test]
    async fn relay_request_selects_region() {
        let nodes = make_nodes();
        let (tx, mut rx) = mpsc::channel(8);

        let result = handle_relay_request(
            "sess-region",
            Some("eu-west-1"),
            &nodes,
            b"secret",
            None,
            &tx,
        )
        .await;
        assert!(result.is_ok());

        let msg = rx.recv().await.unwrap();
        match msg {
            WsMessage::RelayAssigned { relay_addr, .. } => {
                assert_eq!(
                    relay_addr, "relay-eu.example.com",
                    "should select the EU relay node"
                );
            }
            other => panic!("expected RelayAssigned, got: {other:?}"),
        }
    }

    /// AC: relay_request_lowest_load — when no region preference, lowest-load node selected.
    #[tokio::test]
    async fn relay_request_lowest_load() {
        let nodes = make_nodes();
        nodes[0].current_sessions.store(80, Ordering::Relaxed); // ap: high load
        nodes[1].current_sessions.store(5, Ordering::Relaxed);  // us: low load
        nodes[2].current_sessions.store(40, Ordering::Relaxed); // eu: medium load

        let (tx, mut rx) = mpsc::channel(8);

        let result = handle_relay_request(
            "sess-load",
            None, // no region preference
            &nodes,
            b"secret",
            None,
            &tx,
        )
        .await;
        assert!(result.is_ok());

        let msg = rx.recv().await.unwrap();
        match msg {
            WsMessage::RelayAssigned { relay_addr, .. } => {
                assert_eq!(
                    relay_addr, "relay-us.example.com",
                    "should select the US node (lowest load = 5)"
                );
            }
            other => panic!("expected RelayAssigned, got: {other:?}"),
        }
    }

    /// When all nodes are at capacity, an error is returned to the client.
    #[tokio::test]
    async fn relay_request_no_capacity_sends_error() {
        let nodes = vec![
            RelayNode::new("full".into(), 4443, "ap-east-1".into(), 1),
        ];
        nodes[0].current_sessions.store(1, Ordering::Relaxed);

        let (tx, mut rx) = mpsc::channel(8);

        let result = handle_relay_request(
            "sess-full",
            None,
            &nodes,
            b"secret",
            None,
            &tx,
        )
        .await;
        assert!(result.is_ok());

        let msg = rx.recv().await.unwrap();
        match msg {
            WsMessage::Error { code, .. } => {
                assert_eq!(code, "no_relay_available");
            }
            other => panic!("expected Error, got: {other:?}"),
        }
    }

    /// When no relay nodes are configured, an error is returned.
    #[tokio::test]
    async fn relay_request_no_nodes_sends_error() {
        let (tx, mut rx) = mpsc::channel(8);

        let result = handle_relay_request(
            "sess-empty",
            None,
            &[],
            b"secret",
            None,
            &tx,
        )
        .await;
        assert!(result.is_ok());

        let msg = rx.recv().await.unwrap();
        match msg {
            WsMessage::Error { code, .. } => {
                assert_eq!(code, "no_relay_available");
            }
            other => panic!("expected Error, got: {other:?}"),
        }
    }

    /// After allocation, the node's session counter is incremented.
    #[tokio::test]
    async fn relay_request_increments_session_counter() {
        let nodes = make_nodes();
        let initial = nodes[0].load();

        let (tx, mut rx) = mpsc::channel(8);

        // Force selection of nodes[0] by setting region preference.
        handle_relay_request(
            "sess-inc",
            Some("ap-east-1"),
            &nodes,
            b"secret",
            None,
            &tx,
        )
        .await
        .unwrap();

        // Drain the response.
        let _ = rx.recv().await;

        assert_eq!(
            nodes[0].load(),
            initial + 1,
            "session counter should be incremented after allocation"
        );
    }

    /// Token differs across different sessions (nonces are unique).
    #[tokio::test]
    async fn relay_request_tokens_differ_per_session() {
        let nodes = make_nodes();

        let (tx1, mut rx1) = mpsc::channel(8);
        handle_relay_request("sess-a", None, &nodes, b"key", None, &tx1)
            .await
            .unwrap();
        let token_a = match rx1.recv().await.unwrap() {
            WsMessage::RelayAssigned { token, .. } => token,
            other => panic!("expected RelayAssigned, got: {other:?}"),
        };

        let (tx2, mut rx2) = mpsc::channel(8);
        handle_relay_request("sess-b", None, &nodes, b"key", None, &tx2)
            .await
            .unwrap();
        let token_b = match rx2.recv().await.unwrap() {
            WsMessage::RelayAssigned { token, .. } => token,
            other => panic!("expected RelayAssigned, got: {other:?}"),
        };

        assert_ne!(token_a, token_b, "different sessions must get different tokens");
    }

    /// RelayNode::new creates a node with zero sessions.
    #[test]
    fn relay_node_new_has_zero_sessions() {
        let node = RelayNode::new("addr".into(), 4443, "region".into(), 100);
        assert_eq!(node.load(), 0);
        assert!(node.has_capacity());
    }

    /// RelayNode clone shares the atomic counter.
    #[test]
    fn relay_node_clone_shares_counter() {
        let node = RelayNode::new("addr".into(), 4443, "region".into(), 100);
        let cloned = node.clone();

        node.current_sessions.store(42, Ordering::Relaxed);
        assert_eq!(cloned.load(), 42);
    }
}
