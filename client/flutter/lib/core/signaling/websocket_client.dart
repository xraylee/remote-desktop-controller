// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:web_socket_channel/io.dart';
import 'package:rxdart/rxdart.dart';

import 'models/signaling_message.dart';

/// WebSocket connection state.
enum WsConnectionState {
  disconnected,
  connecting,
  connected,
  reconnecting,
  error,
}

/// WebSocket client for signaling server communication.
///
/// Handles:
/// - Connection lifecycle (connect, disconnect, reconnect)
/// - Message serialization/deserialization
/// - Automatic heartbeat
/// - Exponential backoff reconnection
class WebSocketClient {
  WebSocketClient({
    required this.serverUrl,
    this.heartbeatInterval = const Duration(seconds: 30),
    this.reconnectDelay = const Duration(seconds: 2),
    this.maxReconnectDelay = const Duration(seconds: 30),
  });

  final String serverUrl;
  final Duration heartbeatInterval;
  final Duration reconnectDelay;
  final Duration maxReconnectDelay;

  WebSocketChannel? _channel;
  Timer? _heartbeatTimer;
  Timer? _reconnectTimer;
  StreamSubscription? _channelSubscription;

  /// Current connection state.
  final _stateController = BehaviorSubject<WsConnectionState>.seeded(
    WsConnectionState.disconnected,
  );
  Stream<WsConnectionState> get state => _stateController.stream;
  WsConnectionState get currentState => _stateController.value;

  /// Incoming messages from the server.
  final _messageController = StreamController<SignalingMessage>.broadcast();
  Stream<SignalingMessage> get messages => _messageController.stream;

  /// Device code for heartbeat messages (set after registration).
  String? _deviceCode;

  /// Current reconnection attempt count.
  int _reconnectAttempts = 0;

  /// Whether the client is intentionally disconnected (no auto-reconnect).
  bool _manualDisconnect = false;

  // ── Public API ─────────────────────────────────────────────────

  /// Connect to the signaling server.
  Future<void> connect() async {
    if (currentState == WsConnectionState.connected ||
        currentState == WsConnectionState.connecting) {
      return;
    }

    _manualDisconnect = false;
    _stateController.add(WsConnectionState.connecting);

    try {
      // Create WebSocket with no proxy to avoid system proxy interference
      final uri = Uri.parse(serverUrl);
      final socket = await WebSocket.connect(
        serverUrl,
        customClient: HttpClient()..findProxy = (uri) => 'DIRECT',
      );
      _channel = IOWebSocketChannel(socket);

      // Listen for incoming messages
      _channelSubscription = _channel!.stream.listen(
        _onMessage,
        onError: _onError,
        onDone: _onDisconnect,
      );

      // Wait a moment to ensure connection is established
      await Future.delayed(const Duration(milliseconds: 500));

      _stateController.add(WsConnectionState.connected);
      _reconnectAttempts = 0;

      print('✅ WebSocket connected to $serverUrl');
    } catch (e) {
      print('❌ WebSocket connection failed: $e');
      _stateController.add(WsConnectionState.error);
      _scheduleReconnect();
    }
  }

  /// Disconnect from the server (no auto-reconnect).
  void disconnect() {
    _manualDisconnect = true;
    _cleanup();
    _stateController.add(WsConnectionState.disconnected);
    print('🔌 WebSocket disconnected');
  }

  /// Send a message to the server.
  void send(SignalingMessage message) {
    if (currentState != WsConnectionState.connected) {
      print('⚠️  Cannot send message: not connected');
      return;
    }

    try {
      final json = message.toJson();
      final encoded = jsonEncode(json);
      _channel?.sink.add(encoded);

      // Debug log (exclude heartbeat to reduce noise)
      if (message is! HeartbeatMessage) {
        print('📤 Sent: ${json['type']}');
      }
    } catch (e) {
      print('❌ Failed to send message: $e');
    }
  }

  /// Set device code and start heartbeat timer.
  void startHeartbeat(String deviceCode) {
    _deviceCode = deviceCode;
    _heartbeatTimer?.cancel();

    _heartbeatTimer = Timer.periodic(heartbeatInterval, (_) {
      if (currentState == WsConnectionState.connected && _deviceCode != null) {
        send(SignalingMessage.heartbeat(
          deviceCode: _deviceCode!,
          ts: DateTime.now().millisecondsSinceEpoch ~/ 1000,
        ));
      }
    });

    print('💓 Heartbeat started (interval: ${heartbeatInterval.inSeconds}s)');
  }

  /// Stop heartbeat timer.
  void stopHeartbeat() {
    _heartbeatTimer?.cancel();
    _heartbeatTimer = null;
    _deviceCode = null;
    print('💔 Heartbeat stopped');
  }

  /// Dispose resources.
  void dispose() {
    _manualDisconnect = true;
    _cleanup();
    _stateController.close();
    _messageController.close();
  }

  // ── Private Methods ────────────────────────────────────────────

  /// Handle incoming WebSocket message.
  void _onMessage(dynamic data) {
    try {
      final json = jsonDecode(data as String) as Map<String, dynamic>;
      final message = SignalingMessage.fromJson(json);

      // Debug log (exclude heartbeat responses if any)
      final type = json['type'] as String?;
      if (type != 'heartbeat') {
        print('📥 Received: $type');
      }

      _messageController.add(message);
    } catch (e) {
      print('❌ Failed to parse message: $e');
    }
  }

  /// Handle WebSocket error.
  void _onError(dynamic error) {
    print('❌ WebSocket error: $error');
    _stateController.add(WsConnectionState.error);
  }

  /// Handle WebSocket disconnection.
  void _onDisconnect() {
    print('🔌 WebSocket disconnected');
    _cleanup();

    if (!_manualDisconnect) {
      _stateController.add(WsConnectionState.reconnecting);
      _scheduleReconnect();
    } else {
      _stateController.add(WsConnectionState.disconnected);
    }
  }

  /// Schedule reconnection with exponential backoff.
  void _scheduleReconnect() {
    if (_manualDisconnect) return;

    _reconnectTimer?.cancel();

    // Calculate delay with exponential backoff
    final delay = Duration(
      milliseconds: reconnectDelay.inMilliseconds *
          (1 << _reconnectAttempts).clamp(1, 16),
    );

    final actualDelay = delay > maxReconnectDelay ? maxReconnectDelay : delay;

    print('🔄 Reconnecting in ${actualDelay.inSeconds}s (attempt ${_reconnectAttempts + 1})');

    _reconnectTimer = Timer(actualDelay, () {
      _reconnectAttempts++;
      connect();
    });
  }

  /// Clean up resources (timers, subscriptions, channel).
  void _cleanup() {
    _heartbeatTimer?.cancel();
    _heartbeatTimer = null;
    _reconnectTimer?.cancel();
    _reconnectTimer = null;
    _channelSubscription?.cancel();
    _channelSubscription = null;
    _channel?.sink.close();
    _channel = null;
  }
}
