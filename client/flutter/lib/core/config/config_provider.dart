// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'config_model.dart';
import 'config_repository.dart';

// ── Repository provider ────────────────────────────────────────

/// Singleton [ConfigRepository] instance.
final configRepositoryProvider = Provider<ConfigRepository>((ref) {
  return ConfigRepository();
});

// ── Async initial load ─────────────────────────────────────────

/// Loads the configuration once at app startup.
///
/// Usage:
/// ```dart
/// final configAsync = ref.watch(configInitProvider);
/// configAsync.when(
///   data: (config) => ...,
///   loading: () => ...,
///   error: (e, _) => ...,
/// );
/// ```
final configInitProvider = FutureProvider<RdcsConfig>((ref) async {
  final repo = ref.watch(configRepositoryProvider);
  return repo.load();
});

// ── StateNotifier ──────────────────────────────────────────────

/// Mutable wrapper around [RdcsConfig] that auto-persists changes.
class ConfigNotifier extends StateNotifier<RdcsConfig> {
  ConfigNotifier(this._repository) : super(const RdcsConfig());

  final ConfigRepository _repository;

  /// Hydrates the notifier from disk. Call once at startup.
  Future<void> init() async {
    state = await _repository.load();
    // Ensure we have a device code.
    final code = await _repository.ensureDeviceCode(state);
    if (code != state.deviceCode) {
      state = state.copyWith(deviceCode: code);
    }
  }

  /// Updates the server configuration and persists.
  Future<void> updateServer(ServerConfig server) async {
    state = state.copyWith(server: server);
    await _repository.save(state);
  }

  /// Updates the quality configuration and persists.
  Future<void> updateQuality(QualityConfig quality) async {
    state = state.copyWith(quality: quality);
    await _repository.save(state);
  }

  /// Updates general preferences and persists.
  Future<void> updateGeneral(GeneralConfig general) async {
    state = state.copyWith(general: general);
    await _repository.save(state);
  }

  /// Replaces the entire configuration and persists.
  Future<void> replace(RdcsConfig config) async {
    state = config;
    await _repository.save(state);
  }
}

/// Primary provider for reading and mutating RDCS configuration.
///
/// ```dart
/// final config = ref.watch(configProvider);
/// ref.read(configProvider.notifier).updateQuality(...);
/// ```
final configProvider =
    StateNotifierProvider<ConfigNotifier, RdcsConfig>((ref) {
  final repo = ref.watch(configRepositoryProvider);
  return ConfigNotifier(repo);
});
