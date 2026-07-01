// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'signaling_message.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_$IceCandidateImpl _$$IceCandidateImplFromJson(Map<String, dynamic> json) =>
    _$IceCandidateImpl(
      candidate: json['candidate'] as String,
      sdpMid: json['sdpMid'] as String?,
      sdpMLineIndex: (json['sdpMLineIndex'] as num?)?.toInt(),
    );

Map<String, dynamic> _$$IceCandidateImplToJson(_$IceCandidateImpl instance) =>
    <String, dynamic>{
      'candidate': instance.candidate,
      'sdpMid': instance.sdpMid,
      'sdpMLineIndex': instance.sdpMLineIndex,
    };

_$DeviceInfoImpl _$$DeviceInfoImplFromJson(Map<String, dynamic> json) =>
    _$DeviceInfoImpl(
      code: json['code'] as String,
      name: json['name'] as String,
      platform: json['platform'] as String,
      online: json['online'] as bool,
    );

Map<String, dynamic> _$$DeviceInfoImplToJson(_$DeviceInfoImpl instance) =>
    <String, dynamic>{
      'code': instance.code,
      'name': instance.name,
      'platform': instance.platform,
      'online': instance.online,
    };

_$RegisterMessageImpl _$$RegisterMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$RegisterMessageImpl(
      deviceCode: json['device_code'] as String,
      platform: json['platform'] as String,
      version: json['version'] as String,
      teamId: json['team_id'] as String?,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$RegisterMessageImplToJson(
        _$RegisterMessageImpl instance) =>
    <String, dynamic>{
      'device_code': instance.deviceCode,
      'platform': instance.platform,
      'version': instance.version,
      'team_id': instance.teamId,
      'type': instance.$type,
    };

_$HeartbeatMessageImpl _$$HeartbeatMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$HeartbeatMessageImpl(
      deviceCode: json['device_code'] as String,
      ts: (json['ts'] as num).toInt(),
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$HeartbeatMessageImplToJson(
        _$HeartbeatMessageImpl instance) =>
    <String, dynamic>{
      'device_code': instance.deviceCode,
      'ts': instance.ts,
      'type': instance.$type,
    };

_$ConnectRequestMessageImpl _$$ConnectRequestMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$ConnectRequestMessageImpl(
      fromCode: json['from_code'] as String,
      toCode: json['to_code'] as String,
      inviteCode: json['invite_code'] as String?,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$ConnectRequestMessageImplToJson(
        _$ConnectRequestMessageImpl instance) =>
    <String, dynamic>{
      'from_code': instance.fromCode,
      'to_code': instance.toCode,
      'invite_code': instance.inviteCode,
      'type': instance.$type,
    };

_$ConnectResponseMessageImpl _$$ConnectResponseMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$ConnectResponseMessageImpl(
      accepted: json['accepted'] as bool,
      sessionId: json['session_id'] as String,
      fromCode: json['from_code'] as String,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$ConnectResponseMessageImplToJson(
        _$ConnectResponseMessageImpl instance) =>
    <String, dynamic>{
      'accepted': instance.accepted,
      'session_id': instance.sessionId,
      'from_code': instance.fromCode,
      'type': instance.$type,
    };

_$IceOfferMessageImpl _$$IceOfferMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$IceOfferMessageImpl(
      sessionId: json['session_id'] as String,
      sdp: json['sdp'] as String,
      candidates: (json['candidates'] as List<dynamic>)
          .map((e) => IceCandidate.fromJson(e as Map<String, dynamic>))
          .toList(),
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$IceOfferMessageImplToJson(
        _$IceOfferMessageImpl instance) =>
    <String, dynamic>{
      'session_id': instance.sessionId,
      'sdp': instance.sdp,
      'candidates': instance.candidates,
      'type': instance.$type,
    };

_$IceAnswerMessageImpl _$$IceAnswerMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$IceAnswerMessageImpl(
      sessionId: json['session_id'] as String,
      sdp: json['sdp'] as String,
      candidates: (json['candidates'] as List<dynamic>)
          .map((e) => IceCandidate.fromJson(e as Map<String, dynamic>))
          .toList(),
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$IceAnswerMessageImplToJson(
        _$IceAnswerMessageImpl instance) =>
    <String, dynamic>{
      'session_id': instance.sessionId,
      'sdp': instance.sdp,
      'candidates': instance.candidates,
      'type': instance.$type,
    };

_$IceTrickleMessageImpl _$$IceTrickleMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$IceTrickleMessageImpl(
      sessionId: json['session_id'] as String,
      candidate:
          IceCandidate.fromJson(json['candidate'] as Map<String, dynamic>),
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$IceTrickleMessageImplToJson(
        _$IceTrickleMessageImpl instance) =>
    <String, dynamic>{
      'session_id': instance.sessionId,
      'candidate': instance.candidate,
      'type': instance.$type,
    };

_$RelayRequestMessageImpl _$$RelayRequestMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$RelayRequestMessageImpl(
      sessionId: json['session_id'] as String,
      preferredRegion: json['preferred_region'] as String?,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$RelayRequestMessageImplToJson(
        _$RelayRequestMessageImpl instance) =>
    <String, dynamic>{
      'session_id': instance.sessionId,
      'preferred_region': instance.preferredRegion,
      'type': instance.$type,
    };

_$GenerateInviteMessageImpl _$$GenerateInviteMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$GenerateInviteMessageImpl(
      deviceCode: json['device_code'] as String,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$GenerateInviteMessageImplToJson(
        _$GenerateInviteMessageImpl instance) =>
    <String, dynamic>{
      'device_code': instance.deviceCode,
      'type': instance.$type,
    };

_$UseInviteMessageImpl _$$UseInviteMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$UseInviteMessageImpl(
      fromCode: json['from_code'] as String,
      inviteCode: json['invite_code'] as String,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$UseInviteMessageImplToJson(
        _$UseInviteMessageImpl instance) =>
    <String, dynamic>{
      'from_code': instance.fromCode,
      'invite_code': instance.inviteCode,
      'type': instance.$type,
    };

_$NearbyUpdateMessageImpl _$$NearbyUpdateMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$NearbyUpdateMessageImpl(
      devices: (json['devices'] as List<dynamic>)
          .map((e) => DeviceInfo.fromJson(e as Map<String, dynamic>))
          .toList(),
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$NearbyUpdateMessageImplToJson(
        _$NearbyUpdateMessageImpl instance) =>
    <String, dynamic>{
      'devices': instance.devices,
      'type': instance.$type,
    };

_$PeerOfflineMessageImpl _$$PeerOfflineMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$PeerOfflineMessageImpl(
      deviceCode: json['device_code'] as String,
      reason: json['reason'] as String,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$PeerOfflineMessageImplToJson(
        _$PeerOfflineMessageImpl instance) =>
    <String, dynamic>{
      'device_code': instance.deviceCode,
      'reason': instance.reason,
      'type': instance.$type,
    };

_$RelayAssignedMessageImpl _$$RelayAssignedMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$RelayAssignedMessageImpl(
      sessionId: json['session_id'] as String,
      relayAddr: json['relay_addr'] as String,
      relayPort: (json['relay_port'] as num).toInt(),
      token: json['token'] as String,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$RelayAssignedMessageImplToJson(
        _$RelayAssignedMessageImpl instance) =>
    <String, dynamic>{
      'session_id': instance.sessionId,
      'relay_addr': instance.relayAddr,
      'relay_port': instance.relayPort,
      'token': instance.token,
      'type': instance.$type,
    };

_$InviteGeneratedMessageImpl _$$InviteGeneratedMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$InviteGeneratedMessageImpl(
      inviteCode: json['invite_code'] as String,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$InviteGeneratedMessageImplToJson(
        _$InviteGeneratedMessageImpl instance) =>
    <String, dynamic>{
      'invite_code': instance.inviteCode,
      'type': instance.$type,
    };

_$InviteResultMessageImpl _$$InviteResultMessageImplFromJson(
        Map<String, dynamic> json) =>
    _$InviteResultMessageImpl(
      sessionId: json['session_id'] as String,
      toCode: json['to_code'] as String,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$InviteResultMessageImplToJson(
        _$InviteResultMessageImpl instance) =>
    <String, dynamic>{
      'session_id': instance.sessionId,
      'to_code': instance.toCode,
      'type': instance.$type,
    };

_$ErrorMessageImpl _$$ErrorMessageImplFromJson(Map<String, dynamic> json) =>
    _$ErrorMessageImpl(
      code: json['code'] as String,
      message: json['message'] as String,
      $type: json['type'] as String?,
    );

Map<String, dynamic> _$$ErrorMessageImplToJson(_$ErrorMessageImpl instance) =>
    <String, dynamic>{
      'code': instance.code,
      'message': instance.message,
      'type': instance.$type,
    };
