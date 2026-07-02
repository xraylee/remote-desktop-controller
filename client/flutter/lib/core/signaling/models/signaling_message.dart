// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:freezed_annotation/freezed_annotation.dart';

part 'signaling_message.freezed.dart';
part 'signaling_message.g.dart';

/// ICE candidate for WebRTC negotiation.
@freezed
class IceCandidate with _$IceCandidate {
  const factory IceCandidate({
    required String candidate,
    String? sdpMid,
    int? sdpMLineIndex,
  }) = _IceCandidate;

  factory IceCandidate.fromJson(Map<String, dynamic> json) =>
      _$IceCandidateFromJson(json);
}

/// Device information from nearby_update.
@freezed
class DeviceInfo with _$DeviceInfo {
  const factory DeviceInfo({
    required String code,
    required String name,
    required String platform,
    required bool online,
  }) = _DeviceInfo;

  factory DeviceInfo.fromJson(Map<String, dynamic> json) =>
      _$DeviceInfoFromJson(json);
}

/// Signaling message types exchanged between client and server.
///
/// All messages are JSON-encoded with a "type" discriminator field.
@Freezed(unionKey: 'type', unionValueCase: FreezedUnionCase.snake)
sealed class SignalingMessage with _$SignalingMessage {
  // ── Client → Server ──────────────────────────────────────────

  /// Register this device with the signaling server.
  const factory SignalingMessage.register({
    @JsonKey(name: 'device_code') required String deviceCode,
    required String platform,
    required String version,
    @JsonKey(name: 'team_id') String? teamId,
  }) = RegisterMessage;

  /// Heartbeat keep-alive message.
  const factory SignalingMessage.heartbeat({
    @JsonKey(name: 'device_code') required String deviceCode,
    required int ts,
  }) = HeartbeatMessage;

  /// Request a connection to another device.
  const factory SignalingMessage.connectRequest({
    @JsonKey(name: 'from_code') required String fromCode,
    @JsonKey(name: 'to_code') required String toCode,
    @JsonKey(name: 'invite_code') String? inviteCode,
  }) = ConnectRequestMessage;

  /// Accept or reject a connection request.
  const factory SignalingMessage.connectResponse({
    required bool accepted,
    @JsonKey(name: 'session_id') required String sessionId,
    @JsonKey(name: 'from_code') required String fromCode,
  }) = ConnectResponseMessage;

  /// Send ICE offer (SDP + candidates).
  const factory SignalingMessage.iceOffer({
    @JsonKey(name: 'session_id') required String sessionId,
    required String sdp,
    required List<IceCandidate> candidates,
  }) = IceOfferMessage;

  /// Send ICE answer (SDP + candidates).
  const factory SignalingMessage.iceAnswer({
    @JsonKey(name: 'session_id') required String sessionId,
    required String sdp,
    required List<IceCandidate> candidates,
  }) = IceAnswerMessage;

  /// Trickle ICE: send a single candidate after initial offer/answer.
  const factory SignalingMessage.iceTrickle({
    @JsonKey(name: 'session_id') required String sessionId,
    required IceCandidate candidate,
  }) = IceTrickleMessage;

  /// Request a relay server when P2P fails.
  const factory SignalingMessage.relayRequest({
    @JsonKey(name: 'session_id') required String sessionId,
    @JsonKey(name: 'preferred_region') String? preferredRegion,
  }) = RelayRequestMessage;

  /// Generate an invite code for this device.
  const factory SignalingMessage.generateInvite({
    @JsonKey(name: 'device_code') required String deviceCode,
  }) = GenerateInviteMessage;

  /// Use an invite code to connect to a device.
  const factory SignalingMessage.useInvite({
    @JsonKey(name: 'from_code') required String fromCode,
    @JsonKey(name: 'invite_code') required String inviteCode,
  }) = UseInviteMessage;

  // ── Server → Client ──────────────────────────────────────────

  /// Server notifies client of nearby devices (same team).
  const factory SignalingMessage.nearbyUpdate({
    required List<DeviceInfo> devices,
  }) = NearbyUpdateMessage;

  /// Server notifies client that a peer went offline.
  const factory SignalingMessage.peerOffline({
    @JsonKey(name: 'device_code') required String deviceCode,
    required String reason,
  }) = PeerOfflineMessage;

  /// Server assigns a relay server to the session.
  const factory SignalingMessage.relayAssigned({
    @JsonKey(name: 'session_id') required String sessionId,
    @JsonKey(name: 'relay_addr') required String relayAddr,
    @JsonKey(name: 'relay_port') required int relayPort,
    required String token,
  }) = RelayAssignedMessage;

  /// Server responds with the generated invite code.
  const factory SignalingMessage.inviteGenerated({
    @JsonKey(name: 'invite_code') required String inviteCode,
  }) = InviteGeneratedMessage;

  /// Server responds after invite code consumption.
  const factory SignalingMessage.inviteResult({
    @JsonKey(name: 'session_id') required String sessionId,
    @JsonKey(name: 'to_code') required String toCode,
  }) = InviteResultMessage;

  /// Server error response.
  const factory SignalingMessage.error({
    required String code,
    required String message,
  }) = ErrorMessage;

  factory SignalingMessage.fromJson(Map<String, dynamic> json) =>
      _$SignalingMessageFromJson(json);
}
