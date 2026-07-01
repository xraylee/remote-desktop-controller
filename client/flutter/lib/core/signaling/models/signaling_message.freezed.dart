// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'signaling_message.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

IceCandidate _$IceCandidateFromJson(Map<String, dynamic> json) {
  return _IceCandidate.fromJson(json);
}

/// @nodoc
mixin _$IceCandidate {
  String get candidate => throw _privateConstructorUsedError;
  String? get sdpMid => throw _privateConstructorUsedError;
  int? get sdpMLineIndex => throw _privateConstructorUsedError;

  /// Serializes this IceCandidate to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of IceCandidate
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $IceCandidateCopyWith<IceCandidate> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $IceCandidateCopyWith<$Res> {
  factory $IceCandidateCopyWith(
          IceCandidate value, $Res Function(IceCandidate) then) =
      _$IceCandidateCopyWithImpl<$Res, IceCandidate>;
  @useResult
  $Res call({String candidate, String? sdpMid, int? sdpMLineIndex});
}

/// @nodoc
class _$IceCandidateCopyWithImpl<$Res, $Val extends IceCandidate>
    implements $IceCandidateCopyWith<$Res> {
  _$IceCandidateCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of IceCandidate
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? candidate = null,
    Object? sdpMid = freezed,
    Object? sdpMLineIndex = freezed,
  }) {
    return _then(_value.copyWith(
      candidate: null == candidate
          ? _value.candidate
          : candidate // ignore: cast_nullable_to_non_nullable
              as String,
      sdpMid: freezed == sdpMid
          ? _value.sdpMid
          : sdpMid // ignore: cast_nullable_to_non_nullable
              as String?,
      sdpMLineIndex: freezed == sdpMLineIndex
          ? _value.sdpMLineIndex
          : sdpMLineIndex // ignore: cast_nullable_to_non_nullable
              as int?,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$IceCandidateImplCopyWith<$Res>
    implements $IceCandidateCopyWith<$Res> {
  factory _$$IceCandidateImplCopyWith(
          _$IceCandidateImpl value, $Res Function(_$IceCandidateImpl) then) =
      __$$IceCandidateImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String candidate, String? sdpMid, int? sdpMLineIndex});
}

/// @nodoc
class __$$IceCandidateImplCopyWithImpl<$Res>
    extends _$IceCandidateCopyWithImpl<$Res, _$IceCandidateImpl>
    implements _$$IceCandidateImplCopyWith<$Res> {
  __$$IceCandidateImplCopyWithImpl(
      _$IceCandidateImpl _value, $Res Function(_$IceCandidateImpl) _then)
      : super(_value, _then);

  /// Create a copy of IceCandidate
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? candidate = null,
    Object? sdpMid = freezed,
    Object? sdpMLineIndex = freezed,
  }) {
    return _then(_$IceCandidateImpl(
      candidate: null == candidate
          ? _value.candidate
          : candidate // ignore: cast_nullable_to_non_nullable
              as String,
      sdpMid: freezed == sdpMid
          ? _value.sdpMid
          : sdpMid // ignore: cast_nullable_to_non_nullable
              as String?,
      sdpMLineIndex: freezed == sdpMLineIndex
          ? _value.sdpMLineIndex
          : sdpMLineIndex // ignore: cast_nullable_to_non_nullable
              as int?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$IceCandidateImpl implements _IceCandidate {
  const _$IceCandidateImpl(
      {required this.candidate, this.sdpMid, this.sdpMLineIndex});

  factory _$IceCandidateImpl.fromJson(Map<String, dynamic> json) =>
      _$$IceCandidateImplFromJson(json);

  @override
  final String candidate;
  @override
  final String? sdpMid;
  @override
  final int? sdpMLineIndex;

  @override
  String toString() {
    return 'IceCandidate(candidate: $candidate, sdpMid: $sdpMid, sdpMLineIndex: $sdpMLineIndex)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$IceCandidateImpl &&
            (identical(other.candidate, candidate) ||
                other.candidate == candidate) &&
            (identical(other.sdpMid, sdpMid) || other.sdpMid == sdpMid) &&
            (identical(other.sdpMLineIndex, sdpMLineIndex) ||
                other.sdpMLineIndex == sdpMLineIndex));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, candidate, sdpMid, sdpMLineIndex);

  /// Create a copy of IceCandidate
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$IceCandidateImplCopyWith<_$IceCandidateImpl> get copyWith =>
      __$$IceCandidateImplCopyWithImpl<_$IceCandidateImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$IceCandidateImplToJson(
      this,
    );
  }
}

abstract class _IceCandidate implements IceCandidate {
  const factory _IceCandidate(
      {required final String candidate,
      final String? sdpMid,
      final int? sdpMLineIndex}) = _$IceCandidateImpl;

  factory _IceCandidate.fromJson(Map<String, dynamic> json) =
      _$IceCandidateImpl.fromJson;

  @override
  String get candidate;
  @override
  String? get sdpMid;
  @override
  int? get sdpMLineIndex;

  /// Create a copy of IceCandidate
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$IceCandidateImplCopyWith<_$IceCandidateImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

DeviceInfo _$DeviceInfoFromJson(Map<String, dynamic> json) {
  return _DeviceInfo.fromJson(json);
}

/// @nodoc
mixin _$DeviceInfo {
  String get code => throw _privateConstructorUsedError;
  String get name => throw _privateConstructorUsedError;
  String get platform => throw _privateConstructorUsedError;
  bool get online => throw _privateConstructorUsedError;

  /// Serializes this DeviceInfo to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of DeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $DeviceInfoCopyWith<DeviceInfo> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $DeviceInfoCopyWith<$Res> {
  factory $DeviceInfoCopyWith(
          DeviceInfo value, $Res Function(DeviceInfo) then) =
      _$DeviceInfoCopyWithImpl<$Res, DeviceInfo>;
  @useResult
  $Res call({String code, String name, String platform, bool online});
}

/// @nodoc
class _$DeviceInfoCopyWithImpl<$Res, $Val extends DeviceInfo>
    implements $DeviceInfoCopyWith<$Res> {
  _$DeviceInfoCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of DeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? code = null,
    Object? name = null,
    Object? platform = null,
    Object? online = null,
  }) {
    return _then(_value.copyWith(
      code: null == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      platform: null == platform
          ? _value.platform
          : platform // ignore: cast_nullable_to_non_nullable
              as String,
      online: null == online
          ? _value.online
          : online // ignore: cast_nullable_to_non_nullable
              as bool,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$DeviceInfoImplCopyWith<$Res>
    implements $DeviceInfoCopyWith<$Res> {
  factory _$$DeviceInfoImplCopyWith(
          _$DeviceInfoImpl value, $Res Function(_$DeviceInfoImpl) then) =
      __$$DeviceInfoImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String code, String name, String platform, bool online});
}

/// @nodoc
class __$$DeviceInfoImplCopyWithImpl<$Res>
    extends _$DeviceInfoCopyWithImpl<$Res, _$DeviceInfoImpl>
    implements _$$DeviceInfoImplCopyWith<$Res> {
  __$$DeviceInfoImplCopyWithImpl(
      _$DeviceInfoImpl _value, $Res Function(_$DeviceInfoImpl) _then)
      : super(_value, _then);

  /// Create a copy of DeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? code = null,
    Object? name = null,
    Object? platform = null,
    Object? online = null,
  }) {
    return _then(_$DeviceInfoImpl(
      code: null == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String,
      name: null == name
          ? _value.name
          : name // ignore: cast_nullable_to_non_nullable
              as String,
      platform: null == platform
          ? _value.platform
          : platform // ignore: cast_nullable_to_non_nullable
              as String,
      online: null == online
          ? _value.online
          : online // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$DeviceInfoImpl implements _DeviceInfo {
  const _$DeviceInfoImpl(
      {required this.code,
      required this.name,
      required this.platform,
      required this.online});

  factory _$DeviceInfoImpl.fromJson(Map<String, dynamic> json) =>
      _$$DeviceInfoImplFromJson(json);

  @override
  final String code;
  @override
  final String name;
  @override
  final String platform;
  @override
  final bool online;

  @override
  String toString() {
    return 'DeviceInfo(code: $code, name: $name, platform: $platform, online: $online)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$DeviceInfoImpl &&
            (identical(other.code, code) || other.code == code) &&
            (identical(other.name, name) || other.name == name) &&
            (identical(other.platform, platform) ||
                other.platform == platform) &&
            (identical(other.online, online) || other.online == online));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, code, name, platform, online);

  /// Create a copy of DeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$DeviceInfoImplCopyWith<_$DeviceInfoImpl> get copyWith =>
      __$$DeviceInfoImplCopyWithImpl<_$DeviceInfoImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$DeviceInfoImplToJson(
      this,
    );
  }
}

abstract class _DeviceInfo implements DeviceInfo {
  const factory _DeviceInfo(
      {required final String code,
      required final String name,
      required final String platform,
      required final bool online}) = _$DeviceInfoImpl;

  factory _DeviceInfo.fromJson(Map<String, dynamic> json) =
      _$DeviceInfoImpl.fromJson;

  @override
  String get code;
  @override
  String get name;
  @override
  String get platform;
  @override
  bool get online;

  /// Create a copy of DeviceInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$DeviceInfoImplCopyWith<_$DeviceInfoImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

SignalingMessage _$SignalingMessageFromJson(Map<String, dynamic> json) {
  switch (json['type']) {
    case 'register':
      return RegisterMessage.fromJson(json);
    case 'heartbeat':
      return HeartbeatMessage.fromJson(json);
    case 'connectRequest':
      return ConnectRequestMessage.fromJson(json);
    case 'connectResponse':
      return ConnectResponseMessage.fromJson(json);
    case 'iceOffer':
      return IceOfferMessage.fromJson(json);
    case 'iceAnswer':
      return IceAnswerMessage.fromJson(json);
    case 'iceTrickle':
      return IceTrickleMessage.fromJson(json);
    case 'relayRequest':
      return RelayRequestMessage.fromJson(json);
    case 'generateInvite':
      return GenerateInviteMessage.fromJson(json);
    case 'useInvite':
      return UseInviteMessage.fromJson(json);
    case 'nearbyUpdate':
      return NearbyUpdateMessage.fromJson(json);
    case 'peerOffline':
      return PeerOfflineMessage.fromJson(json);
    case 'relayAssigned':
      return RelayAssignedMessage.fromJson(json);
    case 'inviteGenerated':
      return InviteGeneratedMessage.fromJson(json);
    case 'inviteResult':
      return InviteResultMessage.fromJson(json);
    case 'error':
      return ErrorMessage.fromJson(json);

    default:
      throw CheckedFromJsonException(json, 'type', 'SignalingMessage',
          'Invalid union type "${json['type']}"!');
  }
}

/// @nodoc
mixin _$SignalingMessage {
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) =>
      throw _privateConstructorUsedError;
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) =>
      throw _privateConstructorUsedError;

  /// Serializes this SignalingMessage to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SignalingMessageCopyWith<$Res> {
  factory $SignalingMessageCopyWith(
          SignalingMessage value, $Res Function(SignalingMessage) then) =
      _$SignalingMessageCopyWithImpl<$Res, SignalingMessage>;
}

/// @nodoc
class _$SignalingMessageCopyWithImpl<$Res, $Val extends SignalingMessage>
    implements $SignalingMessageCopyWith<$Res> {
  _$SignalingMessageCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
}

/// @nodoc
abstract class _$$RegisterMessageImplCopyWith<$Res> {
  factory _$$RegisterMessageImplCopyWith(_$RegisterMessageImpl value,
          $Res Function(_$RegisterMessageImpl) then) =
      __$$RegisterMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {@JsonKey(name: 'device_code') String deviceCode,
      String platform,
      String version,
      @JsonKey(name: 'team_id') String? teamId});
}

/// @nodoc
class __$$RegisterMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$RegisterMessageImpl>
    implements _$$RegisterMessageImplCopyWith<$Res> {
  __$$RegisterMessageImplCopyWithImpl(
      _$RegisterMessageImpl _value, $Res Function(_$RegisterMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceCode = null,
    Object? platform = null,
    Object? version = null,
    Object? teamId = freezed,
  }) {
    return _then(_$RegisterMessageImpl(
      deviceCode: null == deviceCode
          ? _value.deviceCode
          : deviceCode // ignore: cast_nullable_to_non_nullable
              as String,
      platform: null == platform
          ? _value.platform
          : platform // ignore: cast_nullable_to_non_nullable
              as String,
      version: null == version
          ? _value.version
          : version // ignore: cast_nullable_to_non_nullable
              as String,
      teamId: freezed == teamId
          ? _value.teamId
          : teamId // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$RegisterMessageImpl implements RegisterMessage {
  const _$RegisterMessageImpl(
      {@JsonKey(name: 'device_code') required this.deviceCode,
      required this.platform,
      required this.version,
      @JsonKey(name: 'team_id') this.teamId,
      final String? $type})
      : $type = $type ?? 'register';

  factory _$RegisterMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$RegisterMessageImplFromJson(json);

  @override
  @JsonKey(name: 'device_code')
  final String deviceCode;
  @override
  final String platform;
  @override
  final String version;
  @override
  @JsonKey(name: 'team_id')
  final String? teamId;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.register(deviceCode: $deviceCode, platform: $platform, version: $version, teamId: $teamId)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$RegisterMessageImpl &&
            (identical(other.deviceCode, deviceCode) ||
                other.deviceCode == deviceCode) &&
            (identical(other.platform, platform) ||
                other.platform == platform) &&
            (identical(other.version, version) || other.version == version) &&
            (identical(other.teamId, teamId) || other.teamId == teamId));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, deviceCode, platform, version, teamId);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$RegisterMessageImplCopyWith<_$RegisterMessageImpl> get copyWith =>
      __$$RegisterMessageImplCopyWithImpl<_$RegisterMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return register(deviceCode, platform, version, teamId);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return register?.call(deviceCode, platform, version, teamId);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (register != null) {
      return register(deviceCode, platform, version, teamId);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return register(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return register?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (register != null) {
      return register(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$RegisterMessageImplToJson(
      this,
    );
  }
}

abstract class RegisterMessage implements SignalingMessage {
  const factory RegisterMessage(
      {@JsonKey(name: 'device_code') required final String deviceCode,
      required final String platform,
      required final String version,
      @JsonKey(name: 'team_id') final String? teamId}) = _$RegisterMessageImpl;

  factory RegisterMessage.fromJson(Map<String, dynamic> json) =
      _$RegisterMessageImpl.fromJson;

  @JsonKey(name: 'device_code')
  String get deviceCode;
  String get platform;
  String get version;
  @JsonKey(name: 'team_id')
  String? get teamId;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$RegisterMessageImplCopyWith<_$RegisterMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$HeartbeatMessageImplCopyWith<$Res> {
  factory _$$HeartbeatMessageImplCopyWith(_$HeartbeatMessageImpl value,
          $Res Function(_$HeartbeatMessageImpl) then) =
      __$$HeartbeatMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call({@JsonKey(name: 'device_code') String deviceCode, int ts});
}

/// @nodoc
class __$$HeartbeatMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$HeartbeatMessageImpl>
    implements _$$HeartbeatMessageImplCopyWith<$Res> {
  __$$HeartbeatMessageImplCopyWithImpl(_$HeartbeatMessageImpl _value,
      $Res Function(_$HeartbeatMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceCode = null,
    Object? ts = null,
  }) {
    return _then(_$HeartbeatMessageImpl(
      deviceCode: null == deviceCode
          ? _value.deviceCode
          : deviceCode // ignore: cast_nullable_to_non_nullable
              as String,
      ts: null == ts
          ? _value.ts
          : ts // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$HeartbeatMessageImpl implements HeartbeatMessage {
  const _$HeartbeatMessageImpl(
      {@JsonKey(name: 'device_code') required this.deviceCode,
      required this.ts,
      final String? $type})
      : $type = $type ?? 'heartbeat';

  factory _$HeartbeatMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$HeartbeatMessageImplFromJson(json);

  @override
  @JsonKey(name: 'device_code')
  final String deviceCode;
  @override
  final int ts;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.heartbeat(deviceCode: $deviceCode, ts: $ts)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$HeartbeatMessageImpl &&
            (identical(other.deviceCode, deviceCode) ||
                other.deviceCode == deviceCode) &&
            (identical(other.ts, ts) || other.ts == ts));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, deviceCode, ts);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$HeartbeatMessageImplCopyWith<_$HeartbeatMessageImpl> get copyWith =>
      __$$HeartbeatMessageImplCopyWithImpl<_$HeartbeatMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return heartbeat(deviceCode, ts);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return heartbeat?.call(deviceCode, ts);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (heartbeat != null) {
      return heartbeat(deviceCode, ts);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return heartbeat(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return heartbeat?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (heartbeat != null) {
      return heartbeat(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$HeartbeatMessageImplToJson(
      this,
    );
  }
}

abstract class HeartbeatMessage implements SignalingMessage {
  const factory HeartbeatMessage(
      {@JsonKey(name: 'device_code') required final String deviceCode,
      required final int ts}) = _$HeartbeatMessageImpl;

  factory HeartbeatMessage.fromJson(Map<String, dynamic> json) =
      _$HeartbeatMessageImpl.fromJson;

  @JsonKey(name: 'device_code')
  String get deviceCode;
  int get ts;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$HeartbeatMessageImplCopyWith<_$HeartbeatMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ConnectRequestMessageImplCopyWith<$Res> {
  factory _$$ConnectRequestMessageImplCopyWith(
          _$ConnectRequestMessageImpl value,
          $Res Function(_$ConnectRequestMessageImpl) then) =
      __$$ConnectRequestMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {@JsonKey(name: 'from_code') String fromCode,
      @JsonKey(name: 'to_code') String toCode,
      @JsonKey(name: 'invite_code') String? inviteCode});
}

/// @nodoc
class __$$ConnectRequestMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$ConnectRequestMessageImpl>
    implements _$$ConnectRequestMessageImplCopyWith<$Res> {
  __$$ConnectRequestMessageImplCopyWithImpl(_$ConnectRequestMessageImpl _value,
      $Res Function(_$ConnectRequestMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? fromCode = null,
    Object? toCode = null,
    Object? inviteCode = freezed,
  }) {
    return _then(_$ConnectRequestMessageImpl(
      fromCode: null == fromCode
          ? _value.fromCode
          : fromCode // ignore: cast_nullable_to_non_nullable
              as String,
      toCode: null == toCode
          ? _value.toCode
          : toCode // ignore: cast_nullable_to_non_nullable
              as String,
      inviteCode: freezed == inviteCode
          ? _value.inviteCode
          : inviteCode // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ConnectRequestMessageImpl implements ConnectRequestMessage {
  const _$ConnectRequestMessageImpl(
      {@JsonKey(name: 'from_code') required this.fromCode,
      @JsonKey(name: 'to_code') required this.toCode,
      @JsonKey(name: 'invite_code') this.inviteCode,
      final String? $type})
      : $type = $type ?? 'connectRequest';

  factory _$ConnectRequestMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$ConnectRequestMessageImplFromJson(json);

  @override
  @JsonKey(name: 'from_code')
  final String fromCode;
  @override
  @JsonKey(name: 'to_code')
  final String toCode;
  @override
  @JsonKey(name: 'invite_code')
  final String? inviteCode;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.connectRequest(fromCode: $fromCode, toCode: $toCode, inviteCode: $inviteCode)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ConnectRequestMessageImpl &&
            (identical(other.fromCode, fromCode) ||
                other.fromCode == fromCode) &&
            (identical(other.toCode, toCode) || other.toCode == toCode) &&
            (identical(other.inviteCode, inviteCode) ||
                other.inviteCode == inviteCode));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, fromCode, toCode, inviteCode);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ConnectRequestMessageImplCopyWith<_$ConnectRequestMessageImpl>
      get copyWith => __$$ConnectRequestMessageImplCopyWithImpl<
          _$ConnectRequestMessageImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return connectRequest(fromCode, toCode, inviteCode);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return connectRequest?.call(fromCode, toCode, inviteCode);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (connectRequest != null) {
      return connectRequest(fromCode, toCode, inviteCode);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return connectRequest(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return connectRequest?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (connectRequest != null) {
      return connectRequest(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$ConnectRequestMessageImplToJson(
      this,
    );
  }
}

abstract class ConnectRequestMessage implements SignalingMessage {
  const factory ConnectRequestMessage(
          {@JsonKey(name: 'from_code') required final String fromCode,
          @JsonKey(name: 'to_code') required final String toCode,
          @JsonKey(name: 'invite_code') final String? inviteCode}) =
      _$ConnectRequestMessageImpl;

  factory ConnectRequestMessage.fromJson(Map<String, dynamic> json) =
      _$ConnectRequestMessageImpl.fromJson;

  @JsonKey(name: 'from_code')
  String get fromCode;
  @JsonKey(name: 'to_code')
  String get toCode;
  @JsonKey(name: 'invite_code')
  String? get inviteCode;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ConnectRequestMessageImplCopyWith<_$ConnectRequestMessageImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ConnectResponseMessageImplCopyWith<$Res> {
  factory _$$ConnectResponseMessageImplCopyWith(
          _$ConnectResponseMessageImpl value,
          $Res Function(_$ConnectResponseMessageImpl) then) =
      __$$ConnectResponseMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {bool accepted,
      @JsonKey(name: 'session_id') String sessionId,
      @JsonKey(name: 'from_code') String fromCode});
}

/// @nodoc
class __$$ConnectResponseMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$ConnectResponseMessageImpl>
    implements _$$ConnectResponseMessageImplCopyWith<$Res> {
  __$$ConnectResponseMessageImplCopyWithImpl(
      _$ConnectResponseMessageImpl _value,
      $Res Function(_$ConnectResponseMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? accepted = null,
    Object? sessionId = null,
    Object? fromCode = null,
  }) {
    return _then(_$ConnectResponseMessageImpl(
      accepted: null == accepted
          ? _value.accepted
          : accepted // ignore: cast_nullable_to_non_nullable
              as bool,
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as String,
      fromCode: null == fromCode
          ? _value.fromCode
          : fromCode // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ConnectResponseMessageImpl implements ConnectResponseMessage {
  const _$ConnectResponseMessageImpl(
      {required this.accepted,
      @JsonKey(name: 'session_id') required this.sessionId,
      @JsonKey(name: 'from_code') required this.fromCode,
      final String? $type})
      : $type = $type ?? 'connectResponse';

  factory _$ConnectResponseMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$ConnectResponseMessageImplFromJson(json);

  @override
  final bool accepted;
  @override
  @JsonKey(name: 'session_id')
  final String sessionId;
  @override
  @JsonKey(name: 'from_code')
  final String fromCode;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.connectResponse(accepted: $accepted, sessionId: $sessionId, fromCode: $fromCode)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ConnectResponseMessageImpl &&
            (identical(other.accepted, accepted) ||
                other.accepted == accepted) &&
            (identical(other.sessionId, sessionId) ||
                other.sessionId == sessionId) &&
            (identical(other.fromCode, fromCode) ||
                other.fromCode == fromCode));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, accepted, sessionId, fromCode);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ConnectResponseMessageImplCopyWith<_$ConnectResponseMessageImpl>
      get copyWith => __$$ConnectResponseMessageImplCopyWithImpl<
          _$ConnectResponseMessageImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return connectResponse(accepted, sessionId, fromCode);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return connectResponse?.call(accepted, sessionId, fromCode);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (connectResponse != null) {
      return connectResponse(accepted, sessionId, fromCode);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return connectResponse(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return connectResponse?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (connectResponse != null) {
      return connectResponse(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$ConnectResponseMessageImplToJson(
      this,
    );
  }
}

abstract class ConnectResponseMessage implements SignalingMessage {
  const factory ConnectResponseMessage(
          {required final bool accepted,
          @JsonKey(name: 'session_id') required final String sessionId,
          @JsonKey(name: 'from_code') required final String fromCode}) =
      _$ConnectResponseMessageImpl;

  factory ConnectResponseMessage.fromJson(Map<String, dynamic> json) =
      _$ConnectResponseMessageImpl.fromJson;

  bool get accepted;
  @JsonKey(name: 'session_id')
  String get sessionId;
  @JsonKey(name: 'from_code')
  String get fromCode;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ConnectResponseMessageImplCopyWith<_$ConnectResponseMessageImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$IceOfferMessageImplCopyWith<$Res> {
  factory _$$IceOfferMessageImplCopyWith(_$IceOfferMessageImpl value,
          $Res Function(_$IceOfferMessageImpl) then) =
      __$$IceOfferMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {@JsonKey(name: 'session_id') String sessionId,
      String sdp,
      List<IceCandidate> candidates});
}

/// @nodoc
class __$$IceOfferMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$IceOfferMessageImpl>
    implements _$$IceOfferMessageImplCopyWith<$Res> {
  __$$IceOfferMessageImplCopyWithImpl(
      _$IceOfferMessageImpl _value, $Res Function(_$IceOfferMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sessionId = null,
    Object? sdp = null,
    Object? candidates = null,
  }) {
    return _then(_$IceOfferMessageImpl(
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as String,
      sdp: null == sdp
          ? _value.sdp
          : sdp // ignore: cast_nullable_to_non_nullable
              as String,
      candidates: null == candidates
          ? _value._candidates
          : candidates // ignore: cast_nullable_to_non_nullable
              as List<IceCandidate>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$IceOfferMessageImpl implements IceOfferMessage {
  const _$IceOfferMessageImpl(
      {@JsonKey(name: 'session_id') required this.sessionId,
      required this.sdp,
      required final List<IceCandidate> candidates,
      final String? $type})
      : _candidates = candidates,
        $type = $type ?? 'iceOffer';

  factory _$IceOfferMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$IceOfferMessageImplFromJson(json);

  @override
  @JsonKey(name: 'session_id')
  final String sessionId;
  @override
  final String sdp;
  final List<IceCandidate> _candidates;
  @override
  List<IceCandidate> get candidates {
    if (_candidates is EqualUnmodifiableListView) return _candidates;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_candidates);
  }

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.iceOffer(sessionId: $sessionId, sdp: $sdp, candidates: $candidates)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$IceOfferMessageImpl &&
            (identical(other.sessionId, sessionId) ||
                other.sessionId == sessionId) &&
            (identical(other.sdp, sdp) || other.sdp == sdp) &&
            const DeepCollectionEquality()
                .equals(other._candidates, _candidates));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, sessionId, sdp,
      const DeepCollectionEquality().hash(_candidates));

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$IceOfferMessageImplCopyWith<_$IceOfferMessageImpl> get copyWith =>
      __$$IceOfferMessageImplCopyWithImpl<_$IceOfferMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return iceOffer(sessionId, sdp, candidates);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return iceOffer?.call(sessionId, sdp, candidates);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (iceOffer != null) {
      return iceOffer(sessionId, sdp, candidates);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return iceOffer(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return iceOffer?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (iceOffer != null) {
      return iceOffer(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$IceOfferMessageImplToJson(
      this,
    );
  }
}

abstract class IceOfferMessage implements SignalingMessage {
  const factory IceOfferMessage(
      {@JsonKey(name: 'session_id') required final String sessionId,
      required final String sdp,
      required final List<IceCandidate> candidates}) = _$IceOfferMessageImpl;

  factory IceOfferMessage.fromJson(Map<String, dynamic> json) =
      _$IceOfferMessageImpl.fromJson;

  @JsonKey(name: 'session_id')
  String get sessionId;
  String get sdp;
  List<IceCandidate> get candidates;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$IceOfferMessageImplCopyWith<_$IceOfferMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$IceAnswerMessageImplCopyWith<$Res> {
  factory _$$IceAnswerMessageImplCopyWith(_$IceAnswerMessageImpl value,
          $Res Function(_$IceAnswerMessageImpl) then) =
      __$$IceAnswerMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {@JsonKey(name: 'session_id') String sessionId,
      String sdp,
      List<IceCandidate> candidates});
}

/// @nodoc
class __$$IceAnswerMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$IceAnswerMessageImpl>
    implements _$$IceAnswerMessageImplCopyWith<$Res> {
  __$$IceAnswerMessageImplCopyWithImpl(_$IceAnswerMessageImpl _value,
      $Res Function(_$IceAnswerMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sessionId = null,
    Object? sdp = null,
    Object? candidates = null,
  }) {
    return _then(_$IceAnswerMessageImpl(
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as String,
      sdp: null == sdp
          ? _value.sdp
          : sdp // ignore: cast_nullable_to_non_nullable
              as String,
      candidates: null == candidates
          ? _value._candidates
          : candidates // ignore: cast_nullable_to_non_nullable
              as List<IceCandidate>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$IceAnswerMessageImpl implements IceAnswerMessage {
  const _$IceAnswerMessageImpl(
      {@JsonKey(name: 'session_id') required this.sessionId,
      required this.sdp,
      required final List<IceCandidate> candidates,
      final String? $type})
      : _candidates = candidates,
        $type = $type ?? 'iceAnswer';

  factory _$IceAnswerMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$IceAnswerMessageImplFromJson(json);

  @override
  @JsonKey(name: 'session_id')
  final String sessionId;
  @override
  final String sdp;
  final List<IceCandidate> _candidates;
  @override
  List<IceCandidate> get candidates {
    if (_candidates is EqualUnmodifiableListView) return _candidates;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_candidates);
  }

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.iceAnswer(sessionId: $sessionId, sdp: $sdp, candidates: $candidates)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$IceAnswerMessageImpl &&
            (identical(other.sessionId, sessionId) ||
                other.sessionId == sessionId) &&
            (identical(other.sdp, sdp) || other.sdp == sdp) &&
            const DeepCollectionEquality()
                .equals(other._candidates, _candidates));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, sessionId, sdp,
      const DeepCollectionEquality().hash(_candidates));

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$IceAnswerMessageImplCopyWith<_$IceAnswerMessageImpl> get copyWith =>
      __$$IceAnswerMessageImplCopyWithImpl<_$IceAnswerMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return iceAnswer(sessionId, sdp, candidates);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return iceAnswer?.call(sessionId, sdp, candidates);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (iceAnswer != null) {
      return iceAnswer(sessionId, sdp, candidates);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return iceAnswer(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return iceAnswer?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (iceAnswer != null) {
      return iceAnswer(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$IceAnswerMessageImplToJson(
      this,
    );
  }
}

abstract class IceAnswerMessage implements SignalingMessage {
  const factory IceAnswerMessage(
      {@JsonKey(name: 'session_id') required final String sessionId,
      required final String sdp,
      required final List<IceCandidate> candidates}) = _$IceAnswerMessageImpl;

  factory IceAnswerMessage.fromJson(Map<String, dynamic> json) =
      _$IceAnswerMessageImpl.fromJson;

  @JsonKey(name: 'session_id')
  String get sessionId;
  String get sdp;
  List<IceCandidate> get candidates;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$IceAnswerMessageImplCopyWith<_$IceAnswerMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$IceTrickleMessageImplCopyWith<$Res> {
  factory _$$IceTrickleMessageImplCopyWith(_$IceTrickleMessageImpl value,
          $Res Function(_$IceTrickleMessageImpl) then) =
      __$$IceTrickleMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {@JsonKey(name: 'session_id') String sessionId, IceCandidate candidate});

  $IceCandidateCopyWith<$Res> get candidate;
}

/// @nodoc
class __$$IceTrickleMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$IceTrickleMessageImpl>
    implements _$$IceTrickleMessageImplCopyWith<$Res> {
  __$$IceTrickleMessageImplCopyWithImpl(_$IceTrickleMessageImpl _value,
      $Res Function(_$IceTrickleMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sessionId = null,
    Object? candidate = null,
  }) {
    return _then(_$IceTrickleMessageImpl(
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as String,
      candidate: null == candidate
          ? _value.candidate
          : candidate // ignore: cast_nullable_to_non_nullable
              as IceCandidate,
    ));
  }

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $IceCandidateCopyWith<$Res> get candidate {
    return $IceCandidateCopyWith<$Res>(_value.candidate, (value) {
      return _then(_value.copyWith(candidate: value));
    });
  }
}

/// @nodoc
@JsonSerializable()
class _$IceTrickleMessageImpl implements IceTrickleMessage {
  const _$IceTrickleMessageImpl(
      {@JsonKey(name: 'session_id') required this.sessionId,
      required this.candidate,
      final String? $type})
      : $type = $type ?? 'iceTrickle';

  factory _$IceTrickleMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$IceTrickleMessageImplFromJson(json);

  @override
  @JsonKey(name: 'session_id')
  final String sessionId;
  @override
  final IceCandidate candidate;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.iceTrickle(sessionId: $sessionId, candidate: $candidate)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$IceTrickleMessageImpl &&
            (identical(other.sessionId, sessionId) ||
                other.sessionId == sessionId) &&
            (identical(other.candidate, candidate) ||
                other.candidate == candidate));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, sessionId, candidate);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$IceTrickleMessageImplCopyWith<_$IceTrickleMessageImpl> get copyWith =>
      __$$IceTrickleMessageImplCopyWithImpl<_$IceTrickleMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return iceTrickle(sessionId, candidate);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return iceTrickle?.call(sessionId, candidate);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (iceTrickle != null) {
      return iceTrickle(sessionId, candidate);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return iceTrickle(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return iceTrickle?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (iceTrickle != null) {
      return iceTrickle(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$IceTrickleMessageImplToJson(
      this,
    );
  }
}

abstract class IceTrickleMessage implements SignalingMessage {
  const factory IceTrickleMessage(
      {@JsonKey(name: 'session_id') required final String sessionId,
      required final IceCandidate candidate}) = _$IceTrickleMessageImpl;

  factory IceTrickleMessage.fromJson(Map<String, dynamic> json) =
      _$IceTrickleMessageImpl.fromJson;

  @JsonKey(name: 'session_id')
  String get sessionId;
  IceCandidate get candidate;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$IceTrickleMessageImplCopyWith<_$IceTrickleMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$RelayRequestMessageImplCopyWith<$Res> {
  factory _$$RelayRequestMessageImplCopyWith(_$RelayRequestMessageImpl value,
          $Res Function(_$RelayRequestMessageImpl) then) =
      __$$RelayRequestMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {@JsonKey(name: 'session_id') String sessionId,
      @JsonKey(name: 'preferred_region') String? preferredRegion});
}

/// @nodoc
class __$$RelayRequestMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$RelayRequestMessageImpl>
    implements _$$RelayRequestMessageImplCopyWith<$Res> {
  __$$RelayRequestMessageImplCopyWithImpl(_$RelayRequestMessageImpl _value,
      $Res Function(_$RelayRequestMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sessionId = null,
    Object? preferredRegion = freezed,
  }) {
    return _then(_$RelayRequestMessageImpl(
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as String,
      preferredRegion: freezed == preferredRegion
          ? _value.preferredRegion
          : preferredRegion // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$RelayRequestMessageImpl implements RelayRequestMessage {
  const _$RelayRequestMessageImpl(
      {@JsonKey(name: 'session_id') required this.sessionId,
      @JsonKey(name: 'preferred_region') this.preferredRegion,
      final String? $type})
      : $type = $type ?? 'relayRequest';

  factory _$RelayRequestMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$RelayRequestMessageImplFromJson(json);

  @override
  @JsonKey(name: 'session_id')
  final String sessionId;
  @override
  @JsonKey(name: 'preferred_region')
  final String? preferredRegion;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.relayRequest(sessionId: $sessionId, preferredRegion: $preferredRegion)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$RelayRequestMessageImpl &&
            (identical(other.sessionId, sessionId) ||
                other.sessionId == sessionId) &&
            (identical(other.preferredRegion, preferredRegion) ||
                other.preferredRegion == preferredRegion));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, sessionId, preferredRegion);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$RelayRequestMessageImplCopyWith<_$RelayRequestMessageImpl> get copyWith =>
      __$$RelayRequestMessageImplCopyWithImpl<_$RelayRequestMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return relayRequest(sessionId, preferredRegion);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return relayRequest?.call(sessionId, preferredRegion);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (relayRequest != null) {
      return relayRequest(sessionId, preferredRegion);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return relayRequest(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return relayRequest?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (relayRequest != null) {
      return relayRequest(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$RelayRequestMessageImplToJson(
      this,
    );
  }
}

abstract class RelayRequestMessage implements SignalingMessage {
  const factory RelayRequestMessage(
          {@JsonKey(name: 'session_id') required final String sessionId,
          @JsonKey(name: 'preferred_region') final String? preferredRegion}) =
      _$RelayRequestMessageImpl;

  factory RelayRequestMessage.fromJson(Map<String, dynamic> json) =
      _$RelayRequestMessageImpl.fromJson;

  @JsonKey(name: 'session_id')
  String get sessionId;
  @JsonKey(name: 'preferred_region')
  String? get preferredRegion;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$RelayRequestMessageImplCopyWith<_$RelayRequestMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$GenerateInviteMessageImplCopyWith<$Res> {
  factory _$$GenerateInviteMessageImplCopyWith(
          _$GenerateInviteMessageImpl value,
          $Res Function(_$GenerateInviteMessageImpl) then) =
      __$$GenerateInviteMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call({@JsonKey(name: 'device_code') String deviceCode});
}

/// @nodoc
class __$$GenerateInviteMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$GenerateInviteMessageImpl>
    implements _$$GenerateInviteMessageImplCopyWith<$Res> {
  __$$GenerateInviteMessageImplCopyWithImpl(_$GenerateInviteMessageImpl _value,
      $Res Function(_$GenerateInviteMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceCode = null,
  }) {
    return _then(_$GenerateInviteMessageImpl(
      deviceCode: null == deviceCode
          ? _value.deviceCode
          : deviceCode // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$GenerateInviteMessageImpl implements GenerateInviteMessage {
  const _$GenerateInviteMessageImpl(
      {@JsonKey(name: 'device_code') required this.deviceCode,
      final String? $type})
      : $type = $type ?? 'generateInvite';

  factory _$GenerateInviteMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$GenerateInviteMessageImplFromJson(json);

  @override
  @JsonKey(name: 'device_code')
  final String deviceCode;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.generateInvite(deviceCode: $deviceCode)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$GenerateInviteMessageImpl &&
            (identical(other.deviceCode, deviceCode) ||
                other.deviceCode == deviceCode));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, deviceCode);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$GenerateInviteMessageImplCopyWith<_$GenerateInviteMessageImpl>
      get copyWith => __$$GenerateInviteMessageImplCopyWithImpl<
          _$GenerateInviteMessageImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return generateInvite(deviceCode);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return generateInvite?.call(deviceCode);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (generateInvite != null) {
      return generateInvite(deviceCode);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return generateInvite(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return generateInvite?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (generateInvite != null) {
      return generateInvite(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$GenerateInviteMessageImplToJson(
      this,
    );
  }
}

abstract class GenerateInviteMessage implements SignalingMessage {
  const factory GenerateInviteMessage(
          {@JsonKey(name: 'device_code') required final String deviceCode}) =
      _$GenerateInviteMessageImpl;

  factory GenerateInviteMessage.fromJson(Map<String, dynamic> json) =
      _$GenerateInviteMessageImpl.fromJson;

  @JsonKey(name: 'device_code')
  String get deviceCode;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$GenerateInviteMessageImplCopyWith<_$GenerateInviteMessageImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$UseInviteMessageImplCopyWith<$Res> {
  factory _$$UseInviteMessageImplCopyWith(_$UseInviteMessageImpl value,
          $Res Function(_$UseInviteMessageImpl) then) =
      __$$UseInviteMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {@JsonKey(name: 'from_code') String fromCode,
      @JsonKey(name: 'invite_code') String inviteCode});
}

/// @nodoc
class __$$UseInviteMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$UseInviteMessageImpl>
    implements _$$UseInviteMessageImplCopyWith<$Res> {
  __$$UseInviteMessageImplCopyWithImpl(_$UseInviteMessageImpl _value,
      $Res Function(_$UseInviteMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? fromCode = null,
    Object? inviteCode = null,
  }) {
    return _then(_$UseInviteMessageImpl(
      fromCode: null == fromCode
          ? _value.fromCode
          : fromCode // ignore: cast_nullable_to_non_nullable
              as String,
      inviteCode: null == inviteCode
          ? _value.inviteCode
          : inviteCode // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$UseInviteMessageImpl implements UseInviteMessage {
  const _$UseInviteMessageImpl(
      {@JsonKey(name: 'from_code') required this.fromCode,
      @JsonKey(name: 'invite_code') required this.inviteCode,
      final String? $type})
      : $type = $type ?? 'useInvite';

  factory _$UseInviteMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$UseInviteMessageImplFromJson(json);

  @override
  @JsonKey(name: 'from_code')
  final String fromCode;
  @override
  @JsonKey(name: 'invite_code')
  final String inviteCode;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.useInvite(fromCode: $fromCode, inviteCode: $inviteCode)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$UseInviteMessageImpl &&
            (identical(other.fromCode, fromCode) ||
                other.fromCode == fromCode) &&
            (identical(other.inviteCode, inviteCode) ||
                other.inviteCode == inviteCode));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, fromCode, inviteCode);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$UseInviteMessageImplCopyWith<_$UseInviteMessageImpl> get copyWith =>
      __$$UseInviteMessageImplCopyWithImpl<_$UseInviteMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return useInvite(fromCode, inviteCode);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return useInvite?.call(fromCode, inviteCode);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (useInvite != null) {
      return useInvite(fromCode, inviteCode);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return useInvite(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return useInvite?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (useInvite != null) {
      return useInvite(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$UseInviteMessageImplToJson(
      this,
    );
  }
}

abstract class UseInviteMessage implements SignalingMessage {
  const factory UseInviteMessage(
          {@JsonKey(name: 'from_code') required final String fromCode,
          @JsonKey(name: 'invite_code') required final String inviteCode}) =
      _$UseInviteMessageImpl;

  factory UseInviteMessage.fromJson(Map<String, dynamic> json) =
      _$UseInviteMessageImpl.fromJson;

  @JsonKey(name: 'from_code')
  String get fromCode;
  @JsonKey(name: 'invite_code')
  String get inviteCode;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$UseInviteMessageImplCopyWith<_$UseInviteMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$NearbyUpdateMessageImplCopyWith<$Res> {
  factory _$$NearbyUpdateMessageImplCopyWith(_$NearbyUpdateMessageImpl value,
          $Res Function(_$NearbyUpdateMessageImpl) then) =
      __$$NearbyUpdateMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call({List<DeviceInfo> devices});
}

/// @nodoc
class __$$NearbyUpdateMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$NearbyUpdateMessageImpl>
    implements _$$NearbyUpdateMessageImplCopyWith<$Res> {
  __$$NearbyUpdateMessageImplCopyWithImpl(_$NearbyUpdateMessageImpl _value,
      $Res Function(_$NearbyUpdateMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? devices = null,
  }) {
    return _then(_$NearbyUpdateMessageImpl(
      devices: null == devices
          ? _value._devices
          : devices // ignore: cast_nullable_to_non_nullable
              as List<DeviceInfo>,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$NearbyUpdateMessageImpl implements NearbyUpdateMessage {
  const _$NearbyUpdateMessageImpl(
      {required final List<DeviceInfo> devices, final String? $type})
      : _devices = devices,
        $type = $type ?? 'nearbyUpdate';

  factory _$NearbyUpdateMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$NearbyUpdateMessageImplFromJson(json);

  final List<DeviceInfo> _devices;
  @override
  List<DeviceInfo> get devices {
    if (_devices is EqualUnmodifiableListView) return _devices;
    // ignore: implicit_dynamic_type
    return EqualUnmodifiableListView(_devices);
  }

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.nearbyUpdate(devices: $devices)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$NearbyUpdateMessageImpl &&
            const DeepCollectionEquality().equals(other._devices, _devices));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, const DeepCollectionEquality().hash(_devices));

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$NearbyUpdateMessageImplCopyWith<_$NearbyUpdateMessageImpl> get copyWith =>
      __$$NearbyUpdateMessageImplCopyWithImpl<_$NearbyUpdateMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return nearbyUpdate(devices);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return nearbyUpdate?.call(devices);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (nearbyUpdate != null) {
      return nearbyUpdate(devices);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return nearbyUpdate(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return nearbyUpdate?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (nearbyUpdate != null) {
      return nearbyUpdate(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$NearbyUpdateMessageImplToJson(
      this,
    );
  }
}

abstract class NearbyUpdateMessage implements SignalingMessage {
  const factory NearbyUpdateMessage({required final List<DeviceInfo> devices}) =
      _$NearbyUpdateMessageImpl;

  factory NearbyUpdateMessage.fromJson(Map<String, dynamic> json) =
      _$NearbyUpdateMessageImpl.fromJson;

  List<DeviceInfo> get devices;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$NearbyUpdateMessageImplCopyWith<_$NearbyUpdateMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$PeerOfflineMessageImplCopyWith<$Res> {
  factory _$$PeerOfflineMessageImplCopyWith(_$PeerOfflineMessageImpl value,
          $Res Function(_$PeerOfflineMessageImpl) then) =
      __$$PeerOfflineMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call({@JsonKey(name: 'device_code') String deviceCode, String reason});
}

/// @nodoc
class __$$PeerOfflineMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$PeerOfflineMessageImpl>
    implements _$$PeerOfflineMessageImplCopyWith<$Res> {
  __$$PeerOfflineMessageImplCopyWithImpl(_$PeerOfflineMessageImpl _value,
      $Res Function(_$PeerOfflineMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceCode = null,
    Object? reason = null,
  }) {
    return _then(_$PeerOfflineMessageImpl(
      deviceCode: null == deviceCode
          ? _value.deviceCode
          : deviceCode // ignore: cast_nullable_to_non_nullable
              as String,
      reason: null == reason
          ? _value.reason
          : reason // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$PeerOfflineMessageImpl implements PeerOfflineMessage {
  const _$PeerOfflineMessageImpl(
      {@JsonKey(name: 'device_code') required this.deviceCode,
      required this.reason,
      final String? $type})
      : $type = $type ?? 'peerOffline';

  factory _$PeerOfflineMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$PeerOfflineMessageImplFromJson(json);

  @override
  @JsonKey(name: 'device_code')
  final String deviceCode;
  @override
  final String reason;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.peerOffline(deviceCode: $deviceCode, reason: $reason)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$PeerOfflineMessageImpl &&
            (identical(other.deviceCode, deviceCode) ||
                other.deviceCode == deviceCode) &&
            (identical(other.reason, reason) || other.reason == reason));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, deviceCode, reason);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$PeerOfflineMessageImplCopyWith<_$PeerOfflineMessageImpl> get copyWith =>
      __$$PeerOfflineMessageImplCopyWithImpl<_$PeerOfflineMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return peerOffline(deviceCode, reason);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return peerOffline?.call(deviceCode, reason);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (peerOffline != null) {
      return peerOffline(deviceCode, reason);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return peerOffline(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return peerOffline?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (peerOffline != null) {
      return peerOffline(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$PeerOfflineMessageImplToJson(
      this,
    );
  }
}

abstract class PeerOfflineMessage implements SignalingMessage {
  const factory PeerOfflineMessage(
      {@JsonKey(name: 'device_code') required final String deviceCode,
      required final String reason}) = _$PeerOfflineMessageImpl;

  factory PeerOfflineMessage.fromJson(Map<String, dynamic> json) =
      _$PeerOfflineMessageImpl.fromJson;

  @JsonKey(name: 'device_code')
  String get deviceCode;
  String get reason;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$PeerOfflineMessageImplCopyWith<_$PeerOfflineMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$RelayAssignedMessageImplCopyWith<$Res> {
  factory _$$RelayAssignedMessageImplCopyWith(_$RelayAssignedMessageImpl value,
          $Res Function(_$RelayAssignedMessageImpl) then) =
      __$$RelayAssignedMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {@JsonKey(name: 'session_id') String sessionId,
      @JsonKey(name: 'relay_addr') String relayAddr,
      @JsonKey(name: 'relay_port') int relayPort,
      String token});
}

/// @nodoc
class __$$RelayAssignedMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$RelayAssignedMessageImpl>
    implements _$$RelayAssignedMessageImplCopyWith<$Res> {
  __$$RelayAssignedMessageImplCopyWithImpl(_$RelayAssignedMessageImpl _value,
      $Res Function(_$RelayAssignedMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sessionId = null,
    Object? relayAddr = null,
    Object? relayPort = null,
    Object? token = null,
  }) {
    return _then(_$RelayAssignedMessageImpl(
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as String,
      relayAddr: null == relayAddr
          ? _value.relayAddr
          : relayAddr // ignore: cast_nullable_to_non_nullable
              as String,
      relayPort: null == relayPort
          ? _value.relayPort
          : relayPort // ignore: cast_nullable_to_non_nullable
              as int,
      token: null == token
          ? _value.token
          : token // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$RelayAssignedMessageImpl implements RelayAssignedMessage {
  const _$RelayAssignedMessageImpl(
      {@JsonKey(name: 'session_id') required this.sessionId,
      @JsonKey(name: 'relay_addr') required this.relayAddr,
      @JsonKey(name: 'relay_port') required this.relayPort,
      required this.token,
      final String? $type})
      : $type = $type ?? 'relayAssigned';

  factory _$RelayAssignedMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$RelayAssignedMessageImplFromJson(json);

  @override
  @JsonKey(name: 'session_id')
  final String sessionId;
  @override
  @JsonKey(name: 'relay_addr')
  final String relayAddr;
  @override
  @JsonKey(name: 'relay_port')
  final int relayPort;
  @override
  final String token;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.relayAssigned(sessionId: $sessionId, relayAddr: $relayAddr, relayPort: $relayPort, token: $token)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$RelayAssignedMessageImpl &&
            (identical(other.sessionId, sessionId) ||
                other.sessionId == sessionId) &&
            (identical(other.relayAddr, relayAddr) ||
                other.relayAddr == relayAddr) &&
            (identical(other.relayPort, relayPort) ||
                other.relayPort == relayPort) &&
            (identical(other.token, token) || other.token == token));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, sessionId, relayAddr, relayPort, token);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$RelayAssignedMessageImplCopyWith<_$RelayAssignedMessageImpl>
      get copyWith =>
          __$$RelayAssignedMessageImplCopyWithImpl<_$RelayAssignedMessageImpl>(
              this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return relayAssigned(sessionId, relayAddr, relayPort, token);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return relayAssigned?.call(sessionId, relayAddr, relayPort, token);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (relayAssigned != null) {
      return relayAssigned(sessionId, relayAddr, relayPort, token);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return relayAssigned(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return relayAssigned?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (relayAssigned != null) {
      return relayAssigned(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$RelayAssignedMessageImplToJson(
      this,
    );
  }
}

abstract class RelayAssignedMessage implements SignalingMessage {
  const factory RelayAssignedMessage(
      {@JsonKey(name: 'session_id') required final String sessionId,
      @JsonKey(name: 'relay_addr') required final String relayAddr,
      @JsonKey(name: 'relay_port') required final int relayPort,
      required final String token}) = _$RelayAssignedMessageImpl;

  factory RelayAssignedMessage.fromJson(Map<String, dynamic> json) =
      _$RelayAssignedMessageImpl.fromJson;

  @JsonKey(name: 'session_id')
  String get sessionId;
  @JsonKey(name: 'relay_addr')
  String get relayAddr;
  @JsonKey(name: 'relay_port')
  int get relayPort;
  String get token;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$RelayAssignedMessageImplCopyWith<_$RelayAssignedMessageImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$InviteGeneratedMessageImplCopyWith<$Res> {
  factory _$$InviteGeneratedMessageImplCopyWith(
          _$InviteGeneratedMessageImpl value,
          $Res Function(_$InviteGeneratedMessageImpl) then) =
      __$$InviteGeneratedMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call({@JsonKey(name: 'invite_code') String inviteCode});
}

/// @nodoc
class __$$InviteGeneratedMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$InviteGeneratedMessageImpl>
    implements _$$InviteGeneratedMessageImplCopyWith<$Res> {
  __$$InviteGeneratedMessageImplCopyWithImpl(
      _$InviteGeneratedMessageImpl _value,
      $Res Function(_$InviteGeneratedMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? inviteCode = null,
  }) {
    return _then(_$InviteGeneratedMessageImpl(
      inviteCode: null == inviteCode
          ? _value.inviteCode
          : inviteCode // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$InviteGeneratedMessageImpl implements InviteGeneratedMessage {
  const _$InviteGeneratedMessageImpl(
      {@JsonKey(name: 'invite_code') required this.inviteCode,
      final String? $type})
      : $type = $type ?? 'inviteGenerated';

  factory _$InviteGeneratedMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$InviteGeneratedMessageImplFromJson(json);

  @override
  @JsonKey(name: 'invite_code')
  final String inviteCode;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.inviteGenerated(inviteCode: $inviteCode)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$InviteGeneratedMessageImpl &&
            (identical(other.inviteCode, inviteCode) ||
                other.inviteCode == inviteCode));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, inviteCode);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$InviteGeneratedMessageImplCopyWith<_$InviteGeneratedMessageImpl>
      get copyWith => __$$InviteGeneratedMessageImplCopyWithImpl<
          _$InviteGeneratedMessageImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return inviteGenerated(inviteCode);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return inviteGenerated?.call(inviteCode);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (inviteGenerated != null) {
      return inviteGenerated(inviteCode);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return inviteGenerated(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return inviteGenerated?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (inviteGenerated != null) {
      return inviteGenerated(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$InviteGeneratedMessageImplToJson(
      this,
    );
  }
}

abstract class InviteGeneratedMessage implements SignalingMessage {
  const factory InviteGeneratedMessage(
          {@JsonKey(name: 'invite_code') required final String inviteCode}) =
      _$InviteGeneratedMessageImpl;

  factory InviteGeneratedMessage.fromJson(Map<String, dynamic> json) =
      _$InviteGeneratedMessageImpl.fromJson;

  @JsonKey(name: 'invite_code')
  String get inviteCode;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$InviteGeneratedMessageImplCopyWith<_$InviteGeneratedMessageImpl>
      get copyWith => throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$InviteResultMessageImplCopyWith<$Res> {
  factory _$$InviteResultMessageImplCopyWith(_$InviteResultMessageImpl value,
          $Res Function(_$InviteResultMessageImpl) then) =
      __$$InviteResultMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call(
      {@JsonKey(name: 'session_id') String sessionId,
      @JsonKey(name: 'to_code') String toCode});
}

/// @nodoc
class __$$InviteResultMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$InviteResultMessageImpl>
    implements _$$InviteResultMessageImplCopyWith<$Res> {
  __$$InviteResultMessageImplCopyWithImpl(_$InviteResultMessageImpl _value,
      $Res Function(_$InviteResultMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sessionId = null,
    Object? toCode = null,
  }) {
    return _then(_$InviteResultMessageImpl(
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as String,
      toCode: null == toCode
          ? _value.toCode
          : toCode // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$InviteResultMessageImpl implements InviteResultMessage {
  const _$InviteResultMessageImpl(
      {@JsonKey(name: 'session_id') required this.sessionId,
      @JsonKey(name: 'to_code') required this.toCode,
      final String? $type})
      : $type = $type ?? 'inviteResult';

  factory _$InviteResultMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$InviteResultMessageImplFromJson(json);

  @override
  @JsonKey(name: 'session_id')
  final String sessionId;
  @override
  @JsonKey(name: 'to_code')
  final String toCode;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.inviteResult(sessionId: $sessionId, toCode: $toCode)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$InviteResultMessageImpl &&
            (identical(other.sessionId, sessionId) ||
                other.sessionId == sessionId) &&
            (identical(other.toCode, toCode) || other.toCode == toCode));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, sessionId, toCode);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$InviteResultMessageImplCopyWith<_$InviteResultMessageImpl> get copyWith =>
      __$$InviteResultMessageImplCopyWithImpl<_$InviteResultMessageImpl>(
          this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return inviteResult(sessionId, toCode);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return inviteResult?.call(sessionId, toCode);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (inviteResult != null) {
      return inviteResult(sessionId, toCode);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return inviteResult(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return inviteResult?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (inviteResult != null) {
      return inviteResult(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$InviteResultMessageImplToJson(
      this,
    );
  }
}

abstract class InviteResultMessage implements SignalingMessage {
  const factory InviteResultMessage(
          {@JsonKey(name: 'session_id') required final String sessionId,
          @JsonKey(name: 'to_code') required final String toCode}) =
      _$InviteResultMessageImpl;

  factory InviteResultMessage.fromJson(Map<String, dynamic> json) =
      _$InviteResultMessageImpl.fromJson;

  @JsonKey(name: 'session_id')
  String get sessionId;
  @JsonKey(name: 'to_code')
  String get toCode;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$InviteResultMessageImplCopyWith<_$InviteResultMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class _$$ErrorMessageImplCopyWith<$Res> {
  factory _$$ErrorMessageImplCopyWith(
          _$ErrorMessageImpl value, $Res Function(_$ErrorMessageImpl) then) =
      __$$ErrorMessageImplCopyWithImpl<$Res>;
  @useResult
  $Res call({String code, String message});
}

/// @nodoc
class __$$ErrorMessageImplCopyWithImpl<$Res>
    extends _$SignalingMessageCopyWithImpl<$Res, _$ErrorMessageImpl>
    implements _$$ErrorMessageImplCopyWith<$Res> {
  __$$ErrorMessageImplCopyWithImpl(
      _$ErrorMessageImpl _value, $Res Function(_$ErrorMessageImpl) _then)
      : super(_value, _then);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? code = null,
    Object? message = null,
  }) {
    return _then(_$ErrorMessageImpl(
      code: null == code
          ? _value.code
          : code // ignore: cast_nullable_to_non_nullable
              as String,
      message: null == message
          ? _value.message
          : message // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ErrorMessageImpl implements ErrorMessage {
  const _$ErrorMessageImpl(
      {required this.code, required this.message, final String? $type})
      : $type = $type ?? 'error';

  factory _$ErrorMessageImpl.fromJson(Map<String, dynamic> json) =>
      _$$ErrorMessageImplFromJson(json);

  @override
  final String code;
  @override
  final String message;

  @JsonKey(name: 'type')
  final String $type;

  @override
  String toString() {
    return 'SignalingMessage.error(code: $code, message: $message)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ErrorMessageImpl &&
            (identical(other.code, code) || other.code == code) &&
            (identical(other.message, message) || other.message == message));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, code, message);

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ErrorMessageImplCopyWith<_$ErrorMessageImpl> get copyWith =>
      __$$ErrorMessageImplCopyWithImpl<_$ErrorMessageImpl>(this, _$identity);

  @override
  @optionalTypeArgs
  TResult when<TResult extends Object?>({
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)
        register,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, int ts)
        heartbeat,
    required TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)
        connectRequest,
    required TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)
        connectResponse,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceOffer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            String sdp, List<IceCandidate> candidates)
        iceAnswer,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)
        iceTrickle,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)
        relayRequest,
    required TResult Function(@JsonKey(name: 'device_code') String deviceCode)
        generateInvite,
    required TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)
        useInvite,
    required TResult Function(List<DeviceInfo> devices) nearbyUpdate,
    required TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)
        peerOffline,
    required TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)
        relayAssigned,
    required TResult Function(@JsonKey(name: 'invite_code') String inviteCode)
        inviteGenerated,
    required TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)
        inviteResult,
    required TResult Function(String code, String message) error,
  }) {
    return error(code, message);
  }

  @override
  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>({
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult? Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult? Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult? Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult? Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult? Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult? Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult? Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult? Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult? Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult? Function(String code, String message)? error,
  }) {
    return error?.call(code, message);
  }

  @override
  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>({
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode,
            String platform,
            String version,
            @JsonKey(name: 'team_id') String? teamId)?
        register,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode, int ts)?
        heartbeat,
    TResult Function(
            @JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'to_code') String toCode,
            @JsonKey(name: 'invite_code') String? inviteCode)?
        connectRequest,
    TResult Function(
            bool accepted,
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'from_code') String fromCode)?
        connectResponse,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceOffer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId, String sdp,
            List<IceCandidate> candidates)?
        iceAnswer,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            IceCandidate candidate)?
        iceTrickle,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'preferred_region') String? preferredRegion)?
        relayRequest,
    TResult Function(@JsonKey(name: 'device_code') String deviceCode)?
        generateInvite,
    TResult Function(@JsonKey(name: 'from_code') String fromCode,
            @JsonKey(name: 'invite_code') String inviteCode)?
        useInvite,
    TResult Function(List<DeviceInfo> devices)? nearbyUpdate,
    TResult Function(
            @JsonKey(name: 'device_code') String deviceCode, String reason)?
        peerOffline,
    TResult Function(
            @JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'relay_addr') String relayAddr,
            @JsonKey(name: 'relay_port') int relayPort,
            String token)?
        relayAssigned,
    TResult Function(@JsonKey(name: 'invite_code') String inviteCode)?
        inviteGenerated,
    TResult Function(@JsonKey(name: 'session_id') String sessionId,
            @JsonKey(name: 'to_code') String toCode)?
        inviteResult,
    TResult Function(String code, String message)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(code, message);
    }
    return orElse();
  }

  @override
  @optionalTypeArgs
  TResult map<TResult extends Object?>({
    required TResult Function(RegisterMessage value) register,
    required TResult Function(HeartbeatMessage value) heartbeat,
    required TResult Function(ConnectRequestMessage value) connectRequest,
    required TResult Function(ConnectResponseMessage value) connectResponse,
    required TResult Function(IceOfferMessage value) iceOffer,
    required TResult Function(IceAnswerMessage value) iceAnswer,
    required TResult Function(IceTrickleMessage value) iceTrickle,
    required TResult Function(RelayRequestMessage value) relayRequest,
    required TResult Function(GenerateInviteMessage value) generateInvite,
    required TResult Function(UseInviteMessage value) useInvite,
    required TResult Function(NearbyUpdateMessage value) nearbyUpdate,
    required TResult Function(PeerOfflineMessage value) peerOffline,
    required TResult Function(RelayAssignedMessage value) relayAssigned,
    required TResult Function(InviteGeneratedMessage value) inviteGenerated,
    required TResult Function(InviteResultMessage value) inviteResult,
    required TResult Function(ErrorMessage value) error,
  }) {
    return error(this);
  }

  @override
  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>({
    TResult? Function(RegisterMessage value)? register,
    TResult? Function(HeartbeatMessage value)? heartbeat,
    TResult? Function(ConnectRequestMessage value)? connectRequest,
    TResult? Function(ConnectResponseMessage value)? connectResponse,
    TResult? Function(IceOfferMessage value)? iceOffer,
    TResult? Function(IceAnswerMessage value)? iceAnswer,
    TResult? Function(IceTrickleMessage value)? iceTrickle,
    TResult? Function(RelayRequestMessage value)? relayRequest,
    TResult? Function(GenerateInviteMessage value)? generateInvite,
    TResult? Function(UseInviteMessage value)? useInvite,
    TResult? Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult? Function(PeerOfflineMessage value)? peerOffline,
    TResult? Function(RelayAssignedMessage value)? relayAssigned,
    TResult? Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult? Function(InviteResultMessage value)? inviteResult,
    TResult? Function(ErrorMessage value)? error,
  }) {
    return error?.call(this);
  }

  @override
  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>({
    TResult Function(RegisterMessage value)? register,
    TResult Function(HeartbeatMessage value)? heartbeat,
    TResult Function(ConnectRequestMessage value)? connectRequest,
    TResult Function(ConnectResponseMessage value)? connectResponse,
    TResult Function(IceOfferMessage value)? iceOffer,
    TResult Function(IceAnswerMessage value)? iceAnswer,
    TResult Function(IceTrickleMessage value)? iceTrickle,
    TResult Function(RelayRequestMessage value)? relayRequest,
    TResult Function(GenerateInviteMessage value)? generateInvite,
    TResult Function(UseInviteMessage value)? useInvite,
    TResult Function(NearbyUpdateMessage value)? nearbyUpdate,
    TResult Function(PeerOfflineMessage value)? peerOffline,
    TResult Function(RelayAssignedMessage value)? relayAssigned,
    TResult Function(InviteGeneratedMessage value)? inviteGenerated,
    TResult Function(InviteResultMessage value)? inviteResult,
    TResult Function(ErrorMessage value)? error,
    required TResult orElse(),
  }) {
    if (error != null) {
      return error(this);
    }
    return orElse();
  }

  @override
  Map<String, dynamic> toJson() {
    return _$$ErrorMessageImplToJson(
      this,
    );
  }
}

abstract class ErrorMessage implements SignalingMessage {
  const factory ErrorMessage(
      {required final String code,
      required final String message}) = _$ErrorMessageImpl;

  factory ErrorMessage.fromJson(Map<String, dynamic> json) =
      _$ErrorMessageImpl.fromJson;

  String get code;
  String get message;

  /// Create a copy of SignalingMessage
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ErrorMessageImplCopyWith<_$ErrorMessageImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
