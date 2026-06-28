// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Device registration handler.
//!
//! When a client sends a `Register` message over WebSocket, this handler:
//!
//! 1. Stores the device's online status in Redis as JSON with a 60-second TTL.
//! 2. Adds the device code to the team's `online_devices` set (if a team is
//!    specified).
//! 3. Registers the session in the in-memory [`SessionManager`].
//! 4. Broadcasts a `nearby_update` to all sessions with the same team ID.

use std::time::Instant;

use serde::Serialize;
use tokio::sync::{mpsc, Mutex};

use crate::redis::keys;
use crate::redis::ttl;
use crate::redis::RedisPool;
use crate::ws::message::{DeviceInfo, WsMessage};
use crate::ws::session::SessionManager;

/// Metadata persisted alongside the online status for connection tracking.
#[derive(Clone)]
pub struct ConnectionInfo {
    /// Unique device code reported by the client.
    pub device_code: String,
    /// Optional team/organization this device belongs to.
    pub team_id: Option<String>,
}

/// JSON payload stored in the `device:{code}:online` Redis key.
#[derive(Serialize)]
struct OnlineInfo {
    platform: String,
    version: String,
    team_id: Option<String>,
}

/// Handle a device registration request.
///
/// Performs the following steps in order:
///
/// 1. Stores `device:{code}:online` as JSON with a 60-second TTL.
/// 2. If `team_id` is present, adds `code` to `team:{team_id}:online_devices`.
/// 3. Registers the session in [`SessionManager`].
/// 4. Stores connection info for disconnect cleanup.
/// 5. Broadcasts `nearby_update` to all sessions with the same `team_id`.
#[allow(clippy::too_many_arguments)]
pub async fn handle_register(
    device_code: &str,
    platform: &str,
    version: &str,
    team_id: Option<&str>,
    redis: Option<&mut RedisPool>,
    session_mgr: &SessionManager,
    tx: &mpsc::Sender<WsMessage>,
    connection_info: &Mutex<Option<ConnectionInfo>>,
) {
    // 1. Store device online status in Redis as JSON with TTL.
    if let Some(pool) = redis {
        let key = keys::device_online_key(device_code);
        let info = OnlineInfo {
            platform: platform.to_string(),
            version: version.to_string(),
            team_id: team_id.map(String::from),
        };
        if let Err(err) =
            ttl::set_json_with_ttl(pool, &key, &info, ttl::DEVICE_ONLINE_TTL).await
        {
            tracing::warn!(%err, device_code, "failed to store device online status");
        }

        // 2. Add device to team's online_devices set.
        if let Some(tid) = team_id {
            let team_key = keys::team_online_key(tid);
            if let Err(err) = ttl::sadd(pool, &team_key, device_code).await {
                tracing::warn!(%err, device_code, team_id = tid, "failed to add to team set");
            }
        }
    }

    // 3. Register session in the in-memory session manager.
    let session = crate::ws::session::Session {
        device_code: device_code.to_string(),
        team_id: team_id.map(String::from),
        tx: tx.clone(),
        connected_at: Instant::now(),
    };
    session_mgr
        .insert(device_code.to_string(), session)
        .await;

    // 4. Store connection info for disconnect cleanup.
    *connection_info.lock().await = Some(ConnectionInfo {
        device_code: device_code.to_string(),
        team_id: team_id.map(String::from),
    });

    // 5. Broadcast nearby_update to same-team devices.
    if let Some(tid) = team_id {
        let update = WsMessage::NearbyUpdate {
            devices: vec![DeviceInfo {
                code: device_code.to_string(),
                name: device_code.to_string(),
                platform: platform.to_string(),
                online: true,
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
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn register_stores_session_and_connection_info() {
        let mgr = SessionManager::new();
        let (tx, _rx) = mpsc::channel(8);
        let conn = Mutex::new(None);

        handle_register(
            "DEV-001",
            "linux",
            "0.1.0",
            Some("team-42"),
            None, // no Redis
            &mgr,
            &tx,
            &conn,
        )
        .await;

        assert!(mgr.contains("DEV-001").await);
        let info = conn.lock().await;
        assert!(info.is_some());
        assert_eq!(info.as_ref().unwrap().device_code, "DEV-001");
        assert_eq!(info.as_ref().unwrap().team_id.as_deref(), Some("team-42"));
    }

    #[tokio::test]
    async fn register_without_team_id_does_not_broadcast() {
        let mgr = SessionManager::new();

        // Pre-existing team member that should NOT receive any message.
        let (existing_tx, mut existing_rx) = mpsc::channel(8);
        let existing_session = crate::ws::session::Session {
            device_code: "EXISTING".to_string(),
            team_id: Some("team-42".to_string()),
            tx: existing_tx,
            connected_at: Instant::now(),
        };
        mgr.insert("EXISTING".to_string(), existing_session).await;

        let (tx, _rx) = mpsc::channel(8);
        let conn = Mutex::new(None);

        // Register without team_id.
        handle_register("DEV-NEW", "macos", "1.0", None, None, &mgr, &tx, &conn)
            .await;

        assert!(mgr.contains("DEV-NEW").await);

        // The existing session should NOT have received a broadcast.
        assert!(
            tokio::time::timeout(
                std::time::Duration::from_millis(50),
                existing_rx.recv()
            )
            .await
            .is_err(),
            "no broadcast should be sent when team_id is None"
        );

        // Connection info should have None team_id.
        let info = conn.lock().await;
        assert!(info.as_ref().unwrap().team_id.is_none());
    }

    #[tokio::test]
    async fn register_broadcasts_nearby_update_to_same_team() {
        let mgr = SessionManager::new();

        // Pre-existing team members.
        let (member_a_tx, mut member_a_rx) = mpsc::channel(8);
        let member_a = crate::ws::session::Session {
            device_code: "MEMBER-A".to_string(),
            team_id: Some("team-42".to_string()),
            tx: member_a_tx,
            connected_at: Instant::now(),
        };
        mgr.insert("MEMBER-A".to_string(), member_a).await;

        let (member_b_tx, mut member_b_rx) = mpsc::channel(8);
        let member_b = crate::ws::session::Session {
            device_code: "MEMBER-B".to_string(),
            team_id: Some("team-42".to_string()),
            tx: member_b_tx,
            connected_at: Instant::now(),
        };
        mgr.insert("MEMBER-B".to_string(), member_b).await;

        let (tx, _rx) = mpsc::channel(8);
        let conn = Mutex::new(None);

        handle_register(
            "DEV-NEW",
            "linux",
            "0.2.0",
            Some("team-42"),
            None, // no Redis
            &mgr,
            &tx,
            &conn,
        )
        .await;

        // Both existing team members should receive a nearby_update.
        let msg_a = member_a_rx.recv().await.unwrap();
        match msg_a {
            WsMessage::NearbyUpdate { devices } => {
                assert_eq!(devices.len(), 1);
                assert_eq!(devices[0].code, "DEV-NEW");
                assert!(devices[0].online);
                assert_eq!(devices[0].platform, "linux");
            }
            other => panic!("expected NearbyUpdate, got: {other:?}"),
        }

        let msg_b = member_b_rx.recv().await.unwrap();
        match msg_b {
            WsMessage::NearbyUpdate { devices } => {
                assert_eq!(devices.len(), 1);
                assert_eq!(devices[0].code, "DEV-NEW");
            }
            other => panic!("expected NearbyUpdate, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn register_replaces_existing_session() {
        let mgr = SessionManager::new();

        // First registration.
        let (tx1, _rx1) = mpsc::channel(8);
        let conn = Mutex::new(None);
        handle_register("DEV-001", "linux", "0.1.0", None, None, &mgr, &tx1, &conn)
            .await;
        assert_eq!(mgr.len().await, 1);

        // Second registration (reconnection) replaces the first.
        let (tx2, _rx2) = mpsc::channel(8);
        handle_register("DEV-001", "macos", "0.2.0", None, None, &mgr, &tx2, &conn)
            .await;
        assert_eq!(mgr.len().await, 1);
    }
}
