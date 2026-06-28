// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Heartbeat handler.
//!
//! When a client sends a `Heartbeat` message, this handler:
//!
//! 1. Refreshes the TTL on the device's `device:{code}:online` Redis key.
//! 2. If the key does not exist (device went offline or was evicted),
//!    sends a `peer_offline` notification to the requester.

use tokio::sync::mpsc;

use crate::redis::keys;
use crate::redis::ttl;
use crate::redis::RedisPool;
use crate::ws::message::WsMessage;

/// Handle a heartbeat message from a client.
///
/// Refreshes the device's online TTL in Redis. If the key has already
/// expired (device is considered offline), a `peer_offline` notification
/// is sent back to the requester via `tx`.
pub async fn handle_heartbeat(
    device_code: &str,
    redis: Option<&mut RedisPool>,
    tx: &mpsc::Sender<WsMessage>,
) {
    if let Some(pool) = redis {
        let key = keys::device_online_key(device_code);
        match ttl::refresh_ttl(pool, &key, ttl::DEVICE_ONLINE_TTL).await {
            Ok(exists) if !exists => {
                // Key not found — device has been evicted or never registered.
                tracing::debug!(
                    device_code,
                    "heartbeat for unknown/expired device, notifying requester"
                );
                let _ = tx
                    .send(WsMessage::PeerOffline {
                        device_code: device_code.to_string(),
                        reason: "heartbeat_timeout".to_string(),
                    })
                    .await;
            }
            Ok(_) => {
                tracing::trace!(device_code, "heartbeat TTL refreshed");
            }
            Err(err) => {
                tracing::warn!(%err, device_code, "failed to refresh heartbeat TTL");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn heartbeat_without_redis_does_not_panic() {
        let (tx, _rx) = mpsc::channel(8);
        handle_heartbeat("DEV1", None, &tx).await;
    }

    #[tokio::test]
    async fn heartbeat_with_closed_channel_does_not_panic() {
        let (tx, rx) = mpsc::channel(8);
        drop(rx); // Drop receiver to close the channel.
        handle_heartbeat("DEV1", None, &tx).await;
    }

    #[tokio::test]
    async fn heartbeat_without_redis_does_not_send_peer_offline() {
        let (tx, mut rx) = mpsc::channel(8);

        // Without Redis, the heartbeat handler does nothing (no peer_offline sent).
        handle_heartbeat("DEV1", None, &tx).await;

        // Verify no message was sent.
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), rx.recv())
                .await
                .is_err(),
            "no message should be sent when Redis is unavailable"
        );
    }
}
