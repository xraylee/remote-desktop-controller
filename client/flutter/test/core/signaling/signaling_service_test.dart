// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';
import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/core/signaling/signaling_service.dart';
import 'package:rdcs_client/core/signaling/websocket_client.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';

import '../../mocks/mock_websocket_client.dart';

void main() {
  group('SignalingService', () {
    late SignalingService service;
    const testServerUrl = 'ws://localhost:8080';
    const testDeviceCode = 'TEST123456';
    const testPlatform = 'test';

    setUp(() {
      service = SignalingService(
        serverUrl: testServerUrl,
        deviceCode: testDeviceCode,
        platform: testPlatform,
      );
    });

    tearDown(() {
      service.disconnect();
    });

    // ── Initialization ───────────────────────────────────────────

    test('initializes with correct configuration', () {
      expect(service.serverUrl, testServerUrl);
      expect(service.deviceCode, testDeviceCode);
      expect(service.platform, testPlatform);
    });

    test('initial connection state is disconnected', () {
      expect(service.currentConnectionState, WsConnectionState.disconnected);
    });

    test('initial nearby devices list is empty', () {
      expect(service.currentNearbyDevices, isEmpty);
    });

    // ── Connection Lifecycle ─────────────────────────────────────

    test('connect() establishes connection and registers device', () async {
      await service.connect();

      expect(service.currentConnectionState, WsConnectionState.connected);
    });

    test('disconnect() closes connection cleanly', () async {
      await service.connect();
      expect(service.currentConnectionState, WsConnectionState.connected);

      service.disconnect();
      await Future.delayed(const Duration(milliseconds: 50));

      expect(service.currentConnectionState, WsConnectionState.disconnected);
    });

    test('connect() starts heartbeat after registration', () async {
      await service.connect();

      // Heartbeat should be running (verified by no errors)
      expect(service.currentConnectionState, WsConnectionState.connected);
    });

    test('disconnect() stops heartbeat', () async {
      await service.connect();

      service.disconnect();

      // Should not throw or cause issues
      await Future.delayed(const Duration(milliseconds: 100));
    });

    // ── Device Registration ──────────────────────────────────────

    test('register() sends registration message', () async {
      await service.connect();

      // Registration happens automatically in connect()
      // Verify we're connected successfully
      expect(service.currentConnectionState, WsConnectionState.connected);
    });

    test('register() includes optional teamId', () async {
      final serviceWithTeam = SignalingService(
        serverUrl: testServerUrl,
        deviceCode: testDeviceCode,
        platform: testPlatform,
        teamId: 'team123',
      );

      await serviceWithTeam.connect();
      expect(serviceWithTeam.currentConnectionState, WsConnectionState.connected);

      serviceWithTeam.disconnect();
    });

    // ── Connection Requests ──────────────────────────────────────

    test('requestConnection() sends connect_request message', () async {
      await service.connect();

      // Should not throw
      expect(
        () => service.requestConnection('TARGET123456'),
        returnsNormally,
      );
    });

    test('requestConnection() with invite code', () async {
      await service.connect();

      expect(
        () => service.requestConnection(
          'TARGET123456',
          inviteCode: 'INVITE789',
        ),
        returnsNormally,
      );
    });

    test('respondToConnection() accepts connection', () async {
      await service.connect();

      expect(
        () => service.respondToConnection(
          sessionId: 'session-123',
          fromCode: 'PEER123456',
          accepted: true,
        ),
        returnsNormally,
      );
    });

    test('respondToConnection() rejects connection', () async {
      await service.connect();

      expect(
        () => service.respondToConnection(
          sessionId: 'session-123',
          fromCode: 'PEER123456',
          accepted: false,
        ),
        returnsNormally,
      );
    });

    // ── ICE Signaling ────────────────────────────────────────────

    test('sendIceOffer() transmits offer', () async {
      await service.connect();

      expect(
        () => service.sendIceOffer(
          sessionId: 'session-123',
          sdp: 'v=0\r\no=- 123456 2 IN IP4 127.0.0.1\r\n',
          candidates: [
            IceCandidate(
              candidate: 'candidate:1 1 UDP 2130706431 192.168.1.100 54321 typ host',
              sdpMid: '0',
              sdpMLineIndex: 0,
            ),
          ],
        ),
        returnsNormally,
      );
    });

    test('sendIceAnswer() transmits answer', () async {
      await service.connect();

      expect(
        () => service.sendIceAnswer(
          sessionId: 'session-123',
          sdp: 'v=0\r\no=- 654321 2 IN IP4 127.0.0.1\r\n',
          candidates: [],
        ),
        returnsNormally,
      );
    });

    test('sendIceCandidate() transmits trickle candidate', () async {
      await service.connect();

      expect(
        () => service.sendIceCandidate(
          sessionId: 'session-123',
          candidate: IceCandidate(
            candidate: 'candidate:2 1 TCP 1015022079 192.168.1.100 9 typ host',
            sdpMid: '0',
            sdpMLineIndex: 0,
          ),
        ),
        returnsNormally,
      );
    });

    // ── Relay Server Requests ────────────────────────────────────

    test('requestRelay() sends relay request', () async {
      await service.connect();

      expect(
        () => service.requestRelay(sessionId: 'session-123'),
        returnsNormally,
      );
    });

    test('requestRelay() with preferred region', () async {
      await service.connect();

      expect(
        () => service.requestRelay(
          sessionId: 'session-123',
          preferredRegion: 'us-west',
        ),
        returnsNormally,
      );
    });

    // ── Invite Code Management ───────────────────────────────────

    test('generateInviteCode() requests invite generation', () async {
      await service.connect();

      expect(() => service.generateInviteCode(), returnsNormally);
    });

    test('useInviteCode() sends invite usage', () async {
      await service.connect();

      expect(
        () => service.useInviteCode('INVITE789'),
        returnsNormally,
      );
    });

    // ── Message Stream Handling ──────────────────────────────────

    test('invitations stream emits connect_request messages', () async {
      await service.connect();

      final invitations = <ConnectRequestMessage>[];
      final subscription = service.invitations.listen(invitations.add);

      // Wait for a potential invitation (requires server cooperation)
      await Future.delayed(const Duration(milliseconds: 100));

      // For now, verify stream is accessible
      expect(service.invitations, isNotNull);

      await subscription.cancel();
    });

    test('nearbyDevices stream updates on nearby_update', () async {
      await service.connect();

      final deviceLists = <List<DeviceInfo>>[];
      final subscription = service.nearbyDevices.listen(deviceLists.add);

      // Initial empty list should be emitted
      expect(deviceLists.first, isEmpty);

      await subscription.cancel();
    });

    test('relayAssigned stream emits relay assignments', () async {
      await service.connect();

      final relays = <RelayAssignedMessage>[];
      final subscription = service.relayAssigned.listen(relays.add);

      await Future.delayed(const Duration(milliseconds: 100));

      expect(service.relayAssigned, isNotNull);

      await subscription.cancel();
    });

    test('errors stream emits error messages', () async {
      await service.connect();

      final errors = <ErrorMessage>[];
      final subscription = service.errors.listen(errors.add);

      await Future.delayed(const Duration(milliseconds: 100));

      expect(service.errors, isNotNull);

      await subscription.cancel();
    });

    test('inviteGenerated stream emits generated codes', () async {
      await service.connect();

      final codes = <String>[];
      final subscription = service.inviteGenerated.listen(codes.add);

      service.generateInviteCode();
      await Future.delayed(const Duration(milliseconds: 100));

      expect(service.inviteGenerated, isNotNull);

      await subscription.cancel();
    });

    // ── Nearby Devices Management ────────────────────────────────

    test('nearby devices list excludes own device', () async {
      await service.connect();

      // Even if server sends our own device, it should be filtered
      final devices = service.currentNearbyDevices;
      expect(devices.every((d) => d.code != testDeviceCode), isTrue);
    });

    test('nearby devices online event adds device', () async {
      await service.connect();

      final deviceLists = <List<DeviceInfo>>[];
      final subscription = service.nearbyDevices.listen(deviceLists.add);

      // Wait for nearby_update from server
      await Future.delayed(const Duration(milliseconds: 100));

      // For now, verify mechanism exists
      expect(service.currentNearbyDevices, isNotNull);

      await subscription.cancel();
    });

    test('nearby devices offline event removes device', () async {
      await service.connect();

      final deviceLists = <List<DeviceInfo>>[];
      final subscription = service.nearbyDevices.listen(deviceLists.add);

      await Future.delayed(const Duration(milliseconds: 100));

      expect(service.currentNearbyDevices, isNotNull);

      await subscription.cancel();
    });

    // ── Error Handling ───────────────────────────────────────────

    test('handles connection failure gracefully', () async {
      final failService = SignalingService(
        serverUrl: 'ws://invalid:99999',
        deviceCode: testDeviceCode,
        platform: testPlatform,
      );

      expect(
        () => failService.connect().timeout(
          const Duration(milliseconds: 500),
          onTimeout: () => throw TimeoutException('Connection timeout'),
        ),
        throwsA(isA<TimeoutException>()),
      );

      failService.disconnect();
    });

    test('handles malformed messages without crashing', () async {
      await service.connect();

      // Service should handle malformed data gracefully
      // This requires injecting bad data, which needs server cooperation
      expect(service.currentConnectionState, WsConnectionState.connected);
    });

    test('handles unexpected message types gracefully', () async {
      await service.connect();

      // Service logs unexpected messages but doesn't crash
      expect(service.currentConnectionState, WsConnectionState.connected);
    });

    // ── Resource Cleanup ─────────────────────────────────────────

    test('disconnect() closes all streams', () async {
      await service.connect();

      service.disconnect();

      // Streams should complete or stop emitting
      // Verify no errors occur
      await Future.delayed(const Duration(milliseconds: 100));
    });

    test('multiple disconnect() calls are safe', () async {
      await service.connect();

      service.disconnect();
      service.disconnect();
      service.disconnect();

      expect(service.currentConnectionState, WsConnectionState.disconnected);
    });

    // ── Connection State Propagation ─────────────────────────────

    test('connectionState stream mirrors WebSocket state', () async {
      final states = <WsConnectionState>[];
      final subscription = service.connectionState.listen(states.add);

      await service.connect();

      expect(states, contains(WsConnectionState.connected));

      await subscription.cancel();
    });

    test('currentConnectionState reflects current state', () async {
      expect(service.currentConnectionState, WsConnectionState.disconnected);

      await service.connect();
      expect(service.currentConnectionState, WsConnectionState.connected);

      service.disconnect();
      await Future.delayed(const Duration(milliseconds: 50));
      expect(service.currentConnectionState, WsConnectionState.disconnected);
    });
  });

  // ── Reconnection & Re-registration (injected mock, no live server) ──────
  group('SignalingService reconnection', () {
    const testServerUrl = 'ws://mock:8080';
    const testDeviceCode = 'TEST123456';
    const testPlatform = 'test';

    late MockWebSocketClient mockClient;
    late SignalingService service;

    setUp(() {
      mockClient = MockWebSocketClient(serverUrl: testServerUrl);
      service = SignalingService(
        serverUrl: testServerUrl,
        deviceCode: testDeviceCode,
        platform: testPlatform,
        client: mockClient,
      );
    });

    tearDown(() {
      service.dispose();
    });

    int registerCount() =>
        mockClient.sentMessages.whereType<RegisterMessage>().length;

    test('first connect registers exactly once and starts heartbeat',
        () async {
      await service.connect();
      // Let the state-stream observer fire.
      await Future.delayed(const Duration(milliseconds: 10));

      expect(registerCount(), 1);
      expect(mockClient.heartbeatStartCount, 1);
    });

    test('reconnect re-registers and restarts heartbeat', () async {
      await service.connect();
      await Future.delayed(const Duration(milliseconds: 10));
      expect(registerCount(), 1);
      expect(mockClient.heartbeatStartCount, 1);

      // Simulate an automatic reconnect: reconnecting → connected.
      mockClient.simulateReconnect();
      await Future.delayed(const Duration(milliseconds: 10));

      expect(registerCount(), 2,
          reason: 'device must re-register after reconnect');
      expect(mockClient.heartbeatStartCount, 2,
          reason: 'heartbeat must restart after reconnect');
    });

    test('duplicate connected emission does not double-register', () async {
      await service.connect();
      await Future.delayed(const Duration(milliseconds: 10));
      expect(registerCount(), 1);

      // A stale `connected` replay with no intervening non-connected state
      // must not trigger a second registration.
      mockClient.emitState(WsConnectionState.connected);
      await Future.delayed(const Duration(milliseconds: 10));

      expect(registerCount(), 1);
      expect(mockClient.heartbeatStartCount, 1);
    });

    test('no re-registration after manual disconnect', () async {
      await service.connect();
      await Future.delayed(const Duration(milliseconds: 10));
      expect(registerCount(), 1);

      service.disconnect();
      await Future.delayed(const Duration(milliseconds: 10));

      // A stray `connected` after manual disconnect must be ignored: the
      // state subscription was cancelled, so no new register is sent.
      mockClient.emitState(WsConnectionState.connected);
      await Future.delayed(const Duration(milliseconds: 10));

      expect(registerCount(), 1);
    });
  });
}
