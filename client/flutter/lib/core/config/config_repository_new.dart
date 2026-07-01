// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:convert';
import 'package:shared_preferences/shared_preferences.dart';
import 'app_config.dart';

/// Repository for loading and saving RDCS configuration.
///
/// Configuration is stored in SharedPreferences as JSON.
class ConfigRepository {
  final SharedPreferences _prefs;
  static const _configKey = 'app_config';

  ConfigRepository(this._prefs);

  /// Loads the configuration from storage.
  ///
  /// If no config exists, returns default configuration.
  /// If config is corrupted, returns default and overwrites storage.
  Future<AppConfig> load() async {
    final jsonString = _prefs.getString(_configKey);

    if (jsonString == null) {
      return const AppConfig();
    }

    try {
      final json = jsonDecode(jsonString) as Map<String, dynamic>;

      // Handle migration from v1 to v2 format
      final migrated = _migrateIfNeeded(json);

      final config = AppConfig.fromJson(migrated);

      // Save back if migration occurred
      if (migrated != json) {
        await save(config);
      }

      return config;
    } catch (e) {
      // Corrupted data - return defaults
      final defaults = const AppConfig();
      await save(defaults);
      return defaults;
    }
  }

  /// Persists the given configuration to storage.
  ///
  /// Validates config before saving.
  Future<void> save(AppConfig config) async {
    _validate(config);

    final json = config.toJson();
    final jsonString = jsonEncode(json);
    await _prefs.setString(_configKey, jsonString);
  }

  /// Updates configuration by applying a transformation function.
  ///
  /// Loads current config, applies the updater function, and saves.
  Future<void> update(AppConfig Function(AppConfig) updater) async {
    final current = await load();
    final updated = updater(current);
    await save(updated);
  }

  /// Clears all saved configuration.
  Future<void> clear() async {
    await _prefs.remove(_configKey);
  }

  /// Validates configuration values.
  void _validate(AppConfig config) {
    // Validate signaling server URL
    if (!_isValidWebSocketUrl(config.signalingServerUrl)) {
      throw ArgumentError(
        'Invalid signaling server URL: ${config.signalingServerUrl}',
      );
    }

    // Validate API server URL
    if (!_isValidHttpUrl(config.apiServerUrl)) {
      throw ArgumentError('Invalid API server URL: ${config.apiServerUrl}');
    }

    // Validate quality mode (0=Low, 1=Medium, 2=High)
    if (config.qualityMode < 0 || config.qualityMode > 2) {
      throw ArgumentError(
        'Invalid quality mode: ${config.qualityMode} (must be 0-2)',
      );
    }

    // Validate max bitrate
    if (config.maxBitrate < 0) {
      throw ArgumentError(
        'Invalid max bitrate: ${config.maxBitrate} (must be >= 0)',
      );
    }
  }

  /// Checks if a URL is a valid WebSocket URL.
  bool _isValidWebSocketUrl(String url) {
    if (url.isEmpty) return false;
    return url.startsWith('ws://') || url.startsWith('wss://');
  }

  /// Checks if a URL is a valid HTTP URL.
  bool _isValidHttpUrl(String url) {
    if (url.isEmpty) return false;
    return url.startsWith('http://') || url.startsWith('https://');
  }

  /// Migrates old config format to new format.
  Map<String, dynamic> _migrateIfNeeded(Map<String, dynamic> json) {
    final migrated = Map<String, dynamic>.from(json);

    // Migrate v1 field names to v2
    if (json.containsKey('serverUrl')) {
      migrated['signalingServerUrl'] = json['serverUrl'];
      migrated.remove('serverUrl');
    }

    if (json.containsKey('apiUrl')) {
      migrated['apiServerUrl'] = json['apiUrl'];
      migrated.remove('apiUrl');
    }

    if (json.containsKey('autoStart')) {
      migrated['autoConnect'] = json['autoStart'];
      migrated.remove('autoStart');
    }

    if (json.containsKey('notifications')) {
      migrated['showNotifications'] = json['notifications'];
      migrated.remove('notifications');
    }

    return migrated;
  }
}
