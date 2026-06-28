// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Integration test: reliable UDP transport layer end-to-end.
//!
//! Exercises the rdcs-transport crate components working together:
//!   - TransportChannel (unified send/receive with sequencing, NACK, FEC)
//!   - CongestionController (GCC-inspired AIMD adaptation)
//!   - FecEncoder / FecDecoder (XOR-based forward error correction)
//!   - NackTracker (selective NACK retransmission)
//!   - SendSequencer / ReceiveSequencer (sequence management and reordering)
//!   - Packet encode/decode (wire format roundtrip)

use rdcs_transport::channel::{ChannelConfig, ChannelId, TransportChannel};
use rdcs_transport::congestion::{CongestionController, CongestionState};
use rdcs_transport::fec::{self, FecDecoder, FecEncoder};
use rdcs_transport::packet::{decode_packet, PacketType, VERSION};
use rdcs_transport::sequencer::ReceiveSequencer;

// ---------------------------------------------------------------------------
// TransportChannel send/receive roundtrip
// ---------------------------------------------------------------------------

#[test]
fn transport_channel_send_receive_roundtrip() {
    // Sender and receiver share the same session ID and channel config.
    let session_id = 7777u64;
    let config = ChannelConfig::default_for(ChannelId::Video);

    let mut sender = TransportChannel::new(config.clone(), session_id);
    let mut receiver = TransportChannel::new(config, session_id);

    // Prepare a payload that simulates an encoded video frame.
    let payload: Vec<u8> = (0..200).map(|i| (i % 251) as u8).collect();
    let timestamp = 16_667u32; // one frame at ~60fps

    // Encode on the sender side — produces one or more wire packets.
    let wire_packets = sender.prepare_send(&payload, timestamp).unwrap();
    assert!(!wire_packets.is_empty(), "prepare_send must produce at least one packet");

    // Simulate a perfect network: pass every wire packet to the receiver in order.
    let mut delivered = Vec::new();
    for pkt in &wire_packets {
        let payloads = receiver.on_receive(pkt).unwrap();
        delivered.extend(payloads);
    }

    // The receiver should deliver the original payload to the application.
    assert!(!delivered.is_empty(), "receiver should deliver at least one payload");
    assert_eq!(delivered[0], payload, "delivered payload must match the original");

    // Verify the wire packet header is well-formed.
    let (header, wire_payload) = decode_packet(&wire_packets[0]).unwrap();
    assert_eq!(header.version, VERSION);
    assert_eq!(header.packet_type, PacketType::Data);
    assert_eq!(header.session_id, session_id);
    assert_eq!(header.sequence, 0, "first packet should have sequence 0");
    assert_eq!(header.timestamp, timestamp);
    assert_eq!(wire_payload, payload.as_slice());

    // Send a second payload and verify monotonically increasing sequence numbers.
    let payload2: Vec<u8> = vec![0xAB; 64];
    let wire_packets2 = sender.prepare_send(&payload2, timestamp * 2).unwrap();
    let (header2, _) = decode_packet(&wire_packets2[0]).unwrap();
    assert_eq!(header2.sequence, 1, "second packet should have sequence 1");

    let mut delivered2 = Vec::new();
    for pkt in &wire_packets2 {
        let payloads = receiver.on_receive(pkt).unwrap();
        delivered2.extend(payloads);
    }
    assert_eq!(delivered2[0], payload2);
}

// ---------------------------------------------------------------------------
// NACK retransmission flow
// ---------------------------------------------------------------------------

#[test]
fn transport_nack_retransmission_flow() {
    let session_id = 42u64;
    let config = ChannelConfig::default_for(ChannelId::Video);

    let mut sender = TransportChannel::new(config.clone(), session_id);
    let mut receiver = TransportChannel::new(config, session_id);

    // Sender creates 5 packets.
    let mut sent_packets: Vec<Vec<Vec<u8>>> = Vec::new();
    for i in 0u8..5 {
        let payload = vec![i; 32];
        let pkts = sender.prepare_send(&payload, i as u32 * 100).unwrap();
        sent_packets.push(pkts);
    }

    // Receiver gets packets 0, 1, 3, 4 — packet 2 is lost in transit.
    receiver.on_receive(&sent_packets[0][0]).unwrap(); // seq 0
    receiver.on_receive(&sent_packets[1][0]).unwrap(); // seq 1
    // sent_packets[2] dropped
    receiver.on_receive(&sent_packets[3][0]).unwrap(); // seq 3 — triggers gap detection
    receiver.on_receive(&sent_packets[4][0]).unwrap(); // seq 4

    // Receiver should detect the gap and generate a NACK for seq 2.
    let nacks = receiver.poll_nacks();
    assert!(
        nacks.contains(&2),
        "NACK list should contain missing seq 2, got: {nacks:?}"
    );

    // Sender retransmits the lost packet (seq 2) upon receiving the NACK.
    // We simulate this by feeding the original wire packet for seq 2 to the receiver.
    let retransmit_payloads = receiver.on_receive(&sent_packets[2][0]).unwrap();

    // After retransmission, the gap is filled and the buffered packets are delivered.
    // Packet 2 arrives, which should trigger delivery of packets 2, 3, and 4 in order.
    assert!(
        !retransmit_payloads.is_empty(),
        "filling the gap should deliver buffered packets"
    );

    // Verify that the delivered payloads include the retransmitted packet's data.
    let has_retransmitted = retransmit_payloads.iter().any(|p| p == &vec![2u8; 32]);
    assert!(has_retransmitted, "retransmitted packet 2 data should be delivered");

    // After the gap is filled, subsequent NACK polls should not include seq 2.
    let nacks_after = receiver.poll_nacks();
    assert!(
        !nacks_after.contains(&2),
        "seq 2 should no longer appear in NACK list after retransmission"
    );
}

// ---------------------------------------------------------------------------
// FEC recovery of lost packet
// ---------------------------------------------------------------------------

#[test]
fn transport_fec_recovery_of_lost_packet() {
    // Use a small 4+1 FEC group for a focused test.
    let data_count = 4;
    let parity_count = 1;

    let mut encoder = FecEncoder::new(data_count, parity_count);
    let decoder = FecDecoder::new(data_count, parity_count);

    // Create 4 data packets with distinct, recognizable content.
    let packets: Vec<Vec<u8>> = (0u8..4)
        .map(|i| vec![i.wrapping_mul(37).wrapping_add(11); 64])
        .collect();

    // Feed all 4 packets into the FEC encoder to produce repair packets.
    let mut repairs: Option<Vec<Vec<u8>>> = None;
    for pkt in &packets {
        if let Some(r) = encoder.feed(pkt) {
            repairs = Some(r);
        }
    }
    let repairs = repairs.expect("FEC encoder should produce repairs after group is complete");
    assert_eq!(repairs.len(), parity_count);

    // Simulate losing packet at index 2 (third data packet).
    let mut received: Vec<Option<Vec<u8>>> = packets.iter().map(|p| Some(p.clone())).collect();
    received[2] = None; // lost

    // Attempt recovery using the FEC decoder.
    let repair_opts: Vec<Option<Vec<u8>>> = repairs.into_iter().map(Some).collect();
    let recovered = decoder
        .decode_group(&received, &repair_opts)
        .expect("FEC should recover one lost packet in a 4+1 group");

    assert_eq!(recovered.len(), data_count);
    for (i, pkt) in recovered.iter().enumerate() {
        assert_eq!(
            pkt, &packets[i],
            "recovered packet {i} must match the original"
        );
    }

    // Also verify the convenience `recover()` function for single-repair groups.
    // Lose packet 0, recover from remaining [1, 2, 3] + repair.
    let remaining: Vec<Vec<u8>> = packets[1..].to_vec();

    // Re-encode with a fresh encoder to obtain the repair packet cleanly.
    let mut enc2 = FecEncoder::new(data_count, parity_count);
    let mut repair_pkt = Vec::new();
    for pkt in &packets {
        if let Some(r) = enc2.feed(pkt) {
            repair_pkt = r;
        }
    }

    let recovered_single = fec::recover(&remaining, &repair_pkt[0]);
    assert_eq!(
        recovered_single, packets[0],
        "fec::recover should reconstruct the lost packet from remaining + repair"
    );
}

// ---------------------------------------------------------------------------
// Congestion controller adaptation
// ---------------------------------------------------------------------------

#[test]
fn transport_congestion_controller_adaptation() {
    let mut cc = CongestionController::new();

    // Initial state: SlowStart at default bitrate (2 Mbps).
    assert_eq!(cc.state(), CongestionState::SlowStart);
    let initial_bitrate = cc.target_bitrate();
    assert_eq!(initial_bitrate, 2_000_000);

    // Simulate steady ACKs to move through SlowStart into CongestionAvoidance.
    // ssthresh = 64, cwnd starts at 10, each ACK increments by 1.
    for _ in 0..54 {
        cc.on_ack(5_000); // 5ms RTT
    }
    assert_eq!(cc.state(), CongestionState::CongestionAvoidance);
    assert_eq!(cc.rtt_us(), 5_000);

    // Simulate a loss event via on_loss (window-based).
    let cwnd_before_loss = cc.window();
    cc.on_loss();
    assert_eq!(cc.state(), CongestionState::Recovery);
    assert!(
        cc.window() < cwnd_before_loss,
        "congestion window should shrink after loss: before={}, after={}",
        cwnd_before_loss,
        cc.window()
    );

    // Recovery: a single ACK transitions back to CongestionAvoidance.
    cc.on_ack(5_000);
    assert_eq!(cc.state(), CongestionState::CongestionAvoidance);

    // Now exercise the bitrate-based (GCC-style) path.
    // Start with a known bitrate and simulate sustained 15% loss over 4 rounds.
    let bitrate_stable = cc.target_bitrate();
    for _ in 0..4 {
        cc.on_round(0.15);
    }
    let bitrate_after_loss = cc.target_bitrate();

    // After 4 rounds of 15% loss: factor = 0.85^4 ~ 0.522
    assert!(
        bitrate_after_loss < bitrate_stable,
        "bitrate should decrease under sustained loss: before={bitrate_stable}, after={bitrate_after_loss}"
    );
    assert!(
        bitrate_after_loss >= 100_000,
        "bitrate should not drop below minimum (100 Kbps), got {bitrate_after_loss}"
    );

    // Simulate recovery: 10 consecutive rounds with zero loss.
    let mut prev = bitrate_after_loss;
    for _ in 0..10 {
        cc.on_round(0.0);
        let current = cc.target_bitrate();
        assert!(
            current >= prev,
            "bitrate should not decrease during zero-loss recovery: prev={prev}, current={current}"
        );
        prev = current;
    }
    assert!(
        prev > bitrate_after_loss,
        "bitrate should grow during recovery period"
    );
    assert_eq!(
        cc.state(),
        CongestionState::CongestionAvoidance,
        "should be in CongestionAvoidance after recovery"
    );
}

// ---------------------------------------------------------------------------
// Out-of-order delivery and reordering
// ---------------------------------------------------------------------------

#[test]
fn transport_out_of_order_delivery_reordering() {
    // Use ReceiveSequencer directly to test buffering and reordering behavior.
    let mut recv_seq = ReceiveSequencer::new(100);

    // Packets arrive out of order: 2, 0, 4, 1, 3
    // Expected delivery order after reordering: 0, 1, 2, 3, 4

    // Packet 2 arrives first — buffered, nothing delivered.
    let out = recv_seq.insert(2, vec![0x22]);
    assert!(out.is_empty(), "packet 2 ahead of expected 0, should be buffered");
    assert_eq!(recv_seq.buffered_count(), 1);

    // Packet 0 arrives — expected seq, delivered immediately.
    // This also drains any contiguous buffered packets, but 2 is not contiguous
    // (1 is still missing).
    let out = recv_seq.insert(0, vec![0x00]);
    assert_eq!(out, vec![vec![0x00]], "packet 0 should be delivered immediately");
    assert_eq!(recv_seq.expected(), 1);

    // Packet 4 arrives — buffered (gap at 1 and 3).
    let out = recv_seq.insert(4, vec![0x44]);
    assert!(out.is_empty());
    assert_eq!(recv_seq.buffered_count(), 2); // {2, 4}

    // Packet 1 arrives — fills gap, delivers 1 and then 2 (buffered, contiguous).
    let out = recv_seq.insert(1, vec![0x11]);
    assert_eq!(
        out,
        vec![vec![0x11], vec![0x22]],
        "filling seq 1 should deliver seq 1 and drain buffered seq 2"
    );
    assert_eq!(recv_seq.expected(), 3);
    assert_eq!(recv_seq.buffered_count(), 1); // {4} remains

    // Verify gap report shows seq 3 is still missing.
    let gaps = recv_seq.gap_report();
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0], 3..4);

    // Packet 3 arrives — fills gap, delivers 3 and then 4 (buffered, contiguous).
    let out = recv_seq.insert(3, vec![0x33]);
    assert_eq!(
        out,
        vec![vec![0x33], vec![0x44]],
        "filling seq 3 should deliver seq 3 and drain buffered seq 4"
    );
    assert_eq!(recv_seq.expected(), 5);
    assert_eq!(recv_seq.buffered_count(), 0);

    // No gaps remain.
    let gaps = recv_seq.gap_report();
    assert!(gaps.is_empty());
}

// ---------------------------------------------------------------------------
// Multi-channel isolation
// ---------------------------------------------------------------------------

#[test]
fn transport_multi_channel_isolation() {
    let session_id = 5555u64;

    // Create separate TransportChannels for Video and Audio.
    let video_config = ChannelConfig::default_for(ChannelId::Video);
    let audio_config = ChannelConfig::default_for(ChannelId::Audio);

    let mut video_sender = TransportChannel::new(video_config.clone(), session_id);
    let mut video_receiver = TransportChannel::new(video_config, session_id);

    let mut audio_sender = TransportChannel::new(audio_config.clone(), session_id);
    let mut audio_receiver = TransportChannel::new(audio_config, session_id);

    // Send video frame.
    let video_payload: Vec<u8> = (0..128).map(|i| (i % 200) as u8).collect();
    let video_packets = video_sender.prepare_send(&video_payload, 1000).unwrap();

    // Send audio frame.
    let audio_payload: Vec<u8> = vec![0xAA; 48];
    let audio_packets = audio_sender.prepare_send(&audio_payload, 1000).unwrap();

    // Verify that the wire packets carry the correct channel metadata.
    let (v_header, _) = decode_packet(&video_packets[0]).unwrap();
    let (a_header, _) = decode_packet(&audio_packets[0]).unwrap();

    // Both channels share the same session but have independent sequence counters.
    assert_eq!(v_header.session_id, session_id);
    assert_eq!(a_header.session_id, session_id);
    assert_eq!(v_header.sequence, 0, "video channel starts at seq 0");
    assert_eq!(a_header.sequence, 0, "audio channel starts at seq 0");

    // Cross-feed: sending video packets to the audio receiver should still
    // decode at the wire level (same protocol) but the application must
    // demultiplex by channel. The transport layer handles it correctly.
    let mut video_delivered = Vec::new();
    for pkt in &video_packets {
        video_delivered.extend(video_receiver.on_receive(pkt).unwrap());
    }

    let mut audio_delivered = Vec::new();
    for pkt in &audio_packets {
        audio_delivered.extend(audio_receiver.on_receive(pkt).unwrap());
    }

    // Each channel delivers its own payload independently.
    assert_eq!(video_delivered.len(), 1, "video channel should deliver one payload");
    assert_eq!(audio_delivered.len(), 1, "audio channel should deliver one payload");
    assert_eq!(video_delivered[0], video_payload);
    assert_eq!(audio_delivered[0], audio_payload);

    // Send more packets on both channels and verify sequence independence.
    let video_payload2 = vec![0xBB; 64];
    let audio_payload2 = vec![0xCC; 32];

    let v_pkts2 = video_sender.prepare_send(&video_payload2, 2000).unwrap();
    let a_pkts2 = audio_sender.prepare_send(&audio_payload2, 2000).unwrap();

    let (v2_header, _) = decode_packet(&v_pkts2[0]).unwrap();
    let (a2_header, _) = decode_packet(&a_pkts2[0]).unwrap();

    assert_eq!(v2_header.sequence, 1, "video seq advances independently");
    assert_eq!(a2_header.sequence, 1, "audio seq advances independently");

    // Verify channel config properties are preserved.
    assert_eq!(video_receiver.config().id, ChannelId::Video);
    assert_eq!(audio_receiver.config().id, ChannelId::Audio);
    assert!(
        video_receiver.config().max_bandwidth_fraction > audio_receiver.config().max_bandwidth_fraction,
        "video should have higher bandwidth allocation than audio"
    );

    // Verify congestion state is independent per channel.
    video_receiver.on_loss_report(0.20);
    assert_eq!(
        video_receiver.congestion_state(),
        CongestionState::Recovery,
        "video channel should enter Recovery after loss report"
    );
    assert_eq!(
        audio_receiver.congestion_state(),
        CongestionState::SlowStart,
        "audio channel should remain in SlowStart (unaffected by video loss)"
    );
}
