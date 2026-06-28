// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Disconnect cleanup handler.
//!
//! When a WebSocket connection closes, this handler:
//!
//! 1. Deletes the device's `device:{code}:online` Redis key.
//! 2. Removes the device from the team's `online_devices` set (if applicable).
//! 3. Broadcasts a `nearby_update` to same-team devices indicating the device
//!    is now offline.

use crate::handlers::register::ConnectionInfo;
use crate::redis::keys;
use crate::redis::ttl;
use crate::redis::RedisPool;
use crate::ws::message::{DeviceInfo, WsMessage};
use crate::ws::session::SessionManager;

/// Clean up after a device disconnects.
///
/// Removes Redis online state and team membership, then broadcasts
/// an offline `nearby_update` to same-team devices.
pub async fn handle_disconnect(
    info: &ConnectionInfo,
    redis: Option<&mut RedisPool>,
    session_mgr: &SessionManager,
) {
    let device_code = &info.device_code;
    let team_id = info.team_id.as_deref();

    // 1. Remove device from Redis.
    if let Some(pool) = redis {
        let key = keys::device_online_key(device_code);
        let _ = ttl::del_key(pool, &key).await;

        // 2. Remove from team's online_devices set.
        if let Some(tid) = team_id {
            let team_key = keys::team_online_key(tid);
            let _ = ttl::srem(pool, &team_key, device_code).await;
        }
    }

    // 3. Remove from session manager.
    session_mgr.remove(device_code).await;

    // 4. Broadcast nearby_update to same-team devices (device now offline).
    if let Some(tid) = team_id {
        let update = WsMessage::NearbyUpdate {
            devices: vec![DeviceInfo {
                code: device_code.to_string(),
                name: device_code.to_string(),
                platform: String::new(),
                online: false,
            }],
        };
        session_mgr.broadcast_to_team(tid, update, None).await;
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

    #[tokio::test]
    async fn disconnect_removes_session_from_manager() {
        let mgr = SessionManager::new();

        // Pre-register a session.
        let (tx, _rx) = mpsc::channel(8);
        let session = Session {
            device_code: "DEV-001".to_string(),
            team_id: Some("team-42".to_string()),
            tx,
            connected_at: Instant::now(),
        };
        mgr.insert("DEV-001".to_string(), session).await;
        assert!(mgr.contains("DEV-001").await);

        let info = ConnectionInfo {
            device_code: "DEV-001".to_string(),
            team_id: Some("team-42".to_string()),
        };

        handle_disconnect(&info, None, &mgr).await;

        assert!(!mgr.contains("DEV-001").await);
    }

    #[tokio::test]
    async fn disconnect_broadcasts_offline_to_team() {
        let mgr = SessionManager::new();

        // Another team member who should receive the broadcast.
        let (member_tx, mut member_rx) = mpsc::channel(8);
        let member = Session {
            device_code: "MEMBER-1".to_string(),
            team_id: Some("team-42".to_string()),
            tx: member_tx,
            connected_at: Instant::now(),
        };
        mgr.insert("MEMBER-1".to_string(), member).await;

        let info = ConnectionInfo {
            device_code: "DEV-001".to_string(),
            team_id: Some("team-42".to_string()),
        };

        handle_disconnect(&info, None, &mgr).await;

        // MEMBER-1 should receive a nearby_update with the disconnected device.
        let msg = member_rx.recv().await.unwrap();
        match msg {
            WsMessage::NearbyUpdate { devices } => {
                assert_eq!(devices.len(), 1);
                assert_eq!(devices[0].code, "DEV-001");
                assert!(!devices[0].online);
            }
            other => panic!("expected NearbyUpdate, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn disconnect_without_team_does_not_broadcast() {
        let mgr = SessionManager::new();

        let (other_tx, mut other_rx) = mpsc::channel(8);
        let other = Session {
            device_code: "OTHER".to_string(),
            team_id: Some("team-42".to_string()),
            tx: other_tx,
            connected_at: Instant::now(),
        };
        mgr.insert("OTHER".to_string(), other).await;

        let info = ConnectionInfo {
            device_code: "LONELY-DEV".to_string(),
            team_id: None, // No team.
        };

        handle_disconnect(&info, None, &mgr).await;

        // OTHER should NOT receive any broadcast.
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), other_rx.recv())
                .await
                .is_err(),
            "no broadcast should be sent when team_id is None"
        );
    }

    #[tokio::test]
    async fn disconnect_without_redis_still_removes_session() {
        let mgr = SessionManager::new();

        let (tx, _rx) = mpsc::channel(8);
        let session = Session {
            device_code: "DEV-001".to_string(),
            team_id: None,
            tx,
            connected_at: Instant::now(),
        };
        mgr.insert("DEV-001".to_string(), session).await;

        let info = ConnectionInfo {
            device_code: "DEV-001".to_string(),
            team_id: None,
        };

        // No Redis pool.
        handle_disconnect(&info, None, &mgr).await;

        // Session should still be removed.
        assert!(!mgr.contains("DEV-001").await);
    }

    #[tokio::test]
    async fn disconnect_broadcasts_to_remaining_members_only() {
        let mgr = SessionManager::new();

        // Member A (will be disconnected).
        let (a_tx, _a_rx) = mpsc::channel(8);
        let member_a = Session {
            device_code: "MEMBER-A".to_string(),
            team_id: Some("team-42".to_string()),
            tx: a_tx,
            connected_at: Instant::now(),
        };
        mgr.insert("MEMBER-A".to_string(), member_a).await;

        // Member B (stays online, should receive broadcast).
        let (b_tx, mut b_rx) = mpsc::channel(8);
        let member_b = Session {
            device_code: "MEMBER-B".to_string(),
            team_id: Some("team-42".to_string()),
            tx: b_tx,
            connected_at: Instant::now(),
        };
        mgr.insert("MEMBER-B".to_string(), member_b).await;

        let info = ConnectionInfo {
            device_code: "MEMBER-A".to_string(),
            team_id: Some("team-42".to_string()),
        };

        handle_disconnect(&info, None, &mgr).await;

        // MEMBER-B should receive the offline notification.
        let msg = b_rx.recv().await.unwrap();
        match msg {
            WsMessage::NearbyUpdate { devices } => {
                assert_eq!(devices[0].code, "MEMBER-A");
                assert!(!devices[0].online);
            }
            other => panic!("expected NearbyUpdate, got: {other:?}"),
        }

        // MEMBER-A should be removed from session manager.
        assert!(!mgr.contains("MEMBER-A").await);
        // MEMBER-B should still be in session manager.
        assert!(mgr.contains("MEMBER-B").await);
    }
}
