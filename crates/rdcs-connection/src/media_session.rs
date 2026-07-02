// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! [`MediaSession`] — a library-level media connection over a real WebRTC
//! ICE P2P DataChannel.
//!
//! This lifts the handshake glue that previously lived only in the
//! `video_e2e_test` example into a reusable type, with a **signaling-style
//! interface**: the two peers never hold references to each other. Instead the
//! offerer produces an [`SdpOffer`], the answerer consumes it and produces an
//! [`SdpAnswer`], and the offerer consumes that — each of which is a plain
//! `serde` struct that can be shipped verbatim over `rdcs-signaling`'s
//! `ice_offer` / `ice_answer` messages. That property is what makes an
//! integration test a genuine end-to-end proof rather than an in-process
//! shortcut.
//!
//! The frame wire format (8-byte [`FrameHeader`] + chunk, reassembled by
//! [`FrameReassembler`]) is encapsulated here so callers (FFI, tests, the
//! desktop app) send and receive whole frames without re-implementing chunking.
//!
//! # Example (single process, two sessions, messages exchanged by value)
//! ```no_run
//! # use rdcs_connection::MediaSession;
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let stun = vec!["stun:stun.l.google.com:19302".to_string()];
//! let mut offerer = MediaSession::new_offerer(stun.clone()).await?;
//! let mut answerer = MediaSession::new_answerer(stun).await?;
//!
//! let offer = offerer.create_local_offer()?;           // -> ship as ice_offer
//! let answer = answerer.accept_offer(&offer)?;          // -> ship as ice_answer
//! offerer.accept_answer(answer)?;
//!
//! offerer.wait_connected(std::time::Duration::from_secs(10)).await?;
//! answerer.wait_connected(std::time::Duration::from_secs(10)).await?;
//! # Ok(())
//! # }
//! ```

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tracing::debug;

use crate::frame_reassembler::{FrameHeader, FrameReassembler};
use crate::ice::{IceAgent, IceState, SdpAnswer, SdpOffer};
use crate::real_ice_agent::RealIceAgent;
use crate::video_channel::VideoChannel;
use crate::ConnectionError;

/// Max DataChannel message payload minus the 8-byte frame header.
const CHUNK_SIZE: usize = 16_384 - FrameHeader::SIZE;

/// Which side of the handshake this session drives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaRole {
    /// Creates the DataChannel and the SDP offer.
    Offerer,
    /// Waits for the remote DataChannel and answers the offer.
    Answerer,
}

/// A media connection over a real ICE P2P DataChannel.
pub struct MediaSession {
    agent: RealIceAgent,
    role: MediaRole,
    /// Built lazily once the DataChannel exists (after connection).
    video: Option<VideoChannel>,
}

impl MediaSession {
    /// Create the offerer side (creates the `rdcs-control` DataChannel).
    pub async fn new_offerer(ice_servers: Vec<String>) -> Result<Self, ConnectionError> {
        Ok(Self {
            agent: RealIceAgent::new_with_options(ice_servers, true).await?,
            role: MediaRole::Offerer,
            video: None,
        })
    }

    /// Create the answerer side (waits for the remote DataChannel).
    pub async fn new_answerer(ice_servers: Vec<String>) -> Result<Self, ConnectionError> {
        Ok(Self {
            agent: RealIceAgent::new_with_options(ice_servers, false).await?,
            role: MediaRole::Answerer,
            video: None,
        })
    }

    /// This session's role.
    pub fn role(&self) -> MediaRole {
        self.role
    }

    // ── Signaling-style handshake ──────────────────────────────────────

    /// Offerer: gather candidates and produce the SDP offer to ship over
    /// signaling as `ice_offer`.
    pub fn create_local_offer(&mut self) -> Result<SdpOffer, ConnectionError> {
        debug_assert_eq!(self.role, MediaRole::Offerer);
        self.agent.gather_candidates()?;
        self.agent.create_offer()
    }

    /// Answerer: consume the remote offer and produce the SDP answer to ship
    /// back as `ice_answer`. The returned answer already carries this side's
    /// gathered candidates.
    pub fn accept_offer(&mut self, offer: &SdpOffer) -> Result<SdpAnswer, ConnectionError> {
        debug_assert_eq!(self.role, MediaRole::Answerer);

        // Must set the remote offer BEFORE gathering, so webrtc-rs produces an
        // answer (not another offer) for our local description.
        self.agent.set_remote_offer(offer)?;
        let candidates = self.agent.gather_candidates()?;
        let (ufrag, pwd, fingerprint) = self.agent.get_local_credentials_with_fingerprint()?;

        // Learn the offerer's candidates now (they rode on the offer).
        self.agent.set_remote_candidates(offer.candidates.clone())?;

        Ok(SdpAnswer {
            session_id: offer.session_id.clone(),
            ufrag,
            pwd,
            fingerprint,
            candidates,
        })
    }

    /// Offerer: consume the remote answer (with its candidates) and finish the
    /// handshake.
    pub fn accept_answer(&mut self, answer: SdpAnswer) -> Result<(), ConnectionError> {
        debug_assert_eq!(self.role, MediaRole::Offerer);
        let remote_candidates = answer.candidates.clone();
        self.agent.handle_answer(answer)?;
        self.agent.set_remote_candidates(remote_candidates)?;
        Ok(())
    }

    // ── Connection lifecycle ───────────────────────────────────────────

    /// Current ICE connection state.
    pub fn connection_state(&self) -> IceState {
        self.agent.connection_state()
    }

    /// Poll until ICE reaches `Connected`, or error on `Failed`/timeout.
    pub async fn wait_connected(&self, timeout: Duration) -> Result<(), ConnectionError> {
        let start = tokio::time::Instant::now();
        loop {
            match self.agent.connection_state() {
                IceState::Connected => return Ok(()),
                IceState::Failed => return Err(ConnectionError::PeerUnreachable(
                    "ICE connection failed".into(),
                )),
                _ => {}
            }
            if start.elapsed() > timeout {
                return Err(ConnectionError::Timeout);
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Poll until the DataChannel is open, then build the [`VideoChannel`].
    /// Must be called after [`wait_connected`].
    pub async fn open_media(&mut self, timeout: Duration) -> Result<(), ConnectionError> {
        let dc = {
            let start = tokio::time::Instant::now();
            loop {
                if let Ok(dc) = self.agent.get_data_channel() {
                    use webrtc::data_channel::data_channel_state::RTCDataChannelState;
                    if dc.ready_state() == RTCDataChannelState::Open {
                        break dc;
                    }
                }
                if start.elapsed() > timeout {
                    return Err(ConnectionError::Timeout);
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        };
        self.video = Some(VideoChannel::new(dc));
        Ok(())
    }

    // ── Media I/O ──────────────────────────────────────────────────────

    /// Send one encoded frame, chunked with an 8-byte [`FrameHeader`].
    /// Requires [`open_media`] to have succeeded.
    pub async fn send_frame(
        &self,
        frame_id: u32,
        frame_data: &[u8],
        is_keyframe: bool,
    ) -> Result<(), ConnectionError> {
        let video = self.video.as_ref().ok_or(ConnectionError::Reset)?;
        let total_chunks = frame_data.len().div_ceil(CHUNK_SIZE).max(1) as u8;

        for (chunk_index, chunk) in frame_data.chunks(CHUNK_SIZE).enumerate() {
            let header = FrameHeader {
                frame_id,
                is_keyframe,
                chunk_index: chunk_index as u8,
                total_chunks,
            };
            let mut msg = header.serialize().to_vec();
            msg.extend_from_slice(chunk);
            video.send_frame(&msg).await?;
        }
        debug!("sent frame {frame_id} ({} bytes)", frame_data.len());
        Ok(())
    }

    /// Register a callback invoked with each fully-reassembled frame:
    /// `(frame_id, frame_bytes, is_keyframe)`. Requires [`open_media`].
    pub fn on_frame<F>(&self, callback: F) -> Result<(), ConnectionError>
    where
        F: Fn(u32, Vec<u8>, bool) + Send + Sync + 'static,
    {
        let video = self.video.as_ref().ok_or(ConnectionError::Reset)?;
        let reassembler = Arc::new(Mutex::new(FrameReassembler::new(16)));
        let callback = Arc::new(callback);

        video.on_message(move |chunk| {
            if chunk.len() < FrameHeader::SIZE {
                return;
            }
            let header = match FrameHeader::deserialize(&chunk[..FrameHeader::SIZE]) {
                Ok(h) => h,
                Err(_) => return,
            };
            let data = chunk[FrameHeader::SIZE..].to_vec();
            let reassembler = reassembler.clone();
            let callback = callback.clone();
            tokio::spawn(async move {
                if let Some((id, frame, is_kf)) =
                    reassembler.lock().await.add_chunk(header, data)
                {
                    callback(id, frame, is_kf);
                }
            });
        });
        Ok(())
    }
}
