// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Logical channel multiplexing and transport channel integration.
//!
//! Each channel has a priority and bandwidth allocation. Video, audio,
//! file transfer, and clipboard data each use separate channels.
//!
//! [`TransportChannel`] combines packet encoding/decoding with sequencing,
//! NACK retransmission, FEC, and congestion control into a single interface.

use serde::{Deserialize, Serialize};

use crate::congestion::{CongestionController, CongestionState};
use crate::fec::{FecDecoder, FecEncoder};
use crate::nack::NackTracker;
use crate::packet::{decode_packet, encode_packet, PacketHeader, PacketType, VERSION};
use crate::sequencer::{ReceiveSequencer, SendSequencer};

/// Well-known channel identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ChannelId {
    /// Video stream channel (highest priority).
    Video = 0,
    /// Audio stream channel.
    Audio = 1,
    /// File transfer channel.
    FileTransfer = 2,
    /// Clipboard synchronization channel.
    Clipboard = 3,
    /// Control messages channel (input, heartbeat).
    Control = 4,
}

impl TryFrom<u8> for ChannelId {
    type Error = crate::TransportError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Video),
            1 => Ok(Self::Audio),
            2 => Ok(Self::FileTransfer),
            3 => Ok(Self::Clipboard),
            4 => Ok(Self::Control),
            _ => Err(crate::TransportError::PacketError(format!(
                "unknown channel id: {value}"
            ))),
        }
    }
}

/// Priority level for channel scheduling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Lowest priority (file transfers).
    Low = 0,
    /// Normal priority (clipboard, control).
    Normal = 1,
    /// High priority (audio).
    High = 2,
    /// Highest priority (video, input).
    Critical = 3,
}

/// Configuration for a single logical channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Channel identifier.
    pub id: ChannelId,
    /// Priority for bandwidth scheduling.
    pub priority: Priority,
    /// Maximum bandwidth allocation as a fraction (0.0 to 1.0).
    pub max_bandwidth_fraction: f64,
    /// Whether this channel requires reliable delivery.
    pub reliable: bool,
    /// Whether in-order delivery is required.
    pub ordered: bool,
}

impl ChannelConfig {
    /// Create a default configuration for well-known channels.
    pub fn default_for(id: ChannelId) -> Self {
        match id {
            ChannelId::Video => Self {
                id,
                priority: Priority::Critical,
                max_bandwidth_fraction: 0.70,
                reliable: false, // video uses FEC instead of retransmission
                ordered: true,
            },
            ChannelId::Audio => Self {
                id,
                priority: Priority::High,
                max_bandwidth_fraction: 0.15,
                reliable: false,
                ordered: true,
            },
            ChannelId::FileTransfer => Self {
                id,
                priority: Priority::Low,
                max_bandwidth_fraction: 0.30,
                reliable: true,
                ordered: true,
            },
            ChannelId::Clipboard => Self {
                id,
                priority: Priority::Normal,
                max_bandwidth_fraction: 0.05,
                reliable: true,
                ordered: false,
            },
            ChannelId::Control => Self {
                id,
                priority: Priority::Normal,
                max_bandwidth_fraction: 0.05,
                reliable: true,
                ordered: false,
            },
        }
    }
}

/// Integrated transport channel combining all transport-layer components.
///
/// Provides a unified interface for:
/// - **Sending**: Sequence assignment, packet encoding, FEC encoding
/// - **Receiving**: Packet decoding, in-order delivery, gap detection, NACK generation
/// - **Congestion control**: Loss-based bitrate adaptation
///
/// # Example
/// ```ignore
/// let config = ChannelConfig::default_for(ChannelId::Video);
/// let mut channel = TransportChannel::new(config, session_id);
///
/// // Send
/// let packets = channel.prepare_send(payload, timestamp)?;
///
/// // Receive
/// let delivered = channel.on_receive(raw_packet)?;
/// let nacks = channel.poll_nacks();
/// ```
pub struct TransportChannel {
    config: ChannelConfig,
    send_seq: SendSequencer,
    recv_seq: ReceiveSequencer,
    congestion: CongestionController,
    nack: NackTracker,
    fec_enc: FecEncoder,
    fec_dec: FecDecoder,
    session_id: u64,
}

impl TransportChannel {
    /// Create a new transport channel with the given configuration and session ID.
    pub fn new(config: ChannelConfig, session_id: u64) -> Self {
        Self {
            config,
            send_seq: SendSequencer::new(1024),
            recv_seq: ReceiveSequencer::new(1000),
            congestion: CongestionController::new(),
            nack: NackTracker::new(3),
            fec_enc: FecEncoder::with_defaults(),
            fec_dec: FecDecoder::with_defaults(),
            session_id,
        }
    }

    /// Prepare a payload for sending.
    ///
    /// Assigns a sequence number, encodes the packet, and feeds the FEC encoder.
    /// Returns a list of encoded packets: the data packet plus any FEC repair
    /// packets if a group was completed.
    pub fn prepare_send(
        &mut self,
        payload: &[u8],
        timestamp: u32,
    ) -> Result<Vec<Vec<u8>>, crate::TransportError> {
        let seq = self.send_seq.next_seq().ok_or_else(|| {
            crate::TransportError::Congestion("send window full".into())
        })?;

        let header = PacketHeader {
            version: VERSION,
            packet_type: PacketType::Data,
            session_id: self.session_id,
            sequence: seq,
            timestamp,
            payload_len: payload.len() as u16,
        };

        let mut packets = vec![encode_packet(&header, payload)];

        // Feed FEC encoder; emit repair packets if group completed
        if let Some(repairs) = self.fec_enc.feed(payload) {
            for repair_data in &repairs {
                let repair_header = PacketHeader {
                    version: VERSION,
                    packet_type: PacketType::FecRepair,
                    session_id: self.session_id,
                    sequence: seq,
                    timestamp,
                    payload_len: repair_data.len() as u16,
                };
                packets.push(encode_packet(&repair_header, repair_data));
            }
        }

        Ok(packets)
    }

    /// Process a received raw packet.
    ///
    /// For data packets: decodes, inserts into receive sequencer for in-order
    /// delivery, detects gaps, and updates the NACK tracker.
    ///
    /// Returns a list of in-order payloads ready for the application.
    pub fn on_receive(
        &mut self,
        raw: &[u8],
    ) -> Result<Vec<Vec<u8>>, crate::TransportError> {
        let (header, payload) = decode_packet(raw)?;

        match header.packet_type {
            PacketType::Data => {
                // Detect gaps before inserting
                let expected = self.recv_seq.expected();
                if header.sequence > expected {
                    self.nack.report_gap(expected, header.sequence);
                }

                // Insert into receive sequencer for in-order delivery
                let delivered = self.recv_seq.insert(header.sequence, payload.to_vec());

                // Mark as received in NACK tracker (fills gaps)
                self.nack.mark_received(header.sequence);

                Ok(delivered)
            }
            PacketType::Ack => {
                // Acknowledge a sent packet
                self.send_seq.acknowledge(header.sequence);
                self.congestion.on_ack(header.timestamp as u64);
                Ok(Vec::new())
            }
            PacketType::Nack => {
                // Peer is requesting retransmission — payload contains requested seq numbers
                // Caller is responsible for retransmitting
                Ok(vec![payload.to_vec()])
            }
            PacketType::FecRepair => {
                // Store FEC repair for later group decoding
                // Full FEC decode_group integration would buffer groups here
                Ok(Vec::new())
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Generate NACK sequence numbers for retransmission requests.
    pub fn poll_nacks(&mut self) -> Vec<u32> {
        self.nack.generate_nack_list()
    }

    /// Report observed loss rate for congestion control adaptation.
    pub fn on_loss_report(&mut self, loss_rate: f64) {
        self.congestion.on_round(loss_rate);
    }

    /// Return the current target bitrate in bits per second.
    pub fn target_bitrate(&self) -> u64 {
        self.congestion.target_bitrate()
    }

    /// Return the current congestion state.
    pub fn congestion_state(&self) -> CongestionState {
        self.congestion.state()
    }

    /// Return the channel configuration.
    pub fn config(&self) -> &ChannelConfig {
        &self.config
    }

    /// Return the session ID.
    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    /// Return a reference to the FEC decoder (for external group decoding).
    pub fn fec_decoder(&self) -> &FecDecoder {
        &self.fec_dec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_id_conversion() {
        assert_eq!(ChannelId::try_from(0), Ok(ChannelId::Video));
        assert_eq!(ChannelId::try_from(4), Ok(ChannelId::Control));
        assert!(ChannelId::try_from(99).is_err());
    }

    #[test]
    fn default_configs() {
        let video = ChannelConfig::default_for(ChannelId::Video);
        assert!(video.priority == Priority::Critical);
        assert!(!video.reliable); // uses FEC
    }

    #[test]
    fn channel_prepare_send() {
        let config = ChannelConfig::default_for(ChannelId::Video);
        let mut ch = TransportChannel::new(config, 12345);

        let packets = ch.prepare_send(b"hello world", 100).unwrap();
        assert!(!packets.is_empty());

        // Decode the first packet and verify
        let (header, payload) = decode_packet(&packets[0]).unwrap();
        assert_eq!(header.packet_type, PacketType::Data);
        assert_eq!(header.session_id, 12345);
        assert_eq!(header.sequence, 0);
        assert_eq!(header.timestamp, 100);
        assert_eq!(payload, b"hello world");
    }

    #[test]
    fn channel_receive_in_order() {
        let config = ChannelConfig::default_for(ChannelId::Video);
        let mut sender = TransportChannel::new(config.clone(), 1);
        let mut receiver = TransportChannel::new(config, 1);

        // Send 3 packets in order
        let p0 = sender.prepare_send(b"first", 0).unwrap();
        let p1 = sender.prepare_send(b"second", 1).unwrap();
        let p2 = sender.prepare_send(b"third", 2).unwrap();

        // Receive in order
        let d0 = receiver.on_receive(&p0[0]).unwrap();
        assert_eq!(d0, vec![b"first".to_vec()]);

        let d1 = receiver.on_receive(&p1[0]).unwrap();
        assert_eq!(d1, vec![b"second".to_vec()]);

        let d2 = receiver.on_receive(&p2[0]).unwrap();
        assert_eq!(d2, vec![b"third".to_vec()]);
    }

    #[test]
    fn channel_nack_detection() {
        let config = ChannelConfig::default_for(ChannelId::Video);
        let mut sender = TransportChannel::new(config.clone(), 1);
        let mut receiver = TransportChannel::new(config, 1);

        // Send packets 0, 1, 2, 3
        let p0 = sender.prepare_send(b"pkt0", 0).unwrap();
        let p1 = sender.prepare_send(b"pkt1", 1).unwrap();
        let _p2 = sender.prepare_send(b"pkt2", 2).unwrap(); // will be skipped
        let p3 = sender.prepare_send(b"pkt3", 3).unwrap();

        // Receive 0, 1, 3 (skip 2)
        receiver.on_receive(&p0[0]).unwrap();
        receiver.on_receive(&p1[0]).unwrap();
        receiver.on_receive(&p3[0]).unwrap();

        // Should detect missing packet 2
        let nacks = receiver.poll_nacks();
        assert!(
            nacks.contains(&2),
            "should detect missing packet 2, got nacks: {nacks:?}"
        );
    }

    #[test]
    fn channel_congestion_feedback() {
        let config = ChannelConfig::default_for(ChannelId::Video);
        let mut ch = TransportChannel::new(config, 1);

        let initial = ch.target_bitrate();

        // Report 10% loss for 5 rounds
        for _ in 0..5 {
            ch.on_loss_report(0.10);
        }

        assert!(
            ch.target_bitrate() < initial,
            "bitrate should decrease after loss"
        );
        assert_eq!(ch.congestion_state(), CongestionState::Recovery);
    }

    #[test]
    fn channel_send_window_full() {
        let config = ChannelConfig::default_for(ChannelId::Video);
        let mut ch = TransportChannel::new(config, 1);

        // Fill the send window (1024 packets)
        for _ in 0..1024 {
            ch.prepare_send(b"x", 0).unwrap();
        }

        // Next send should fail
        let result = ch.prepare_send(b"overflow", 0);
        assert!(result.is_err());
    }
}
