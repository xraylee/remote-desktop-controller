// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Horizontal scaling via Redis Pub/Sub for cross-instance messaging.
//!
//! When multiple signaling server instances run behind a load balancer,
//! devices belonging to the same team may be connected to different
//! instances. [`PubSubBridge`] publishes team-scoped signaling messages
//! to Redis Pub/Sub channels so that other instances can deliver them
//! to their locally-connected devices.
//!
//! ## Architecture
//!
//! * **Publishing**: when a signaling message targets a team whose members
//!   span multiple instances, [`PubSubBridge::publish_to_team`] wraps the
//!   [`WsMessage`] in a [`PubSubEnvelope`] (which includes the sender's
//!   unique instance ID) and publishes it to `team:{team_id}:events`.
//!
//! * **Subscribing**: [`PubSubBridge::subscribe_all_teams`] uses Redis
//!   `PSUBSCRIBE` on the `team:*:events` pattern. On each message from a
//!   **different** instance, the bridge delivers it to locally-connected
//!   team members via [`SessionManager::broadcast_to_team`].
//!
//! * **Self-filtering**: messages whose `instance_id` matches the local
//!   instance are silently dropped to avoid duplicate delivery.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::redis::keys;
use crate::redis::RedisPool;
use crate::ws::message::WsMessage;
use crate::ws::session::SessionManager;

// ---------------------------------------------------------------------------
// Pub/Sub message envelope
// ---------------------------------------------------------------------------

/// Envelope wrapping a [`WsMessage`] published over Redis Pub/Sub.
///
/// The `instance_id` allows receiving instances to filter out messages
/// they published themselves, preventing duplicate delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSubEnvelope {
    /// Unique identifier of the publishing instance.
    pub instance_id: String,
    /// Team ID this message is scoped to.
    pub team_id: String,
    /// The signaling message to deliver.
    pub message: WsMessage,
}

// ---------------------------------------------------------------------------
// PubSubBridge
// ---------------------------------------------------------------------------

/// Pub/Sub bridge for cross-instance messaging.
///
/// Each signaling server instance creates one [`PubSubBridge`] at startup.
/// It publishes team-scoped messages and subscribes to all team event
/// channels, delivering cross-instance messages to local sessions.
pub struct PubSubBridge {
    /// Redis connection pool used for `PUBLISH` commands.
    redis: RedisPool,
    /// Separate Redis client used for `PSUBSCRIBE` (requires a dedicated
    /// pub/sub connection that cannot be shared with regular commands).
    redis_client: ::redis::Client,
    /// Shared session manager for local delivery.
    session_mgr: Arc<SessionManager>,
    /// Unique identifier for this instance (UUID v4).
    instance_id: String,
}

impl PubSubBridge {
    /// Create a new Pub/Sub bridge.
    ///
    /// Generates a unique instance ID (UUID v4) to identify this instance
    /// in pub/sub messages.
    pub fn new(
        redis: RedisPool,
        redis_client: ::redis::Client,
        session_mgr: Arc<SessionManager>,
    ) -> Self {
        let instance_id = uuid::Uuid::new_v4().to_string();
        tracing::info!(instance_id = %instance_id, "pub/sub bridge created");
        Self {
            redis,
            redis_client,
            session_mgr,
            instance_id,
        }
    }

    /// Returns the unique instance ID for this bridge.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Publish a signaling message to the team's Pub/Sub channel.
    ///
    /// The message is wrapped in a [`PubSubEnvelope`] that includes the
    /// instance ID so other instances can filter out their own messages.
    pub async fn publish_to_team(
        &self,
        team_id: &str,
        msg: &WsMessage,
    ) -> Result<(), anyhow::Error> {
        let envelope = PubSubEnvelope {
            instance_id: self.instance_id.clone(),
            team_id: team_id.to_string(),
            message: msg.clone(),
        };

        let channel = keys::team_events_channel(team_id);
        let payload = serde_json::to_string(&envelope)?;

        let mut pool = self.redis.clone();
        ::redis::cmd("PUBLISH")
            .arg(&channel)
            .arg(&payload)
            .query_async::<u64>(&mut pool)
            .await
            .map_err(|e| anyhow::anyhow!("failed to publish to {channel}: {e}"))?;

        tracing::debug!(
            channel = %channel,
            instance_id = %self.instance_id,
            "published message to team channel"
        );

        Ok(())
    }

    /// Subscribe to all team event channels (`team:*:events`).
    ///
    /// On receiving a message from another instance:
    /// - If the device is connected to THIS instance, deliver locally.
    /// - Messages from the same instance are silently filtered out.
    pub async fn subscribe_all_teams(&self) -> Result<(), anyhow::Error> {
        use futures_util::StreamExt;

        let mut pubsub = self
            .redis_client
            .get_async_pubsub()
            .await
            .map_err(|e| anyhow::anyhow!("failed to create pubsub connection: {e}"))?;

        let pattern = "team:*:events";
        pubsub
            .psubscribe(pattern)
            .await
            .map_err(|e| anyhow::anyhow!("failed to psubscribe to {pattern}: {e}"))?;

        tracing::info!(
            pattern = pattern,
            instance_id = %self.instance_id,
            "subscribed to team event channels"
        );

        let mut on_message = pubsub.into_on_message();
        while let Some(msg) = on_message.next().await {
            let payload: String = match msg.get_payload() {
                Ok(p) => p,
                Err(err) => {
                    tracing::warn!(%err, "failed to get pubsub payload");
                    continue;
                }
            };

            handle_pubsub_message(&payload, &self.instance_id, &self.session_mgr).await;
        }

        Ok(())
    }

    /// Long-running Pub/Sub loop with automatic reconnection.
    ///
    /// If the subscription stream ends or an error occurs, the bridge
    /// waits 5 seconds and retries.
    pub async fn run(&self) -> Result<(), anyhow::Error> {
        loop {
            match self.subscribe_all_teams().await {
                Ok(()) => {
                    tracing::warn!("pubsub subscription ended, reconnecting in 5s");
                }
                Err(err) => {
                    tracing::error!(%err, "pubsub error, reconnecting in 5s");
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    }
}

// ---------------------------------------------------------------------------
// Message handling (testable without Redis)
// ---------------------------------------------------------------------------

/// Process a single pub/sub message payload.
///
/// Filters out messages from the same instance and delivers cross-instance
/// messages to locally-connected team members via
/// [`SessionManager::broadcast_to_team`].
pub(crate) async fn handle_pubsub_message(
    payload: &str,
    instance_id: &str,
    session_mgr: &SessionManager,
) {
    let envelope: PubSubEnvelope = match serde_json::from_str(payload) {
        Ok(e) => e,
        Err(err) => {
            tracing::warn!(%err, "failed to deserialize pubsub envelope");
            return;
        }
    };

    // Filter out messages from the same instance.
    if envelope.instance_id == instance_id {
        tracing::trace!(
            instance_id = %instance_id,
            "ignoring pubsub message from same instance"
        );
        return;
    }

    tracing::debug!(
        from_instance = %envelope.instance_id,
        team_id = %envelope.team_id,
        "received cross-instance pubsub message, delivering locally"
    );

    // Deliver to all local sessions of the same team.
    session_mgr
        .broadcast_to_team(&envelope.team_id, envelope.message, None)
        .await;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::message::DeviceInfo;
    use crate::ws::session::Session;
    use std::time::Instant;
    use tokio::sync::mpsc;

    /// Helper: create a team session with a receiver for verifying delivery.
    fn make_team_session(
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

    /// Helper: create a session without a team.
    fn make_session(
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
    // PubSubEnvelope serialization tests
    // -----------------------------------------------------------------------

    #[test]
    fn envelope_serializes_with_instance_id() {
        let envelope = PubSubEnvelope {
            instance_id: "inst-001".to_string(),
            team_id: "team-42".to_string(),
            message: WsMessage::Heartbeat {
                device_code: "DEV1".into(),
                ts: 100,
            },
        };
        let json = serde_json::to_string(&envelope).unwrap();
        assert!(json.contains(r#""instance_id":"inst-001""#));
        assert!(json.contains(r#""team_id":"team-42""#));
        assert!(json.contains(r#""type":"heartbeat""#));
    }

    #[test]
    fn envelope_round_trip() {
        let envelope = PubSubEnvelope {
            instance_id: "inst-abc".to_string(),
            team_id: "team-X".to_string(),
            message: WsMessage::PeerOffline {
                device_code: "DEV9".into(),
                reason: "timeout".into(),
            },
        };
        let json = serde_json::to_string(&envelope).unwrap();
        let parsed: PubSubEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.instance_id, "inst-abc");
        assert_eq!(parsed.team_id, "team-X");
        assert_eq!(parsed.message, envelope.message);
    }

    #[test]
    fn envelope_deserialization_failure_on_bad_json() {
        let result = serde_json::from_str::<PubSubEnvelope>("not json");
        assert!(result.is_err());
    }

    // -----------------------------------------------------------------------
    // AC: publish_to_team_channel — message published to Redis Pub/Sub
    // (Requires a running Redis server)
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires a running Redis server at redis://127.0.0.1:6379"]
    async fn publish_to_team_channel() {
        let pool = crate::redis::create_pool("redis://127.0.0.1:6379")
            .await
            .expect("failed to connect to Redis");
        let client = ::redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let session_mgr = Arc::new(SessionManager::new());

        let bridge = PubSubBridge::new(pool, client, session_mgr);

        let msg = WsMessage::NearbyUpdate {
            devices: vec![DeviceInfo {
                code: "DEV1".into(),
                name: "DEV1".into(),
                platform: "linux".into(),
                online: true,
            }],
        };

        let result = bridge.publish_to_team("team-test", &msg).await;
        assert!(result.is_ok(), "publish should succeed: {:?}", result.err());
    }

    // -----------------------------------------------------------------------
    // AC: cross_instance_delivery — message from other instance delivered
    //     to local session.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn cross_instance_delivery() {
        let mgr = SessionManager::new();

        // Register a local team member.
        let (s1, mut rx1) = make_team_session("D1", "team-A");
        let (s2, mut rx2) = make_team_session("D2", "team-A");
        mgr.insert("D1".into(), s1).await;
        mgr.insert("D2".into(), s2).await;

        // Simulate a pubsub message from a DIFFERENT instance.
        let envelope = PubSubEnvelope {
            instance_id: "other-instance".to_string(),
            team_id: "team-A".to_string(),
            message: WsMessage::PeerOffline {
                device_code: "D3".into(),
                reason: "key_expired".into(),
            },
        };
        let payload = serde_json::to_string(&envelope).unwrap();

        handle_pubsub_message(&payload, "local-instance", &mgr).await;

        // Both local team members should receive the message.
        let msg1 = rx1.recv().await.expect("D1 should receive cross-instance message");
        match msg1 {
            WsMessage::PeerOffline { device_code, reason } => {
                assert_eq!(device_code, "D3");
                assert_eq!(reason, "key_expired");
            }
            other => panic!("expected PeerOffline, got: {other:?}"),
        }

        let msg2 = rx2.recv().await.expect("D2 should receive cross-instance message");
        assert!(matches!(msg2, WsMessage::PeerOffline { .. }));
    }

    // -----------------------------------------------------------------------
    // AC: same_instance_ignored — messages from same instance are filtered.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn same_instance_ignored() {
        let mgr = SessionManager::new();

        // Register a local team member.
        let (s1, mut rx1) = make_team_session("D1", "team-A");
        mgr.insert("D1".into(), s1).await;

        // Simulate a pubsub message from the SAME instance.
        let envelope = PubSubEnvelope {
            instance_id: "my-instance".to_string(),
            team_id: "team-A".to_string(),
            message: WsMessage::PeerOffline {
                device_code: "D2".into(),
                reason: "key_expired".into(),
            },
        };
        let payload = serde_json::to_string(&envelope).unwrap();

        handle_pubsub_message(&payload, "my-instance", &mgr).await;

        // D1 should NOT receive the message (same instance filtered out).
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), rx1.recv())
                .await
                .is_err(),
            "messages from same instance should be filtered out"
        );
    }

    // -----------------------------------------------------------------------
    // Edge case: malformed payload is handled gracefully.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn malformed_payload_does_not_panic() {
        let mgr = SessionManager::new();
        handle_pubsub_message("not valid json", "inst-1", &mgr).await;
        // Should not panic.
    }

    // -----------------------------------------------------------------------
    // Edge case: cross-instance message to team with no local members.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn cross_instance_no_local_members() {
        let mgr = SessionManager::new();

        // Register a device on a different team.
        let (s1, mut rx1) = make_team_session("D1", "team-B");
        mgr.insert("D1".into(), s1).await;

        // Cross-instance message for team-A (no local members).
        let envelope = PubSubEnvelope {
            instance_id: "other".to_string(),
            team_id: "team-A".to_string(),
            message: WsMessage::PeerOffline {
                device_code: "D2".into(),
                reason: "timeout".into(),
            },
        };
        let payload = serde_json::to_string(&envelope).unwrap();

        handle_pubsub_message(&payload, "local", &mgr).await;

        // D1 (team-B) should NOT receive the message.
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), rx1.recv())
                .await
                .is_err(),
            "team-B should not receive team-A cross-instance message"
        );
    }

    // -----------------------------------------------------------------------
    // Edge case: cross-instance delivery to device without team.
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn cross_instance_device_without_team_not_affected() {
        let mgr = SessionManager::new();

        // Register a device without a team.
        let (s1, mut rx1) = make_session("LONELY");
        mgr.insert("LONELY".into(), s1).await;

        // Cross-instance message for team-A.
        let envelope = PubSubEnvelope {
            instance_id: "other".to_string(),
            team_id: "team-A".to_string(),
            message: WsMessage::PeerOffline {
                device_code: "X".into(),
                reason: "timeout".into(),
            },
        };
        let payload = serde_json::to_string(&envelope).unwrap();

        handle_pubsub_message(&payload, "local", &mgr).await;

        // LONELY (no team) should NOT receive the message.
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), rx1.recv())
                .await
                .is_err(),
            "device without team should not receive team-scoped message"
        );
    }

    // -----------------------------------------------------------------------
    // Instance ID generation
    // -----------------------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires a running Redis server at redis://127.0.0.1:6379"]
    async fn new_generates_unique_instance_id() {
        let pool = crate::redis::create_pool("redis://127.0.0.1:6379")
            .await
            .unwrap();
        let client = ::redis::Client::open("redis://127.0.0.1:6379").unwrap();
        let mgr = Arc::new(SessionManager::new());

        let bridge1 = PubSubBridge::new(pool.clone(), client.clone(), mgr.clone());
        let bridge2 = PubSubBridge::new(pool, client, mgr);

        assert_ne!(
            bridge1.instance_id(),
            bridge2.instance_id(),
            "each bridge should have a unique instance ID"
        );

        // UUID v4 format check.
        assert_eq!(bridge1.instance_id().len(), 36);
        assert!(bridge1.instance_id().contains('-'));
    }
}
