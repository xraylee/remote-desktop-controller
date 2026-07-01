// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'engine_events.dart';
import 'engine_isolate.dart';

/// Singleton engine isolate provider.
///
/// Creates and manages the background isolate that handles all FFI
/// communication with the Rust core engine. Automatically disposes
/// the isolate when the provider is disposed.
final engineIsolateProvider = Provider<EngineIsolate>((ref) {
  final engine = EngineIsolate();

  // Ensure the engine is disposed when the provider is disposed
  ref.onDispose(() {
    engine.dispose();
  });

  return engine;
});

/// Stream provider for engine events.
///
/// Provides a broadcast stream of all events emitted by the Rust engine.
/// Events include connection state changes, frame updates, input events,
/// file transfers, and more.
///
/// Usage:
/// ```dart
/// ref.listen(engineEventsProvider, (previous, next) {
///   next.when(
///     data: (event) {
///       if (event.type == EngineEventId.frameReady) {
///         // Handle video frame
///       }
///     },
///     loading: () {},
///     error: (err, stack) => debugPrint('Engine error: $err'),
///   );
/// });
/// ```
final engineEventsProvider = StreamProvider<EngineEvent>((ref) {
  final engine = ref.watch(engineIsolateProvider);
  return engine.events;
});

/// Provider for the current session state.
///
/// Tracks the active session ID and connection status.
final currentSessionProvider = StateProvider<int?>((ref) => null);

/// Provider for checking if the engine is currently capturing.
final isCapturingProvider = StateProvider<bool>((ref) => false);

/// Provider for checking if a session is connected.
final isConnectedProvider = Provider<bool>((ref) {
  final sessionId = ref.watch(currentSessionProvider);
  return sessionId != null && sessionId > 0;
});
