// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';

import 'package:rdcs_client/core/config/config_model.dart';
import 'package:rdcs_client/core/config/config_provider.dart';
import 'package:rdcs_client/core/config/config_repository.dart';
import 'package:rdcs_client/core/ffi/engine_events.dart';
import 'package:rdcs_client/core/ffi/engine_isolate.dart';
import 'package:rdcs_client/core/signaling/session_signaling.dart';
import 'package:rdcs_client/core/signaling/websocket_client.dart';
import 'package:rdcs_client/features/connect/connect_page.dart';
import 'package:rdcs_client/features/home/home_page.dart';
import 'package:rdcs_client/features/session/session_providers.dart';
import 'package:rdcs_client/features/session/session_screen.dart';
import 'package:rdcs_client/features/settings/settings_screen.dart';

// =============================================================================
// Fake ConfigRepository — in-memory, no disk I/O.
// =============================================================================

class FakeConfigRepository implements ConfigRepository {
  FakeConfigRepository([RdcsConfig? initial])
      : _config = initial ?? const RdcsConfig();

  RdcsConfig _config;

  @override
  Future<RdcsConfig> load() async => _config;

  @override
  Future<void> save(RdcsConfig config) async {
    _config = config;
  }

  @override
  Future<String> ensureDeviceCode(RdcsConfig config) async {
    if (config.deviceCode.isNotEmpty) return config.deviceCode;
    return '999888777';
  }
}

// =============================================================================
// Fake EngineIsolate — records calls, returns canned responses.
// =============================================================================

class FakeEngineIsolate implements EngineIsolate {
  FakeEngineIsolate({
    this.connectResult = 1,
    this.inviteCode = 'ABC123',
  });

  int connectResult;
  String inviteCode;

  // Tracking fields.
  String? lastConnectCode;
  int? lastDisconnectSessionId;
  int? lastSetQualityMode;
  String? lastSentInput;
  String? lastSentMessage;
  bool initCalled = false;
  bool destroyCalled = false;

  final _eventController = StreamController<EngineEvent>.broadcast();

  @override
  int currentSessionId = 0;

  @override
  Stream<EngineEvent> get events => _eventController.stream;

  /// Inject an event into the stream (useful for testing event handling).
  void emitEvent(EngineEvent event) {
    _eventController.add(event);
  }

  @override
  Future<void> init() async {
    initCalled = true;
  }

  @override
  Future<int> create(String configJson) async => 0;

  @override
  Future<void> destroy() async {
    destroyCalled = true;
  }

  @override
  Future<int> startCapture(String configJson) async => 0;

  @override
  Future<int> stopCapture() async => 0;

  @override
  Future<int> connect(String targetCode) async {
    lastConnectCode = targetCode;
    currentSessionId = connectResult;
    return connectResult;
  }

  @override
  Future<int> disconnect(int sessionId) async {
    lastDisconnectSessionId = sessionId;
    return 0;
  }

  @override
  Future<int> sendInput(int sessionId, String eventJson) async {
    lastSentInput = eventJson;
    return 0;
  }

  @override
  Future<int> sendFile(int sessionId, String path, String dest) async => 0;

  @override
  Future<int> sendMessage(int sessionId, String text) async {
    lastSentMessage = text;
    return 0;
  }

  @override
  Future<int> setQuality(int sessionId, int mode) async {
    lastSetQualityMode = mode;
    return 0;
  }

  @override
  Future<String> generateInvite() async => inviteCode;

  @override
  Future<void> dispose() async {
    await _eventController.close();
  }
}

// =============================================================================
// Fake SessionSignaling — records signaling calls without network I/O.
// =============================================================================

class FakeSessionSignaling implements SessionSignaling {
  FakeSessionSignaling({
    this.deviceCode = '123456789',
    this.currentConnectionState = WsConnectionState.connected,
  });

  @override
  final String deviceCode;

  @override
  WsConnectionState currentConnectionState;

  bool connectCalled = false;
  String? lastRequestTargetCode;
  String? lastInviteCode;

  @override
  Future<void> connect() async {
    connectCalled = true;
    currentConnectionState = WsConnectionState.connected;
  }

  @override
  void requestConnection(String targetCode, {String? inviteCode}) {
    lastRequestTargetCode = targetCode;
    lastInviteCode = inviteCode;
  }
}

// =============================================================================
// TestSessionNotifier — exposes a public setState for test control.
// =============================================================================

class TestSessionNotifier extends SessionNotifier {
  TestSessionNotifier(super.engine, super.signaling);

  /// Public setter so tests can directly control session state.
  void setSessionState(SessionInfo? info) {
    state = info;
  }
}

// =============================================================================
// Helper: pump a test app with provider overrides and GoRouter.
// =============================================================================

/// Creates a [ProviderContainer] with standard test overrides and pumps
/// a [MaterialApp.router] that includes all major routes.
///
/// Returns the container so tests can inspect provider state.
Future<ProviderContainer> pumpTestApp(
  WidgetTester tester, {
  String initialLocation = '/',
  RdcsConfig? config,
  FakeEngineIsolate? fakeEngine,
  FakeSessionSignaling? fakeSignaling,
  SessionInfo? initialSession,
}) async {
  final engine = fakeEngine ?? FakeEngineIsolate();
  final signaling = fakeSignaling ?? FakeSessionSignaling();
  final configData = config ?? const RdcsConfig(deviceCode: '123456789');
  final fakeRepo = FakeConfigRepository(configData);

  final container = ProviderContainer(overrides: [
    configRepositoryProvider.overrideWithValue(fakeRepo),
    engineProvider.overrideWithValue(engine),
    sessionProvider.overrideWith((ref) {
      final notifier = TestSessionNotifier(ref.watch(engineProvider), signaling);
      if (initialSession != null) {
        notifier.setSessionState(initialSession);
      }
      return notifier;
    }),
  ]);

  final router = GoRouter(
    initialLocation: initialLocation,
    routes: [
      GoRoute(
        path: '/',
        builder: (context, state) => const HomePage(),
      ),
      GoRoute(
        path: '/connect',
        builder: (context, state) => const ConnectPage(),
      ),
      GoRoute(
        path: '/session',
        builder: (context, state) => const SessionScreen(),
      ),
      GoRoute(
        path: '/settings',
        builder: (context, state) => const SettingsScreen(),
      ),
    ],
  );

  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MaterialApp.router(routerConfig: router),
    ),
  );

  return container;
}

// =============================================================================
// Convenience: standard test device code used across tests.
// =============================================================================

const testDeviceCode = '123456789';
const testDeviceCodeFormatted = '123-456-789';
