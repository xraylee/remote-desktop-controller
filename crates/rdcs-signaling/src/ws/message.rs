// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! WebSocket message type definitions for the signaling protocol.
//!
//! All messages exchanged between clients and the signaling server are
//! JSON-encoded with a `"type"` discriminator field (serde tagged enum).
//! The 10 message types correspond to the signaling protocol defined in
//! the architecture specification (Section 3.1).

use serde::{Deserialize, Serialize};

/// Top-level WebSocket message envelope.
///
/// Each variant is discriminated by the `"type"` JSON field. For example,
/// a Register message serializes as:
///
/// ```json
/// {"type":"register","device_code":"ABC","platform":"macos","version":"1.0"}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Client announces itself to the signaling server.
    #[serde(rename = "register")]
    Register {
        device_code: String,
        platform: String,
        version: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        team_id: Option<String>,
    },

    /// Periodic keep-alive from client; server refreshes Redis device TTL.
    #[serde(rename = "heartbeat")]
    Heartbeat { device_code: String, ts: u64 },

    /// Controller requests a connection to a target device.
    ///
    /// `session_id` is present only on the server→target forward (the server
    /// mints it); the controller→server request omits it.
    #[serde(rename = "connect_request")]
    ConnectRequest {
        from_code: String,
        to_code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        invite_code: Option<String>,
    },

    /// Target device accepts or rejects a connection request.
    #[serde(rename = "connect_response")]
    ConnectResponse {
        accepted: bool,
        session_id: String,
        from_code: String,
    },

    /// Initiator sends its WebRTC SDP offer and ICE candidates.
    #[serde(rename = "ice_offer")]
    IceOffer {
        session_id: String,
        sdp: String,
        candidates: Vec<IceCandidate>,
    },

    /// Responder sends its WebRTC SDP answer and ICE candidates.
    #[serde(rename = "ice_answer")]
    IceAnswer {
        session_id: String,
        sdp: String,
        candidates: Vec<IceCandidate>,
    },

    /// Trickle ICE: a single ICE candidate exchanged after the initial offer/answer.
    #[serde(rename = "ice_trickle")]
    IceTrickle {
        session_id: String,
        candidate: IceCandidate,
    },

    /// Either peer requests a relay server when P2P fails.
    #[serde(rename = "relay_request")]
    RelayRequest {
        session_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        preferred_region: Option<String>,
    },

    /// Server assigns a relay endpoint to the session.
    #[serde(rename = "relay_assigned")]
    RelayAssigned {
        session_id: String,
        relay_addr: String,
        relay_port: u16,
        token: String,
    },

    /// Server notifies a client that a peer went offline.
    #[serde(rename = "peer_offline")]
    PeerOffline { device_code: String, reason: String },

    /// Server pushes updated nearby-device list to a client.
    #[serde(rename = "nearby_update")]
    NearbyUpdate { devices: Vec<DeviceInfo> },

    /// Client requests generation of an invite code.
    #[serde(rename = "generate_invite")]
    GenerateInvite { device_code: String },

    /// Client uses an invite code to initiate a connection.
    #[serde(rename = "use_invite")]
    UseInvite {
        from_code: String,
        invite_code: String,
    },

    /// Server responds with the generated invite code.
    #[serde(rename = "invite_generated")]
    InviteGenerated { invite_code: String },

    /// Server responds after a successful invite consumption with session info.
    #[serde(rename = "invite_result")]
    InviteResult {
        session_id: String,
        to_code: String,
    },

    /// Server-to-client error response (e.g. unknown message type).
    #[serde(rename = "error")]
    Error { code: String, message: String },
}

/// A single ICE candidate exchanged during WebRTC negotiation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IceCandidate {
    pub candidate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp_mid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp_m_line_index: Option<u16>,
}

/// Lightweight descriptor for a device visible to the current user.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DeviceInfo {
    pub code: String,
    pub name: String,
    pub platform: String,
    pub online: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Serialization round-trip tests for every message type
    // -----------------------------------------------------------------------

    #[test]
    fn register_round_trip() {
        let msg = WsMessage::Register {
            device_code: "ABC123".into(),
            platform: "macos".into(),
            version: "1.0.0".into(),
            team_id: Some("team-1".into()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"register""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn register_without_team_id_omits_field() {
        let msg = WsMessage::Register {
            device_code: "X".into(),
            platform: "win".into(),
            version: "0.1".into(),
            team_id: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.contains("team_id"), "None team_id should be omitted: {json}");
    }

    #[test]
    fn heartbeat_round_trip() {
        let msg = WsMessage::Heartbeat {
            device_code: "DEV1".into(),
            ts: 1_700_000_000,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"heartbeat""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn connect_request_round_trip() {
        // server→B direction: carries session_id
        let msg = WsMessage::ConnectRequest {
            from_code: "CTRL".into(),
            to_code: "TARGET".into(),
            session_id: Some("sess-abc".into()),
            invite_code: Some("INV001".into()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"connect_request""#));
        assert!(json.contains(r#""session_id":"sess-abc""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);

        // controller→server direction: no session_id, should be omitted
        let msg_none = WsMessage::ConnectRequest {
            from_code: "CTRL".into(),
            to_code: "TARGET".into(),
            session_id: None,
            invite_code: None,
        };
        let json_none = serde_json::to_string(&msg_none).unwrap();
        assert!(!json_none.contains("session_id"));
        let parsed_none: WsMessage = serde_json::from_str(&json_none).unwrap();
        assert_eq!(parsed_none, msg_none);
    }

    #[test]
    fn connect_response_round_trip() {
        let msg = WsMessage::ConnectResponse {
            accepted: true,
            session_id: "sess-001".into(),
            from_code: "CTRL".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"connect_response""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn ice_offer_round_trip() {
        let msg = WsMessage::IceOffer {
            session_id: "sess-001".into(),
            sdp: "v=0\r\n...".into(),
            candidates: vec![IceCandidate {
                candidate: "candidate:1 ...".into(),
                sdp_mid: Some("0".into()),
                sdp_m_line_index: Some(0),
            }],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"ice_offer""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn ice_answer_round_trip() {
        let msg = WsMessage::IceAnswer {
            session_id: "sess-002".into(),
            sdp: "v=0\r\n...".into(),
            candidates: vec![],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"ice_answer""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn ice_trickle_round_trip() {
        let msg = WsMessage::IceTrickle {
            session_id: "sess-002".into(),
            candidate: IceCandidate {
                candidate: "candidate:1 udp 2130706431 192.168.1.10 5000 typ host".into(),
                sdp_mid: Some("0".into()),
                sdp_m_line_index: Some(0),
            },
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"ice_trickle""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn relay_request_round_trip() {
        let msg = WsMessage::RelayRequest {
            session_id: "sess-003".into(),
            preferred_region: Some("ap-east-1".into()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"relay_request""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn relay_assigned_round_trip() {
        let msg = WsMessage::RelayAssigned {
            session_id: "sess-004".into(),
            relay_addr: "relay.example.com".into(),
            relay_port: 4443,
            token: "tok-abc".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"relay_assigned""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn peer_offline_round_trip() {
        let msg = WsMessage::PeerOffline {
            device_code: "DEV9".into(),
            reason: "heartbeat_timeout".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"peer_offline""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn nearby_update_round_trip() {
        let msg = WsMessage::NearbyUpdate {
            devices: vec![
                DeviceInfo {
                    code: "D1".into(),
                    name: "Workstation".into(),
                    platform: "linux".into(),
                    online: true,
                },
                DeviceInfo {
                    code: "D2".into(),
                    name: "Laptop".into(),
                    platform: "windows".into(),
                    online: false,
                },
            ],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"nearby_update""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn error_round_trip() {
        let msg = WsMessage::Error {
            code: "unknown_type".into(),
            message: "unknown message type: foo".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"error""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn generate_invite_round_trip() {
        let msg = WsMessage::GenerateInvite {
            device_code: "DEV1".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"generate_invite""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn use_invite_round_trip() {
        let msg = WsMessage::UseInvite {
            from_code: "CTRL".into(),
            invite_code: "1234".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"use_invite""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn invite_generated_round_trip() {
        let msg = WsMessage::InviteGenerated {
            invite_code: "5678".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"invite_generated""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn invite_result_round_trip() {
        let msg = WsMessage::InviteResult {
            session_id: "sess-inv-001".into(),
            to_code: "TARGET".into(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"invite_result""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);
    }

    // -----------------------------------------------------------------------
    // Deserialization edge cases
    // -----------------------------------------------------------------------

    #[test]
    fn unknown_type_fails_deserialization() {
        let json = r#"{"type":"bogus_type","foo":"bar"}"#;
        let result = serde_json::from_str::<WsMessage>(json);
        assert!(result.is_err(), "unknown type should fail to deserialize");
    }

    #[test]
    fn missing_type_fails_deserialization() {
        let json = r#"{"device_code":"X","platform":"macos","version":"1.0"}"#;
        let result = serde_json::from_str::<WsMessage>(json);
        assert!(result.is_err(), "missing type field should fail");
    }

    #[test]
    fn malformed_json_fails_deserialization() {
        let json = r#"{"type":"register", broken}"#;
        let result = serde_json::from_str::<WsMessage>(json);
        assert!(result.is_err());
    }

    #[test]
    fn all_fourteen_message_types_have_distinct_type_tags() {
        // Verify that all 15 required types produce different "type" values.
        let types = [
            "register",
            "heartbeat",
            "connect_request",
            "connect_response",
            "ice_offer",
            "ice_answer",
            "ice_trickle",
            "relay_request",
            "relay_assigned",
            "peer_offline",
            "nearby_update",
            "generate_invite",
            "use_invite",
            "invite_generated",
            "invite_result",
        ];
        let unique: std::collections::HashSet<&str> = types.iter().copied().collect();
        assert_eq!(unique.len(), 15, "all 15 message types must have distinct tags");
    }
}
