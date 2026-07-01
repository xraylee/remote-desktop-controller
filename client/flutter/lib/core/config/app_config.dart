// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:freezed_annotation/freezed_annotation.dart';

part 'app_config.freezed.dart';
part 'app_config.g.dart';

/// Application configuration for RDCS client.
///
/// Stored in SharedPreferences as JSON.
@freezed
class AppConfig with _$AppConfig {
  const factory AppConfig({
    @Default('ws://localhost:8080') String signalingServerUrl,
    @Default('http://localhost:3000') String apiServerUrl,
    @Default(false) bool autoConnect,
    @Default(true) bool showNotifications,
    @Default('system') String theme,
    @Default('system') String language,
    @Default(1) int qualityMode,
    @Default(5000) int maxBitrate,
    @Default(true) bool enableHardwareAcceleration,
  }) = _AppConfig;

  factory AppConfig.fromJson(Map<String, dynamic> json) =>
      _$AppConfigFromJson(json);
}
