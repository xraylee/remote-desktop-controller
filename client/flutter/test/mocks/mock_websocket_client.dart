// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';
import 'package:rdcs_client/core/signaling/websocket_client.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';
import 'package:rxdart/rxdart.dart';

/// Mock WebSocket client for testing.
class MockWebSocketClient implements WebSocketClient {
  MockWebSocketClient({
    this.shouldFailConnection = false,
    this.connectionDelay = Duration.zero,
    this.serverUrl = 'ws://mock:8080',
    this.heartbeatInterval = const Duration(seconds: 30),
    this.reconnectDelay = const Duration(seconds: 5),
    this.maxReconnectDelay = const Duration(seconds: 60),
  });

  final bool shouldFailConnection;
  final Duration connectionDelay;

  @override
  final String serverUrl;

  @override
  final Duration heartbeatInterval;

  @override
  final Duration reconnectDelay;

  @override
  final Duration maxReconnectDelay;

  final _stateController = BehaviorSubject<WsConnectionState>.seeded(
    WsConnectionState.disconnected,
  );
  final _messagesController = StreamController<SignalingMessage>.broadcast();

  final List<SignalingMessage> sentMessages = [];
  Timer? _heartbeatTimer;

  @override
  Stream<WsConnectionState> get state => _stateController.stream;

  @override
  WsConnectionState get currentState => _stateController.value;

  @override
  Stream<SignalingMessage> get messages => _messagesController.stream;

  @override
  Future<void> connect() async {
    _stateController.add(WsConnectionState.connecting);

    if (connectionDelay > Duration.zero) {
      await Future.delayed(connectionDelay);
    }

    if (shouldFailConnection) {
      _stateController.add(WsConnectionState.error);
      throw Exception('Connection failed');
    }

    _stateController.add(WsConnectionState.connected);
  }

  @override
  void disconnect() {
    stopHeartbeat();
    _stateController.add(WsConnectionState.disconnected);
  }

  @override
  void send(SignalingMessage message) {
    if (currentState != WsConnectionState.connected) {
      throw StateError('Cannot send message when not connected');
    }
    sentMessages.add(message);
  }

  @override
  void startHeartbeat(String deviceCode) {
    _heartbeatTimer?.cancel();
    _heartbeatTimer = Timer.periodic(const Duration(seconds: 30), (_) {
      if (currentState == WsConnectionState.connected) {
        send(SignalingMessage.heartbeat(
          deviceCode: deviceCode,
          ts: DateTime.now().millisecondsSinceEpoch,
        ));
      }
    });
  }

  @override
  void stopHeartbeat() {
    _heartbeatTimer?.cancel();
    _heartbeatTimer = null;
  }

  /// Simulate receiving a message from the server.
  void simulateMessage(SignalingMessage message) {
    _messagesController.add(message);
  }

  /// Simulate connection loss.
  void simulateDisconnection() {
    _stateController.add(WsConnectionState.disconnected);
  }

  /// Clean up resources.
  void dispose() {
    _heartbeatTimer?.cancel();
    _stateController.close();
    _messagesController.close();
  }
}
