// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'session_providers.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

/// @nodoc
mixin _$SessionInfo {
  int get sessionId => throw _privateConstructorUsedError;
  String get remoteDeviceCode => throw _privateConstructorUsedError;
  String get remoteDeviceName => throw _privateConstructorUsedError;
  SessionState get state => throw _privateConstructorUsedError;
  int get latencyMs => throw _privateConstructorUsedError;
  double get fps => throw _privateConstructorUsedError;

  /// 0 = auto, 1 = clarity priority, 2 = fluidity priority.
  int get qualityMode => throw _privateConstructorUsedError;

  /// Create a copy of SessionInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $SessionInfoCopyWith<SessionInfo> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $SessionInfoCopyWith<$Res> {
  factory $SessionInfoCopyWith(
          SessionInfo value, $Res Function(SessionInfo) then) =
      _$SessionInfoCopyWithImpl<$Res, SessionInfo>;
  @useResult
  $Res call(
      {int sessionId,
      String remoteDeviceCode,
      String remoteDeviceName,
      SessionState state,
      int latencyMs,
      double fps,
      int qualityMode});
}

/// @nodoc
class _$SessionInfoCopyWithImpl<$Res, $Val extends SessionInfo>
    implements $SessionInfoCopyWith<$Res> {
  _$SessionInfoCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of SessionInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sessionId = null,
    Object? remoteDeviceCode = null,
    Object? remoteDeviceName = null,
    Object? state = null,
    Object? latencyMs = null,
    Object? fps = null,
    Object? qualityMode = null,
  }) {
    return _then(_value.copyWith(
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as int,
      remoteDeviceCode: null == remoteDeviceCode
          ? _value.remoteDeviceCode
          : remoteDeviceCode // ignore: cast_nullable_to_non_nullable
              as String,
      remoteDeviceName: null == remoteDeviceName
          ? _value.remoteDeviceName
          : remoteDeviceName // ignore: cast_nullable_to_non_nullable
              as String,
      state: null == state
          ? _value.state
          : state // ignore: cast_nullable_to_non_nullable
              as SessionState,
      latencyMs: null == latencyMs
          ? _value.latencyMs
          : latencyMs // ignore: cast_nullable_to_non_nullable
              as int,
      fps: null == fps
          ? _value.fps
          : fps // ignore: cast_nullable_to_non_nullable
              as double,
      qualityMode: null == qualityMode
          ? _value.qualityMode
          : qualityMode // ignore: cast_nullable_to_non_nullable
              as int,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$SessionInfoImplCopyWith<$Res>
    implements $SessionInfoCopyWith<$Res> {
  factory _$$SessionInfoImplCopyWith(
          _$SessionInfoImpl value, $Res Function(_$SessionInfoImpl) then) =
      __$$SessionInfoImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {int sessionId,
      String remoteDeviceCode,
      String remoteDeviceName,
      SessionState state,
      int latencyMs,
      double fps,
      int qualityMode});
}

/// @nodoc
class __$$SessionInfoImplCopyWithImpl<$Res>
    extends _$SessionInfoCopyWithImpl<$Res, _$SessionInfoImpl>
    implements _$$SessionInfoImplCopyWith<$Res> {
  __$$SessionInfoImplCopyWithImpl(
      _$SessionInfoImpl _value, $Res Function(_$SessionInfoImpl) _then)
      : super(_value, _then);

  /// Create a copy of SessionInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? sessionId = null,
    Object? remoteDeviceCode = null,
    Object? remoteDeviceName = null,
    Object? state = null,
    Object? latencyMs = null,
    Object? fps = null,
    Object? qualityMode = null,
  }) {
    return _then(_$SessionInfoImpl(
      sessionId: null == sessionId
          ? _value.sessionId
          : sessionId // ignore: cast_nullable_to_non_nullable
              as int,
      remoteDeviceCode: null == remoteDeviceCode
          ? _value.remoteDeviceCode
          : remoteDeviceCode // ignore: cast_nullable_to_non_nullable
              as String,
      remoteDeviceName: null == remoteDeviceName
          ? _value.remoteDeviceName
          : remoteDeviceName // ignore: cast_nullable_to_non_nullable
              as String,
      state: null == state
          ? _value.state
          : state // ignore: cast_nullable_to_non_nullable
              as SessionState,
      latencyMs: null == latencyMs
          ? _value.latencyMs
          : latencyMs // ignore: cast_nullable_to_non_nullable
              as int,
      fps: null == fps
          ? _value.fps
          : fps // ignore: cast_nullable_to_non_nullable
              as double,
      qualityMode: null == qualityMode
          ? _value.qualityMode
          : qualityMode // ignore: cast_nullable_to_non_nullable
              as int,
    ));
  }
}

/// @nodoc

class _$SessionInfoImpl implements _SessionInfo {
  const _$SessionInfoImpl(
      {required this.sessionId,
      required this.remoteDeviceCode,
      required this.remoteDeviceName,
      required this.state,
      this.latencyMs = 0,
      this.fps = 0.0,
      this.qualityMode = 0});

  @override
  final int sessionId;
  @override
  final String remoteDeviceCode;
  @override
  final String remoteDeviceName;
  @override
  final SessionState state;
  @override
  @JsonKey()
  final int latencyMs;
  @override
  @JsonKey()
  final double fps;

  /// 0 = auto, 1 = clarity priority, 2 = fluidity priority.
  @override
  @JsonKey()
  final int qualityMode;

  @override
  String toString() {
    return 'SessionInfo(sessionId: $sessionId, remoteDeviceCode: $remoteDeviceCode, remoteDeviceName: $remoteDeviceName, state: $state, latencyMs: $latencyMs, fps: $fps, qualityMode: $qualityMode)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$SessionInfoImpl &&
            (identical(other.sessionId, sessionId) ||
                other.sessionId == sessionId) &&
            (identical(other.remoteDeviceCode, remoteDeviceCode) ||
                other.remoteDeviceCode == remoteDeviceCode) &&
            (identical(other.remoteDeviceName, remoteDeviceName) ||
                other.remoteDeviceName == remoteDeviceName) &&
            (identical(other.state, state) || other.state == state) &&
            (identical(other.latencyMs, latencyMs) ||
                other.latencyMs == latencyMs) &&
            (identical(other.fps, fps) || other.fps == fps) &&
            (identical(other.qualityMode, qualityMode) ||
                other.qualityMode == qualityMode));
  }

  @override
  int get hashCode => Object.hash(runtimeType, sessionId, remoteDeviceCode,
      remoteDeviceName, state, latencyMs, fps, qualityMode);

  /// Create a copy of SessionInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$SessionInfoImplCopyWith<_$SessionInfoImpl> get copyWith =>
      __$$SessionInfoImplCopyWithImpl<_$SessionInfoImpl>(this, _$identity);
}

abstract class _SessionInfo implements SessionInfo {
  const factory _SessionInfo(
      {required final int sessionId,
      required final String remoteDeviceCode,
      required final String remoteDeviceName,
      required final SessionState state,
      final int latencyMs,
      final double fps,
      final int qualityMode}) = _$SessionInfoImpl;

  @override
  int get sessionId;
  @override
  String get remoteDeviceCode;
  @override
  String get remoteDeviceName;
  @override
  SessionState get state;
  @override
  int get latencyMs;
  @override
  double get fps;

  /// 0 = auto, 1 = clarity priority, 2 = fluidity priority.
  @override
  int get qualityMode;

  /// Create a copy of SessionInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$SessionInfoImplCopyWith<_$SessionInfoImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
mixin _$NearbyDevice {
  String get deviceCode => throw _privateConstructorUsedError;
  String get deviceName => throw _privateConstructorUsedError;

  /// Create a copy of NearbyDevice
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $NearbyDeviceCopyWith<NearbyDevice> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $NearbyDeviceCopyWith<$Res> {
  factory $NearbyDeviceCopyWith(
          NearbyDevice value, $Res Function(NearbyDevice) then) =
      _$NearbyDeviceCopyWithImpl<$Res, NearbyDevice>;
  @useResult
  $Res call({String deviceCode, String deviceName});
}

/// @nodoc
class _$NearbyDeviceCopyWithImpl<$Res, $Val extends NearbyDevice>
    implements $NearbyDeviceCopyWith<$Res> {
  _$NearbyDeviceCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of NearbyDevice
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceCode = null,
    Object? deviceName = null,
  }) {
    return _then(_value.copyWith(
      deviceCode: null == deviceCode
          ? _value.deviceCode
          : deviceCode // ignore: cast_nullable_to_non_nullable
              as String,
      deviceName: null == deviceName
          ? _value.deviceName
          : deviceName // ignore: cast_nullable_to_non_nullable
              as String,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$NearbyDeviceImplCopyWith<$Res>
    implements $NearbyDeviceCopyWith<$Res> {
  factory _$$NearbyDeviceImplCopyWith(
          _$NearbyDeviceImpl value, $Res Function(_$NearbyDeviceImpl) then) =
      __$$NearbyDeviceImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call({String deviceCode, String deviceName});
}

/// @nodoc
class __$$NearbyDeviceImplCopyWithImpl<$Res>
    extends _$NearbyDeviceCopyWithImpl<$Res, _$NearbyDeviceImpl>
    implements _$$NearbyDeviceImplCopyWith<$Res> {
  __$$NearbyDeviceImplCopyWithImpl(
      _$NearbyDeviceImpl _value, $Res Function(_$NearbyDeviceImpl) _then)
      : super(_value, _then);

  /// Create a copy of NearbyDevice
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? deviceCode = null,
    Object? deviceName = null,
  }) {
    return _then(_$NearbyDeviceImpl(
      deviceCode: null == deviceCode
          ? _value.deviceCode
          : deviceCode // ignore: cast_nullable_to_non_nullable
              as String,
      deviceName: null == deviceName
          ? _value.deviceName
          : deviceName // ignore: cast_nullable_to_non_nullable
              as String,
    ));
  }
}

/// @nodoc

class _$NearbyDeviceImpl implements _NearbyDevice {
  const _$NearbyDeviceImpl({required this.deviceCode, this.deviceName = ''});

  @override
  final String deviceCode;
  @override
  @JsonKey()
  final String deviceName;

  @override
  String toString() {
    return 'NearbyDevice(deviceCode: $deviceCode, deviceName: $deviceName)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$NearbyDeviceImpl &&
            (identical(other.deviceCode, deviceCode) ||
                other.deviceCode == deviceCode) &&
            (identical(other.deviceName, deviceName) ||
                other.deviceName == deviceName));
  }

  @override
  int get hashCode => Object.hash(runtimeType, deviceCode, deviceName);

  /// Create a copy of NearbyDevice
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$NearbyDeviceImplCopyWith<_$NearbyDeviceImpl> get copyWith =>
      __$$NearbyDeviceImplCopyWithImpl<_$NearbyDeviceImpl>(this, _$identity);
}

abstract class _NearbyDevice implements NearbyDevice {
  const factory _NearbyDevice(
      {required final String deviceCode,
      final String deviceName}) = _$NearbyDeviceImpl;

  @override
  String get deviceCode;
  @override
  String get deviceName;

  /// Create a copy of NearbyDevice
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$NearbyDeviceImplCopyWith<_$NearbyDeviceImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
