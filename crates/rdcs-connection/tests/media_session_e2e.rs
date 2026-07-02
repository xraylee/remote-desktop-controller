// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Signaling-style end-to-end test for [`MediaSession`].
//!
//! Two `MediaSession`s connect using ONLY serialized offer/answer JSON — the
//! same bytes that would travel over `rdcs-signaling`'s `ice_offer`/
//! `ice_answer` messages. Neither session holds a reference to the other, so a
//! pass proves the handshake is genuinely transportable over signaling, not an
//! in-process shortcut. Real H.264 frames are then streamed offerer→answerer
//! over the ICE DataChannel and decoded.
//!
//! Requires the software (OpenH264) codec — the VideoToolbox hardware decoder
//! currently SIGSEGVs on decode (tracked separately). Run with:
//!   cargo test -p rdcs-connection --features software-encoder \
//!     --test media_session_e2e -- --nocapture
#![cfg(feature = "software-encoder")]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use rdcs_codec::platform::{NativeVideoDecoder, NativeVideoEncoder};
use rdcs_codec::types::{VideoCodec, VideoResolution};
use rdcs_connection::MediaSession;
use rdcs_platform::{CapturedFrame, PixelFormat};
use tokio::sync::Mutex;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 360;
const FPS: u32 = 30;
const BITRATE: u32 = 1_000_000;
const NUM_FRAMES: u32 = 15;

/// Generate a deterministic BGRA test frame with a moving band so successive
/// frames differ (exercises inter-frame encoding, not just a static keyframe).
fn generate_test_frame(frame_id: u32) -> CapturedFrame {
    let mut data = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
    let band = (frame_id * 12) % HEIGHT;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let offset = ((y * WIDTH + x) * 4) as usize;
            let r = ((x * 255) / WIDTH) as u8;
            let g = if y == band { 255 } else { ((y * 255) / HEIGHT) as u8 };
            let b = (frame_id.wrapping_mul(7) & 0xFF) as u8;
            data[offset] = b;
            data[offset + 1] = g;
            data[offset + 2] = r;
            data[offset + 3] = 255;
        }
    }
    CapturedFrame {
        data: data.into(),
        width: WIDTH,
        height: HEIGHT,
        pixel_format: PixelFormat::Bgra,
        stride: WIDTH * 4,
        display_id: 0,
        timestamp_us: 0,
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn media_session_streams_frames_over_serialized_signaling() {
    // Loopback host candidates connect without public STUN; keep the list so
    // the agents still gather host candidates.
    let ice_servers = vec!["stun:stun.l.google.com:19302".to_string()];

    let mut offerer = MediaSession::new_offerer(ice_servers.clone())
        .await
        .expect("create offerer");
    let mut answerer = MediaSession::new_answerer(ice_servers)
        .await
        .expect("create answerer");

    // ── Handshake — cross ONLY serialized JSON between the two sessions ──
    let offer = offerer.create_local_offer().expect("create offer");
    let offer_wire = serde_json::to_string(&offer).expect("serialize offer");
    let offer_rx = serde_json::from_str(&offer_wire).expect("deserialize offer");

    let answer = answerer.accept_offer(&offer_rx).expect("accept offer");
    let answer_wire = serde_json::to_string(&answer).expect("serialize answer");
    let answer_rx = serde_json::from_str(&answer_wire).expect("deserialize answer");

    offerer.accept_answer(answer_rx).expect("accept answer");

    // ── Connect ─────────────────────────────────────────────────────────
    let connect = Duration::from_secs(15);
    offerer.wait_connected(connect).await.expect("offerer connect");
    answerer.wait_connected(connect).await.expect("answerer connect");

    let open = Duration::from_secs(5);
    offerer.open_media(open).await.expect("offerer media");
    answerer.open_media(open).await.expect("answerer media");

    // ── Receiver: decode each reassembled frame ─────────────────────────
    let received = Arc::new(AtomicUsize::new(0));
    let decoded_ok = Arc::new(AtomicUsize::new(0));
    let decoder = Arc::new(Mutex::new(
        NativeVideoDecoder::new(VideoCodec::H264).expect("decoder"),
    ));

    {
        let received = received.clone();
        let decoded_ok = decoded_ok.clone();
        let decoder = decoder.clone();
        answerer
            .on_frame(move |_id, frame, _is_kf| {
                received.fetch_add(1, Ordering::SeqCst);
                let decoder = decoder.clone();
                let decoded_ok = decoded_ok.clone();
                tokio::spawn(async move {
                    if decoder.lock().await.decode_to_captured_frame(&frame).is_ok() {
                        decoded_ok.fetch_add(1, Ordering::SeqCst);
                    }
                });
            })
            .expect("register on_frame");
    }

    // ── Sender: encode + send real H.264 frames ─────────────────────────
    let mut encoder = NativeVideoEncoder::new(
        VideoCodec::H264,
        VideoResolution::Custom(WIDTH, HEIGHT),
        FPS,
        BITRATE,
    )
    .expect("encoder");

    for frame_id in 0..NUM_FRAMES {
        if frame_id == 0 {
            encoder.request_keyframe();
        }
        let captured = generate_test_frame(frame_id);
        let encoded = encoder.encode_captured_frame(&captured).expect("encode");
        if encoded.is_empty() {
            continue; // encoder may buffer; skip empties
        }
        offerer
            .send_frame(frame_id, &encoded, frame_id == 0)
            .await
            .expect("send frame");
        tokio::time::sleep(Duration::from_millis(30)).await;
    }

    // Allow in-flight chunks + decode tasks to drain.
    tokio::time::sleep(Duration::from_secs(2)).await;

    let got = received.load(Ordering::SeqCst);
    let ok = decoded_ok.load(Ordering::SeqCst);
    println!("frames received: {got}, decoded ok: {ok} (sent up to {NUM_FRAMES})");

    // At least the keyframe plus most inter frames must arrive and decode.
    assert!(got >= 1, "no frames received over the DataChannel");
    assert!(
        ok >= 1,
        "no frames decoded (received {got}, decoded {ok})"
    );
    // Loss should be minimal on loopback — require the large majority through.
    assert!(
        got * 100 >= (NUM_FRAMES as usize) * 80,
        "too many frames lost: received {got}/{NUM_FRAMES}"
    );
}
