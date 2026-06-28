// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests exercising the full signaling flow over real
//! WebSocket connections.
//!
//! These tests start a real axum server (with no Redis pool) and connect
//! clients via `tokio-tungstenite` to verify end-to-end behavior:
//!
//! - `full_connection_flow`: register → connect → ICE exchange
//! - `disconnect_cleanup`: disconnect removes session and notifies peer
//! - `invite_code_flow`: invite generation → consumption (requires Redis)

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use rdcs_signaling::ws::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// WebSocket stream type returned by `connect_async`.
type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Start a test server with default [`AppState`] (no Redis) and return
/// the bound address and a clone of the state for assertions.
async fn start_test_server() -> (std::net::SocketAddr, AppState) {
    let state = AppState::default();
    let app = rdcs_signaling::router_with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    // Give the server a moment to start accepting.
    tokio::time::sleep(Duration::from_millis(20)).await;
    (addr, state)
}

/// Connect a WebSocket client to the test server.
async fn connect_ws(addr: std::net::SocketAddr) -> WsStream {
    let url = format!("ws://{addr}/ws");
    let (ws, _resp) = connect_async(&url).await.unwrap();
    ws
}

/// Send a JSON value over the WebSocket as a text frame.
async fn send_json(ws: &mut WsStream, value: &Value) {
    ws.send(TungsteniteMessage::Text(value.to_string().into()))
        .await
        .unwrap();
}

/// Receive the next JSON message from the WebSocket, with a timeout.
async fn recv_json(ws: &mut WsStream) -> Value {
    let msg = tokio::time::timeout(Duration::from_secs(3), ws.next())
        .await
        .expect("timed out waiting for WebSocket message")
        .expect("WebSocket stream ended unexpectedly")
        .expect("WebSocket error");
    let text = msg.into_text().unwrap();
    serde_json::from_str(&text).unwrap()
}

/// Try to receive a message, returning `None` on timeout.
async fn try_recv_json(ws: &mut WsStream) -> Option<Value> {
    match tokio::time::timeout(Duration::from_millis(200), ws.next()).await {
        Ok(Some(Ok(msg))) => {
            let text = msg.into_text().unwrap();
            serde_json::from_str(&text).ok()
        }
        _ => None,
    }
}

/// Drain all pending messages from the WebSocket (with a short timeout).
async fn drain_messages(ws: &mut WsStream) -> Vec<Value> {
    let mut msgs = Vec::new();
    while let Some(msg) = try_recv_json(ws).await {
        msgs.push(msg);
    }
    msgs
}

/// Send a Register message.
async fn register(ws: &mut WsStream, device_code: &str) {
    send_json(
        ws,
        &serde_json::json!({
            "type": "register",
            "device_code": device_code,
            "platform": "test",
            "version": "0.0.1"
        }),
    )
    .await;
}

/// Send a Register message with a team_id.
async fn register_with_team(ws: &mut WsStream, device_code: &str, team_id: &str) {
    send_json(
        ws,
        &serde_json::json!({
            "type": "register",
            "device_code": device_code,
            "platform": "test",
            "version": "0.0.1",
            "team_id": team_id
        }),
    )
    .await;
}

// ---------------------------------------------------------------------------
// AC: full_connection_flow
//
// register → connect → ICE → all messages delivered
// ---------------------------------------------------------------------------

#[tokio::test]
async fn full_connection_flow() {
    let (addr, state) = start_test_server().await;

    // 1. Client A connects and registers (no team to avoid broadcasts).
    let mut ws_a = connect_ws(addr).await;
    register(&mut ws_a, "CLIENT-A").await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 2. Client B connects and registers.
    let mut ws_b = connect_ws(addr).await;
    register(&mut ws_b, "CLIENT-B").await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Both should be in the session manager.
    assert!(state.session_manager.contains("CLIENT-A").await);
    assert!(state.session_manager.contains("CLIENT-B").await);

    // 3. Client A sends ConnectRequest to B.
    send_json(
        &mut ws_a,
        &serde_json::json!({
            "type": "connect_request",
            "from_code": "CLIENT-A",
            "to_code": "CLIENT-B"
        }),
    )
    .await;

    // 4. Client B receives the forwarded ConnectRequest.
    let msg = recv_json(&mut ws_b).await;
    assert_eq!(msg["type"], "connect_request");
    assert_eq!(msg["from_code"], "CLIENT-A");
    assert_eq!(msg["to_code"], "CLIENT-B");

    // 5. Client B sends ConnectResponse (B generates the session_id).
    send_json(
        &mut ws_b,
        &serde_json::json!({
            "type": "connect_response",
            "accepted": true,
            "session_id": "test-session-001",
            "from_code": "CLIENT-B"
        }),
    )
    .await;

    // 6. Client A receives the forwarded ConnectResponse.
    let msg = recv_json(&mut ws_a).await;
    assert_eq!(msg["type"], "connect_response");
    assert_eq!(msg["accepted"], true);
    assert_eq!(msg["session_id"], "test-session-001");
    assert_eq!(msg["from_code"], "CLIENT-B");

    // 7. Client A sends IceOffer.
    send_json(
        &mut ws_a,
        &serde_json::json!({
            "type": "ice_offer",
            "session_id": "test-session-001",
            "sdp": "v=0\r\no=- 123 1 IN IP4 0.0.0.0\r\n",
            "candidates": [{
                "candidate": "candidate:1 udp 2130706431 192.168.1.1 5000 typ host",
                "sdp_mid": "0",
                "sdp_m_line_index": 0
            }]
        }),
    )
    .await;

    // 8. Client B receives the IceOffer.
    let msg = recv_json(&mut ws_b).await;
    assert_eq!(msg["type"], "ice_offer");
    assert_eq!(msg["session_id"], "test-session-001");
    assert!(msg["sdp"].as_str().unwrap().contains("v=0"));
    assert_eq!(msg["candidates"].as_array().unwrap().len(), 1);

    // 9. Client B sends IceAnswer.
    send_json(
        &mut ws_b,
        &serde_json::json!({
            "type": "ice_answer",
            "session_id": "test-session-001",
            "sdp": "v=0\r\no=- 456 1 IN IP4 0.0.0.0\r\n",
            "candidates": []
        }),
    )
    .await;

    // 10. Client A receives the IceAnswer.
    let msg = recv_json(&mut ws_a).await;
    assert_eq!(msg["type"], "ice_answer");
    assert_eq!(msg["session_id"], "test-session-001");
    assert!(msg["sdp"].as_str().unwrap().contains("v=0"));

    // Cleanup.
    ws_a.close(None).await.ok();
    ws_b.close(None).await.ok();
}

// ---------------------------------------------------------------------------
// AC: disconnect_cleanup
//
// disconnect removes session + notifies peer
// ---------------------------------------------------------------------------

#[tokio::test]
async fn disconnect_cleanup() {
    let (addr, state) = start_test_server().await;

    // 1. Client B registers first with team "team-1".
    let mut ws_b = connect_ws(addr).await;
    register_with_team(&mut ws_b, "CLIENT-B", "team-1").await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 2. Client A registers with the same team.
    let mut ws_a = connect_ws(addr).await;
    register_with_team(&mut ws_a, "CLIENT-A", "team-1").await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Both should be registered.
    assert!(state.session_manager.contains("CLIENT-A").await);
    assert!(state.session_manager.contains("CLIENT-B").await);

    // Drain any nearby_update broadcasts from registration.
    // B may receive a nearby_update about A joining.
    // A may receive a nearby_update about itself (broadcast includes sender).
    let _ = drain_messages(&mut ws_b).await;
    let _ = drain_messages(&mut ws_a).await;

    // 3. Client A disconnects.
    ws_a.close(None).await.ok();

    // Wait for server-side cleanup.
    tokio::time::sleep(Duration::from_millis(200)).await;

    // 4. A's session should be removed from the SessionManager.
    assert!(
        !state.session_manager.contains("CLIENT-A").await,
        "CLIENT-A session should be removed after disconnect"
    );

    // 5. B should receive a nearby_update indicating A went offline.
    let msg = tokio::time::timeout(Duration::from_secs(2), async {
        while let Some(msg) = try_recv_json(&mut ws_b).await {
            if msg["type"] == "nearby_update" {
                return msg;
            }
        }
        // One final blocking wait.
        recv_json(&mut ws_b).await
    })
    .await
    .expect("timed out waiting for offline notification");

    assert_eq!(msg["type"], "nearby_update");
    let devices = msg["devices"].as_array().unwrap();
    assert!(!devices.is_empty(), "should have at least one device in update");

    // Find CLIENT-A in the devices list.
    let a_device = devices
        .iter()
        .find(|d| d["code"] == "CLIENT-A")
        .expect("CLIENT-A should be in the nearby_update");
    assert_eq!(a_device["online"], false, "CLIENT-A should be marked offline");

    // B should still be in the session manager.
    assert!(state.session_manager.contains("CLIENT-B").await);

    ws_b.close(None).await.ok();
}

// ---------------------------------------------------------------------------
// AC: invite_code_flow
//
// invite generates → used → connects → consumed
// (Requires a running Redis server)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires a running Redis server at redis://127.0.0.1:6379"]
async fn invite_code_flow() {
    use rdcs_signaling::redis as rdcs_redis;

    let redis_url = "redis://127.0.0.1:6379";
    let redis_pool = rdcs_redis::create_pool(redis_url)
        .await
        .expect("failed to connect to Redis");

    let state = AppState {
        redis_pool: Some(redis_pool),
        ..AppState::default()
    };

    let app = rdcs_signaling::router_with_state(state.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(20)).await;

    // 1. Client A connects and registers.
    let mut ws_a = connect_ws(addr).await;
    register(&mut ws_a, "INVITE-A").await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 2. Client A generates an invite code.
    send_json(
        &mut ws_a,
        &serde_json::json!({
            "type": "generate_invite",
            "device_code": "INVITE-A"
        }),
    )
    .await;

    // A receives the invite_generated response.
    let msg = recv_json(&mut ws_a).await;
    assert_eq!(msg["type"], "invite_generated");
    let invite_code = msg["invite_code"]
        .as_str()
        .expect("invite_code should be a string")
        .to_string();
    assert_eq!(invite_code.len(), 4, "invite code should be 4 digits");

    // 3. Client B connects and registers.
    let mut ws_b = connect_ws(addr).await;
    register(&mut ws_b, "INVITE-B").await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    // 4. Client B uses the invite code.
    send_json(
        &mut ws_b,
        &serde_json::json!({
            "type": "use_invite",
            "from_code": "INVITE-B",
            "invite_code": invite_code
        }),
    )
    .await;

    // B receives the invite_result.
    let msg = recv_json(&mut ws_b).await;
    assert_eq!(msg["type"], "invite_result");
    assert_eq!(msg["to_code"], "INVITE-A");

    // A receives the forwarded connect_request (from the invite consumption).
    let msg = recv_json(&mut ws_a).await;
    assert_eq!(msg["type"], "connect_request");
    assert_eq!(msg["from_code"], "INVITE-B");
    assert_eq!(msg["to_code"], "INVITE-A");
    assert_eq!(msg["invite_code"], invite_code);

    // 5. Verify invite is consumed: B tries to use the same code again.
    send_json(
        &mut ws_b,
        &serde_json::json!({
            "type": "use_invite",
            "from_code": "INVITE-B",
            "invite_code": invite_code
        }),
    )
    .await;

    // B should receive an error (invite already consumed).
    let msg = recv_json(&mut ws_b).await;
    assert_eq!(msg["type"], "error");
    assert_eq!(msg["code"], "invite_error");

    // Cleanup.
    ws_a.close(None).await.ok();
    ws_b.close(None).await.ok();
}
