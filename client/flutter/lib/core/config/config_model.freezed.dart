// coverage:ignore-file
// GENERATED CODE - DO NOT MODIFY BY HAND
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'config_model.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

T _$identity<T>(T value) => value;

final _privateConstructorUsedError = UnsupportedError(
    'It seems like you constructed your class using `MyClass._()`. This constructor is only meant to be used by freezed and you are not supposed to need it nor use it.\nPlease check the documentation here for more information: https://github.com/rrousselGit/freezed#adding-getters-and-methods-to-our-models');

ServerConfig _$ServerConfigFromJson(Map<String, dynamic> json) {
  return _ServerConfig.fromJson(json);
}

/// @nodoc
mixin _$ServerConfig {
  /// Rendezvous / signaling server URL.
  String get rendezvousUrl => throw _privateConstructorUsedError;

  /// Relay server URL (fallback when P2P fails).
  String get relayUrl => throw _privateConstructorUsedError;

  /// Management API base URL.
  String get apiUrl => throw _privateConstructorUsedError;

  /// Whether TLS is required for all connections.
  bool get requireTls => throw _privateConstructorUsedError;

  /// Serializes this ServerConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of ServerConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $ServerConfigCopyWith<ServerConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $ServerConfigCopyWith<$Res> {
  factory $ServerConfigCopyWith(
          ServerConfig value, $Res Function(ServerConfig) then) =
      _$ServerConfigCopyWithImpl<$Res, ServerConfig>;
  @useResult
  $Res call(
      {String rendezvousUrl, String relayUrl, String apiUrl, bool requireTls});
}

/// @nodoc
class _$ServerConfigCopyWithImpl<$Res, $Val extends ServerConfig>
    implements $ServerConfigCopyWith<$Res> {
  _$ServerConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of ServerConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? rendezvousUrl = null,
    Object? relayUrl = null,
    Object? apiUrl = null,
    Object? requireTls = null,
  }) {
    return _then(_value.copyWith(
      rendezvousUrl: null == rendezvousUrl
          ? _value.rendezvousUrl
          : rendezvousUrl // ignore: cast_nullable_to_non_nullable
              as String,
      relayUrl: null == relayUrl
          ? _value.relayUrl
          : relayUrl // ignore: cast_nullable_to_non_nullable
              as String,
      apiUrl: null == apiUrl
          ? _value.apiUrl
          : apiUrl // ignore: cast_nullable_to_non_nullable
              as String,
      requireTls: null == requireTls
          ? _value.requireTls
          : requireTls // ignore: cast_nullable_to_non_nullable
              as bool,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$ServerConfigImplCopyWith<$Res>
    implements $ServerConfigCopyWith<$Res> {
  factory _$$ServerConfigImplCopyWith(
          _$ServerConfigImpl value, $Res Function(_$ServerConfigImpl) then) =
      __$$ServerConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String rendezvousUrl, String relayUrl, String apiUrl, bool requireTls});
}

/// @nodoc
class __$$ServerConfigImplCopyWithImpl<$Res>
    extends _$ServerConfigCopyWithImpl<$Res, _$ServerConfigImpl>
    implements _$$ServerConfigImplCopyWith<$Res> {
  __$$ServerConfigImplCopyWithImpl(
      _$ServerConfigImpl _value, $Res Function(_$ServerConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of ServerConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? rendezvousUrl = null,
    Object? relayUrl = null,
    Object? apiUrl = null,
    Object? requireTls = null,
  }) {
    return _then(_$ServerConfigImpl(
      rendezvousUrl: null == rendezvousUrl
          ? _value.rendezvousUrl
          : rendezvousUrl // ignore: cast_nullable_to_non_nullable
              as String,
      relayUrl: null == relayUrl
          ? _value.relayUrl
          : relayUrl // ignore: cast_nullable_to_non_nullable
              as String,
      apiUrl: null == apiUrl
          ? _value.apiUrl
          : apiUrl // ignore: cast_nullable_to_non_nullable
              as String,
      requireTls: null == requireTls
          ? _value.requireTls
          : requireTls // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$ServerConfigImpl implements _ServerConfig {
  const _$ServerConfigImpl(
      {this.rendezvousUrl = '',
      this.relayUrl = '',
      this.apiUrl = '',
      this.requireTls = true});

  factory _$ServerConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$ServerConfigImplFromJson(json);

  /// Rendezvous / signaling server URL.
  @override
  @JsonKey()
  final String rendezvousUrl;

  /// Relay server URL (fallback when P2P fails).
  @override
  @JsonKey()
  final String relayUrl;

  /// Management API base URL.
  @override
  @JsonKey()
  final String apiUrl;

  /// Whether TLS is required for all connections.
  @override
  @JsonKey()
  final bool requireTls;

  @override
  String toString() {
    return 'ServerConfig(rendezvousUrl: $rendezvousUrl, relayUrl: $relayUrl, apiUrl: $apiUrl, requireTls: $requireTls)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$ServerConfigImpl &&
            (identical(other.rendezvousUrl, rendezvousUrl) ||
                other.rendezvousUrl == rendezvousUrl) &&
            (identical(other.relayUrl, relayUrl) ||
                other.relayUrl == relayUrl) &&
            (identical(other.apiUrl, apiUrl) || other.apiUrl == apiUrl) &&
            (identical(other.requireTls, requireTls) ||
                other.requireTls == requireTls));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode =>
      Object.hash(runtimeType, rendezvousUrl, relayUrl, apiUrl, requireTls);

  /// Create a copy of ServerConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$ServerConfigImplCopyWith<_$ServerConfigImpl> get copyWith =>
      __$$ServerConfigImplCopyWithImpl<_$ServerConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$ServerConfigImplToJson(
      this,
    );
  }
}

abstract class _ServerConfig implements ServerConfig {
  const factory _ServerConfig(
      {final String rendezvousUrl,
      final String relayUrl,
      final String apiUrl,
      final bool requireTls}) = _$ServerConfigImpl;

  factory _ServerConfig.fromJson(Map<String, dynamic> json) =
      _$ServerConfigImpl.fromJson;

  /// Rendezvous / signaling server URL.
  @override
  String get rendezvousUrl;

  /// Relay server URL (fallback when P2P fails).
  @override
  String get relayUrl;

  /// Management API base URL.
  @override
  String get apiUrl;

  /// Whether TLS is required for all connections.
  @override
  bool get requireTls;

  /// Create a copy of ServerConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$ServerConfigImplCopyWith<_$ServerConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

QualityConfig _$QualityConfigFromJson(Map<String, dynamic> json) {
  return _QualityConfig.fromJson(json);
}

/// @nodoc
mixin _$QualityConfig {
  /// Preferred video codec (h264, h265, vp9, av1).
  String get codec => throw _privateConstructorUsedError;

  /// Maximum frames per second.
  int get maxFps => throw _privateConstructorUsedError;

  /// Maximum resolution width in pixels (0 = auto).
  int get maxWidth => throw _privateConstructorUsedError;

  /// Maximum resolution height in pixels (0 = auto).
  int get maxHeight => throw _privateConstructorUsedError;

  /// Target bitrate in kbps (0 = auto).
  int get bitrateKbps => throw _privateConstructorUsedError;

  /// Enable GPU-accelerated encoding when available.
  bool get hardwareAccel => throw _privateConstructorUsedError;

  /// Serializes this QualityConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of QualityConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $QualityConfigCopyWith<QualityConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $QualityConfigCopyWith<$Res> {
  factory $QualityConfigCopyWith(
          QualityConfig value, $Res Function(QualityConfig) then) =
      _$QualityConfigCopyWithImpl<$Res, QualityConfig>;
  @useResult
  $Res call(
      {String codec,
      int maxFps,
      int maxWidth,
      int maxHeight,
      int bitrateKbps,
      bool hardwareAccel});
}

/// @nodoc
class _$QualityConfigCopyWithImpl<$Res, $Val extends QualityConfig>
    implements $QualityConfigCopyWith<$Res> {
  _$QualityConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of QualityConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? codec = null,
    Object? maxFps = null,
    Object? maxWidth = null,
    Object? maxHeight = null,
    Object? bitrateKbps = null,
    Object? hardwareAccel = null,
  }) {
    return _then(_value.copyWith(
      codec: null == codec
          ? _value.codec
          : codec // ignore: cast_nullable_to_non_nullable
              as String,
      maxFps: null == maxFps
          ? _value.maxFps
          : maxFps // ignore: cast_nullable_to_non_nullable
              as int,
      maxWidth: null == maxWidth
          ? _value.maxWidth
          : maxWidth // ignore: cast_nullable_to_non_nullable
              as int,
      maxHeight: null == maxHeight
          ? _value.maxHeight
          : maxHeight // ignore: cast_nullable_to_non_nullable
              as int,
      bitrateKbps: null == bitrateKbps
          ? _value.bitrateKbps
          : bitrateKbps // ignore: cast_nullable_to_non_nullable
              as int,
      hardwareAccel: null == hardwareAccel
          ? _value.hardwareAccel
          : hardwareAccel // ignore: cast_nullable_to_non_nullable
              as bool,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$QualityConfigImplCopyWith<$Res>
    implements $QualityConfigCopyWith<$Res> {
  factory _$$QualityConfigImplCopyWith(
          _$QualityConfigImpl value, $Res Function(_$QualityConfigImpl) then) =
      __$$QualityConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String codec,
      int maxFps,
      int maxWidth,
      int maxHeight,
      int bitrateKbps,
      bool hardwareAccel});
}

/// @nodoc
class __$$QualityConfigImplCopyWithImpl<$Res>
    extends _$QualityConfigCopyWithImpl<$Res, _$QualityConfigImpl>
    implements _$$QualityConfigImplCopyWith<$Res> {
  __$$QualityConfigImplCopyWithImpl(
      _$QualityConfigImpl _value, $Res Function(_$QualityConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of QualityConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? codec = null,
    Object? maxFps = null,
    Object? maxWidth = null,
    Object? maxHeight = null,
    Object? bitrateKbps = null,
    Object? hardwareAccel = null,
  }) {
    return _then(_$QualityConfigImpl(
      codec: null == codec
          ? _value.codec
          : codec // ignore: cast_nullable_to_non_nullable
              as String,
      maxFps: null == maxFps
          ? _value.maxFps
          : maxFps // ignore: cast_nullable_to_non_nullable
              as int,
      maxWidth: null == maxWidth
          ? _value.maxWidth
          : maxWidth // ignore: cast_nullable_to_non_nullable
              as int,
      maxHeight: null == maxHeight
          ? _value.maxHeight
          : maxHeight // ignore: cast_nullable_to_non_nullable
              as int,
      bitrateKbps: null == bitrateKbps
          ? _value.bitrateKbps
          : bitrateKbps // ignore: cast_nullable_to_non_nullable
              as int,
      hardwareAccel: null == hardwareAccel
          ? _value.hardwareAccel
          : hardwareAccel // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$QualityConfigImpl implements _QualityConfig {
  const _$QualityConfigImpl(
      {this.codec = 'h264',
      this.maxFps = 60,
      this.maxWidth = 0,
      this.maxHeight = 0,
      this.bitrateKbps = 0,
      this.hardwareAccel = true});

  factory _$QualityConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$QualityConfigImplFromJson(json);

  /// Preferred video codec (h264, h265, vp9, av1).
  @override
  @JsonKey()
  final String codec;

  /// Maximum frames per second.
  @override
  @JsonKey()
  final int maxFps;

  /// Maximum resolution width in pixels (0 = auto).
  @override
  @JsonKey()
  final int maxWidth;

  /// Maximum resolution height in pixels (0 = auto).
  @override
  @JsonKey()
  final int maxHeight;

  /// Target bitrate in kbps (0 = auto).
  @override
  @JsonKey()
  final int bitrateKbps;

  /// Enable GPU-accelerated encoding when available.
  @override
  @JsonKey()
  final bool hardwareAccel;

  @override
  String toString() {
    return 'QualityConfig(codec: $codec, maxFps: $maxFps, maxWidth: $maxWidth, maxHeight: $maxHeight, bitrateKbps: $bitrateKbps, hardwareAccel: $hardwareAccel)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$QualityConfigImpl &&
            (identical(other.codec, codec) || other.codec == codec) &&
            (identical(other.maxFps, maxFps) || other.maxFps == maxFps) &&
            (identical(other.maxWidth, maxWidth) ||
                other.maxWidth == maxWidth) &&
            (identical(other.maxHeight, maxHeight) ||
                other.maxHeight == maxHeight) &&
            (identical(other.bitrateKbps, bitrateKbps) ||
                other.bitrateKbps == bitrateKbps) &&
            (identical(other.hardwareAccel, hardwareAccel) ||
                other.hardwareAccel == hardwareAccel));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, codec, maxFps, maxWidth,
      maxHeight, bitrateKbps, hardwareAccel);

  /// Create a copy of QualityConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$QualityConfigImplCopyWith<_$QualityConfigImpl> get copyWith =>
      __$$QualityConfigImplCopyWithImpl<_$QualityConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$QualityConfigImplToJson(
      this,
    );
  }
}

abstract class _QualityConfig implements QualityConfig {
  const factory _QualityConfig(
      {final String codec,
      final int maxFps,
      final int maxWidth,
      final int maxHeight,
      final int bitrateKbps,
      final bool hardwareAccel}) = _$QualityConfigImpl;

  factory _QualityConfig.fromJson(Map<String, dynamic> json) =
      _$QualityConfigImpl.fromJson;

  /// Preferred video codec (h264, h265, vp9, av1).
  @override
  String get codec;

  /// Maximum frames per second.
  @override
  int get maxFps;

  /// Maximum resolution width in pixels (0 = auto).
  @override
  int get maxWidth;

  /// Maximum resolution height in pixels (0 = auto).
  @override
  int get maxHeight;

  /// Target bitrate in kbps (0 = auto).
  @override
  int get bitrateKbps;

  /// Enable GPU-accelerated encoding when available.
  @override
  bool get hardwareAccel;

  /// Create a copy of QualityConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$QualityConfigImplCopyWith<_$QualityConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

GeneralConfig _$GeneralConfigFromJson(Map<String, dynamic> json) {
  return _GeneralConfig.fromJson(json);
}

/// @nodoc
mixin _$GeneralConfig {
  /// Display language code (zh-CN, en-US, etc.).
  String get locale => throw _privateConstructorUsedError;

  /// Start the client minimized to system tray.
  bool get startMinimized => throw _privateConstructorUsedError;

  /// Automatically connect to the last remote session.
  bool get autoReconnect => throw _privateConstructorUsedError;

  /// Allow the remote side to control clipboard.
  bool get syncClipboard => throw _privateConstructorUsedError;

  /// Allow remote audio playback.
  bool get enableRemoteAudio => throw _privateConstructorUsedError;

  /// Serializes this GeneralConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of GeneralConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $GeneralConfigCopyWith<GeneralConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $GeneralConfigCopyWith<$Res> {
  factory $GeneralConfigCopyWith(
          GeneralConfig value, $Res Function(GeneralConfig) then) =
      _$GeneralConfigCopyWithImpl<$Res, GeneralConfig>;
  @useResult
  $Res call(
      {String locale,
      bool startMinimized,
      bool autoReconnect,
      bool syncClipboard,
      bool enableRemoteAudio});
}

/// @nodoc
class _$GeneralConfigCopyWithImpl<$Res, $Val extends GeneralConfig>
    implements $GeneralConfigCopyWith<$Res> {
  _$GeneralConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of GeneralConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? locale = null,
    Object? startMinimized = null,
    Object? autoReconnect = null,
    Object? syncClipboard = null,
    Object? enableRemoteAudio = null,
  }) {
    return _then(_value.copyWith(
      locale: null == locale
          ? _value.locale
          : locale // ignore: cast_nullable_to_non_nullable
              as String,
      startMinimized: null == startMinimized
          ? _value.startMinimized
          : startMinimized // ignore: cast_nullable_to_non_nullable
              as bool,
      autoReconnect: null == autoReconnect
          ? _value.autoReconnect
          : autoReconnect // ignore: cast_nullable_to_non_nullable
              as bool,
      syncClipboard: null == syncClipboard
          ? _value.syncClipboard
          : syncClipboard // ignore: cast_nullable_to_non_nullable
              as bool,
      enableRemoteAudio: null == enableRemoteAudio
          ? _value.enableRemoteAudio
          : enableRemoteAudio // ignore: cast_nullable_to_non_nullable
              as bool,
    ) as $Val);
  }
}

/// @nodoc
abstract class _$$GeneralConfigImplCopyWith<$Res>
    implements $GeneralConfigCopyWith<$Res> {
  factory _$$GeneralConfigImplCopyWith(
          _$GeneralConfigImpl value, $Res Function(_$GeneralConfigImpl) then) =
      __$$GeneralConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {String locale,
      bool startMinimized,
      bool autoReconnect,
      bool syncClipboard,
      bool enableRemoteAudio});
}

/// @nodoc
class __$$GeneralConfigImplCopyWithImpl<$Res>
    extends _$GeneralConfigCopyWithImpl<$Res, _$GeneralConfigImpl>
    implements _$$GeneralConfigImplCopyWith<$Res> {
  __$$GeneralConfigImplCopyWithImpl(
      _$GeneralConfigImpl _value, $Res Function(_$GeneralConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of GeneralConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? locale = null,
    Object? startMinimized = null,
    Object? autoReconnect = null,
    Object? syncClipboard = null,
    Object? enableRemoteAudio = null,
  }) {
    return _then(_$GeneralConfigImpl(
      locale: null == locale
          ? _value.locale
          : locale // ignore: cast_nullable_to_non_nullable
              as String,
      startMinimized: null == startMinimized
          ? _value.startMinimized
          : startMinimized // ignore: cast_nullable_to_non_nullable
              as bool,
      autoReconnect: null == autoReconnect
          ? _value.autoReconnect
          : autoReconnect // ignore: cast_nullable_to_non_nullable
              as bool,
      syncClipboard: null == syncClipboard
          ? _value.syncClipboard
          : syncClipboard // ignore: cast_nullable_to_non_nullable
              as bool,
      enableRemoteAudio: null == enableRemoteAudio
          ? _value.enableRemoteAudio
          : enableRemoteAudio // ignore: cast_nullable_to_non_nullable
              as bool,
    ));
  }
}

/// @nodoc
@JsonSerializable()
class _$GeneralConfigImpl implements _GeneralConfig {
  const _$GeneralConfigImpl(
      {this.locale = 'zh-CN',
      this.startMinimized = false,
      this.autoReconnect = false,
      this.syncClipboard = true,
      this.enableRemoteAudio = true});

  factory _$GeneralConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$GeneralConfigImplFromJson(json);

  /// Display language code (zh-CN, en-US, etc.).
  @override
  @JsonKey()
  final String locale;

  /// Start the client minimized to system tray.
  @override
  @JsonKey()
  final bool startMinimized;

  /// Automatically connect to the last remote session.
  @override
  @JsonKey()
  final bool autoReconnect;

  /// Allow the remote side to control clipboard.
  @override
  @JsonKey()
  final bool syncClipboard;

  /// Allow remote audio playback.
  @override
  @JsonKey()
  final bool enableRemoteAudio;

  @override
  String toString() {
    return 'GeneralConfig(locale: $locale, startMinimized: $startMinimized, autoReconnect: $autoReconnect, syncClipboard: $syncClipboard, enableRemoteAudio: $enableRemoteAudio)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$GeneralConfigImpl &&
            (identical(other.locale, locale) || other.locale == locale) &&
            (identical(other.startMinimized, startMinimized) ||
                other.startMinimized == startMinimized) &&
            (identical(other.autoReconnect, autoReconnect) ||
                other.autoReconnect == autoReconnect) &&
            (identical(other.syncClipboard, syncClipboard) ||
                other.syncClipboard == syncClipboard) &&
            (identical(other.enableRemoteAudio, enableRemoteAudio) ||
                other.enableRemoteAudio == enableRemoteAudio));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(runtimeType, locale, startMinimized,
      autoReconnect, syncClipboard, enableRemoteAudio);

  /// Create a copy of GeneralConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$GeneralConfigImplCopyWith<_$GeneralConfigImpl> get copyWith =>
      __$$GeneralConfigImplCopyWithImpl<_$GeneralConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$GeneralConfigImplToJson(
      this,
    );
  }
}

abstract class _GeneralConfig implements GeneralConfig {
  const factory _GeneralConfig(
      {final String locale,
      final bool startMinimized,
      final bool autoReconnect,
      final bool syncClipboard,
      final bool enableRemoteAudio}) = _$GeneralConfigImpl;

  factory _GeneralConfig.fromJson(Map<String, dynamic> json) =
      _$GeneralConfigImpl.fromJson;

  /// Display language code (zh-CN, en-US, etc.).
  @override
  String get locale;

  /// Start the client minimized to system tray.
  @override
  bool get startMinimized;

  /// Automatically connect to the last remote session.
  @override
  bool get autoReconnect;

  /// Allow the remote side to control clipboard.
  @override
  bool get syncClipboard;

  /// Allow remote audio playback.
  @override
  bool get enableRemoteAudio;

  /// Create a copy of GeneralConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$GeneralConfigImplCopyWith<_$GeneralConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}

RdcsConfig _$RdcsConfigFromJson(Map<String, dynamic> json) {
  return _RdcsConfig.fromJson(json);
}

/// @nodoc
mixin _$RdcsConfig {
  ServerConfig get server => throw _privateConstructorUsedError;
  QualityConfig get quality => throw _privateConstructorUsedError;
  GeneralConfig get general => throw _privateConstructorUsedError;

  /// Unique 9-digit device code assigned by admin.
  String get deviceCode => throw _privateConstructorUsedError;

  /// Human-readable device name shown in the management console.
  String get deviceName => throw _privateConstructorUsedError;

  /// Serializes this RdcsConfig to a JSON map.
  Map<String, dynamic> toJson() => throw _privateConstructorUsedError;

  /// Create a copy of RdcsConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  $RdcsConfigCopyWith<RdcsConfig> get copyWith =>
      throw _privateConstructorUsedError;
}

/// @nodoc
abstract class $RdcsConfigCopyWith<$Res> {
  factory $RdcsConfigCopyWith(
          RdcsConfig value, $Res Function(RdcsConfig) then) =
      _$RdcsConfigCopyWithImpl<$Res, RdcsConfig>;
  @useResult
  $Res call(
      {ServerConfig server,
      QualityConfig quality,
      GeneralConfig general,
      String deviceCode,
      String deviceName});

  $ServerConfigCopyWith<$Res> get server;
  $QualityConfigCopyWith<$Res> get quality;
  $GeneralConfigCopyWith<$Res> get general;
}

/// @nodoc
class _$RdcsConfigCopyWithImpl<$Res, $Val extends RdcsConfig>
    implements $RdcsConfigCopyWith<$Res> {
  _$RdcsConfigCopyWithImpl(this._value, this._then);

  // ignore: unused_field
  final $Val _value;
  // ignore: unused_field
  final $Res Function($Val) _then;

  /// Create a copy of RdcsConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? server = null,
    Object? quality = null,
    Object? general = null,
    Object? deviceCode = null,
    Object? deviceName = null,
  }) {
    return _then(_value.copyWith(
      server: null == server
          ? _value.server
          : server // ignore: cast_nullable_to_non_nullable
              as ServerConfig,
      quality: null == quality
          ? _value.quality
          : quality // ignore: cast_nullable_to_non_nullable
              as QualityConfig,
      general: null == general
          ? _value.general
          : general // ignore: cast_nullable_to_non_nullable
              as GeneralConfig,
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

  /// Create a copy of RdcsConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $ServerConfigCopyWith<$Res> get server {
    return $ServerConfigCopyWith<$Res>(_value.server, (value) {
      return _then(_value.copyWith(server: value) as $Val);
    });
  }

  /// Create a copy of RdcsConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $QualityConfigCopyWith<$Res> get quality {
    return $QualityConfigCopyWith<$Res>(_value.quality, (value) {
      return _then(_value.copyWith(quality: value) as $Val);
    });
  }

  /// Create a copy of RdcsConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $GeneralConfigCopyWith<$Res> get general {
    return $GeneralConfigCopyWith<$Res>(_value.general, (value) {
      return _then(_value.copyWith(general: value) as $Val);
    });
  }
}

/// @nodoc
abstract class _$$RdcsConfigImplCopyWith<$Res>
    implements $RdcsConfigCopyWith<$Res> {
  factory _$$RdcsConfigImplCopyWith(
          _$RdcsConfigImpl value, $Res Function(_$RdcsConfigImpl) then) =
      __$$RdcsConfigImplCopyWithImpl<$Res>;
  @override
  @useResult
  $Res call(
      {ServerConfig server,
      QualityConfig quality,
      GeneralConfig general,
      String deviceCode,
      String deviceName});

  @override
  $ServerConfigCopyWith<$Res> get server;
  @override
  $QualityConfigCopyWith<$Res> get quality;
  @override
  $GeneralConfigCopyWith<$Res> get general;
}

/// @nodoc
class __$$RdcsConfigImplCopyWithImpl<$Res>
    extends _$RdcsConfigCopyWithImpl<$Res, _$RdcsConfigImpl>
    implements _$$RdcsConfigImplCopyWith<$Res> {
  __$$RdcsConfigImplCopyWithImpl(
      _$RdcsConfigImpl _value, $Res Function(_$RdcsConfigImpl) _then)
      : super(_value, _then);

  /// Create a copy of RdcsConfig
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? server = null,
    Object? quality = null,
    Object? general = null,
    Object? deviceCode = null,
    Object? deviceName = null,
  }) {
    return _then(_$RdcsConfigImpl(
      server: null == server
          ? _value.server
          : server // ignore: cast_nullable_to_non_nullable
              as ServerConfig,
      quality: null == quality
          ? _value.quality
          : quality // ignore: cast_nullable_to_non_nullable
              as QualityConfig,
      general: null == general
          ? _value.general
          : general // ignore: cast_nullable_to_non_nullable
              as GeneralConfig,
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
@JsonSerializable()
class _$RdcsConfigImpl implements _RdcsConfig {
  const _$RdcsConfigImpl(
      {this.server = const ServerConfig(),
      this.quality = const QualityConfig(),
      this.general = const GeneralConfig(),
      this.deviceCode = '',
      this.deviceName = ''});

  factory _$RdcsConfigImpl.fromJson(Map<String, dynamic> json) =>
      _$$RdcsConfigImplFromJson(json);

  @override
  @JsonKey()
  final ServerConfig server;
  @override
  @JsonKey()
  final QualityConfig quality;
  @override
  @JsonKey()
  final GeneralConfig general;

  /// Unique 9-digit device code assigned by admin.
  @override
  @JsonKey()
  final String deviceCode;

  /// Human-readable device name shown in the management console.
  @override
  @JsonKey()
  final String deviceName;

  @override
  String toString() {
    return 'RdcsConfig(server: $server, quality: $quality, general: $general, deviceCode: $deviceCode, deviceName: $deviceName)';
  }

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _$RdcsConfigImpl &&
            (identical(other.server, server) || other.server == server) &&
            (identical(other.quality, quality) || other.quality == quality) &&
            (identical(other.general, general) || other.general == general) &&
            (identical(other.deviceCode, deviceCode) ||
                other.deviceCode == deviceCode) &&
            (identical(other.deviceName, deviceName) ||
                other.deviceName == deviceName));
  }

  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  int get hashCode => Object.hash(
      runtimeType, server, quality, general, deviceCode, deviceName);

  /// Create a copy of RdcsConfig
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @override
  @pragma('vm:prefer-inline')
  _$$RdcsConfigImplCopyWith<_$RdcsConfigImpl> get copyWith =>
      __$$RdcsConfigImplCopyWithImpl<_$RdcsConfigImpl>(this, _$identity);

  @override
  Map<String, dynamic> toJson() {
    return _$$RdcsConfigImplToJson(
      this,
    );
  }
}

abstract class _RdcsConfig implements RdcsConfig {
  const factory _RdcsConfig(
      {final ServerConfig server,
      final QualityConfig quality,
      final GeneralConfig general,
      final String deviceCode,
      final String deviceName}) = _$RdcsConfigImpl;

  factory _RdcsConfig.fromJson(Map<String, dynamic> json) =
      _$RdcsConfigImpl.fromJson;

  @override
  ServerConfig get server;
  @override
  QualityConfig get quality;
  @override
  GeneralConfig get general;

  /// Unique 9-digit device code assigned by admin.
  @override
  String get deviceCode;

  /// Human-readable device name shown in the management console.
  @override
  String get deviceName;

  /// Create a copy of RdcsConfig
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  _$$RdcsConfigImplCopyWith<_$RdcsConfigImpl> get copyWith =>
      throw _privateConstructorUsedError;
}
