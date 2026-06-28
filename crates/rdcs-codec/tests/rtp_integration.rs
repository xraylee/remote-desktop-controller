// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for RTP/SRTP pipeline.
//!
//! Tests the complete flow: H.264 → RTP Packetizer → SRTP Encrypt → Network →
//! SRTP Decrypt → RTP Depacketizer → H.264.

use rdcs_codec::rtp::{
    H264Depacketizer, H264Packetizer, PacketizerConfig, SrtpConfig, SrtpContext, SrtpProfile,
};

/// Generate test SRTP keys (same keys for sender and receiver in loopback test).
fn create_test_srtp_config() -> SrtpConfig {
    SrtpConfig {
        master_key: vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
            0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10,
        ],
        master_salt: vec![
            0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18,
            0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E,
        ],
        profile: SrtpProfile::Aead_Aes128Gcm,
    }
}

#[tokio::test]
async fn test_end_to_end_single_nal_unit() {
    // Setup
    let packetizer_config = PacketizerConfig {
        mtu: 1200,
        clock_rate: 90000,
        ssrc: 0x12345678,
        payload_type: 96,
    };

    let mut packetizer = H264Packetizer::new(packetizer_config);
    let mut depacketizer = H264Depacketizer::new();

    let srtp_config = create_test_srtp_config();
    let sender_srtp = SrtpContext::new(srtp_config.clone()).await.unwrap();
    let receiver_srtp = SrtpContext::new(srtp_config).await.unwrap();

    // Create H.264 test data (Annex B format)
    let original_h264 = vec![
        0x00, 0x00, 0x00, 0x01, // Start code
        0x65, // IDR NAL header
        0xAA, 0xBB, 0xCC, 0xDD, 0xEE, // Payload
    ];

    // Sender side: Packetize
    let rtp_packets = packetizer.packetize(&original_h264, 1000).unwrap();
    assert_eq!(rtp_packets.len(), 1, "Single NAL unit should fit in one packet");

    // Sender side: Encrypt
    let srtp_packet = sender_srtp.encrypt(&rtp_packets[0]).await.unwrap();
    assert!(
        srtp_packet.len() > rtp_packets[0].len(),
        "SRTP packet should be larger (auth tag)"
    );

    // === Network transmission (simulated) ===

    // Receiver side: Decrypt
    let decrypted_rtp = receiver_srtp.decrypt(&srtp_packet).await.unwrap();
    assert_eq!(
        decrypted_rtp, rtp_packets[0],
        "Decrypted RTP should match original"
    );

    // Receiver side: Depacketize
    let reconstructed_h264 = depacketizer.depacketize(&decrypted_rtp).unwrap();
    assert!(reconstructed_h264.is_some(), "Should have complete NAL unit");

    let output = reconstructed_h264.unwrap();
    assert_eq!(
        output, original_h264,
        "Reconstructed H.264 should match original"
    );

    // Verify stats
    assert_eq!(packetizer.stats().packets_sent, 1);
    assert_eq!(depacketizer.stats().packets_received, 1);
    assert_eq!(sender_srtp.stats().await.packets_encrypted, 1);
    assert_eq!(receiver_srtp.stats().await.packets_decrypted, 1);
}

#[tokio::test]
async fn test_end_to_end_fragmented_nal_unit() {
    // Setup with small MTU to force fragmentation
    let packetizer_config = PacketizerConfig {
        mtu: 100, // Small MTU
        clock_rate: 90000,
        ssrc: 0x87654321,
        payload_type: 96,
    };

    let mut packetizer = H264Packetizer::new(packetizer_config);
    let mut depacketizer = H264Depacketizer::new();

    let srtp_config = create_test_srtp_config();
    let sender_srtp = SrtpContext::new(srtp_config.clone()).await.unwrap();
    let receiver_srtp = SrtpContext::new(srtp_config).await.unwrap();

    // Create large H.264 NAL unit (will be fragmented)
    let mut original_h264 = vec![0x00, 0x00, 0x00, 0x01, 0x65]; // Start code + IDR header
    original_h264.extend(vec![0x42; 300]); // Large payload

    // Sender side: Packetize
    let rtp_packets = packetizer.packetize(&original_h264, 2000).unwrap();
    assert!(rtp_packets.len() > 1, "Large NAL should be fragmented");

    // Process all fragments
    let mut last_output = None;
    for rtp_packet in rtp_packets {
        // Encrypt
        let srtp_packet = sender_srtp.encrypt(&rtp_packet).await.unwrap();

        // === Network ===

        // Decrypt
        let decrypted_rtp = receiver_srtp.decrypt(&srtp_packet).await.unwrap();

        // Depacketize
        let result = depacketizer.depacketize(&decrypted_rtp).unwrap();
        if result.is_some() {
            last_output = result;
        }
    }

    // Should have complete NAL unit after last fragment
    assert!(last_output.is_some(), "Should reconstruct fragmented NAL");
    let output = last_output.unwrap();
    assert_eq!(
        output, original_h264,
        "Reconstructed H.264 should match original"
    );

    // Verify fragmentation stats
    assert_eq!(packetizer.stats().fragmented_nal_units, 1);
    assert_eq!(depacketizer.stats().fragments_reassembled, 1);
}

#[tokio::test]
async fn test_end_to_end_multiple_nal_units() {
    // Setup
    let packetizer_config = PacketizerConfig::default();
    let mut packetizer = H264Packetizer::new(packetizer_config);
    let mut depacketizer = H264Depacketizer::new();

    let srtp_config = create_test_srtp_config();
    let sender_srtp = SrtpContext::new(srtp_config.clone()).await.unwrap();
    let receiver_srtp = SrtpContext::new(srtp_config).await.unwrap();

    // Create Annex B with multiple NAL units (SPS, PPS, IDR)
    let mut original_h264 = vec![];

    // SPS
    original_h264.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0x00, 0x1F]);

    // PPS
    original_h264.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x68, 0xCE, 0x3C, 0x80]);

    // IDR
    original_h264.extend_from_slice(&[0x00, 0x00, 0x00, 0x01, 0x65]);
    original_h264.extend(vec![0x99; 50]);

    // Sender side: Packetize
    let rtp_packets = packetizer.packetize(&original_h264, 3000).unwrap();
    assert_eq!(rtp_packets.len(), 3, "Should have 3 RTP packets for 3 NAL units");

    // Collect reconstructed NAL units
    let mut reconstructed = vec![];

    for rtp_packet in rtp_packets {
        // Encrypt
        let srtp_packet = sender_srtp.encrypt(&rtp_packet).await.unwrap();

        // === Network ===

        // Decrypt
        let decrypted_rtp = receiver_srtp.decrypt(&srtp_packet).await.unwrap();

        // Depacketize
        if let Some(nal_unit) = depacketizer.depacketize(&decrypted_rtp).unwrap() {
            reconstructed.extend(nal_unit);
        }
    }

    // Verify complete reconstruction
    assert_eq!(
        reconstructed, original_h264,
        "All NAL units should be reconstructed"
    );

    // Verify stats
    assert_eq!(packetizer.stats().nal_units_processed, 3);
    assert_eq!(depacketizer.stats().nal_units_assembled, 3);
    assert_eq!(depacketizer.stats().keyframes_received, 3); // SPS, PPS, IDR
}

#[tokio::test]
async fn test_packet_loss_detection() {
    // Setup
    let packetizer_config = PacketizerConfig::default();
    let mut packetizer = H264Packetizer::new(packetizer_config);
    let mut depacketizer = H264Depacketizer::new();

    let srtp_config = create_test_srtp_config();
    let sender_srtp = SrtpContext::new(srtp_config.clone()).await.unwrap();
    let receiver_srtp = SrtpContext::new(srtp_config).await.unwrap();

    // Create three separate NAL units
    let nal1 = vec![0x00, 0x00, 0x00, 0x01, 0x67, 0x01]; // SPS
    let nal2 = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0x02]; // PPS
    let nal3 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0x03]; // IDR

    let packets1 = packetizer.packetize(&nal1, 1000).unwrap();
    let packets2 = packetizer.packetize(&nal2, 2000).unwrap();
    let packets3 = packetizer.packetize(&nal3, 3000).unwrap();

    // Process first packet
    let srtp1 = sender_srtp.encrypt(&packets1[0]).await.unwrap();
    let rtp1 = receiver_srtp.decrypt(&srtp1).await.unwrap();
    depacketizer.depacketize(&rtp1).unwrap();

    // Skip second packet (simulate loss)

    // Process third packet
    let srtp3 = sender_srtp.encrypt(&packets3[0]).await.unwrap();
    let rtp3 = receiver_srtp.decrypt(&srtp3).await.unwrap();
    depacketizer.depacketize(&rtp3).unwrap();

    // Should detect packet loss
    assert_eq!(
        depacketizer.stats().packets_lost,
        1,
        "Should detect 1 lost packet"
    );
}

#[tokio::test]
async fn test_srtp_authentication_failure() {
    // Setup
    let packetizer_config = PacketizerConfig::default();
    let mut packetizer = H264Packetizer::new(packetizer_config);

    let srtp_config = create_test_srtp_config();
    let sender_srtp = SrtpContext::new(srtp_config.clone()).await.unwrap();
    let receiver_srtp = SrtpContext::new(srtp_config).await.unwrap();

    // Create and encrypt a packet
    let h264 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xAA];
    let rtp_packets = packetizer.packetize(&h264, 1000).unwrap();
    let mut srtp_packet = sender_srtp.encrypt(&rtp_packets[0]).await.unwrap();

    // Tamper with encrypted data
    srtp_packet[20] ^= 0xFF;

    // Decryption should fail
    let result = receiver_srtp.decrypt(&srtp_packet).await;
    assert!(result.is_err(), "Tampered packet should fail authentication");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("decryption failed"));
}

#[tokio::test]
async fn test_different_protection_profiles() {
    for profile in [SrtpProfile::Aead_Aes128Gcm, SrtpProfile::Aes128CmHmacSha1_80] {
        let srtp_config = SrtpConfig {
            master_key: vec![0x42; 16],
            master_salt: vec![0x84; 14],
            profile,
        };

        let sender = SrtpContext::new(srtp_config.clone()).await.unwrap();
        let receiver = SrtpContext::new(srtp_config).await.unwrap();

        // Simple encrypt/decrypt test
        let mut packetizer = H264Packetizer::new(PacketizerConfig::default());
        let h264 = vec![0x00, 0x00, 0x00, 0x01, 0x65, 0xFF];
        let rtp_packets = packetizer.packetize(&h264, 1000).unwrap();

        let srtp_packet = sender.encrypt(&rtp_packets[0]).await.unwrap();
        let decrypted = receiver.decrypt(&srtp_packet).await.unwrap();

        assert_eq!(decrypted, rtp_packets[0], "Profile {:?} roundtrip failed", profile);
    }
}
