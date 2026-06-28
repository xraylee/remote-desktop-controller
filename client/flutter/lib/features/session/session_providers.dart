// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';
import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import '../../core/ffi/engine_events.dart';
import '../../core/ffi/engine_isolate.dart';

part 'session_providers.freezed.dart';

// ── Session state enum ─────────────────────────────────────────

/// Possible states of a remote desktop session.
enum SessionState {
  idle,
  connecting,
  connected,
  disconnected,
  error,
}

// ── Session info model ─────────────────────────────────────────

/// Immutable snapshot of a remote desktop session.
///
/// Tracks the current connection state, remote device metadata,
/// and live performance metrics (latency, fps, quality mode).
@freezed
class SessionInfo with _$SessionInfo {
  const factory SessionInfo({
    required int sessionId,
    required String remoteDeviceCode,
    required String remoteDeviceName,
    required SessionState state,
    @Default(0) int latencyMs,
    @Default(0.0) double fps,
    /// 0 = auto, 1 = clarity priority, 2 = fluidity priority.
    @Default(0) int qualityMode,
  }) = _SessionInfo;
}

// ── Session notifier ───────────────────────────────────────────

/// Manages remote desktop session lifecycle through [EngineIsolate].
///
/// Exposes [connect], [disconnect], [setQuality], and [sendInput]
/// methods that delegate to the FFI engine. Automatically listens
/// to the engine event stream and updates [SessionInfo] state when
/// connection, frame, or quality events arrive.
class SessionNotifier extends StateNotifier<SessionInfo?> {
  SessionNotifier(this._engine) : super(null);

  final EngineIsolate _engine;
  StreamSubscription<EngineEvent>? _subscription;

  /// Initiates a connection to a remote device by its 9-digit code.
  ///
  /// Sets the session state to [SessionState.connecting], invokes
  /// [EngineIsolate.connect], and updates state based on the result.
  /// On success the state becomes [SessionState.connected]; on
  /// failure it becomes [SessionState.error].
  Future<void> connect(String targetCode) async {
    // Remove dashes/spaces if the user typed them.
    final code = targetCode.replaceAll(RegExp(r'[\s-]'), '');

    state = SessionInfo(
      sessionId: 0,
      remoteDeviceCode: code,
      remoteDeviceName: code,
      state: SessionState.connecting,
    );

    _subscribeToEvents();

    try {
      final sessionId = await _engine.connect(code);
      if (sessionId > 0) {
        state = state?.copyWith(
          sessionId: sessionId,
          state: SessionState.connected,
        );
      } else {
        state = state?.copyWith(state: SessionState.error);
      }
    } catch (e) {
      state = state?.copyWith(state: SessionState.error);
    }
  }

  /// Disconnects the current remote session and resets state to idle.
  Future<void> disconnect() async {
    final session = state;
    if (session != null && session.sessionId > 0) {
      try {
        await _engine.disconnect(session.sessionId);
      } catch (_) {
        // Best-effort disconnect; the engine may already have torn down.
      }
    }
    _unsubscribe();
    state = null;
  }

  /// Changes the streaming quality mode for the active session.
  ///
  /// [mode]: 0 = auto, 1 = clarity priority, 2 = fluidity priority.
  Future<void> setQuality(int mode) async {
    final session = state;
    if (session != null && session.sessionId > 0) {
      await _engine.setQuality(session.sessionId, mode);
      state = session.copyWith(qualityMode: mode);
    }
  }

  /// Sends a mouse or keyboard input event to the remote session.
  ///
  /// [eventJson] should be a JSON object describing the input event.
  Future<void> sendInput(String eventJson) async {
    final session = state;
    if (session != null && session.sessionId > 0) {
      await _engine.sendInput(session.sessionId, eventJson);
    }
  }

  /// Sends a chat message to the remote session.
  Future<void> sendMessage(String text) async {
    final session = state;
    if (session != null && session.sessionId > 0) {
      await _engine.sendMessage(session.sessionId, text);
    }
  }

  // ── Event handling ─────────────────────────────────────────────

  /// Subscribes to the engine event stream.
  ///
  /// Called once when [connect] is invoked. The subscription
  /// automatically updates session state for connection, frame,
  /// and quality change events.
  void _subscribeToEvents() {
    _unsubscribe();
    _subscription = _engine.events.listen(_handleEvent);
  }

  void _unsubscribe() {
    _subscription?.cancel();
    _subscription = null;
  }

  void _handleEvent(EngineEvent event) {
    if (state == null) return;

    switch (event.type) {
      case EngineEventId.connectionEstablished:
        final payload = event.payload;
        final sessionId = payload['session_id'] as int? ?? state!.sessionId;
        state = state!.copyWith(
          sessionId: sessionId,
          state: SessionState.connected,
        );

      case EngineEventId.connectionLost:
        state = state!.copyWith(state: SessionState.disconnected);
        _unsubscribe();

      case EngineEventId.connectionRestored:
        state = state!.copyWith(state: SessionState.connected);

      case EngineEventId.frameReady:
        // Extract live metrics from the frame payload when available.
        final payload = event.payload;
        final latencyMs = payload['latency_ms'] as int? ?? state!.latencyMs;
        final fps = (payload['fps'] as num?)?.toDouble() ?? state!.fps;
        state = state!.copyWith(latencyMs: latencyMs, fps: fps);

      case EngineEventId.qualityChanged:
        final payload = event.payload;
        final mode = payload['mode'] as int? ?? state!.qualityMode;
        state = state!.copyWith(qualityMode: mode);

      default:
        // Ignore events not relevant to session state (frame data,
        // file transfer progress, chat messages, etc.).
        break;
    }
  }

  @override
  void dispose() {
    _unsubscribe();
    super.dispose();
  }
}

/// Primary provider for the current remote desktop session.
///
/// ```dart
/// final session = ref.watch(sessionProvider);
/// ref.read(sessionProvider.notifier).connect('123456789');
/// ```
final sessionProvider =
    StateNotifierProvider<SessionNotifier, SessionInfo?>((ref) {
  final engine = ref.watch(engineProvider);
  return SessionNotifier(engine);
});

// ── Nearby devices ─────────────────────────────────────────────

/// A LAN device discovered via mDNS / broadcast.
@freezed
class NearbyDevice with _$NearbyDevice {
  const factory NearbyDevice({
    required String deviceCode,
    @Default('') String deviceName,
  }) = _NearbyDevice;
}

/// Tracks LAN devices discovered and lost through engine events.
class NearbyDevicesNotifier extends StateNotifier<List<NearbyDevice>> {
  NearbyDevicesNotifier() : super([]);

  StreamSubscription<EngineEvent>? _subscription;

  /// Starts listening to engine events for nearby device updates.
  void subscribe(Stream<EngineEvent> events) {
    _subscription?.cancel();
    _subscription = events.listen(_handleEvent);
  }

  void _handleEvent(EngineEvent event) {
    switch (event.type) {
      case EngineEventId.nearbyDeviceFound:
        final payload = NearbyDevicePayload.fromJson(event.payload);
        final device = NearbyDevice(
          deviceCode: payload.deviceCode,
          deviceName: payload.deviceName,
        );
        // Avoid duplicates.
        if (!state.any((d) => d.deviceCode == device.deviceCode)) {
          state = [...state, device];
        }

      case EngineEventId.nearbyDeviceLost:
        final payload = NearbyDevicePayload.fromJson(event.payload);
        state = state
            .where((d) => d.deviceCode != payload.deviceCode)
            .toList();

      default:
        break;
    }
  }

  @override
  void dispose() {
    _subscription?.cancel();
    super.dispose();
  }
}

/// Provides the list of nearby LAN devices discovered by the engine.
///
/// The notifier must be initialised by calling `subscribe` with the
/// engine event stream. Typically done once at app startup after
/// the engine has been created.
final nearbyDevicesProvider =
    StateNotifierProvider<NearbyDevicesNotifier, List<NearbyDevice>>((ref) {
  final notifier = NearbyDevicesNotifier();
  final engine = ref.watch(engineProvider);
  notifier.subscribe(engine.events);
  return notifier;
});

// ── Controlled state ───────────────────────────────────────────

/// Whether this device is currently being controlled by a remote user.
///
/// Set to `true` when an incoming connection is accepted and the
/// remote side is viewing/controlling this device's screen.
final isControlledProvider = StateProvider<bool>((ref) => false);

// ── Helpers ────────────────────────────────────────────────────

/// Human-readable labels for quality mode values.
const qualityModeLabels = {
  0: '自动',
  1: '清晰优先',
  2: '流畅优先',
};

/// Returns the display label for a quality mode integer.
String qualityModeLabel(int mode) => qualityModeLabels[mode] ?? '自动';
