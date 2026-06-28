// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! WebSocket upgrade handler and connection-level message loop.
//!
//! The [`ws_handler`] function accepts an HTTP request with the
//! `Upgrade: websocket` header, upgrades the connection, and runs a
//! per-connection loop that:
//!
//! 1. Reads JSON messages from the client.
//! 2. Routes them based on type (register, heartbeat, signaling, etc.).
//! 3. Maintains the session lifecycle in the [`SessionManager`].
//! 4. Cleans up on disconnect (removes session, clears Redis TTL).

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use tracing;

use super::message::WsMessage;
use super::session::SessionManager;
use crate::handlers::connect;
use crate::handlers::disconnect;
use crate::handlers::heartbeat;
use crate::handlers::invite;
use crate::handlers::register::{self, ConnectionInfo};
use crate::handlers::relay::{self, RelayNode};
use crate::redis::RedisPool;

// ---------------------------------------------------------------------------
// Application state shared across all handlers
// ---------------------------------------------------------------------------

/// Shared state threaded through every request handler via axum's `State`
/// extractor.
#[derive(Clone, Default)]
pub struct AppState {
    /// In-memory registry of active WebSocket sessions.
    pub session_manager: SessionManager,
    /// Optional Redis pool (`None` in unit tests that don't need Redis).
    pub redis_pool: Option<RedisPool>,
    /// Relay nodes available for allocation.
    pub relay_nodes: Vec<RelayNode>,
    /// HMAC secret used to sign relay session tokens.
    pub hmac_secret: Vec<u8>,
}

// ---------------------------------------------------------------------------
// WebSocket handler
// ---------------------------------------------------------------------------

/// `GET /ws` — upgrades the HTTP connection to a WebSocket.
///
/// Axum returns `101 Switching Protocols` and hands us a [`WebSocket`]
/// which we pass to [`handle_connection`].
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    tracing::debug!("websocket upgrade requested");
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}

// ---------------------------------------------------------------------------
// Per-connection logic
// ---------------------------------------------------------------------------

/// Runs the full lifecycle of a single WebSocket connection.
///
/// * Splits the socket into a sender/receiver pair.
/// * Spawns a background **write task** that forwards messages from an
///   mpsc channel to the WebSocket sink.
/// * Runs a **read loop** in the current task that deserializes incoming
///   JSON and dispatches to [`process_message`].
/// * Cleans up the session and Redis state on disconnect.
async fn handle_connection(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<WsMessage>(64);

    // Track registration info so we can clean up on disconnect.
    let connection_info: Arc<Mutex<Option<ConnectionInfo>>> = Arc::new(Mutex::new(None));

    // -- Write task: mpsc channel -> WebSocket sink -------------------------
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(json) => {
                    if sender.send(Message::Text(json.into())).await.is_err() {
                        break;
                    }
                }
                Err(err) => {
                    tracing::warn!(%err, "failed to serialize outgoing message");
                }
            }
        }
    });

    // -- Read loop: WebSocket stream -> message router ----------------------
    let ci = connection_info.clone();
    while let Some(result) = receiver.next().await {
        match result {
            Ok(Message::Text(text)) => {
                process_message(&text, &state, &tx, &ci).await;
            }
            Ok(Message::Close(_)) => {
                tracing::debug!("client sent close frame");
                break;
            }
            Ok(_) => {
                // Binary / Ping / Pong — axum handles pong replies;
                // we ignore binary frames in the signaling protocol.
            }
            Err(err) => {
                tracing::warn!(%err, "websocket receive error");
                break;
            }
        }
    }

    // -- Cleanup: remove session + clear Redis online state -----------------
    let info = connection_info.lock().await.take();
    if let Some(ref info) = info {
        tracing::debug!(device_code = %info.device_code, "cleaning up disconnected session");
        disconnect::handle_disconnect(
            info,
            state.redis_pool.clone().as_mut(),
            &state.session_manager,
        )
        .await;
    }

    // Drop the local tx so the write task sees no more senders and exits.
    drop(tx);
    tracing::debug!("connection handler finished");
}

// ---------------------------------------------------------------------------
// Message router
// ---------------------------------------------------------------------------

/// Deserialize an incoming JSON text frame and dispatch by message type.
///
/// On parse failure the server attempts to extract the `"type"` field from
/// the raw JSON to produce a helpful error response.
async fn process_message(
    text: &str,
    state: &AppState,
    tx: &mpsc::Sender<WsMessage>,
    connection_info: &Mutex<Option<ConnectionInfo>>,
) {
    match serde_json::from_str::<WsMessage>(text) {
        Ok(msg) => route_message(msg, state, tx, connection_info).await,
        Err(parse_err) => {
            // Try to pull out the "type" field for a better error.
            let type_name = serde_json::from_str::<serde_json::Value>(text)
                .ok()
                .and_then(|v| {
                    v.get("type")
                        .and_then(|t| t.as_str())
                        .map(String::from)
                })
                .unwrap_or_else(|| "unknown".to_string());

            tracing::warn!(
                msg_type = %type_name,
                err = %parse_err,
                "failed to deserialize incoming message"
            );

            let _ = tx
                .send(WsMessage::Error {
                    code: "unknown_type".to_string(),
                    message: format!("unknown message type: {type_name}"),
                })
                .await;
        }
    }
}

/// Route a successfully parsed [`WsMessage`] to the appropriate handler.
async fn route_message(
    msg: WsMessage,
    state: &AppState,
    tx: &mpsc::Sender<WsMessage>,
    connection_info: &Mutex<Option<ConnectionInfo>>,
) {
    match &msg {
        WsMessage::Register {
            device_code: code,
            platform,
            version,
            team_id,
        } => {
            tracing::info!(device_code = %code, "device registered");
            register::handle_register(
                code,
                platform,
                version,
                team_id.as_deref(),
                state.redis_pool.clone().as_mut(),
                &state.session_manager,
                tx,
                connection_info,
            )
            .await;
        }

        WsMessage::Heartbeat {
            device_code: code, ..
        } => {
            tracing::trace!(device_code = %code, "heartbeat received");
            heartbeat::handle_heartbeat(
                code,
                state.redis_pool.clone().as_mut(),
                tx,
            )
            .await;
        }

        WsMessage::ConnectRequest {
            from_code,
            to_code,
            invite_code,
        } => {
            tracing::debug!(from = %from_code, to = %to_code, "connect_request received");
            if let Err(err) = connect::handle_connect_request(
                from_code,
                to_code,
                invite_code.as_deref(),
                &state.session_manager,
            )
            .await
            {
                tracing::warn!(%err, "connect_request handler error");
            }
        }

        WsMessage::ConnectResponse {
            accepted,
            session_id,
            from_code,
        } => {
            tracing::debug!(
                from = %from_code,
                session_id = %session_id,
                accepted = accepted,
                "connect_response received"
            );
            if let Err(err) = connect::handle_connect_response(
                from_code,
                *accepted,
                session_id,
                from_code,
                &state.session_manager,
                state.redis_pool.clone().as_mut(),
            )
            .await
            {
                tracing::warn!(%err, "connect_response handler error");
            }
        }

        WsMessage::IceOffer {
            session_id,
            sdp,
            candidates,
        } => {
            let device_code = connection_info
                .lock()
                .await
                .as_ref()
                .map(|ci| ci.device_code.clone())
                .unwrap_or_default();
            tracing::debug!(
                from = %device_code,
                session_id = %session_id,
                "ice_offer received"
            );
            if let Err(err) = connect::handle_ice_offer(
                &device_code,
                session_id,
                sdp,
                candidates,
                &state.session_manager,
            )
            .await
            {
                tracing::warn!(%err, "ice_offer handler error");
            }
        }

        WsMessage::IceAnswer {
            session_id,
            sdp,
            candidates,
        } => {
            let device_code = connection_info
                .lock()
                .await
                .as_ref()
                .map(|ci| ci.device_code.clone())
                .unwrap_or_default();
            tracing::debug!(
                from = %device_code,
                session_id = %session_id,
                "ice_answer received"
            );
            if let Err(err) = connect::handle_ice_answer(
                &device_code,
                session_id,
                sdp,
                candidates,
                &state.session_manager,
            )
            .await
            {
                tracing::warn!(%err, "ice_answer handler error");
            }
        }

        WsMessage::IceTrickle {
            session_id,
            candidate,
        } => {
            let device_code = connection_info
                .lock()
                .await
                .as_ref()
                .map(|ci| ci.device_code.clone())
                .unwrap_or_default();
            tracing::debug!(
                from = %device_code,
                session_id = %session_id,
                "ice_trickle received"
            );
            if let Err(err) = connect::handle_ice_trickle(
                &device_code,
                session_id,
                &candidate,
                &state.session_manager,
            )
            .await
            {
                tracing::warn!(%err, "ice_trickle handler error");
            }
        }

        WsMessage::RelayRequest {
            session_id,
            preferred_region,
        } => {
            tracing::debug!(
                session_id = %session_id,
                preferred_region = preferred_region.as_deref().unwrap_or("none"),
                "relay_request received"
            );
            if let Err(err) = relay::handle_relay_request(
                session_id,
                preferred_region.as_deref(),
                &state.relay_nodes,
                &state.hmac_secret,
                state.redis_pool.clone().as_mut(),
                tx,
            )
            .await
            {
                tracing::warn!(%err, "relay_request handler error");
            }
        }

        WsMessage::GenerateInvite { device_code } => {
            tracing::debug!(device_code = %device_code, "generate_invite received");
            let mut pool = state.redis_pool.clone();
            match pool.as_mut() {
                Some(ref mut redis) => {
                    let ci = connection_info.lock().await;
                    let team_id = ci.as_ref().and_then(|c| c.team_id.as_deref());
                    match invite::handle_generate_invite(
                        device_code,
                        team_id,
                        redis,
                    )
                    .await
                    {
                        Ok(code) => {
                            let _ = tx
                                .send(WsMessage::InviteGenerated {
                                    invite_code: code,
                                })
                                .await;
                        }
                        Err(err) => {
                            tracing::warn!(%err, "generate_invite handler error");
                            let _ = tx
                                .send(WsMessage::Error {
                                    code: "invite_error".to_string(),
                                    message: format!("failed to generate invite: {err}"),
                                })
                                .await;
                        }
                    }
                }
                None => {
                    let _ = tx
                        .send(WsMessage::Error {
                            code: "invite_error".to_string(),
                            message: "redis unavailable".to_string(),
                        })
                        .await;
                }
            }
        }

        WsMessage::UseInvite {
            from_code,
            invite_code,
        } => {
            tracing::debug!(
                from = %from_code,
                invite_code = %invite_code,
                "use_invite received"
            );
            let mut pool = state.redis_pool.clone();
            match pool.as_mut() {
                Some(ref mut redis) => {
                    match invite::handle_use_invite(
                        invite_code,
                        from_code,
                        redis,
                        &state.session_manager,
                    )
                    .await
                    {
                        Ok((session_id, to_code)) => {
                            let _ = tx
                                .send(WsMessage::InviteResult {
                                    session_id,
                                    to_code,
                                })
                                .await;
                        }
                        Err(err) => {
                            tracing::warn!(%err, "use_invite handler error");
                            let error_msg = match &err {
                                crate::error::AppError::NotFound(msg) => msg.clone(),
                                other => format!("{other}"),
                            };
                            let _ = tx
                                .send(WsMessage::Error {
                                    code: "invite_error".to_string(),
                                    message: error_msg,
                                })
                                .await;
                        }
                    }
                }
                None => {
                    let _ = tx
                        .send(WsMessage::Error {
                            code: "invite_error".to_string(),
                            message: "redis unavailable".to_string(),
                        })
                        .await;
                }
            }
        }

        // The remaining message types are forwarded/routed by higher-level
        // components that will be implemented in subsequent tasks.
        // For now we log them at debug level.
        other => {
            tracing::debug!(
                msg_type = %message_type_name(other),
                "received signaling message (routing not yet implemented)"
            );
        }
    }
}

/// Return the `"type"` tag string for a [`WsMessage`] variant (for logging).
fn message_type_name(msg: &WsMessage) -> &'static str {
    match msg {
        WsMessage::Register { .. } => "register",
        WsMessage::Heartbeat { .. } => "heartbeat",
        WsMessage::ConnectRequest { .. } => "connect_request",
        WsMessage::ConnectResponse { .. } => "connect_response",
        WsMessage::IceOffer { .. } => "ice_offer",
        WsMessage::IceAnswer { .. } => "ice_answer",
        WsMessage::IceTrickle { .. } => "ice_trickle",
        WsMessage::RelayRequest { .. } => "relay_request",
        WsMessage::RelayAssigned { .. } => "relay_assigned",
        WsMessage::PeerOffline { .. } => "peer_offline",
        WsMessage::NearbyUpdate { .. } => "nearby_update",
        WsMessage::GenerateInvite { .. } => "generate_invite",
        WsMessage::UseInvite { .. } => "use_invite",
        WsMessage::InviteGenerated { .. } => "invite_generated",
        WsMessage::InviteResult { .. } => "invite_result",
        WsMessage::Error { .. } => "error",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::message::WsMessage;

    // -- Unit tests for message routing (no WebSocket needed) ---------------

    #[tokio::test]
    async fn register_stores_session_in_manager() {
        let state = AppState::default();
        let (tx, _rx) = mpsc::channel(8);
        let ci = Mutex::new(None);

        let json = serde_json::json!({
            "type": "register",
            "device_code": "TEST-001",
            "platform": "linux",
            "version": "0.1.0",
            "team_id": "team-42"
        });

        process_message(&json.to_string(), &state, &tx, &ci).await;

        assert!(state.session_manager.contains("TEST-001").await);
        let info = ci.lock().await;
        assert!(info.is_some());
        assert_eq!(info.as_ref().unwrap().device_code, "TEST-001");
        assert_eq!(info.as_ref().unwrap().team_id.as_deref(), Some("team-42"));
    }

    #[tokio::test]
    async fn register_without_team_id() {
        let state = AppState::default();
        let (tx, _rx) = mpsc::channel(8);
        let ci = Mutex::new(None);

        let json = serde_json::json!({
            "type": "register",
            "device_code": "TEST-002",
            "platform": "macos",
            "version": "1.0"
        });

        process_message(&json.to_string(), &state, &tx, &ci).await;
        assert!(state.session_manager.contains("TEST-002").await);
    }

    #[tokio::test]
    async fn unknown_type_sends_error_response() {
        let state = AppState::default();
        let (tx, mut rx) = mpsc::channel(8);
        let ci = Mutex::new(None);

        let json = r#"{"type":"totally_bogus","data":123}"#;
        process_message(json, &state, &tx, &ci).await;

        let response = rx.recv().await.unwrap();
        match response {
            WsMessage::Error { code, message } => {
                assert_eq!(code, "unknown_type");
                assert!(
                    message.contains("totally_bogus"),
                    "error should mention the unknown type, got: {message}"
                );
            }
            other => panic!("expected Error, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn malformed_json_sends_error_response() {
        let state = AppState::default();
        let (tx, mut rx) = mpsc::channel(8);
        let ci = Mutex::new(None);

        let json = r#"this is not json at all"#;
        process_message(json, &state, &tx, &ci).await;

        let response = rx.recv().await.unwrap();
        match response {
            WsMessage::Error { code, .. } => {
                assert_eq!(code, "unknown_type");
            }
            other => panic!("expected Error, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn heartbeat_without_redis_does_not_panic() {
        let state = AppState::default(); // redis_pool = None
        let (tx, _rx) = mpsc::channel(8);
        let ci = Mutex::new(None);

        let json = serde_json::json!({
            "type": "heartbeat",
            "device_code": "DEV1",
            "ts": 1_700_000_000u64
        });

        // Should not panic even without Redis.
        process_message(&json.to_string(), &state, &tx, &ci).await;
    }

    #[tokio::test]
    async fn all_signaling_types_do_not_panic() {
        let state = AppState::default();
        let (tx, _rx) = mpsc::channel(8);
        let ci = Mutex::new(None);

        let messages = [
            r#"{"type":"connect_request","from_code":"A","to_code":"B"}"#,
            r#"{"type":"connect_response","accepted":true,"session_id":"s1","from_code":"A"}"#,
            r#"{"type":"ice_offer","session_id":"s1","sdp":"v=0","candidates":[]}"#,
            r#"{"type":"ice_answer","session_id":"s1","sdp":"v=0","candidates":[]}"#,
            r#"{"type":"ice_trickle","session_id":"s1","candidate":{"candidate":"candidate:1 udp 2130706431 192.168.1.10 5000 typ host"}}"#,
            r#"{"type":"relay_request","session_id":"s1"}"#,
            r#"{"type":"relay_assigned","session_id":"s1","relay_addr":"r.io","relay_port":443,"token":"t"}"#,
            r#"{"type":"peer_offline","device_code":"X","reason":"timeout"}"#,
            r#"{"type":"nearby_update","devices":[]}"#,
        ];

        for json in &messages {
            process_message(json, &state, &tx, &ci).await;
        }
    }

    // -- WebSocket integration tests ----------------------------------------

    /// Spin up a real axum server on a random port and return the address.
    async fn start_test_server() -> std::net::SocketAddr {
        use axum::routing::get;
        use axum::Router;

        let state = AppState::default();
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        addr
    }

    #[tokio::test]
    async fn ws_upgrade_returns_101() {
        let addr = start_test_server().await;
        let url = format!("ws://{addr}/ws");

        let (ws, response) = tokio_tungstenite::connect_async(&url).await.unwrap();
        assert_eq!(response.status(), 101, "WebSocket upgrade should return 101");
        drop(ws);
    }

    #[tokio::test]
    async fn ws_register_and_receive_messages() {
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

        let addr = start_test_server().await;
        let url = format!("ws://{addr}/ws");

        let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

        // Send a Register message.
        let register = serde_json::json!({
            "type": "register",
            "device_code": "WS-TEST-1",
            "platform": "test",
            "version": "0.0.1"
        });
        ws.send(TungsteniteMessage::Text(register.to_string().into()))
            .await
            .unwrap();

        // Send a ConnectRequest to an offline target — should produce a
        // device_offline error response.
        let connect = serde_json::json!({
            "type": "connect_request",
            "from_code": "WS-TEST-1",
            "to_code": "OTHER"
        });
        ws.send(TungsteniteMessage::Text(connect.to_string().into()))
            .await
            .unwrap();

        // Send an unknown type and expect an error response.
        let bogus = r#"{"type":"alien_invasion","data":true}"#;
        ws.send(TungsteniteMessage::Text(bogus.into()))
            .await
            .unwrap();

        // Read the first error response (device_offline from connect_request).
        use futures_util::StreamExt;
        let msg1 = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            ws.next(),
        )
        .await
        .expect("timed out waiting for device_offline error")
        .unwrap()
        .unwrap();

        let text1 = msg1.into_text().unwrap();
        let parsed1: serde_json::Value = serde_json::from_str(&text1).unwrap();
        assert_eq!(parsed1["type"], "error");
        assert_eq!(parsed1["code"], "device_offline");

        // Read the second error response (unknown_type from bogus message).
        let msg2 = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            ws.next(),
        )
        .await
        .expect("timed out waiting for unknown_type error")
        .unwrap()
        .unwrap();

        let text2 = msg2.into_text().unwrap();
        let parsed2: serde_json::Value = serde_json::from_str(&text2).unwrap();
        assert_eq!(parsed2["type"], "error");
        assert_eq!(parsed2["code"], "unknown_type");
        assert!(
            parsed2["message"]
                .as_str()
                .unwrap()
                .contains("alien_invasion")
        );

        ws.close(None).await.ok();
    }

    #[tokio::test]
    async fn ws_disconnect_cleans_up_session() {
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

        let state = AppState::default();

        let app = axum::Router::new()
            .route(
                "/ws",
                axum::routing::get(ws_handler),
            )
            .with_state(state.clone());

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url = format!("ws://{addr}/ws");
        let (mut ws, _) = tokio_tungstenite::connect_async(&url).await.unwrap();

        // Register.
        let register = serde_json::json!({
            "type": "register",
            "device_code": "CLEANUP-1",
            "platform": "test",
            "version": "0.0.1"
        });
        ws.send(TungsteniteMessage::Text(register.to_string().into()))
            .await
            .unwrap();

        // Give the server a moment to process the register.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert!(state.session_manager.contains("CLEANUP-1").await);

        // Close the connection.
        ws.close(None).await.ok();

        // Give the server a moment to clean up.
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        assert!(
            !state.session_manager.contains("CLEANUP-1").await,
            "session should be removed after disconnect"
        );
    }
}
