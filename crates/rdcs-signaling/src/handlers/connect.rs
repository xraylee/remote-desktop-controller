// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Connection negotiation and ICE relay handlers.
//!
//! This module implements the signaling flow for establishing a peer-to-peer
//! remote desktop connection:
//!
//! 1. **connect_request** — the controller asks the server to invite a target
//!    device. The server looks the target up in the [`SessionManager`],
//!    generates a session ID, and forwards the request.
//! 2. **connect_response** — the target accepts (or rejects). The server
//!    records the session participants, stores session metadata in Redis, and
//!    forwards the response to the controller.
//! 3. **ice_offer** — the controller sends its SDP offer + ICE candidates.
//!    The server relays them to the target.
//! 4. **ice_answer** — the target sends its SDP answer + ICE candidates.
//!    The server relays them back to the controller.

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::error::AppError;
use crate::redis::keys;
use crate::redis::ttl;
use crate::redis::RedisPool;
use crate::ws::message::{IceCandidate, WsMessage};
use crate::ws::session::SessionManager;

// ---------------------------------------------------------------------------
// JSON payload stored in Redis for each session
// ---------------------------------------------------------------------------

/// Session metadata persisted in the `session:{id}` Redis key.
#[derive(Serialize)]
struct SessionRecord {
    /// Transport path — `"unknown"` until the relay layer assigns one.
    path: String,
    /// Unix-epoch timestamp (seconds) when the session was created.
    started_at: u64,
    /// Controller (initiator) device code.
    controller: String,
    /// Target (responder) device code.
    target: String,
}

// ---------------------------------------------------------------------------
// Handler: connect_request
// ---------------------------------------------------------------------------

/// Handle a `connect_request` from a controller device.
///
/// 1. Looks up the target device in the [`SessionManager`].
/// 2. If the target is **offline**, sends an [`WsMessage::Error`] back to
///    the controller and returns `Ok(())`.
/// 3. If the target is **online**, generates a UUID session ID, records a
///    pending connection, and forwards the [`WsMessage::ConnectRequest`] to
///    the target.
pub async fn handle_connect_request(
    from_code: &str,
    to_code: &str,
    invite_code: Option<&str>,
    session_mgr: &SessionManager,
) -> Result<(), AppError> {
    // 1. Check that the target is online.
    if !session_mgr.contains(to_code).await {
        tracing::debug!(
            from = from_code,
            target = to_code,
            "connect_request target is offline"
        );
        let _ = session_mgr
            .send_to(
                from_code,
                WsMessage::Error {
                    code: "device_offline".to_string(),
                    message: format!("device {to_code} is offline"),
                },
            )
            .await;
        return Ok(());
    }

    // 2. Generate a session ID and record the pending connection.
    let session_id = uuid::Uuid::new_v4().to_string();
    session_mgr.add_pending_connection(to_code, from_code).await;

    tracing::info!(
        from = from_code,
        target = to_code,
        session_id = %session_id,
        "forwarding connect_request"
    );

    // 3. Forward the ConnectRequest to the target.
    let _ = session_mgr
        .send_to(
            to_code,
            WsMessage::ConnectRequest {
                from_code: from_code.to_string(),
                to_code: to_code.to_string(),
                session_id: Some(session_id),
                invite_code: invite_code.map(String::from),
            },
        )
        .await;

    Ok(())
}

// ---------------------------------------------------------------------------
// Handler: connect_response
// ---------------------------------------------------------------------------

/// Handle a `connect_response` from the target device.
///
/// 1. Consumes the pending connection to discover the controller device code.
/// 2. Records the session participants in the [`SessionManager`] so that
///    subsequent ICE messages can be routed.
/// 3. Forwards the [`WsMessage::ConnectResponse`] to the controller.
/// 4. Stores session metadata in Redis (`session:{session_id}`).
pub async fn handle_connect_response(
    from_device: &str,
    accepted: bool,
    session_id: &str,
    from_code_in_msg: &str,
    session_mgr: &SessionManager,
    redis: Option<&mut RedisPool>,
) -> Result<(), AppError> {
    // 1. Look up the controller that initiated the connection.
    //    `from_device` is the responder (target). The pending connection maps
    //    target -> controller.
    let controller_code = session_mgr
        .take_pending_connection(from_device)
        .await
        .unwrap_or_else(|| from_code_in_msg.to_string());

    // 2. Record session participants for ICE relay routing.
    session_mgr
        .insert_session_participants(session_id, &controller_code, from_device)
        .await;

    tracing::info!(
        controller = %controller_code,
        target = from_device,
        session_id = %session_id,
        accepted = accepted,
        "connect_response processed"
    );

    // 3. Forward the ConnectResponse to the controller.
    let _ = session_mgr
        .send_to(
            &controller_code,
            WsMessage::ConnectResponse {
                accepted,
                session_id: session_id.to_string(),
                from_code: from_device.to_string(),
            },
        )
        .await;

    // 4. Store session metadata in Redis.
    if let Some(pool) = redis {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let record = SessionRecord {
            path: "unknown".to_string(),
            started_at: now,
            controller: controller_code.clone(),
            target: from_device.to_string(),
        };
        let key = keys::session_key(session_id);
        if let Err(err) = ttl::set_json_with_ttl(pool, &key, &record, ttl::SESSION_TTL).await {
            tracing::warn!(
                %err,
                session_id = %session_id,
                "failed to store session metadata in Redis"
            );
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Handler: ice_offer
// ---------------------------------------------------------------------------

/// Relay an ICE offer from one peer to the other.
///
/// Looks up the session participants by `session_id`, determines the other
/// peer, and forwards the [`WsMessage::IceOffer`] to it.
pub async fn handle_ice_offer(
    from_device: &str,
    session_id: &str,
    sdp: &str,
    candidates: &[IceCandidate],
    session_mgr: &SessionManager,
) -> Result<(), AppError> {
    let other = match session_mgr.get_other_peer(session_id, from_device).await {
        Some(peer) => peer,
        None => {
            tracing::warn!(
                session_id = %session_id,
                from = from_device,
                "ice_offer for unknown session or non-participant"
            );
            let _ = session_mgr
                .send_to(
                    from_device,
                    WsMessage::Error {
                        code: "unknown_session".to_string(),
                        message: format!("session {session_id} not found"),
                    },
                )
                .await;
            return Ok(());
        }
    };

    tracing::debug!(
        session_id = %session_id,
        from = from_device,
        to = %other,
        "relaying ice_offer"
    );

    let _ = session_mgr
        .send_to(
            &other,
            WsMessage::IceOffer {
                session_id: session_id.to_string(),
                sdp: sdp.to_string(),
                candidates: candidates.to_vec(),
            },
        )
        .await;

    Ok(())
}

// ---------------------------------------------------------------------------
// Handler: ice_answer
// ---------------------------------------------------------------------------

/// Relay an ICE answer from one peer back to the offerer.
///
/// Looks up the session participants by `session_id`, determines the other
/// peer, and forwards the [`WsMessage::IceAnswer`] to it.
pub async fn handle_ice_answer(
    from_device: &str,
    session_id: &str,
    sdp: &str,
    candidates: &[IceCandidate],
    session_mgr: &SessionManager,
) -> Result<(), AppError> {
    let other = match session_mgr.get_other_peer(session_id, from_device).await {
        Some(peer) => peer,
        None => {
            tracing::warn!(
                session_id = %session_id,
                from = from_device,
                "ice_answer for unknown session or non-participant"
            );
            let _ = session_mgr
                .send_to(
                    from_device,
                    WsMessage::Error {
                        code: "unknown_session".to_string(),
                        message: format!("session {session_id} not found"),
                    },
                )
                .await;
            return Ok(());
        }
    };

    tracing::debug!(
        session_id = %session_id,
        from = from_device,
        to = %other,
        "relaying ice_answer"
    );

    let _ = session_mgr
        .send_to(
            &other,
            WsMessage::IceAnswer {
                session_id: session_id.to_string(),
                sdp: sdp.to_string(),
                candidates: candidates.to_vec(),
            },
        )
        .await;

    Ok(())
}

// ---------------------------------------------------------------------------
// Handler: ice_trickle
// ---------------------------------------------------------------------------

/// Relay a trickle ICE candidate from one peer to the other.
///
/// Looks up the session participants by `session_id`, determines the other
/// peer, and forwards the [`WsMessage::IceTrickle`] to it.
pub async fn handle_ice_trickle(
    from_device: &str,
    session_id: &str,
    candidate: &IceCandidate,
    session_mgr: &SessionManager,
) -> Result<(), AppError> {
    let other = match session_mgr.get_other_peer(session_id, from_device).await {
        Some(peer) => peer,
        None => {
            tracing::warn!(
                session_id = %session_id,
                from = from_device,
                "ice_trickle for unknown session or non-participant"
            );
            let _ = session_mgr
                .send_to(
                    from_device,
                    WsMessage::Error {
                        code: "unknown_session".to_string(),
                        message: format!("session {session_id} not found"),
                    },
                )
                .await;
            return Ok(());
        }
    };

    tracing::debug!(
        session_id = %session_id,
        from = from_device,
        to = %other,
        "relaying ice_trickle"
    );

    let _ = session_mgr
        .send_to(
            &other,
            WsMessage::IceTrickle {
                session_id: session_id.to_string(),
                candidate: candidate.clone(),
            },
        )
        .await;

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::message::IceCandidate;
    use crate::ws::session::Session;
    use std::time::Instant;
    use tokio::sync::mpsc;

    /// Helper: create a session with a receiver for verifying forwarded messages.
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

    /// Helper: register two devices in the session manager and return their
    /// receivers.
    async fn setup_two_devices(
        mgr: &SessionManager,
        controller: &str,
        target: &str,
    ) -> (mpsc::Receiver<WsMessage>, mpsc::Receiver<WsMessage>) {
        let (ctrl_session, ctrl_rx) = make_session_with_rx(controller);
        let (tgt_session, tgt_rx) = make_session_with_rx(target);
        mgr.insert(controller.to_string(), ctrl_session).await;
        mgr.insert(target.to_string(), tgt_session).await;
        (ctrl_rx, tgt_rx)
    }

    // -----------------------------------------------------------------------
    // connect_request tests
    // -----------------------------------------------------------------------

    /// AC: connect_request_forwards_to_target — request sent to target's WebSocket.
    #[tokio::test]
    async fn connect_request_forwards_to_target() {
        let mgr = SessionManager::new();
        let (_ctrl_rx, mut tgt_rx) = setup_two_devices(&mgr, "CTRL", "TARGET").await;

        let result = handle_connect_request("CTRL", "TARGET", None, &mgr).await;
        assert!(result.is_ok());

        // The target should have received a ConnectRequest.
        let msg = tgt_rx.recv().await.expect("target should receive ConnectRequest");
        match msg {
            WsMessage::ConnectRequest {
                from_code,
                to_code,
                session_id,
                invite_code,
            } => {
                assert_eq!(from_code, "CTRL");
                assert_eq!(to_code, "TARGET");
                assert!(
                    session_id.as_deref().is_some_and(|s| !s.is_empty()),
                    "forwarded ConnectRequest must carry a non-empty session_id"
                );
                assert!(invite_code.is_none());
            }
            other => panic!("expected ConnectRequest, got: {other:?}"),
        }
    }

    /// AC: connect_request_offline — offline target returns error.
    #[tokio::test]
    async fn connect_request_offline() {
        let mgr = SessionManager::new();

        // Only register the controller, NOT the target.
        let (ctrl_session, mut ctrl_rx) = make_session_with_rx("CTRL");
        mgr.insert("CTRL".to_string(), ctrl_session).await;

        let result = handle_connect_request("CTRL", "OFFLINE_DEV", None, &mgr).await;
        assert!(result.is_ok());

        // The controller should receive an error.
        let msg = ctrl_rx
            .recv()
            .await
            .expect("controller should receive error for offline target");
        match msg {
            WsMessage::Error { code, message } => {
                assert_eq!(code, "device_offline");
                assert!(
                    message.contains("OFFLINE_DEV"),
                    "error should mention the offline device: {message}"
                );
            }
            other => panic!("expected Error, got: {other:?}"),
        }
    }

    /// Verify that connect_request with invite_code forwards it.
    #[tokio::test]
    async fn connect_request_with_invite_code() {
        let mgr = SessionManager::new();
        let (_ctrl_rx, mut tgt_rx) = setup_two_devices(&mgr, "CTRL", "TARGET").await;

        let result =
            handle_connect_request("CTRL", "TARGET", Some("INV-99"), &mgr).await;
        assert!(result.is_ok());

        let msg = tgt_rx.recv().await.unwrap();
        match msg {
            WsMessage::ConnectRequest { invite_code, session_id, .. } => {
                assert_eq!(invite_code.as_deref(), Some("INV-99"));
                assert!(session_id.is_some());
            }
            other => panic!("expected ConnectRequest, got: {other:?}"),
        }
    }

    /// Verify that a pending connection is recorded.
    #[tokio::test]
    async fn connect_request_records_pending_connection() {
        let mgr = SessionManager::new();
        let (_ctrl_rx, _tgt_rx) = setup_two_devices(&mgr, "CTRL", "TARGET").await;

        handle_connect_request("CTRL", "TARGET", None, &mgr)
            .await
            .unwrap();

        // The pending connection should be recorded: TARGET -> CTRL.
        let controller = mgr.take_pending_connection("TARGET").await;
        assert_eq!(controller.as_deref(), Some("CTRL"));
    }

    // -----------------------------------------------------------------------
    // connect_response tests
    // -----------------------------------------------------------------------

    /// AC: connect_response_stores_session — session created in Redis
    /// (unit test without Redis verifies participant mapping + forwarding).
    #[tokio::test]
    async fn connect_response_forwards_to_controller_and_records_participants() {
        let mgr = SessionManager::new();
        let (mut ctrl_rx, _tgt_rx) = setup_two_devices(&mgr, "CTRL", "TARGET").await;

        // Simulate a prior connect_request that recorded a pending connection.
        mgr.add_pending_connection("TARGET", "CTRL").await;

        let result = handle_connect_response(
            "TARGET",    // from_device = responder
            true,        // accepted
            "sess-001",  // session_id
            "TARGET",    // from_code in the message
            &mgr,
            None,        // no Redis
        )
        .await;
        assert!(result.is_ok());

        // Controller should receive the ConnectResponse.
        let msg = ctrl_rx
            .recv()
            .await
            .expect("controller should receive ConnectResponse");
        match msg {
            WsMessage::ConnectResponse {
                accepted,
                session_id,
                from_code,
            } => {
                assert!(accepted);
                assert_eq!(session_id, "sess-001");
                assert_eq!(from_code, "TARGET");
            }
            other => panic!("expected ConnectResponse, got: {other:?}"),
        }

        // Session participants should be recorded.
        assert_eq!(
            mgr.get_other_peer("sess-001", "CTRL").await.as_deref(),
            Some("TARGET")
        );
        assert_eq!(
            mgr.get_other_peer("sess-001", "TARGET").await.as_deref(),
            Some("CTRL")
        );
    }

    /// Verify connect_response works without Redis (graceful degradation).
    #[tokio::test]
    async fn connect_response_without_redis_does_not_panic() {
        let mgr = SessionManager::new();
        let (_ctrl_rx, _tgt_rx) = setup_two_devices(&mgr, "CTRL", "TARGET").await;
        mgr.add_pending_connection("TARGET", "CTRL").await;

        let result = handle_connect_response(
            "TARGET", true, "sess-002", "TARGET", &mgr, None,
        )
        .await;
        assert!(result.is_ok());
    }

    // -----------------------------------------------------------------------
    // ice_offer tests
    // -----------------------------------------------------------------------

    /// AC: ice_offer_forwards_to_peer — ICE offer forwarded to other peer.
    #[tokio::test]
    async fn ice_offer_forwards_to_peer() {
        let mgr = SessionManager::new();
        let (_ctrl_rx, mut tgt_rx) = setup_two_devices(&mgr, "CTRL", "TARGET").await;

        // Set up session participants (as if connect_response already ran).
        mgr.insert_session_participants("sess-001", "CTRL", "TARGET")
            .await;

        let candidates = vec![IceCandidate {
            candidate: "candidate:1 udp 2130706431 192.168.1.1 5000 typ host".into(),
            sdp_mid: Some("0".into()),
            sdp_m_line_index: Some(0),
        }];

        let result = handle_ice_offer(
            "CTRL",
            "sess-001",
            "v=0\r\no=- 123 1 IN IP4 0.0.0.0\r\n",
            &candidates,
            &mgr,
        )
        .await;
        assert!(result.is_ok());

        // Target should receive the IceOffer.
        let msg = tgt_rx
            .recv()
            .await
            .expect("target should receive IceOffer");
        match msg {
            WsMessage::IceOffer {
                session_id,
                sdp,
                candidates: recv_candidates,
            } => {
                assert_eq!(session_id, "sess-001");
                assert!(sdp.contains("v=0"));
                assert_eq!(recv_candidates.len(), 1);
                assert_eq!(recv_candidates[0].sdp_mid.as_deref(), Some("0"));
            }
            other => panic!("expected IceOffer, got: {other:?}"),
        }
    }

    /// ICE offer for unknown session sends error back to sender.
    #[tokio::test]
    async fn ice_offer_unknown_session_sends_error() {
        let mgr = SessionManager::new();
        let (ctrl_session, mut ctrl_rx) = make_session_with_rx("CTRL");
        mgr.insert("CTRL".to_string(), ctrl_session).await;

        let result =
            handle_ice_offer("CTRL", "no-such-session", "v=0", &[], &mgr).await;
        assert!(result.is_ok());

        let msg = ctrl_rx.recv().await.expect("should receive error");
        match msg {
            WsMessage::Error { code, message } => {
                assert_eq!(code, "unknown_session");
                assert!(message.contains("no-such-session"));
            }
            other => panic!("expected Error, got: {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // ice_answer tests
    // -----------------------------------------------------------------------

    /// AC: ice_answer_forwards_back — ICE answer forwarded back to offerer.
    #[tokio::test]
    async fn ice_answer_forwards_back() {
        let mgr = SessionManager::new();
        let (mut ctrl_rx, _tgt_rx) = setup_two_devices(&mgr, "CTRL", "TARGET").await;

        // Set up session participants.
        mgr.insert_session_participants("sess-001", "CTRL", "TARGET")
            .await;

        let candidates = vec![IceCandidate {
            candidate: "candidate:2 udp 1694498815 10.0.0.1 6000 typ srflx".into(),
            sdp_mid: Some("0".into()),
            sdp_m_line_index: Some(0),
        }];

        // Target sends the answer.
        let result = handle_ice_answer(
            "TARGET",
            "sess-001",
            "v=0\r\no=- 456 1 IN IP4 0.0.0.0\r\n",
            &candidates,
            &mgr,
        )
        .await;
        assert!(result.is_ok());

        // Controller (offerer) should receive the IceAnswer.
        let msg = ctrl_rx
            .recv()
            .await
            .expect("controller should receive IceAnswer");
        match msg {
            WsMessage::IceAnswer {
                session_id,
                sdp,
                candidates: recv_candidates,
            } => {
                assert_eq!(session_id, "sess-001");
                assert!(sdp.contains("v=0"));
                assert_eq!(recv_candidates.len(), 1);
            }
            other => panic!("expected IceAnswer, got: {other:?}"),
        }
    }

    /// ICE answer for unknown session sends error back to sender.
    #[tokio::test]
    async fn ice_answer_unknown_session_sends_error() {
        let mgr = SessionManager::new();
        let (tgt_session, mut tgt_rx) = make_session_with_rx("TARGET");
        mgr.insert("TARGET".to_string(), tgt_session).await;

        let result =
            handle_ice_answer("TARGET", "no-such", "v=0", &[], &mgr).await;
        assert!(result.is_ok());

        let msg = tgt_rx.recv().await.expect("should receive error");
        match msg {
            WsMessage::Error { code, .. } => {
                assert_eq!(code, "unknown_session");
            }
            other => panic!("expected Error, got: {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Full flow integration test
    // -----------------------------------------------------------------------

    /// End-to-end: connect_request -> connect_response -> ice_offer -> ice_answer.
    #[tokio::test]
    async fn full_connection_negotiation_flow() {
        let mgr = SessionManager::new();
        let (mut ctrl_rx, mut tgt_rx) =
            setup_two_devices(&mgr, "CTRL", "TARGET").await;

        // Step 1: Controller sends connect_request.
        handle_connect_request("CTRL", "TARGET", None, &mgr)
            .await
            .unwrap();

        // Target receives the request.
        let req_msg = tgt_rx.recv().await.unwrap();
        assert!(matches!(req_msg, WsMessage::ConnectRequest { .. }));

        // Step 2: Target sends connect_response.
        handle_connect_response(
            "TARGET", true, "sess-e2e", "TARGET", &mgr, None,
        )
        .await
        .unwrap();

        // Controller receives the response.
        let resp_msg = ctrl_rx.recv().await.unwrap();
        match &resp_msg {
            WsMessage::ConnectResponse {
                accepted,
                session_id,
                ..
            } => {
                assert!(accepted);
                assert_eq!(session_id, "sess-e2e");
            }
            other => panic!("expected ConnectResponse, got: {other:?}"),
        }

        // Step 3: Controller sends ICE offer.
        handle_ice_offer(
            "CTRL",
            "sess-e2e",
            "v=0\r\noffer-sdp",
            &[IceCandidate {
                candidate: "c1".into(),
                sdp_mid: None,
                sdp_m_line_index: None,
            }],
            &mgr,
        )
        .await
        .unwrap();

        // Target receives the offer.
        let offer_msg = tgt_rx.recv().await.unwrap();
        assert!(matches!(offer_msg, WsMessage::IceOffer { .. }));

        // Step 4: Target sends ICE answer.
        handle_ice_answer(
            "TARGET",
            "sess-e2e",
            "v=0\r\nanswer-sdp",
            &[],
            &mgr,
        )
        .await
        .unwrap();

        // Controller receives the answer.
        let answer_msg = ctrl_rx.recv().await.unwrap();
        assert!(matches!(answer_msg, WsMessage::IceAnswer { .. }));
    }
}
