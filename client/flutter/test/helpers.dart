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
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';
import 'package:rdcs_client/core/signaling/session_signaling.dart';
import 'package:rdcs_client/core/signaling/signaling_service.dart';
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
    this.failConnectAttempts = 0,
    this.failRequestAttempts = 0,
  });

  @override
  final String deviceCode;

  @override
  WsConnectionState currentConnectionState;

  bool connectCalled = false;
  int connectAttempts = 0;
  int failConnectAttempts;
  int requestAttempts = 0;
  int failRequestAttempts;
  String? lastRequestTargetCode;
  String? lastInviteCode;

  final _connectResponsesController =
      StreamController<ConnectResponseMessage>.broadcast();

  @override
  Stream<ConnectResponseMessage> get connectResponses =>
      _connectResponsesController.stream;

  /// Test hook: push a fake connect_response to the notifier under test.
  void emitConnectResponse(ConnectResponseMessage msg) =>
      _connectResponsesController.add(msg);

  /// Close the fake's stream controller (call in test tearDown if needed).
  void disposeFake() => _connectResponsesController.close();

  @override
  Future<void> connect() async {
    connectCalled = true;
    connectAttempts++;
    if (connectAttempts <= failConnectAttempts) {
      currentConnectionState = WsConnectionState.error;
      throw StateError('signaling connect failed');
    }
    currentConnectionState = WsConnectionState.connected;
  }

  @override
  void requestConnection(String targetCode, {String? inviteCode}) {
    requestAttempts++;
    if (requestAttempts <= failRequestAttempts) {
      throw StateError('connect_request send failed');
    }
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
      final notifier =
          TestSessionNotifier(ref.watch(engineProvider), signaling);
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

// =============================================================================
// LoopbackSignalingBus — in-memory stand-in for the signaling server that
// routes one initiator to one acceptor without any network I/O.
//
// It reproduces the two routing facts the real server enforces (see
// crates/rdcs-signaling/src/handlers/connect.rs) that the UI relies on:
//
//   • connect_request(from=initiator, to=acceptor) is delivered to the
//     acceptor as an `invitation`, carrying a server-minted session_id.
//   • connect_response is delivered back to the *initiator* with `from_code`
//     rewritten to the **responder's own device code** (server line ~159 uses
//     the responder's connection identity via take_pending_connection, NOT the
//     `from_code` the client put in the body). This is why the initiator can
//     match on `fromCode == the code it dialed`, and why InvitationHost's body
//     `fromCode` is a don't-care on the happy path.
//
// A single bus backs both the initiator's [SessionSignaling] view
// ([busInitiatorSignaling]) and the acceptor's [SignalingService] view
// ([BusAcceptorSignalingService]), so a single-process widget test can drive
// the full UI handshake through it.
// =============================================================================

class LoopbackSignalingBus {
  LoopbackSignalingBus({
    required this.initiatorCode,
    required this.acceptorCode,
    this.sessionId = 'loopback-session',
  });

  final String initiatorCode;
  final String acceptorCode;

  /// The session_id the "server" mints for this connection.
  final String sessionId;

  // Stream the acceptor's InvitationHost listens on (its `invitations`).
  final _invitations = StreamController<ConnectRequestMessage>.broadcast();

  // Stream the initiator's SessionNotifier listens on (its `connectResponses`).
  final _connectResponses = StreamController<ConnectResponseMessage>.broadcast();

  Stream<ConnectRequestMessage> get invitations => _invitations.stream;
  Stream<ConnectResponseMessage> get connectResponses =>
      _connectResponses.stream;

  /// Initiator dials the acceptor. The bus mints a session_id and forwards a
  /// connect_request to the acceptor, exactly as the server would.
  void routeConnectRequest(String targetCode, {String? inviteCode}) {
    _invitations.add(ConnectRequestMessage(
      fromCode: initiatorCode,
      toCode: targetCode,
      sessionId: sessionId,
      inviteCode: inviteCode,
    ));
  }

  /// Acceptor responds. The bus rewrites `from_code` to the acceptor's own
  /// code before delivering to the initiator — mirroring the server, and
  /// deliberately ignoring the [bodyFromCode] the client supplied.
  void routeConnectResponse({
    required String sessionId,
    required String bodyFromCode,
    required bool accepted,
  }) {
    _connectResponses.add(ConnectResponseMessage(
      accepted: accepted,
      sessionId: sessionId,
      fromCode: acceptorCode, // server rewrites to responder identity
    ));
  }

  Future<void> dispose() async {
    await _invitations.close();
    await _connectResponses.close();
  }
}

/// Initiator-side [SessionSignaling] view backed by a [LoopbackSignalingBus].
///
/// `requestConnection` routes onto the bus; `connectResponses` is the bus's
/// initiator-facing stream. This is what the `sessionProvider` under test uses.
class BusInitiatorSignaling implements SessionSignaling {
  BusInitiatorSignaling(this.bus);

  final LoopbackSignalingBus bus;

  @override
  String get deviceCode => bus.initiatorCode;

  @override
  WsConnectionState currentConnectionState = WsConnectionState.connected;

  @override
  Stream<ConnectResponseMessage> get connectResponses => bus.connectResponses;

  @override
  Future<void> connect() async {
    currentConnectionState = WsConnectionState.connected;
  }

  @override
  void requestConnection(String targetCode, {String? inviteCode}) {
    bus.routeConnectRequest(targetCode, inviteCode: inviteCode);
  }
}

/// Acceptor-side [SignalingService] view backed by a [LoopbackSignalingBus].
///
/// Only the members the acceptor path touches are implemented:
/// `respondToConnection` routes onto the bus. `invitations` is provided via a
/// provider override in the test, not this object. Everything else falls
/// through to [noSuchMethod] (unused by the dialog flow).
class BusAcceptorSignalingService implements SignalingService {
  BusAcceptorSignalingService(this.bus);

  final LoopbackSignalingBus bus;

  /// Records what InvitationHost actually passed, so the test can pin the
  /// (currently initiator-code) body semantics without changing production.
  String? lastRespondBodyFromCode;

  @override
  void respondToConnection({
    required String sessionId,
    required String fromCode,
    required bool accepted,
  }) {
    lastRespondBodyFromCode = fromCode;
    bus.routeConnectResponse(
      sessionId: sessionId,
      bodyFromCode: fromCode,
      accepted: accepted,
    );
  }

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      super.noSuchMethod(invocation);
}
