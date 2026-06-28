// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:freezed_annotation/freezed_annotation.dart';

part 'config_model.freezed.dart';
part 'config_model.g.dart';

/// Server connection configuration.
@freezed
class ServerConfig with _$ServerConfig {
  const factory ServerConfig({
    /// Rendezvous / signaling server URL.
    @Default('') String rendezvousUrl,

    /// Relay server URL (fallback when P2P fails).
    @Default('') String relayUrl,

    /// Management API base URL.
    @Default('') String apiUrl,

    /// Whether TLS is required for all connections.
    @Default(true) bool requireTls,
  }) = _ServerConfig;

  factory ServerConfig.fromJson(Map<String, dynamic> json) =>
      _$ServerConfigFromJson(json);
}

/// Streaming quality configuration.
@freezed
class QualityConfig with _$QualityConfig {
  const factory QualityConfig({
    /// Preferred video codec (h264, h265, vp9, av1).
    @Default('h264') String codec,

    /// Maximum frames per second.
    @Default(60) int maxFps,

    /// Maximum resolution width in pixels (0 = auto).
    @Default(0) int maxWidth,

    /// Maximum resolution height in pixels (0 = auto).
    @Default(0) int maxHeight,

    /// Target bitrate in kbps (0 = auto).
    @Default(0) int bitrateKbps,

    /// Enable GPU-accelerated encoding when available.
    @Default(true) bool hardwareAccel,
  }) = _QualityConfig;

  factory QualityConfig.fromJson(Map<String, dynamic> json) =>
      _$QualityConfigFromJson(json);
}

/// General user preferences.
@freezed
class GeneralConfig with _$GeneralConfig {
  const factory GeneralConfig({
    /// Display language code (zh-CN, en-US, etc.).
    @Default('zh-CN') String locale,

    /// Start the client minimized to system tray.
    @Default(false) bool startMinimized,

    /// Automatically connect to the last remote session.
    @Default(false) bool autoReconnect,

    /// Allow the remote side to control clipboard.
    @Default(true) bool syncClipboard,

    /// Allow remote audio playback.
    @Default(true) bool enableRemoteAudio,
  }) = _GeneralConfig;

  factory GeneralConfig.fromJson(Map<String, dynamic> json) =>
      _$GeneralConfigFromJson(json);
}

/// Top-level RDCS client configuration.
///
/// Persisted as JSON in `~/.rdcs/config.json`.
@freezed
class RdcsConfig with _$RdcsConfig {
  const factory RdcsConfig({
    @Default(ServerConfig()) ServerConfig server,
    @Default(QualityConfig()) QualityConfig quality,
    @Default(GeneralConfig()) GeneralConfig general,

    /// Unique 9-digit device code assigned by admin.
    @Default('') String deviceCode,

    /// Human-readable device name shown in the management console.
    @Default('') String deviceName,
  }) = _RdcsConfig;

  factory RdcsConfig.fromJson(Map<String, dynamic> json) =>
      _$RdcsConfigFromJson(json);
}
