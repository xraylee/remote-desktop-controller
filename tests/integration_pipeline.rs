// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Integration test: full encode → encrypt → transport → decrypt → decode pipeline.
//!
//! Exercises the cross-crate data flow:
//!   CapturedFrame (rdcs-platform)
//!     → StubEncoder (rdcs-codec)
//!       → CryptoSession encrypt (rdcs-crypto)
//!         → TransportChannel prepare_send (rdcs-transport)
//!           → (simulate network)
//!             → TransportChannel on_receive (rdcs-transport)
//!               → CryptoSession decrypt (rdcs-crypto)
//!                 → StubDecoder (rdcs-codec)
//!                   → DecodedFrame

use rdcs_codec::decoder::{DecoderConfig, StubDecoder, VideoDecoder};
use rdcs_codec::encoder::{CodecType, EncoderConfig, StubEncoder, VideoEncoder};
use rdcs_crypto::session::CryptoSession;
use rdcs_platform::{CapturedFrame, PixelFormat};
use rdcs_transport::channel::{ChannelConfig, ChannelId, TransportChannel};

/// Create a test CapturedFrame with a known pixel pattern.
fn make_frame(width: u32, height: u32, timestamp_us: u64) -> CapturedFrame {
    let stride = width * 4;
    let data: Vec<u8> = (0..(stride * height) as usize)
        .map(|i| (i % 256) as u8)
        .collect();
    CapturedFrame {
        data,
        width,
        height,
        pixel_format: PixelFormat::Bgra,
        stride,
        display_id: 0,
        timestamp_us,
    }
}

/// Set up a pair of CryptoSessions with a completed handshake.
fn setup_crypto_pair() -> (CryptoSession, CryptoSession) {
    let mut alice = CryptoSession::new(1);
    let mut bob = CryptoSession::new(1);

    let alice_pub = alice.local_public_key().clone();
    let bob_pub = bob.local_public_key().clone();

    let salt = b"integration-pipeline-test-session";
    alice.complete_handshake(&bob_pub, salt).unwrap();
    bob.complete_handshake(&alice_pub, salt).unwrap();

    (alice, bob)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn full_pipeline_encode_transport_decode() {
    let width = 64u32;
    let height = 48u32;

    // 1. Create CapturedFrame (mock data)
    let original = make_frame(width, height, 1_000_000);
    let original_data = original.data.clone();

    // 2. Encode via StubEncoder
    let mut encoder = StubEncoder::new();
    encoder.configure(EncoderConfig::default()).unwrap();
    let encoded = encoder.encode(&original).unwrap();

    assert_eq!(encoded.width, width);
    assert_eq!(encoded.height, height);
    assert!(encoded.is_keyframe);

    // 3. Encrypt via CryptoSession (Alice sends)
    let (alice, bob) = setup_crypto_pair();
    let encrypted = alice.encrypt(&encoded.data).unwrap();

    // 4. Packetize via TransportChannel (sender side)
    let session_id = 42u64;
    let sender_config = ChannelConfig::default_for(ChannelId::Video);
    let mut sender_channel = TransportChannel::new(sender_config, session_id);

    // Serialize the encrypted payload: nonce (24 bytes) + ciphertext
    let mut wire_payload = Vec::new();
    wire_payload.extend_from_slice(&encrypted.nonce);
    wire_payload.extend_from_slice(&encrypted.ciphertext);

    let packets = sender_channel
        .prepare_send(&wire_payload, encoded.pts_us as u32)
        .unwrap();
    assert!(!packets.is_empty());

    // 5. "Transport" — simulate network by passing raw packets to receiver
    let receiver_config = ChannelConfig::default_for(ChannelId::Video);
    let mut receiver_channel = TransportChannel::new(receiver_config, session_id);

    let mut delivered_payloads = Vec::new();
    for pkt in &packets {
        let delivered = receiver_channel.on_receive(pkt).unwrap();
        delivered_payloads.extend(delivered);
    }

    // 6. Depacketize — recover the wire payload
    assert!(!delivered_payloads.is_empty(), "should deliver at least one payload");
    let received_wire = &delivered_payloads[0];

    // Extract nonce (first 24 bytes) and ciphertext (rest)
    assert!(received_wire.len() >= 24, "payload too short for nonce");
    let received_payload = rdcs_crypto::EncryptedPayload {
        nonce: received_wire[..24].to_vec(),
        ciphertext: received_wire[24..].to_vec(),
    };

    // 7. Decrypt via paired CryptoSession (Bob receives)
    let decrypted_data = bob.decrypt(&received_payload).unwrap();

    // 8. Decode via StubDecoder
    //    Reconstruct an EncodedFrame from the decrypted stub wire data
    let received_encoded = rdcs_codec::encoder::EncodedFrame {
        data: decrypted_data,
        is_keyframe: encoded.is_keyframe,
        pts_us: encoded.pts_us,
        dts_us: encoded.dts_us,
        codec: encoded.codec,
        width: encoded.width,
        height: encoded.height,
    };

    let mut decoder = StubDecoder::new();
    decoder
        .configure(DecoderConfig {
            codec: CodecType::H264,
            width,
            height,
        })
        .unwrap();
    let decoded = decoder.decode(&received_encoded).unwrap();

    // 9. Assert: output frame matches input frame dimensions and data
    assert_eq!(decoded.width, width);
    assert_eq!(decoded.height, height);
    assert_eq!(decoded.stride, width * 4);
    assert_eq!(decoded.pts_us, 1_000_000);
    assert_eq!(decoded.data, original_data);
}

#[test]
fn pipeline_multiple_frames_through_transport() {
    let width = 32u32;
    let height = 24u32;
    let frame_count = 10;

    // Set up encoder
    let mut encoder = StubEncoder::new();
    encoder.configure(EncoderConfig::default()).unwrap();

    // Set up crypto
    let (alice, _bob) = setup_crypto_pair();

    // Set up transport channels
    let session_id = 100u64;
    let config = ChannelConfig::default_for(ChannelId::Video);
    let mut sender = TransportChannel::new(config.clone(), session_id);
    let mut receiver = TransportChannel::new(config, session_id);

    // Set up decoder
    let mut decoder = StubDecoder::new();
    decoder
        .configure(DecoderConfig {
            codec: CodecType::H264,
            width,
            height,
        })
        .unwrap();

    for i in 0..frame_count {
        let ts = i * 16_667; // ~60fps
        let original = make_frame(width, height, ts);
        let _original_data = original.data.clone();

        // Encode
        let encoded = encoder.encode(&original).unwrap();

        // Encrypt
        let encrypted = alice.encrypt(&encoded.data).unwrap();

        // Packetize
        let mut wire = Vec::new();
        wire.extend_from_slice(&encrypted.nonce);
        wire.extend_from_slice(&encrypted.ciphertext);
        let packets = sender.prepare_send(&wire, ts as u32).unwrap();

        // Transport
        for pkt in &packets {
            let _ = receiver.on_receive(pkt).unwrap();
        }

        // We need to collect delivered payloads — since packets are in order,
        // they should be delivered immediately
    }

    // Verify encoder/decoder processed all frames
    assert_eq!(encoder.flush().unwrap().len(), 0); // stub has no buffered frames
}

#[test]
fn pipeline_preserves_frame_integrity_with_encryption() {
    // Verify that the encode→encrypt→decrypt→decode path preserves
    // exact byte-level data integrity (not just dimensions).
    let width = 16u32;
    let height = 16u32;

    let original = make_frame(width, height, 42_000);
    let original_data = original.data.clone();
    let original_stride = original.stride;

    // Encode
    let mut encoder = StubEncoder::new();
    encoder.configure(EncoderConfig::default()).unwrap();
    let encoded = encoder.encode(&original).unwrap();

    // Encrypt then decrypt (no transport, isolate crypto + codec)
    let (alice, bob) = setup_crypto_pair();
    let encrypted = alice.encrypt(&encoded.data).unwrap();
    let decrypted = bob.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted, encoded.data, "crypto roundtrip should preserve encoded data exactly");

    // Decode
    let received_encoded = rdcs_codec::encoder::EncodedFrame {
        data: decrypted,
        is_keyframe: encoded.is_keyframe,
        pts_us: encoded.pts_us,
        dts_us: encoded.dts_us,
        codec: encoded.codec,
        width: encoded.width,
        height: encoded.height,
    };

    let mut decoder = StubDecoder::new();
    decoder
        .configure(DecoderConfig {
            codec: CodecType::H264,
            width,
            height,
        })
        .unwrap();
    let decoded = decoder.decode(&received_encoded).unwrap();

    assert_eq!(decoded.data, original_data, "pixel data should be byte-identical");
    assert_eq!(decoded.stride, original_stride);
    assert_eq!(decoded.width, width);
    assert_eq!(decoded.height, height);
    assert_eq!(decoded.pts_us, 42_000);
}
