// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Manages platform-specific auto-start (login item) configuration
/// for the RDCS desktop client.
///
/// Currently only macOS is supported via a Launch Agent plist.
/// On other platforms the methods are no-ops.
class AutoStartService {
  AutoStartService();

  // ── Constants ───────────────────────────────────────────────────

  static const _label = 'com.rdcs.client';
  static const _plistFileName = 'com.rdcs.client.plist';

  // ── Public API ──────────────────────────────────────────────────

  /// Returns `true` if auto-start is currently enabled.
  Future<bool> isEnabled() async {
    if (!Platform.isMacOS) return false;

    final plistPath = await _plistPath;
    final file = File(plistPath);

    if (!await file.exists()) return false;

    // Verify the agent is actually loaded via launchctl.
    try {
      final result = await Process.run('launchctl', ['list', _label]);
      return result.exitCode == 0;
    } catch (_) {
      // launchctl may not be available (e.g. sandboxed builds);
      // fall back to plist existence check.
      return await file.exists();
    }
  }

  /// Enables auto-start by creating the Launch Agent plist and
  /// loading it with `launchctl`.
  Future<void> enable() async {
    if (!Platform.isMacOS) return;

    final plistPath = await _plistPath;
    final file = File(plistPath);

    // Ensure the LaunchAgents directory exists.
    final dir = file.parent;
    if (!await dir.exists()) {
      await dir.create(recursive: true);
    }

    // Write the plist file.
    final appPath = Platform.resolvedExecutable;
    final plistContent = _generatePlist(appPath);
    await file.writeAsString(plistContent, flush: true);

    // Load the agent via launchctl.
    try {
      // Unload first in case it was already loaded with a stale
      // configuration — ignore errors on unload.
      await Process.run('launchctl', ['unload', plistPath]);
    } catch (_) {
      // Ignore: plist may not have been loaded before.
    }

    await Process.run('launchctl', ['load', '-w', plistPath]);
  }

  /// Disables auto-start by unloading the Launch Agent and removing
  /// the plist file.
  Future<void> disable() async {
    if (!Platform.isMacOS) return;

    final plistPath = await _plistPath;
    final file = File(plistPath);

    // Unload the agent.
    if (await file.exists()) {
      try {
        await Process.run('launchctl', ['unload', plistPath]);
      } catch (_) {
        // Ignore: may already be unloaded.
      }

      // Remove the plist.
      try {
        await file.delete();
      } catch (_) {
        // Ignore: file may have been removed externally.
      }
    }
  }

  // ── Internal ────────────────────────────────────────────────────

  /// Returns the full path to the Launch Agent plist file.
  Future<String> get _plistPath async {
    final home = Platform.environment['HOME'] ??
        (await Process.run('bash', ['-c', 'echo \$HOME'])).stdout.toString().trim();
    return '$home/Library/LaunchAgents/$_plistFileName';
  }

  /// Generates the XML plist content for the macOS Launch Agent.
  ///
  /// [appPath] is the absolute path to the RDCS executable.
  String _generatePlist(String appPath) {
    return '''<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>$_label</string>
	<key>ProgramArguments</key>
	<array>
		<string>$appPath</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
	<key>KeepAlive</key>
	<false/>
	<key>StandardErrorPath</key>
	<string>/tmp/com.rdcs.client.err</string>
	<key>StandardOutPath</key>
	<string>/tmp/com.rdcs.client.out</string>
</dict>
</plist>
''';
  }
}

// ── Riverpod StateNotifier ─────────────────────────────────────

/// Manages the auto-start enabled/disabled state as reactive UI
/// state so widgets can `watch` it.
class AutoStartNotifier extends StateNotifier<bool> {
  AutoStartNotifier(this._service) : super(false);

  final AutoStartService _service;

  /// Hydrates the state from the platform. Call once at startup.
  Future<void> init() async {
    state = await _service.isEnabled();
  }

  /// Enables auto-start and updates the reactive state.
  Future<void> enable() async {
    await _service.enable();
    state = true;
  }

  /// Disables auto-start and updates the reactive state.
  Future<void> disable() async {
    await _service.disable();
    state = false;
  }

  /// Toggles the current auto-start state.
  Future<void> toggle() async {
    if (state) {
      await disable();
    } else {
      await enable();
    }
  }
}

/// Provides the auto-start enabled state as a reactive boolean.
///
/// Usage:
/// ```dart
/// final autoStart = ref.watch(autoStartProvider);
/// // autoStart is `true` when auto-start is enabled.
///
/// ref.read(autoStartProvider.notifier).toggle();
/// ```
final autoStartProvider =
    StateNotifierProvider<AutoStartNotifier, bool>((ref) {
  final service = AutoStartService();
  return AutoStartNotifier(service);
});
