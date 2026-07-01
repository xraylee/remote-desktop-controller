// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:rdcs_client/core/config/config_repository_new.dart';
import 'package:rdcs_client/core/config/app_config.dart';

void main() {
  group('ConfigRepository', () {
    late ConfigRepository repository;

    setUp(() async {
      // Initialize with empty preferences
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      repository = ConfigRepository(prefs);
    });

    // ── Load Configuration ───────────────────────────────────────

    test('load() returns default config when no saved data exists', () async {
      final config = await repository.load();

      expect(config.signalingServerUrl, isNotEmpty);
      expect(config.apiServerUrl, isNotEmpty);
      expect(config.autoConnect, isFalse);
      expect(config.showNotifications, isTrue);
    });

    test('load() restores previously saved config', () async {
      // Save a config first
      final testConfig = AppConfig(
        signalingServerUrl: 'ws://test:8080',
        apiServerUrl: 'http://test:3000',
        autoConnect: true,
        showNotifications: false,
        theme: 'dark',
        language: 'zh',
      );

      await repository.save(testConfig);

      // Load it back
      final loaded = await repository.load();

      expect(loaded.signalingServerUrl, testConfig.signalingServerUrl);
      expect(loaded.apiServerUrl, testConfig.apiServerUrl);
      expect(loaded.autoConnect, testConfig.autoConnect);
      expect(loaded.showNotifications, testConfig.showNotifications);
      expect(loaded.theme, testConfig.theme);
      expect(loaded.language, testConfig.language);
    });

    test('load() handles corrupted data gracefully', () async {
      // Inject corrupted JSON
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString('app_config', '{invalid json}');

      // Should return default config instead of throwing
      final config = await repository.load();

      expect(config, isNotNull);
      expect(config.signalingServerUrl, isNotEmpty);
    });

    test('load() handles missing fields with defaults', () async {
      // Save partial config (simulating old version)
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString('app_config', '{"signalingServerUrl": "ws://test:8080"}');

      final config = await repository.load();

      expect(config.signalingServerUrl, 'ws://test:8080');
      expect(config.autoConnect, isFalse); // Default value
      expect(config.showNotifications, isTrue); // Default value
    });

    // ── Save Configuration ───────────────────────────────────────

    test('save() persists config to storage', () async {
      final testConfig = AppConfig(
        signalingServerUrl: 'ws://save-test:8080',
        apiServerUrl: 'http://save-test:3000',
        autoConnect: true,
        showNotifications: true,
        theme: 'light',
        language: 'en',
      );

      await repository.save(testConfig);

      // Verify it was saved
      final prefs = await SharedPreferences.getInstance();
      final saved = prefs.getString('app_config');

      expect(saved, isNotNull);
      expect(saved, contains('ws://save-test:8080'));
    });

    test('save() overwrites previous config', () async {
      // Save first config
      final config1 = AppConfig(
        signalingServerUrl: 'ws://first:8080',
        apiServerUrl: 'http://first:3000',
        autoConnect: false,
      );
      await repository.save(config1);

      // Save second config
      final config2 = AppConfig(
        signalingServerUrl: 'ws://second:8080',
        apiServerUrl: 'http://second:3000',
        autoConnect: true,
      );
      await repository.save(config2);

      // Load and verify it's the second config
      final loaded = await repository.load();

      expect(loaded.signalingServerUrl, 'ws://second:8080');
      expect(loaded.autoConnect, isTrue);
    });

    test('save() handles all config fields correctly', () async {
      final testConfig = AppConfig(
        signalingServerUrl: 'ws://complete:8080',
        apiServerUrl: 'http://complete:3000',
        autoConnect: true,
        showNotifications: false,
        theme: 'dark',
        language: 'zh',
        qualityMode: 2,
        maxBitrate: 5000,
        enableHardwareAcceleration: true,
      );

      await repository.save(testConfig);
      final loaded = await repository.load();

      expect(loaded.signalingServerUrl, testConfig.signalingServerUrl);
      expect(loaded.apiServerUrl, testConfig.apiServerUrl);
      expect(loaded.autoConnect, testConfig.autoConnect);
      expect(loaded.showNotifications, testConfig.showNotifications);
      expect(loaded.theme, testConfig.theme);
      expect(loaded.language, testConfig.language);
      expect(loaded.qualityMode, testConfig.qualityMode);
      expect(loaded.maxBitrate, testConfig.maxBitrate);
      expect(loaded.enableHardwareAcceleration, testConfig.enableHardwareAcceleration);
    });

    // ── Default Values ───────────────────────────────────────────

    test('default config has valid signaling server URL', () async {
      final config = await repository.load();

      expect(config.signalingServerUrl, startsWith('ws'));
      expect(config.signalingServerUrl.length, greaterThan(10));
    });

    test('default config has valid API server URL', () async {
      final config = await repository.load();

      expect(config.apiServerUrl, startsWith('http'));
      expect(config.apiServerUrl.length, greaterThan(10));
    });

    test('default config has sensible preferences', () async {
      final config = await repository.load();

      expect(config.autoConnect, isFalse); // Safety: don't auto-connect
      expect(config.showNotifications, isTrue); // UX: show notifications
      expect(config.theme, 'system'); // UX: follow system theme
      expect(config.language, 'system'); // UX: follow system language
    });

    // ── Validation ───────────────────────────────────────────────

    test('validates signaling server URL format', () async {
      final invalidConfig = AppConfig(
        signalingServerUrl: 'invalid-url',
        apiServerUrl: 'http://valid:3000',
      );

      expect(
        () => repository.save(invalidConfig),
        throwsA(isA<ArgumentError>()),
      );
    });

    test('validates API server URL format', () async {
      final invalidConfig = AppConfig(
        signalingServerUrl: 'ws://valid:8080',
        apiServerUrl: 'not-a-url',
      );

      expect(
        () => repository.save(invalidConfig),
        throwsA(isA<ArgumentError>()),
      );
    });

    test('validates quality mode range', () async {
      final invalidConfig = AppConfig(
        signalingServerUrl: 'ws://valid:8080',
        apiServerUrl: 'http://valid:3000',
        qualityMode: 99, // Invalid mode
      );

      expect(
        () => repository.save(invalidConfig),
        throwsA(isA<ArgumentError>()),
      );
    });

    test('validates max bitrate range', () async {
      final invalidConfig = AppConfig(
        signalingServerUrl: 'ws://valid:8080',
        apiServerUrl: 'http://valid:3000',
        maxBitrate: -100, // Negative bitrate
      );

      expect(
        () => repository.save(invalidConfig),
        throwsA(isA<ArgumentError>()),
      );
    });

    // ── Migration ────────────────────────────────────────────────

    test('migrates from v1 to v2 config format', () async {
      // Simulate old v1 config
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString('app_config', '''
        {
          "serverUrl": "ws://old:8080",
          "autoStart": true
        }
      ''');

      final config = await repository.load();

      // Should migrate to new field names
      expect(config.signalingServerUrl, 'ws://old:8080');
      expect(config.autoConnect, true);
    });

    test('migration preserves all user settings', () async {
      // Old format with all fields
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString('app_config', '''
        {
          "serverUrl": "ws://migrate:8080",
          "apiUrl": "http://migrate:3000",
          "autoStart": true,
          "notifications": false,
          "theme": "dark"
        }
      ''');

      final config = await repository.load();

      expect(config.signalingServerUrl, 'ws://migrate:8080');
      expect(config.apiServerUrl, 'http://migrate:3000');
      expect(config.autoConnect, true);
      expect(config.showNotifications, false);
      expect(config.theme, 'dark');
    });

    test('migration writes updated config back to storage', () async {
      // Old format
      final prefs = await SharedPreferences.getInstance();
      await prefs.setString('app_config', '''
        {
          "serverUrl": "ws://migrate:8080"
        }
      ''');

      await repository.load();

      // Load again and verify it's now in new format
      final config = await repository.load();
      expect(config.signalingServerUrl, 'ws://migrate:8080');

      // Verify new format in storage
      final saved = prefs.getString('app_config');
      expect(saved, contains('signalingServerUrl'));
      expect(saved, isNot(contains('serverUrl')));
    });

    // ── Clear Configuration ──────────────────────────────────────

    test('clear() removes all saved config', () async {
      // Save a config
      final testConfig = AppConfig(
        signalingServerUrl: 'ws://clear-test:8080',
        apiServerUrl: 'http://clear-test:3000',
      );
      await repository.save(testConfig);

      // Clear it
      await repository.clear();

      // Load should return defaults
      final config = await repository.load();
      expect(config.signalingServerUrl, isNot('ws://clear-test:8080'));
    });

    test('clear() is safe when no config exists', () async {
      // Clear when nothing is saved
      await repository.clear();

      // Should not throw
      final config = await repository.load();
      expect(config, isNotNull);
    });

    // ── Update Partial Configuration ─────────────────────────────

    test('update() modifies only specified fields', () async {
      // Save initial config
      final initial = AppConfig(
        signalingServerUrl: 'ws://initial:8080',
        apiServerUrl: 'http://initial:3000',
        autoConnect: false,
        theme: 'light',
      );
      await repository.save(initial);

      // Update only theme
      await repository.update((config) => config.copyWith(theme: 'dark'));

      // Load and verify
      final updated = await repository.load();
      expect(updated.signalingServerUrl, 'ws://initial:8080'); // Unchanged
      expect(updated.theme, 'dark'); // Changed
    });

    test('update() handles multiple field changes', () async {
      final initial = AppConfig(
        signalingServerUrl: 'ws://test:8080',
        apiServerUrl: 'http://test:3000',
        autoConnect: false,
        showNotifications: true,
        theme: 'light',
      );
      await repository.save(initial);

      // Update multiple fields
      await repository.update((config) => config.copyWith(
        autoConnect: true,
        showNotifications: false,
        theme: 'dark',
      ));

      final updated = await repository.load();
      expect(updated.autoConnect, true);
      expect(updated.showNotifications, false);
      expect(updated.theme, 'dark');
    });
  });
}
