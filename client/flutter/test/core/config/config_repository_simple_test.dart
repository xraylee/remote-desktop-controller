// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter_test/flutter_test.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:rdcs_client/core/config/config_repository_new.dart';
import 'package:rdcs_client/core/config/app_config.dart';

void main() {
  group('ConfigRepository Basic Tests', () {
    late ConfigRepository repository;

    setUp(() async {
      SharedPreferences.setMockInitialValues({});
      final prefs = await SharedPreferences.getInstance();
      repository = ConfigRepository(prefs);
    });

    test('load() returns default config when no saved data exists', () async {
      final config = await repository.load();

      expect(config.signalingServerUrl, 'ws://localhost:8080');
      expect(config.apiServerUrl, 'http://localhost:3000');
      expect(config.autoConnect, isFalse);
      expect(config.showNotifications, isTrue);
    });

    test('save() and load() persist config correctly', () async {
      final testConfig = AppConfig(
        signalingServerUrl: 'ws://test:8080',
        apiServerUrl: 'http://test:3000',
        autoConnect: true,
        showNotifications: false,
        theme: 'dark',
        language: 'zh',
      );

      await repository.save(testConfig);
      final loaded = await repository.load();

      expect(loaded.signalingServerUrl, testConfig.signalingServerUrl);
      expect(loaded.apiServerUrl, testConfig.apiServerUrl);
      expect(loaded.autoConnect, testConfig.autoConnect);
      expect(loaded.showNotifications, testConfig.showNotifications);
      expect(loaded.theme, testConfig.theme);
      expect(loaded.language, testConfig.language);
    });

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

    test('clear() removes all saved config', () async {
      final testConfig = AppConfig(
        signalingServerUrl: 'ws://clear-test:8080',
        apiServerUrl: 'http://clear-test:3000',
      );
      await repository.save(testConfig);

      await repository.clear();

      final config = await repository.load();
      expect(config.signalingServerUrl, 'ws://localhost:8080'); // Default
    });

    test('update() modifies only specified fields', () async {
      final initial = AppConfig(
        signalingServerUrl: 'ws://initial:8080',
        apiServerUrl: 'http://initial:3000',
        autoConnect: false,
        theme: 'light',
      );
      await repository.save(initial);

      await repository.update((config) => config.copyWith(theme: 'dark'));

      final updated = await repository.load();
      expect(updated.signalingServerUrl, 'ws://initial:8080');
      expect(updated.theme, 'dark');
    });
  });
}
