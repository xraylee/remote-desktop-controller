// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Offline detection via Redis keyspace notifications.
//!
//! When a device's `device:{code}:online` key expires (because heartbeats
//! stopped refreshing the TTL), Redis emits a keyspace notification on the
//! `__keyevent@0__:expired` channel. This module subscribes to that channel
//! and, upon receiving a matching expiration event:
//!
//! 1. Extracts the device code from the expired key.
//! 2. Looks up the device's team membership in the [`SessionManager`].
//! 3. Removes the device from the team's `online_devices` set in Redis.
//! 4. Broadcasts a [`WsMessage::PeerOffline`] to all same-team sessions.
//! 5. Removes the device's session from the in-memory [`SessionManager`].

use crate::redis::keys;
use crate::redis::ttl;
use crate::redis::RedisPool;
use crate::ws::message::WsMessage;
use crate::ws::session::SessionManager;

/// Prefix for device online keys: `device:`.
const DEVICE_ONLINE_PREFIX: &str = "device:";
/// Suffix for device online keys: `:online`.
const DEVICE_ONLINE_SUFFIX: &str = ":online";

/// Extract the device code from a `device:{code}:online` key.
///
/// Returns `None` if the key does not match the expected pattern.
fn extract_device_code(expired_key: &str) -> Option<&str> {
    let rest = expired_key.strip_prefix(DEVICE_ONLINE_PREFIX)?;
    rest.strip_suffix(DEVICE_ONLINE_SUFFIX)
}

// ---------------------------------------------------------------------------
// Core handler (testable without Redis pub/sub)
// ---------------------------------------------------------------------------

/// Process a single expired-key event from Redis keyspace notifications.
///
/// If the expired key matches the `device:{code}:online` pattern:
///
/// 1. Looks up the device's team in the [`SessionManager`].
/// 2. Removes the device from the team's `online_devices` set in Redis.
/// 3. Broadcasts [`WsMessage::PeerOffline`] to same-team sessions.
/// 4. Removes the device from the [`SessionManager`].
pub async fn handle_expired_key(
    expired_key: &str,
    session_mgr: &SessionManager,
    redis: Option<&mut RedisPool>,
) {
    // 1. Only process device online keys.
    let device_code = match extract_device_code(expired_key) {
        Some(code) => code,
        None => return,
    };

    tracing::info!(
        device_code = %device_code,
        expired_key = %expired_key,
        "device online key expired, processing offline detection"
    );

    // 2. Look up the device's team membership.
    let team_id = session_mgr.get_team_id(device_code).await;

    // 3. Remove device from team's online_devices set in Redis.
    if let Some(pool) = redis {
        if let Some(ref tid) = team_id {
            let team_key = keys::team_online_key(tid);
            let _ = ttl::srem(pool, &team_key, device_code).await;
        }
    }

    // 4. Broadcast peer_offline to same-team sessions.
    if let Some(ref tid) = team_id {
        let msg = WsMessage::PeerOffline {
            device_code: device_code.to_string(),
            reason: "key_expired".to_string(),
        };
        session_mgr.broadcast_to_team(tid, msg, None).await;
    }

    // 5. Remove the device from the session manager.
    session_mgr.remove(device_code).await;
}

// ---------------------------------------------------------------------------
// Pub/Sub subscription loop (requires a real Redis connection)
// ---------------------------------------------------------------------------

/// Subscribe to Redis keyspace notifications and process expired-key events.
///
/// This function runs an infinite loop that:
/// 1. Subscribes to `__keyevent@0__:expired`.
/// 2. On each message, calls [`handle_expired_key`] to process the event.
///
/// The function will attempt to reconnect on transient failures.
///
/// # Note
///
/// Redis must have keyspace notifications enabled (`notify-keyspace-events Ex`).
/// This is typically configured in `redis.conf` or via `CONFIG SET`.
pub async fn subscribe_keyspace_notifications(
    client: &::redis::Client,
    mut pool: RedisPool,
    session_mgr: SessionManager,
) {
    use futures_util::StreamExt;

    const CHANNEL: &str = "__keyevent@0__:expired";

    loop {
        tracing::info!("subscribing to Redis keyspace notifications on {CHANNEL}");

        let result = client.get_async_pubsub().await;
        let mut pubsub = match result {
            Ok(ps) => ps,
            Err(err) => {
                tracing::error!(
                    %err,
                    "failed to create Redis pub/sub connection, retrying in 5s"
                );
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        if let Err(err) = pubsub.subscribe(CHANNEL).await {
            tracing::error!(
                %err,
                "failed to subscribe to {CHANNEL}, retrying in 5s"
            );
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            continue;
        }

        tracing::info!("subscribed to {CHANNEL}");

        let mut on_message = pubsub.into_on_message();
        while let Some(msg) = on_message.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(err) => {
                    tracing::warn!(%err, "failed to get keyspace notification payload");
                    continue;
                }
            };

            tracing::debug!(
                channel = %msg.get_channel_name(),
                payload = %payload,
                "received keyspace notification"
            );

            handle_expired_key(&payload, &session_mgr, Some(&mut pool)).await;
        }

        tracing::warn!("keyspace notification stream ended, reconnecting in 5s");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::session::Session;
    use std::time::Instant;
    use tokio::sync::mpsc;

    /// Helper: create a session with a receiver for verifying broadcast messages.
    fn make_team_session_with_rx(
        code: &str,
        team: &str,
    ) -> (Session, mpsc::Receiver<WsMessage>) {
        let (tx, rx) = mpsc::channel(16);
        let session = Session {
            device_code: code.to_string(),
            team_id: Some(team.to_string()),
            tx,
            connected_at: Instant::now(),
        };
        (session, rx)
    }

    fn make_session_with_rx(
        code: &str,
    ) -> (Session, mpsc::Receiver<WsMessage>) {
        let (tx, rx) = mpsc::channel(16);
        let session = Session {
            device_code: code.to_string(),
            team_id: None,
            tx,
            connected_at: Instant::now(),
        };
        (session, rx)
    }

    // -----------------------------------------------------------------------
    // Pattern matching tests
    // -----------------------------------------------------------------------

    #[test]
    fn extract_device_code_valid_key() {
        assert_eq!(extract_device_code("device:ABC123:online"), Some("ABC123"));
    }

    #[test]
    fn extract_device_code_empty_code() {
        assert_eq!(extract_device_code("device::online"), Some(""));
    }

    #[test]
    fn extract_device_code_wrong_prefix() {
        assert_eq!(extract_device_code("session:ABC123:online"), None);
    }

    #[test]
    fn extract_device_code_wrong_suffix() {
        assert_eq!(extract_device_code("device:ABC123:offline"), None);
    }

    #[test]
    fn extract_device_code_unrelated_key() {
        assert_eq!(extract_device_code("device_invite:1234"), None);
    }

    #[test]
    fn extract_device_code_partial_match() {
        assert_eq!(extract_device_code("device:ABC123"), None);
    }

    #[test]
    fn extract_device_code_nested_code() {
        assert_eq!(
            extract_device_code("device:team-42:DEV-001:online"),
            Some("team-42:DEV-001")
        );
    }

    // -----------------------------------------------------------------------
    // expired_key_triggers_offline: when device key expires,
    // peer_offline broadcast sent to team members.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn expired_key_triggers_offline() {
        let mgr = SessionManager::new();

        // Register the device that will go offline.
        let (dev_session, _dev_rx) = make_team_session_with_rx("DEV-001", "team-42");
        mgr.insert("DEV-001".to_string(), dev_session).await;

        // Register a team member who should receive the broadcast.
        let (member_session, mut member_rx) =
            make_team_session_with_rx("MEMBER-1", "team-42");
        mgr.insert("MEMBER-1".to_string(), member_session).await;

        // Simulate key expiration.
        handle_expired_key("device:DEV-001:online", &mgr, None).await;

        // MEMBER-1 should receive a peer_offline broadcast.
        let msg = member_rx
            .recv()
            .await
            .expect("team member should receive peer_offline");
        match msg {
            WsMessage::PeerOffline {
                device_code,
                reason,
            } => {
                assert_eq!(device_code, "DEV-001");
                assert_eq!(reason, "key_expired");
            }
            other => panic!("expected PeerOffline, got: {other:?}"),
        }

        // DEV-001 should be removed from session manager.
        assert!(!mgr.contains("DEV-001").await);
        // MEMBER-1 should still be in session manager.
        assert!(mgr.contains("MEMBER-1").await);
    }

    // -----------------------------------------------------------------------
    // pattern_matching: only device:{code}:online keys trigger offline logic.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn pattern_matching_only_device_online_keys() {
        let mgr = SessionManager::new();

        // Register a device.
        let (dev_session, mut dev_rx) =
            make_team_session_with_rx("DEV-001", "team-42");
        mgr.insert("DEV-001".to_string(), dev_session).await;

        // Simulate expiration of a non-device-online key.
        handle_expired_key("device_invite:1234", &mgr, None).await;
        handle_expired_key("session:sess-001", &mgr, None).await;
        handle_expired_key("device:DEV-001:offline", &mgr, None).await;
        handle_expired_key("random_key", &mgr, None).await;

        // Device should still be in session manager.
        assert!(mgr.contains("DEV-001").await);

        // No messages should have been sent.
        assert!(
            tokio::time::timeout(
                std::time::Duration::from_millis(50),
                dev_rx.recv()
            )
            .await
            .is_err(),
            "no messages should be sent for non-device-online key expirations"
        );
    }

    // -----------------------------------------------------------------------
    // Additional edge case tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn expired_key_unknown_device_no_panic() {
        let mgr = SessionManager::new();

        // Expire a key for a device not in the session manager.
        handle_expired_key("device:UNKNOWN:online", &mgr, None).await;
        // Should not panic; just a no-op.
    }

    #[tokio::test]
    async fn expired_key_device_without_team_no_broadcast() {
        let mgr = SessionManager::new();

        // Register a device without a team.
        let (dev_session, _dev_rx) = make_session_with_rx("DEV-001");
        mgr.insert("DEV-001".to_string(), dev_session).await;

        // Register another device (different team, should not receive).
        let (other_session, mut other_rx) =
            make_team_session_with_rx("OTHER", "team-99");
        mgr.insert("OTHER".to_string(), other_session).await;

        handle_expired_key("device:DEV-001:online", &mgr, None).await;

        // DEV-001 should be removed.
        assert!(!mgr.contains("DEV-001").await);

        // OTHER should NOT receive any broadcast.
        assert!(
            tokio::time::timeout(
                std::time::Duration::from_millis(50),
                other_rx.recv()
            )
            .await
            .is_err(),
            "no broadcast should be sent when device has no team"
        );
    }

    #[tokio::test]
    async fn expired_key_broadcasts_to_all_team_members() {
        let mgr = SessionManager::new();

        // Register the device that will go offline.
        let (dev_session, _dev_rx) = make_team_session_with_rx("DEV-001", "team-42");
        mgr.insert("DEV-001".to_string(), dev_session).await;

        // Register multiple team members.
        let (m1, mut m1_rx) = make_team_session_with_rx("M1", "team-42");
        let (m2, mut m2_rx) = make_team_session_with_rx("M2", "team-42");
        let (m3, mut m3_rx) = make_team_session_with_rx("M3", "team-42");
        mgr.insert("M1".to_string(), m1).await;
        mgr.insert("M2".to_string(), m2).await;
        mgr.insert("M3".to_string(), m3).await;

        handle_expired_key("device:DEV-001:online", &mgr, None).await;

        // All three members should receive peer_offline.
        for rx in [&mut m1_rx, &mut m2_rx, &mut m3_rx] {
            let msg = rx.recv().await.expect("should receive peer_offline");
            match msg {
                WsMessage::PeerOffline { device_code, .. } => {
                    assert_eq!(device_code, "DEV-001");
                }
                other => panic!("expected PeerOffline, got: {other:?}"),
            }
        }
    }

    #[tokio::test]
    async fn expired_key_does_not_broadcast_to_other_teams() {
        let mgr = SessionManager::new();

        // Register the device that will go offline.
        let (dev_session, _dev_rx) = make_team_session_with_rx("DEV-001", "team-A");
        mgr.insert("DEV-001".to_string(), dev_session).await;

        // Register a device in a different team.
        let (other_session, mut other_rx) =
            make_team_session_with_rx("OTHER", "team-B");
        mgr.insert("OTHER".to_string(), other_session).await;

        handle_expired_key("device:DEV-001:online", &mgr, None).await;

        // OTHER (team-B) should NOT receive any broadcast.
        assert!(
            tokio::time::timeout(
                std::time::Duration::from_millis(50),
                other_rx.recv()
            )
            .await
            .is_err(),
            "team-B should not receive broadcast for team-A device"
        );
    }
}
