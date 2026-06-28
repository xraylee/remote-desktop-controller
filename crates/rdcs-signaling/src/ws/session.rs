// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Per-connection session state and the in-memory session manager.
//!
//! Each active WebSocket connection is represented by a [`Session`] that
//! tracks the device code, team membership, and a channel for sending
//! messages back to the client.
//!
//! [`SessionManager`] is a thread-safe registry that maps device codes to
//! their active sessions. It is cheaply cloneable (backed by an `Arc`)
//! and safe to share across async tasks.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::{mpsc, RwLock};

use super::message::WsMessage;

/// Per-connection session state.
///
/// Created when a client sends a `Register` message and removed when the
/// WebSocket connection closes.
pub struct Session {
    /// Unique device code reported by the client.
    pub device_code: String,
    /// Optional team/organization this device belongs to.
    pub team_id: Option<String>,
    /// Channel for sending messages to this client's write task.
    pub tx: mpsc::Sender<WsMessage>,
    /// When the connection was established.
    pub connected_at: Instant,
}

/// Thread-safe, cloneable session registry.
///
/// Internally holds a `HashMap<device_code, Session>` behind an
/// `Arc<RwLock<...>>`, allowing concurrent reads and exclusive writes.
///
/// Additionally tracks:
/// - **pending connections**: a short-lived mapping from target device code
///   to controller device code, set when a `connect_request` is forwarded and
///   consumed when the corresponding `connect_response` arrives.
/// - **session participants**: a mapping from signaling session ID to the two
///   device codes involved, used to route ICE offer/answer messages.
#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// target device_code -> controller device_code.
    pending_connections: Arc<RwLock<HashMap<String, String>>>,
    /// session_id -> (peer_a_device_code, peer_b_device_code).
    session_participants: Arc<RwLock<HashMap<String, (String, String)>>>,
}

impl SessionManager {
    /// Create a new, empty session manager.
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            pending_connections: Arc::new(RwLock::new(HashMap::new())),
            session_participants: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a new session for the given device code.
    ///
    /// If a session already exists for this code, it is replaced and the
    /// old session is returned.
    pub async fn insert(&self, device_code: String, session: Session) -> Option<Session> {
        self.sessions.write().await.insert(device_code, session)
    }

    /// Remove the session for `device_code`, returning it if it existed.
    pub async fn remove(&self, device_code: &str) -> Option<Session> {
        self.sessions.write().await.remove(device_code)
    }

    /// Check whether a session exists for the given device code.
    pub async fn contains(&self, device_code: &str) -> bool {
        self.sessions.read().await.contains_key(device_code)
    }

    /// Send a message to the device identified by `device_code`.
    ///
    /// Returns `Ok(())` on success, or `Err` if the device is not found
    /// or its send channel is closed.
    pub async fn send_to(
        &self,
        device_code: &str,
        msg: WsMessage,
    ) -> Result<(), SessionSendError> {
        let sessions = self.sessions.read().await;
        match sessions.get(device_code) {
            Some(session) => session
                .tx
                .send(msg)
                .await
                .map_err(|_| SessionSendError::ChannelClosed),
            None => Err(SessionSendError::DeviceNotFound(device_code.to_string())),
        }
    }

    /// Return the team ID of the session for `device_code`, if it exists
    /// and has a team assigned.
    pub async fn get_team_id(&self, device_code: &str) -> Option<String> {
        self.sessions
            .read()
            .await
            .get(device_code)
            .and_then(|s| s.team_id.clone())
    }

    /// Number of currently active sessions.
    pub async fn len(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Whether the session registry is empty.
    pub async fn is_empty(&self) -> bool {
        self.sessions.read().await.is_empty()
    }

    /// Broadcast a message to all connected sessions.
    ///
    /// Returns the number of sessions the message was sent to.
    /// Failures (e.g. closed channels) are silently ignored.
    pub async fn broadcast_all(&self, msg: WsMessage) -> usize {
        let sessions = self.sessions.read().await;
        let mut count = 0;
        for (_code, session) in sessions.iter() {
            if session.tx.send(msg.clone()).await.is_ok() {
                count += 1;
            }
        }
        count
    }

    /// Broadcast a message to all sessions belonging to the same team,
    /// optionally skipping one device (typically the sender).
    ///
    /// Messages are sent on a best-effort basis: failures (e.g. closed
    /// channels) are silently ignored.
    pub async fn broadcast_to_team(
        &self,
        team_id: &str,
        msg: WsMessage,
        skip_device: Option<&str>,
    ) {
        let sessions = self.sessions.read().await;
        for (code, session) in sessions.iter() {
            if session.team_id.as_deref() == Some(team_id) {
                if let Some(skip) = skip_device {
                    if code == skip {
                        continue;
                    }
                }
                let _ = session.tx.send(msg.clone()).await;
            }
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Pending-connection and session-participant helpers
// ---------------------------------------------------------------------------

impl SessionManager {
    /// Record a pending connection: the controller (`from_code`) wants to
    /// connect to the target (`to_code`).
    ///
    /// Called by the connect-request handler; consumed by
    /// [`take_pending_connection`] when the target responds.
    pub async fn add_pending_connection(&self, to_code: &str, from_code: &str) {
        self.pending_connections
            .write()
            .await
            .insert(to_code.to_string(), from_code.to_string());
    }

    /// Consume the pending connection for `to_code`, returning the controller
    /// device code if one was recorded.
    pub async fn take_pending_connection(&self, to_code: &str) -> Option<String> {
        self.pending_connections
            .write()
            .await
            .remove(to_code)
    }

    /// Register the two participants of a signaling session.
    ///
    /// `peer_a` is the controller (initiator) and `peer_b` is the target
    /// (responder).
    pub async fn insert_session_participants(
        &self,
        session_id: &str,
        peer_a: &str,
        peer_b: &str,
    ) {
        self.session_participants
            .write()
            .await
            .insert(session_id.to_string(), (peer_a.to_string(), peer_b.to_string()));
    }

    /// Given a `session_id` and the device code of one participant, return
    /// the device code of the **other** participant.
    ///
    /// Returns `None` if the session is unknown or `from_device` is not a
    /// participant.
    pub async fn get_other_peer(
        &self,
        session_id: &str,
        from_device: &str,
    ) -> Option<String> {
        let sessions = self.session_participants.read().await;
        sessions.get(session_id).and_then(|(a, b)| {
            if a == from_device {
                Some(b.clone())
            } else if b == from_device {
                Some(a.clone())
            } else {
                None
            }
        })
    }

    /// Remove the participant mapping for a session.
    pub async fn remove_session_participants(&self, session_id: &str) -> Option<(String, String)> {
        self.session_participants
            .write()
            .await
            .remove(session_id)
    }
}

/// Errors that can occur when sending a message to a session.
#[derive(Debug, thiserror::Error)]
pub enum SessionSendError {
    #[error("device `{0}` not found in session manager")]
    DeviceNotFound(String),
    #[error("send channel closed (client disconnected)")]
    ChannelClosed,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::message::DeviceInfo;
    use tokio::sync::mpsc;

    fn make_session(code: &str) -> (Session, mpsc::Receiver<WsMessage>) {
        let (tx, rx) = mpsc::channel(8);
        let session = Session {
            device_code: code.to_string(),
            team_id: None,
            tx,
            connected_at: Instant::now(),
        };
        (session, rx)
    }

    #[tokio::test]
    async fn insert_and_contains() {
        let mgr = SessionManager::new();
        let (session, _rx) = make_session("DEV1");
        mgr.insert("DEV1".into(), session).await;
        assert!(mgr.contains("DEV1").await);
        assert!(!mgr.contains("DEV2").await);
    }

    #[tokio::test]
    async fn remove_returns_session() {
        let mgr = SessionManager::new();
        let (session, _rx) = make_session("DEV1");
        mgr.insert("DEV1".into(), session).await;

        let removed = mgr.remove("DEV1").await;
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().device_code, "DEV1");
        assert!(!mgr.contains("DEV1").await);
    }

    #[tokio::test]
    async fn remove_nonexistent_returns_none() {
        let mgr = SessionManager::new();
        assert!(mgr.remove("NOPE").await.is_none());
    }

    #[tokio::test]
    async fn send_to_delivers_message() {
        let mgr = SessionManager::new();
        let (session, mut rx) = make_session("DEV1");
        mgr.insert("DEV1".into(), session).await;

        let msg = WsMessage::Heartbeat {
            device_code: "DEV1".into(),
            ts: 42,
        };
        mgr.send_to("DEV1", msg.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received, msg);
    }

    #[tokio::test]
    async fn send_to_unknown_device_errors() {
        let mgr = SessionManager::new();
        let msg = WsMessage::Heartbeat {
            device_code: "NOPE".into(),
            ts: 1,
        };
        let result = mgr.send_to("NOPE", msg).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SessionSendError::DeviceNotFound(_)
        ));
    }

    #[tokio::test]
    async fn len_and_is_empty() {
        let mgr = SessionManager::new();
        assert!(mgr.is_empty().await);
        assert_eq!(mgr.len().await, 0);

        let (s1, _rx1) = make_session("A");
        let (s2, _rx2) = make_session("B");
        mgr.insert("A".into(), s1).await;
        mgr.insert("B".into(), s2).await;
        assert_eq!(mgr.len().await, 2);
        assert!(!mgr.is_empty().await);
    }

    #[tokio::test]
    async fn insert_replaces_existing() {
        let mgr = SessionManager::new();
        let (s1, _rx1) = make_session("DEV1");
        let (s2, _rx2) = make_session("DEV1");
        mgr.insert("DEV1".into(), s1).await;
        let old = mgr.insert("DEV1".into(), s2).await;
        assert!(old.is_some());
        assert_eq!(mgr.len().await, 1);
    }

    #[tokio::test]
    async fn clone_shares_state() {
        let mgr1 = SessionManager::new();
        let mgr2 = mgr1.clone();

        let (session, _rx) = make_session("DEV1");
        mgr1.insert("DEV1".into(), session).await;

        assert!(mgr2.contains("DEV1").await);
    }

    // -- broadcast_to_team tests -------------------------------------------

    /// Helper: create a session with a specific team_id.
    fn make_team_session(
        code: &str,
        team: &str,
    ) -> (Session, mpsc::Receiver<WsMessage>) {
        let (tx, rx) = mpsc::channel(8);
        let session = Session {
            device_code: code.to_string(),
            team_id: Some(team.to_string()),
            tx,
            connected_at: Instant::now(),
        };
        (session, rx)
    }

    #[tokio::test]
    async fn broadcast_to_team_delivers_to_same_team() {
        let mgr = SessionManager::new();
        let (s1, mut rx1) = make_team_session("D1", "team-A");
        let (s2, mut rx2) = make_team_session("D2", "team-A");
        let (s3, mut rx3) = make_team_session("D3", "team-B");
        mgr.insert("D1".into(), s1).await;
        mgr.insert("D2".into(), s2).await;
        mgr.insert("D3".into(), s3).await;

        let msg = WsMessage::NearbyUpdate {
            devices: vec![DeviceInfo {
                code: "D1".into(),
                name: "D1".into(),
                platform: "linux".into(),
                online: true,
            }],
        };

        mgr.broadcast_to_team("team-A", msg, None).await;

        // D1 and D2 should receive the message.
        assert!(rx1.recv().await.is_some());
        assert!(rx2.recv().await.is_some());

        // D3 (different team) should NOT receive the message.
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), rx3.recv())
                .await
                .is_err(),
            "D3 should not receive broadcast from team-A"
        );
    }

    #[tokio::test]
    async fn broadcast_to_team_with_skip_device() {
        let mgr = SessionManager::new();
        let (s1, mut rx1) = make_team_session("D1", "team-A");
        let (s2, mut rx2) = make_team_session("D2", "team-A");
        mgr.insert("D1".into(), s1).await;
        mgr.insert("D2".into(), s2).await;

        let msg = WsMessage::NearbyUpdate { devices: vec![] };

        // Broadcast but skip D1.
        mgr.broadcast_to_team("team-A", msg, Some("D1")).await;

        // D1 should NOT receive (skipped).
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(50), rx1.recv())
                .await
                .is_err(),
            "D1 should be skipped"
        );

        // D2 should receive.
        assert!(rx2.recv().await.is_some());
    }

    #[tokio::test]
    async fn broadcast_to_nonexistent_team_does_nothing() {
        let mgr = SessionManager::new();
        let (s1, _rx1) = make_team_session("D1", "team-A");
        mgr.insert("D1".into(), s1).await;

        // Broadcast to a team with no members -- should not panic.
        let msg = WsMessage::NearbyUpdate { devices: vec![] };
        mgr.broadcast_to_team("team-nonexistent", msg, None).await;
    }

    // -- get_team_id tests --------------------------------------------------

    #[tokio::test]
    async fn get_team_id_returns_team_when_present() {
        let mgr = SessionManager::new();
        let (session, _rx) = make_team_session("D1", "team-A");
        mgr.insert("D1".into(), session).await;
        assert_eq!(mgr.get_team_id("D1").await.as_deref(), Some("team-A"));
    }

    #[tokio::test]
    async fn get_team_id_returns_none_for_unknown_device() {
        let mgr = SessionManager::new();
        assert!(mgr.get_team_id("NOPE").await.is_none());
    }

    #[tokio::test]
    async fn get_team_id_returns_none_when_no_team() {
        let mgr = SessionManager::new();
        let (session, _rx) = make_session("D1");
        mgr.insert("D1".into(), session).await;
        assert!(mgr.get_team_id("D1").await.is_none());
    }

    // -- pending connection tests -------------------------------------------

    #[tokio::test]
    async fn add_and_take_pending_connection() {
        let mgr = SessionManager::new();
        mgr.add_pending_connection("TARGET", "CTRL").await;

        let controller = mgr.take_pending_connection("TARGET").await;
        assert_eq!(controller.as_deref(), Some("CTRL"));

        // Second take should return None (consumed).
        let again = mgr.take_pending_connection("TARGET").await;
        assert!(again.is_none());
    }

    #[tokio::test]
    async fn take_pending_connection_unknown_returns_none() {
        let mgr = SessionManager::new();
        assert!(mgr.take_pending_connection("NOPE").await.is_none());
    }

    // -- session participant tests ------------------------------------------

    #[tokio::test]
    async fn insert_and_get_other_peer() {
        let mgr = SessionManager::new();
        mgr.insert_session_participants("sess-1", "A", "B").await;

        assert_eq!(mgr.get_other_peer("sess-1", "A").await.as_deref(), Some("B"));
        assert_eq!(mgr.get_other_peer("sess-1", "B").await.as_deref(), Some("A"));
    }

    #[tokio::test]
    async fn get_other_peer_unknown_session() {
        let mgr = SessionManager::new();
        assert!(mgr.get_other_peer("no-such", "A").await.is_none());
    }

    #[tokio::test]
    async fn get_other_peer_unknown_device() {
        let mgr = SessionManager::new();
        mgr.insert_session_participants("sess-1", "A", "B").await;

        // Device C is not a participant.
        assert!(mgr.get_other_peer("sess-1", "C").await.is_none());
    }

    #[tokio::test]
    async fn remove_session_participants() {
        let mgr = SessionManager::new();
        mgr.insert_session_participants("sess-1", "A", "B").await;

        let removed = mgr.remove_session_participants("sess-1").await;
        assert!(removed.is_some());

        assert!(mgr.get_other_peer("sess-1", "A").await.is_none());
    }
}
