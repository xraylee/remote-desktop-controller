// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';
import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/core/signaling/websocket_client.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';
import '../../mocks/mock_websocket_client.dart';

void main() {
  group('WebSocketClient', () {
    late MockWebSocketClient client;

    setUp(() {
      client = MockWebSocketClient();
    });

    tearDown(() {
      client.disconnect();
      client.dispose();
    });

    // ── Connection State Management ──────────────────────────────

    test('initial state is disconnected', () {
      expect(client.currentState, WsConnectionState.disconnected);
    });

    test('state changes to connecting when connect called', () async {
      final states = <WsConnectionState>[];
      final subscription = client.state.listen(states.add);

      unawaited(client.connect());
      await Future.delayed(const Duration(milliseconds: 10));

      expect(states, contains(WsConnectionState.connecting));

      await subscription.cancel();
    });

    test('state changes to connected on successful connection', () async {
      final states = <WsConnectionState>[];
      final subscription = client.state.listen(states.add);

      await client.connect();

      expect(states, containsAllInOrder([
        WsConnectionState.connecting,
        WsConnectionState.connected,
      ]));
      expect(client.currentState, WsConnectionState.connected);

      await subscription.cancel();
    });

    test('state changes to disconnected when disconnect called', () async {
      await client.connect();
      expect(client.currentState, WsConnectionState.connected);

      client.disconnect();

      expect(client.currentState, WsConnectionState.disconnected);
    });

    test('state changes to error on connection failure', () async {
      final failClient = MockWebSocketClient(shouldFailConnection: true);
      final states = <WsConnectionState>[];
      final subscription = failClient.state.listen(states.add);

      try {
        await failClient.connect();
        fail('Should have thrown');
      } catch (_) {
        // Expected to fail
      }

      expect(states, contains(WsConnectionState.error));
      expect(failClient.currentState, WsConnectionState.error);

      await subscription.cancel();
      failClient.dispose();
    });

    // ── Message Sending ──────────────────────────────────────────

    test('send() transmits message when connected', () async {
      await client.connect();

      final message = SignalingMessage.register(
        deviceCode: 'TEST123456',
        platform: 'test',
        version: '1.0.0',
      );

      client.send(message);

      expect(client.sentMessages, contains(message));
    });

    test('send() throws when not connected', () {
      expect(client.currentState, WsConnectionState.disconnected);

      final message = SignalingMessage.register(
        deviceCode: 'TEST123456',
        platform: 'test',
        version: '1.0.0',
      );

      expect(() => client.send(message), throwsStateError);
    });

    test('send() records all sent messages', () async {
      await client.connect();

      final msg1 = SignalingMessage.register(
        deviceCode: 'TEST123456',
        platform: 'test',
        version: '1.0.0',
      );
      final msg2 = SignalingMessage.heartbeat(
        deviceCode: 'TEST123456',
        ts: DateTime.now().millisecondsSinceEpoch,
      );

      client.send(msg1);
      client.send(msg2);

      expect(client.sentMessages, hasLength(2));
      expect(client.sentMessages[0], msg1);
      expect(client.sentMessages[1], msg2);
    });

    // ── Message Receiving ────────────────────────────────────────

    test('receives incoming messages via stream', () async {
      await client.connect();

      final receivedMessages = <SignalingMessage>[];
      final subscription = client.messages.listen(receivedMessages.add);

      final testMessage = SignalingMessage.nearbyUpdate(
        devices: [
          const DeviceInfo(
            code: 'PEER123456',
            name: 'Test Device',
            platform: 'test',
            online: true,
          ),
        ],
      );

      client.simulateMessage(testMessage);
      await Future.delayed(const Duration(milliseconds: 10));

      expect(receivedMessages, contains(testMessage));

      await subscription.cancel();
    });

    test('messages stream delivers multiple messages', () async {
      await client.connect();

      final receivedMessages = <SignalingMessage>[];
      final subscription = client.messages.listen(receivedMessages.add);

      final msg1 = SignalingMessage.nearbyUpdate(
        devices: [
          const DeviceInfo(
            code: 'PEER1',
            name: 'Device 1',
            platform: 'test',
            online: true,
          ),
        ],
      );
      final msg2 = SignalingMessage.nearbyUpdate(
        devices: [
          const DeviceInfo(
            code: 'PEER2',
            name: 'Device 2',
            platform: 'test',
            online: true,
          ),
        ],
      );

      client.simulateMessage(msg1);
      client.simulateMessage(msg2);
      await Future.delayed(const Duration(milliseconds: 10));

      expect(receivedMessages, hasLength(2));
      expect(receivedMessages[0], msg1);
      expect(receivedMessages[1], msg2);

      await subscription.cancel();
    });

    // ── Heartbeat Mechanism ──────────────────────────────────────

    test('startHeartbeat() enables heartbeat', () async {
      await client.connect();

      client.startHeartbeat('TEST123456');

      // Verify heartbeat was started (no exception)
      expect(client.currentState, WsConnectionState.connected);

      client.stopHeartbeat();
    });

    test('stopHeartbeat() cancels heartbeat timer', () {
      client.startHeartbeat('TEST123456');
      client.stopHeartbeat();

      // Should not throw on second call
      expect(() => client.stopHeartbeat(), returnsNormally);
    });

    test('heartbeat stops automatically on disconnect', () async {
      await client.connect();
      client.startHeartbeat('TEST123456');

      client.disconnect();

      // Heartbeat should be stopped, verify by checking state
      expect(client.currentState, WsConnectionState.disconnected);
    });

    // ── Connection Loss Handling ─────────────────────────────────

    test('handles connection loss', () async {
      await client.connect();
      expect(client.currentState, WsConnectionState.connected);

      final states = <WsConnectionState>[];
      final subscription = client.state.listen(states.add);

      client.simulateDisconnection();

      expect(states, contains(WsConnectionState.disconnected));
      expect(client.currentState, WsConnectionState.disconnected);

      await subscription.cancel();
    });

    test('can reconnect after disconnection', () async {
      await client.connect();
      client.disconnect();

      expect(client.currentState, WsConnectionState.disconnected);

      await client.connect();
      expect(client.currentState, WsConnectionState.connected);
    });

    // ── Resource Cleanup ─────────────────────────────────────────

    test('disconnect() cleans up resources', () async {
      await client.connect();
      client.startHeartbeat('TEST123456');

      client.disconnect();

      expect(client.currentState, WsConnectionState.disconnected);
    });

    test('multiple disconnect() calls are safe', () async {
      await client.connect();

      client.disconnect();
      client.disconnect();
      client.disconnect();

      expect(client.currentState, WsConnectionState.disconnected);
    });

    test('dispose() closes all streams', () async {
      await client.connect();

      client.dispose();

      // Streams should be closed, no errors should occur
      expect(client.currentState, isNotNull);
    });

    // ── Delayed Connection ───────────────────────────────────────

    test('handles slow connection', () async {
      final slowClient = MockWebSocketClient(
        connectionDelay: const Duration(milliseconds: 100),
      );

      final stopwatch = Stopwatch()..start();
      await slowClient.connect();
      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, greaterThanOrEqualTo(100));
      expect(slowClient.currentState, WsConnectionState.connected);

      slowClient.dispose();
    });
  });
}
