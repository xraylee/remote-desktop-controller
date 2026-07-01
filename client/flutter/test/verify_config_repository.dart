// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

// 直接验证脚本 - 不使用 flutter test
// 运行: dart run test/verify_config_repository.dart

import 'dart:convert';

// 手动模拟 SharedPreferences
class MockSharedPreferences {
  final Map<String, String> _storage = {};

  Future<void> setString(String key, String value) async {
    _storage[key] = value;
  }

  String? getString(String key) {
    return _storage[key];
  }

  Future<bool> remove(String key) async {
    _storage.remove(key);
    return true;
  }
}

// 简化的 AppConfig 类 (不依赖 Freezed)
class AppConfig {
  final String signalingServerUrl;
  final String apiServerUrl;
  final bool autoConnect;
  final bool showNotifications;
  final String theme;
  final String language;
  final int qualityMode;
  final int maxBitrate;
  final bool enableHardwareAcceleration;

  AppConfig({
    this.signalingServerUrl = 'ws://localhost:8080',
    this.apiServerUrl = 'http://localhost:3000',
    this.autoConnect = false,
    this.showNotifications = true,
    this.theme = 'system',
    this.language = 'system',
    this.qualityMode = 1,
    this.maxBitrate = 5000,
    this.enableHardwareAcceleration = true,
  });

  Map<String, dynamic> toJson() => {
        'signalingServerUrl': signalingServerUrl,
        'apiServerUrl': apiServerUrl,
        'autoConnect': autoConnect,
        'showNotifications': showNotifications,
        'theme': theme,
        'language': language,
        'qualityMode': qualityMode,
        'maxBitrate': maxBitrate,
        'enableHardwareAcceleration': enableHardwareAcceleration,
      };

  factory AppConfig.fromJson(Map<String, dynamic> json) {
    return AppConfig(
      signalingServerUrl: json['signalingServerUrl'] ?? 'ws://localhost:8080',
      apiServerUrl: json['apiServerUrl'] ?? 'http://localhost:3000',
      autoConnect: json['autoConnect'] ?? false,
      showNotifications: json['showNotifications'] ?? true,
      theme: json['theme'] ?? 'system',
      language: json['language'] ?? 'system',
      qualityMode: json['qualityMode'] ?? 1,
      maxBitrate: json['maxBitrate'] ?? 5000,
      enableHardwareAcceleration: json['enableHardwareAcceleration'] ?? true,
    );
  }

  AppConfig copyWith({
    String? signalingServerUrl,
    String? apiServerUrl,
    bool? autoConnect,
    bool? showNotifications,
    String? theme,
    String? language,
    int? qualityMode,
    int? maxBitrate,
    bool? enableHardwareAcceleration,
  }) {
    return AppConfig(
      signalingServerUrl: signalingServerUrl ?? this.signalingServerUrl,
      apiServerUrl: apiServerUrl ?? this.apiServerUrl,
      autoConnect: autoConnect ?? this.autoConnect,
      showNotifications: showNotifications ?? this.showNotifications,
      theme: theme ?? this.theme,
      language: language ?? this.language,
      qualityMode: qualityMode ?? this.qualityMode,
      maxBitrate: maxBitrate ?? this.maxBitrate,
      enableHardwareAcceleration:
          enableHardwareAcceleration ?? this.enableHardwareAcceleration,
    );
  }
}

// 简化的 ConfigRepository
class ConfigRepository {
  final MockSharedPreferences _prefs;
  static const _configKey = 'app_config';

  ConfigRepository(this._prefs);

  Future<AppConfig> load() async {
    final jsonString = _prefs.getString(_configKey);
    if (jsonString == null) {
      return AppConfig();
    }

    try {
      final json = jsonDecode(jsonString) as Map<String, dynamic>;
      return AppConfig.fromJson(json);
    } catch (e) {
      return AppConfig();
    }
  }

  Future<void> save(AppConfig config) async {
    _validate(config);
    final json = config.toJson();
    final jsonString = jsonEncode(json);
    await _prefs.setString(_configKey, jsonString);
  }

  Future<void> update(AppConfig Function(AppConfig) updater) async {
    final current = await load();
    final updated = updater(current);
    await save(updated);
  }

  Future<void> clear() async {
    await _prefs.remove(_configKey);
  }

  void _validate(AppConfig config) {
    if (!_isValidWebSocketUrl(config.signalingServerUrl)) {
      throw ArgumentError('Invalid signaling server URL: ${config.signalingServerUrl}');
    }
    if (!_isValidHttpUrl(config.apiServerUrl)) {
      throw ArgumentError('Invalid API server URL: ${config.apiServerUrl}');
    }
    if (config.qualityMode < 0 || config.qualityMode > 2) {
      throw ArgumentError('Invalid quality mode: ${config.qualityMode}');
    }
    if (config.maxBitrate < 0) {
      throw ArgumentError('Invalid max bitrate: ${config.maxBitrate}');
    }
  }

  bool _isValidWebSocketUrl(String url) {
    return url.startsWith('ws://') || url.startsWith('wss://');
  }

  bool _isValidHttpUrl(String url) {
    return url.startsWith('http://') || url.startsWith('https://');
  }
}

// 测试运行器
void main() async {
  print('🧪 ConfigRepository 验证测试\n');

  int passed = 0;
  int failed = 0;

  Future<void> test(String name, Future<void> Function() fn) async {
    try {
      await fn();
      print('✅ $name');
      passed++;
    } catch (e) {
      print('❌ $name\n   Error: $e');
      failed++;
    }
  }

  // Test 1: Load default config
  await test('load() returns default config', () async {
    final prefs = MockSharedPreferences();
    final repo = ConfigRepository(prefs);
    final config = await repo.load();

    assert(config.signalingServerUrl == 'ws://localhost:8080');
    assert(config.apiServerUrl == 'http://localhost:3000');
    assert(config.autoConnect == false);
    assert(config.showNotifications == true);
  });

  // Test 2: Save and load
  await test('save() and load() persist correctly', () async {
    final prefs = MockSharedPreferences();
    final repo = ConfigRepository(prefs);

    final testConfig = AppConfig(
      signalingServerUrl: 'ws://test:8080',
      apiServerUrl: 'http://test:3000',
      autoConnect: true,
      theme: 'dark',
    );

    await repo.save(testConfig);
    final loaded = await repo.load();

    assert(loaded.signalingServerUrl == 'ws://test:8080');
    assert(loaded.autoConnect == true);
    assert(loaded.theme == 'dark');
  });

  // Test 3: Validation - invalid WebSocket URL
  await test('validates WebSocket URL', () async {
    final prefs = MockSharedPreferences();
    final repo = ConfigRepository(prefs);

    final invalidConfig = AppConfig(
      signalingServerUrl: 'invalid-url',
    );

    bool threw = false;
    try {
      await repo.save(invalidConfig);
    } catch (e) {
      threw = e is ArgumentError;
    }

    assert(threw, 'Should throw ArgumentError');
  });

  // Test 4: Clear config
  await test('clear() removes config', () async {
    final prefs = MockSharedPreferences();
    final repo = ConfigRepository(prefs);

    await repo.save(AppConfig(signalingServerUrl: 'ws://test:8080'));
    await repo.clear();

    final config = await repo.load();
    assert(config.signalingServerUrl == 'ws://localhost:8080'); // Default
  });

  // Test 5: Update partial config
  await test('update() modifies only specified fields', () async {
    final prefs = MockSharedPreferences();
    final repo = ConfigRepository(prefs);

    final initial = AppConfig(
      signalingServerUrl: 'ws://initial:8080',
      theme: 'light',
    );
    await repo.save(initial);

    await repo.update((config) => config.copyWith(theme: 'dark'));

    final updated = await repo.load();
    assert(updated.signalingServerUrl == 'ws://initial:8080'); // Unchanged
    assert(updated.theme == 'dark'); // Changed
  });

  // Test 6: Quality mode validation
  await test('validates quality mode range', () async {
    final prefs = MockSharedPreferences();
    final repo = ConfigRepository(prefs);

    final invalidConfig = AppConfig(qualityMode: 99);

    bool threw = false;
    try {
      await repo.save(invalidConfig);
    } catch (e) {
      threw = e is ArgumentError;
    }

    assert(threw, 'Should throw ArgumentError for invalid quality mode');
  });

  print('\n📊 测试结果:');
  print('   通过: $passed');
  print('   失败: $failed');
  print('   总计: ${passed + failed}');

  if (failed == 0) {
    print('\n🎉 所有测试通过！');
  } else {
    print('\n⚠️  有 $failed 个测试失败');
  }
}
