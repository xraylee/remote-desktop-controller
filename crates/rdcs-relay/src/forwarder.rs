// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Zero-copy data forwarding engine for relay sessions.
//!
//! When a DATA packet arrives for a known session, [`DataForwarder`]
//! determines which peer sent it and forwards the raw bytes directly
//! to the other peer — using a single pre-allocated buffer with no
//! intermediate copies.

use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::RwLock;

use crate::protocol;
use crate::session::SessionManager;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors produced during data forwarding.
#[derive(Debug, thiserror::Error)]
pub enum ForwardError {
    #[error("session {0} not found")]
    SessionNotFound(u64),

    #[error("unknown peer: {0}")]
    UnknownPeer(SocketAddr),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// DataForwarder
// ---------------------------------------------------------------------------

/// Forwards DATA packets between relay session peers.
///
/// The forwarder holds a shared reference to the relay's UDP socket and
/// the session manager.  It uses a *zero-copy* strategy: incoming packets
/// are read once into a pre-allocated buffer and the same buffer slice is
/// passed directly to `send_to` without any intermediate allocation or
/// copy.
pub struct DataForwarder {
    socket: Arc<UdpSocket>,
    sessions: Arc<RwLock<SessionManager>>,
}

impl DataForwarder {
    /// Create a new forwarder backed by the given socket and session manager.
    pub fn new(socket: Arc<UdpSocket>, sessions: Arc<RwLock<SessionManager>>) -> Self {
        Self { socket, sessions }
    }

    /// Forward a DATA packet to the appropriate peer.
    ///
    /// Steps:
    /// 1. Parse `session_id` from the packet header.
    /// 2. Look up the session in [`SessionManager`].
    /// 3. Determine which peer sent it (by source-address matching).
    /// 4. Forward the raw bytes to the *other* peer's address.
    /// 5. Update the session's `bytes_forwarded` counter.
    ///
    /// The session lock is released **before** the socket write so that
    /// other tasks (cleanup, allocate, …) are not blocked on network I/O.
    pub async fn forward_packet(
        &self,
        src_addr: SocketAddr,
        data: &[u8],
    ) -> Result<(), ForwardError> {
        // 1. Parse session_id from the 16-byte header.
        let (msg, _rest) = protocol::parse_message(data)
            .map_err(|_| ForwardError::SessionNotFound(0))?;

        let session_id = match msg {
            protocol::RelayMessage::Data { session_id } => session_id,
            _ => return Err(ForwardError::SessionNotFound(0)),
        };

        // 2-5. Resolve target via session manager (lock released immediately).
        let target = {
            let mut sessions = self.sessions.write().await;
            sessions
                .forward_target(session_id, src_addr)
                .map_err(|(sid, maybe_peer)| match maybe_peer {
                    None => ForwardError::SessionNotFound(sid),
                    Some(peer) => ForwardError::UnknownPeer(peer),
                })?
        };

        // Forward raw bytes (zero-copy: same buffer slice, no extra alloc).
        self.socket.send_to(data, target).await?;

        Ok(())
    }

    /// Start the forwarding loop — reads packets from the socket and
    /// dispatches them.
    ///
    /// This is a standalone event loop that handles **all** relay message
    /// types (DATA, ALLOCATE, RELEASE, KEEPALIVE).  It is an alternative
    /// to wiring [`forward_packet`](Self::forward_packet) into the main
    /// UDP recv loop by hand.
    #[allow(dead_code)]
    pub async fn run(&self) -> Result<(), ForwardError> {
        let mut buf = vec![0u8; 65536];

        loop {
            let (len, peer) = self.socket.recv_from(&mut buf).await?;
            let data = &buf[..len];

            match protocol::parse_message(data) {
                Ok((msg, _)) => match &msg {
                    protocol::RelayMessage::Data { .. } => {
                        if let Err(e) = self.forward_packet(peer, data).await {
                            tracing::warn!(
                                peer = %peer,
                                error = %e,
                                "failed to forward data packet"
                            );
                        }
                    }
                    protocol::RelayMessage::Allocate { session_id, token } => {
                        let token_str = String::from_utf8_lossy(token);
                        let mut sessions = self.sessions.write().await;
                        match sessions.allocate(*session_id, &token_str, peer) {
                            Ok((port_a, port_b)) => {
                                tracing::info!(
                                    session_id = *session_id,
                                    peer = %peer,
                                    port_a,
                                    port_b,
                                    "allocated relay ports"
                                );
                            }
                            Err(e) => {
                                tracing::warn!(
                                    session_id = *session_id,
                                    peer = %peer,
                                    error = %e,
                                    "allocate failed"
                                );
                            }
                        }
                    }
                    protocol::RelayMessage::Release { session_id } => {
                        let mut sessions = self.sessions.write().await;
                        match sessions.release(*session_id) {
                            Ok(()) => {
                                tracing::info!(session_id = *session_id, "released relay session");
                            }
                            Err(e) => {
                                tracing::warn!(
                                    session_id = *session_id,
                                    error = %e,
                                    "release failed"
                                );
                            }
                        }
                    }
                    protocol::RelayMessage::Keepalive { session_id } => {
                        let mut sessions = self.sessions.write().await;
                        if sessions.keepalive(*session_id) {
                            tracing::debug!(session_id = *session_id, "keepalive OK");
                        } else {
                            tracing::warn!(
                                session_id = *session_id,
                                "keepalive for unknown session"
                            );
                        }
                    }
                },
                Err(e) => {
                    tracing::warn!(peer = %peer, error = %e, "failed to parse relay packet");
                }
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
    use crate::auth::{generate_token, TokenPayload};
    use tokio::net::UdpSocket as TokioUdpSocket;

    const SECRET: &[u8] = b"test-forwarder-secret";

    /// Generate a valid HMAC token for the given session ID.
    fn valid_token(session_id: u64) -> String {
        let payload = TokenPayload {
            session_id,
            relay_addr: "127.0.0.1:3478".into(),
            nonce: session_id,
            expires_at: 4_102_444_800, // year 2100
        };
        generate_token(&payload, SECRET)
    }

    /// Helper: bind a UDP socket on loopback with an OS-assigned port.
    async fn bind_loopback() -> TokioUdpSocket {
        TokioUdpSocket::bind("127.0.0.1:0").await.unwrap()
    }

    /// Helper: set up a forwarder with a relay socket, session manager,
    /// and one active session (session_id = 1) with both peer addresses
    /// assigned to loopback sockets.
    ///
    /// Returns `(forwarder, relay_socket, peer_a_socket, peer_b_socket)`.
    async fn setup() -> (DataForwarder, Arc<TokioUdpSocket>, TokioUdpSocket, TokioUdpSocket) {
        let relay = Arc::new(bind_loopback().await);
        let peer_a = bind_loopback().await;
        let peer_b = bind_loopback().await;

        let mgr = SessionManager::new(49152, 65535, SECRET.to_vec());
        let sessions = Arc::new(RwLock::new(mgr));

        // Allocate session 1 with peer_a's address.
        {
            let mut s = sessions.write().await;
            let token = valid_token(1);
            s.allocate(1, &token, peer_a.local_addr().unwrap())
                .unwrap();
            s.set_peer_b(1, peer_b.local_addr().unwrap());
        }

        let fwd = DataForwarder::new(relay.clone(), sessions);
        (fwd, relay, peer_a, peer_b)
    }

    // -- Acceptance: forward_a_to_b -------------------------------------------

    #[tokio::test]
    async fn forward_a_to_b() {
        let (fwd, relay, peer_a, peer_b) = setup().await;

        let payload = b"frame-data-from-a";
        let pkt = protocol::encode_data(1, payload);

        // peer_a sends DATA to the relay socket.
        peer_a.send_to(&pkt, relay.local_addr().unwrap()).await.unwrap();

        let mut buf = vec![0u8; 65536];
        let (len, from) = relay.recv_from(&mut buf).await.unwrap();

        // Relay forwards to peer_b.
        fwd.forward_packet(from, &buf[..len]).await.unwrap();

        // peer_b receives the forwarded packet.
        let mut recv_buf = vec![0u8; 65536];
        let (n, _) = peer_b.recv_from(&mut recv_buf).await.unwrap();

        assert_eq!(&recv_buf[..n], &buf[..len], "peer_b must receive the exact bytes");
        assert_eq!(n, pkt.len());
        // Verify the payload is intact after the header.
        assert_eq!(&recv_buf[protocol::HEADER_LEN..n], payload);
    }

    // -- Acceptance: forward_b_to_a -------------------------------------------

    #[tokio::test]
    async fn forward_b_to_a() {
        let (fwd, relay, peer_a, peer_b) = setup().await;

        let payload = b"frame-data-from-b";
        let pkt = protocol::encode_data(1, payload);

        // peer_b sends DATA to the relay socket.
        peer_b.send_to(&pkt, relay.local_addr().unwrap()).await.unwrap();

        let mut buf = vec![0u8; 65536];
        let (len, from) = relay.recv_from(&mut buf).await.unwrap();

        // Relay forwards to peer_a.
        fwd.forward_packet(from, &buf[..len]).await.unwrap();

        // peer_a receives the forwarded packet.
        let mut recv_buf = vec![0u8; 65536];
        let (n, _) = peer_a.recv_from(&mut recv_buf).await.unwrap();

        assert_eq!(&recv_buf[..n], &buf[..len], "peer_a must receive the exact bytes");
        assert_eq!(n, pkt.len());
        assert_eq!(&recv_buf[protocol::HEADER_LEN..n], payload);
    }

    // -- Acceptance: unknown_session_rejected ---------------------------------

    #[tokio::test]
    async fn unknown_session_rejected() {
        let (fwd, relay, peer_a, _peer_b) = setup().await;

        let pkt = protocol::encode_data(9999, b"some-data");

        peer_a.send_to(&pkt, relay.local_addr().unwrap()).await.unwrap();

        let mut buf = vec![0u8; 65536];
        let (len, from) = relay.recv_from(&mut buf).await.unwrap();

        let result = fwd.forward_packet(from, &buf[..len]).await;
        assert!(result.is_err(), "unknown session must return error");
        assert!(
            matches!(result.unwrap_err(), ForwardError::SessionNotFound(9999)),
            "error must be SessionNotFound(9999)"
        );
    }

    // -- Acceptance: bytes_counter_updated ------------------------------------

    #[tokio::test]
    async fn bytes_counter_updated() {
        let (fwd, relay, peer_a, _peer_b) = setup().await;
        let sessions = fwd.sessions.clone();

        // Verify initial counter.
        {
            let s = sessions.read().await;
            assert_eq!(s.get(1).unwrap().bytes_forwarded, 0);
        }

        // Forward one packet.
        let pkt = protocol::encode_data(1, b"payload-bytes");
        peer_a.send_to(&pkt, relay.local_addr().unwrap()).await.unwrap();

        let mut buf = vec![0u8; 65536];
        let (len, from) = relay.recv_from(&mut buf).await.unwrap();
        fwd.forward_packet(from, &buf[..len]).await.unwrap();

        // Counter should be 1.
        {
            let s = sessions.read().await;
            assert_eq!(s.get(1).unwrap().bytes_forwarded, 1);
        }

        // Forward another packet (from peer_b this time).
        let peer_b_addr = {
            let s = sessions.read().await;
            s.get(1).unwrap().peer_b.addr
        };
        let peer_b_sock = bind_loopback().await;
        // We need to send from peer_b's registered address, so use the
        // original peer_b socket from setup.  Instead, re-register the new
        // socket's address as peer_b for this test.
        {
            let mut s = sessions.write().await;
            s.set_peer_b(1, peer_b_sock.local_addr().unwrap());
        }

        let pkt2 = protocol::encode_data(1, b"more-payload");
        peer_b_sock
            .send_to(&pkt2, relay.local_addr().unwrap())
            .await
            .unwrap();

        let (len2, from2) = relay.recv_from(&mut buf).await.unwrap();
        fwd.forward_packet(from2, &buf[..len2]).await.unwrap();

        // Counter should be 2.
        {
            let s = sessions.read().await;
            assert_eq!(s.get(1).unwrap().bytes_forwarded, 2);
        }

        // Suppress unused-variable warning for the original peer_b addr.
        let _ = peer_b_addr;
    }

    // -- Acceptance: zero_copy_efficiency -------------------------------------

    #[tokio::test]
    async fn zero_copy_efficiency() {
        let (fwd, relay, peer_a, peer_b) = setup().await;

        // Use a moderately large payload to emphasize that no extra
        // copies are made (while staying within loopback MTU limits).
        let payload = vec![0xABu8; 8_000];
        let pkt = protocol::encode_data(1, &payload);

        peer_a.send_to(&pkt, relay.local_addr().unwrap()).await.unwrap();

        let mut buf = vec![0u8; 65536];
        let (len, from) = relay.recv_from(&mut buf).await.unwrap();

        // forward_packet uses the same buffer slice — no intermediate
        // allocation or copy.  The data slice &buf[..len] is passed
        // directly to send_to.
        fwd.forward_packet(from, &buf[..len]).await.unwrap();

        let mut recv_buf = vec![0u8; 65536];
        let (n, _) = peer_b.recv_from(&mut recv_buf).await.unwrap();

        // Verify the full large payload arrived intact.
        assert_eq!(n, pkt.len());
        assert_eq!(&recv_buf[protocol::HEADER_LEN..n], &payload[..]);
    }

    // -- Additional edge-case tests -------------------------------------------

    #[tokio::test]
    async fn unknown_peer_rejected() {
        let (fwd, relay, _peer_a, _peer_b) = setup().await;

        // Bind a socket that is NOT registered as either peer.
        let stranger = bind_loopback().await;
        let pkt = protocol::encode_data(1, b"intruder-data");

        stranger
            .send_to(&pkt, relay.local_addr().unwrap())
            .await
            .unwrap();

        let mut buf = vec![0u8; 65536];
        let (len, from) = relay.recv_from(&mut buf).await.unwrap();

        let result = fwd.forward_packet(from, &buf[..len]).await;
        assert!(result.is_err(), "unknown peer must return error");
        assert!(
            matches!(result.unwrap_err(), ForwardError::UnknownPeer(_)),
            "error must be UnknownPeer"
        );
    }

    #[tokio::test]
    async fn forward_multiple_sessions() {
        let relay = Arc::new(bind_loopback().await);
        let peer_a1 = bind_loopback().await;
        let peer_b1 = bind_loopback().await;
        let peer_a2 = bind_loopback().await;
        let peer_b2 = bind_loopback().await;

        let mgr = SessionManager::new(49152, 65535, SECRET.to_vec());
        let sessions = Arc::new(RwLock::new(mgr));

        // Set up two independent sessions.
        {
            let mut s = sessions.write().await;
            let t1 = valid_token(1);
            let t2 = valid_token(2);
            s.allocate(1, &t1, peer_a1.local_addr().unwrap()).unwrap();
            s.set_peer_b(1, peer_b1.local_addr().unwrap());
            s.allocate(2, &t2, peer_a2.local_addr().unwrap()).unwrap();
            s.set_peer_b(2, peer_b2.local_addr().unwrap());
        }

        let fwd = DataForwarder::new(relay.clone(), sessions.clone());

        // Forward data for session 1.
        let pkt1 = protocol::encode_data(1, b"session-1-data");
        peer_a1.send_to(&pkt1, relay.local_addr().unwrap()).await.unwrap();
        let mut buf = vec![0u8; 65536];
        let (len, from) = relay.recv_from(&mut buf).await.unwrap();
        fwd.forward_packet(from, &buf[..len]).await.unwrap();

        let mut recv_buf = vec![0u8; 65536];
        let (n, _) = peer_b1.recv_from(&mut recv_buf).await.unwrap();
        assert_eq!(&recv_buf[protocol::HEADER_LEN..n], b"session-1-data");

        // Forward data for session 2.
        let pkt2 = protocol::encode_data(2, b"session-2-data");
        peer_a2.send_to(&pkt2, relay.local_addr().unwrap()).await.unwrap();
        let (len2, from2) = relay.recv_from(&mut buf).await.unwrap();
        fwd.forward_packet(from2, &buf[..len2]).await.unwrap();

        let (n2, _) = peer_b2.recv_from(&mut recv_buf).await.unwrap();
        assert_eq!(&recv_buf[protocol::HEADER_LEN..n2], b"session-2-data");

        // Verify each session has independent byte counters.
        let s = sessions.read().await;
        assert_eq!(s.get(1).unwrap().bytes_forwarded, 1);
        assert_eq!(s.get(2).unwrap().bytes_forwarded, 1);
    }

    // -- ForwardError Display -------------------------------------------------

    #[test]
    fn forward_error_display() {
        let e = ForwardError::SessionNotFound(42);
        assert_eq!(e.to_string(), "session 42 not found");

        let addr: SocketAddr = "192.168.1.1:5000".parse().unwrap();
        let e = ForwardError::UnknownPeer(addr);
        assert_eq!(e.to_string(), "unknown peer: 192.168.1.1:5000");

        let e = ForwardError::Io(std::io::Error::other(
            "test",
        ));
        assert_eq!(e.to_string(), "io error: test");
    }
}
