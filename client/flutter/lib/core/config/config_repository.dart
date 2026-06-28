// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:convert';
import 'dart:io';
import 'dart:math';

import 'package:path_provider/path_provider.dart';

import 'config_model.dart';

/// Repository for loading and saving RDCS configuration.
///
/// Configuration is stored as a JSON file at `~/.rdcs/config.json`.
/// On first launch the file (and parent directory) are created with
/// sensible defaults.
class ConfigRepository {
  static const _dirName = '.rdcs';
  static const _fileName = 'config.json';

  /// Returns the platform-specific config directory path.
  Future<String> get _configDir async {
    // Use home directory for the config path so it is shared
    // across all user accounts on the same machine (mirrors the
    // convention used by RustDesk, VS Code, etc.).
    final home = Platform.environment['HOME'] ??
        Platform.environment['USERPROFILE'] ??
        (await getApplicationDocumentsDirectory()).path;
    return '$home/$_dirName';
  }

  /// Full path to the config file.
  Future<File> get _configFile async {
    final dir = await _configDir;
    return File('$dir/$_fileName');
  }

  /// Loads the configuration from disk.
  ///
  /// If the config file does not exist, a default configuration
  /// is created, persisted, and returned.
  Future<RdcsConfig> load() async {
    final file = await _configFile;

    if (!await file.exists()) {
      final defaults = RdcsConfig();
      await save(defaults);
      return defaults;
    }

    try {
      final content = await file.readAsString();
      final json = jsonDecode(content) as Map<String, dynamic>;
      return RdcsConfig.fromJson(json);
    } catch (_) {
      // If the file is corrupted, return defaults and overwrite.
      final defaults = RdcsConfig();
      await save(defaults);
      return defaults;
    }
  }

  /// Persists the given configuration to disk.
  Future<void> save(RdcsConfig config) async {
    final file = await _configFile;

    // Ensure the parent directory exists.
    final dir = file.parent;
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }

    const encoder = JsonEncoder.withIndent('  ');
    final content = encoder.convert(config.toJson());
    await file.writeAsString(content, flush: true);
  }

  /// Ensures a device code exists in the configuration.
  ///
  /// If [config.deviceCode] is empty, generates a new 9-digit code,
  /// saves it, and returns it. Otherwise returns the existing code.
  Future<String> ensureDeviceCode(RdcsConfig config) async {
    if (config.deviceCode.isNotEmpty) {
      return config.deviceCode;
    }

    final code = _generateDeviceCode();
    final updated = config.copyWith(deviceCode: code);
    await save(updated);
    return code;
  }

  /// Generates a random 9-digit device code.
  ///
  /// Format: `XXX-XXX-XXX` (stored as `XXXXXXXXX` without dashes).
  /// The first digit is always non-zero to ensure a consistent
  /// 9-character length.
  static String _generateDeviceCode() {
    final rng = Random.secure();
    final first = rng.nextInt(9) + 1; // 1–9
    final rest = List.generate(8, (_) => rng.nextInt(10)).join();
    return '$first$rest';
  }

  /// Formats a 9-digit device code for display: `XXX-XXX-XXX`.
  static String formatDeviceCode(String code) {
    if (code.length != 9) return code;
    return '${code.substring(0, 3)}-${code.substring(3, 6)}-${code.substring(6, 9)}';
  }
}
