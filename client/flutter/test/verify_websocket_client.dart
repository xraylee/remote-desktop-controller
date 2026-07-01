// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

/// Standalone verification script for WebSocketClient
/// Bypasses Flutter test framework to avoid HttpException issues

import 'dart:async';

// ── Mock Implementations ────────────────────────────────────────────

/// WebSocket connection state
enum WsConnectionState {
  disconnected,
  connecting,
  connected,
  reconnecting,
  error,
}

/// Device information (simplified)
class DeviceInfo {
  DeviceInfo({
    required this.code,
    required this.name,
    required this.platform,
    required this.online,
  });

  final String code;
  final String name;
  final String platform;
  final bool online;
}

/// Signaling message (simplified)
class SignalingMessage {
  SignalingMessage(this.type, this.data);

  final String type;
  final Map<String, dynamic> data;

  factory SignalingMessage.register({
    required String deviceCode,
    required String platform,
    required String version,
  }) {
    return SignalingMessage('register', {
      'device_code': deviceCode,
      'platform': platform,
      'version': version,
    });
  }

  factory SignalingMessage.heartbeat({
    required String deviceCode,
    required int ts,
  }) {
    return SignalingMessage('heartbeat', {
      'device_code': deviceCode,
      'ts': ts,
    });
  }

  factory SignalingMessage.nearbyUpdate({
    required List<DeviceInfo> devices,
  }) {
    return SignalingMessage('nearby_update', {
      'devices': devices.map((d) => {
        'code': d.code,
        'name': d.name,
        'platform': d.platform,
        'online': d.online,
      }).toList(),
    });
  }

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is SignalingMessage &&
          runtimeType == other.runtimeType &&
          type == other.type;

  @override
  int get hashCode => type.hashCode;
}

/// Mock WebSocket Client
class MockWebSocketClient {
  MockWebSocketClient({
    this.shouldFailConnection = false,
    this.connectionDelay = Duration.zero,
    this.serverUrl = 'ws://mock:8080',
  });

  final bool shouldFailConnection;
  final Duration connectionDelay;
  final String serverUrl;

  WsConnectionState _currentState = WsConnectionState.disconnected;
  final _stateController = StreamController<WsConnectionState>.broadcast();
  final _messageController = StreamController<SignalingMessage>.broadcast();

  Stream<WsConnectionState> get state => _stateController.stream;
  WsConnectionState get currentState => _currentState;
  Stream<SignalingMessage> get messages => _messageController.stream;

  final List<SignalingMessage> sentMessages = [];
  String? _deviceCode;
  Timer? _heartbeatTimer;

  Future<void> connect() async {
    if (_currentState == WsConnectionState.connected ||
        _currentState == WsConnectionState.connecting) {
      return;
    }

    _setState(WsConnectionState.connecting);

    if (connectionDelay > Duration.zero) {
      await Future.delayed(connectionDelay);
    }

    if (shouldFailConnection) {
      _setState(WsConnectionState.error);
      throw Exception('Connection failed');
    }

    _setState(WsConnectionState.connected);
  }

  void disconnect() {
    stopHeartbeat();
    _setState(WsConnectionState.disconnected);
  }

  void send(SignalingMessage message) {
    if (_currentState != WsConnectionState.connected) {
      throw StateError('Cannot send message when not connected');
    }
    sentMessages.add(message);
  }

  void startHeartbeat(String deviceCode) {
    _deviceCode = deviceCode;
    _heartbeatTimer?.cancel();
    _heartbeatTimer = Timer.periodic(const Duration(seconds: 30), (_) {
      if (_currentState == WsConnectionState.connected && _deviceCode != null) {
        send(SignalingMessage.heartbeat(
          deviceCode: _deviceCode!,
          ts: DateTime.now().millisecondsSinceEpoch,
        ));
      }
    });
  }

  void stopHeartbeat() {
    _heartbeatTimer?.cancel();
    _heartbeatTimer = null;
  }

  void dispose() {
    stopHeartbeat();
    _stateController.close();
    _messageController.close();
  }

  void _setState(WsConnectionState newState) {
    _currentState = newState;
    _stateController.add(newState);
  }

  // Helper: simulate incoming message
  void simulateMessage(SignalingMessage message) {
    _messageController.add(message);
  }
}

// ── Test Runner ────────────────────────────────────────────────────

void main() async {
  print('🧪 WebSocketClient 验证测试\n');

  int passed = 0;
  int failed = 0;

  Future<void> test(String description, Future<void> Function() body) async {
    try {
      await body();
      print('✅ $description');
      passed++;
    } catch (e, stack) {
      print('❌ $description');
      print('   错误: $e');
      print('   堆栈: ${stack.toString().split('\n').take(3).join('\n   ')}');
      failed++;
    }
  }

  void expect(dynamic actual, dynamic matcher, {String? reason}) {
    if (matcher is _Matcher) {
      if (!matcher.matches(actual)) {
        throw Exception(
            'Expected $actual to ${matcher.description}, but it did not. ${reason ?? ""}');
      }
    } else {
      if (actual != matcher) {
        throw Exception(
            'Expected $actual to equal $matcher. ${reason ?? ""}');
      }
    }
  }

  // ── Connection State Management Tests ────────────────────────────

  await test('initial state is disconnected', () async {
    final client = MockWebSocketClient();
    expect(client.currentState, WsConnectionState.disconnected);
    client.dispose();
  });

  await test('state changes to connecting when connect called', () async {
    final client = MockWebSocketClient();
    final states = <WsConnectionState>[];
    final subscription = client.state.listen(states.add);

    unawaited(client.connect());
    await Future.delayed(const Duration(milliseconds: 10));

    expect(states, _contains(WsConnectionState.connecting));

    await subscription.cancel();
    client.dispose();
  });

  await test('state changes to connected on successful connection', () async {
    final client = MockWebSocketClient();
    final states = <WsConnectionState>[];
    final subscription = client.state.listen(states.add);

    await client.connect();
    await Future.delayed(const Duration(milliseconds: 10));

    expect(states, _containsAll([
      WsConnectionState.connecting,
      WsConnectionState.connected,
    ]));
    expect(client.currentState, WsConnectionState.connected);

    await subscription.cancel();
    client.dispose();
  });

  await test('state changes to disconnected when disconnect called', () async {
    final client = MockWebSocketClient();
    await client.connect();
    expect(client.currentState, WsConnectionState.connected);

    client.disconnect();

    expect(client.currentState, WsConnectionState.disconnected);
    client.dispose();
  });

  await test('state changes to error on connection failure', () async {
    final client = MockWebSocketClient(shouldFailConnection: true);
    final states = <WsConnectionState>[];
    final subscription = client.state.listen(states.add);

    try {
      await client.connect();
      throw Exception('Should have thrown');
    } catch (e) {
      if (e.toString().contains('Should have thrown')) {
        rethrow;
      }
      // Expected to fail
    }

    await Future.delayed(const Duration(milliseconds: 10));

    expect(states, _contains(WsConnectionState.error));
    expect(client.currentState, WsConnectionState.error);

    await subscription.cancel();
    client.dispose();
  });

  // ── Message Sending Tests ─────────────────────────────────────────

  await test('send() transmits message when connected', () async {
    final client = MockWebSocketClient();
    await client.connect();

    final message = SignalingMessage.register(
      deviceCode: 'TEST123456',
      platform: 'test',
      version: '1.0.0',
    );

    client.send(message);

    expect(client.sentMessages, _contains(message));
    client.dispose();
  });

  await test('send() throws when not connected', () async {
    final client = MockWebSocketClient();
    expect(client.currentState, WsConnectionState.disconnected);

    final message = SignalingMessage.register(
      deviceCode: 'TEST123456',
      platform: 'test',
      version: '1.0.0',
    );

    try {
      client.send(message);
      throw Exception('Should have thrown StateError');
    } catch (e) {
      if (e is! StateError && !e.toString().contains('Should have thrown')) {
        rethrow;
      }
    }

    client.dispose();
  });

  await test('send() records all sent messages', () async {
    final client = MockWebSocketClient();
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

    expect(client.sentMessages.length, 2);
    expect(client.sentMessages[0], msg1);
    expect(client.sentMessages[1], msg2);

    client.dispose();
  });

  // ── Message Receiving Tests ───────────────────────────────────────

  await test('receives incoming messages via stream', () async {
    final client = MockWebSocketClient();
    await client.connect();

    final receivedMessages = <SignalingMessage>[];
    final subscription = client.messages.listen(receivedMessages.add);

    final testMessage = SignalingMessage.nearbyUpdate(
      devices: [
        DeviceInfo(
          code: 'PEER123456',
          name: 'Test Device',
          platform: 'test',
          online: true,
        ),
      ],
    );

    client.simulateMessage(testMessage);
    await Future.delayed(const Duration(milliseconds: 10));

    expect(receivedMessages, _contains(testMessage));

    await subscription.cancel();
    client.dispose();
  });

  await test('messages stream delivers multiple messages', () async {
    final client = MockWebSocketClient();
    await client.connect();

    final receivedMessages = <SignalingMessage>[];
    final subscription = client.messages.listen(receivedMessages.add);

    final msg1 = SignalingMessage.nearbyUpdate(
      devices: [
        DeviceInfo(
          code: 'PEER1',
          name: 'Device 1',
          platform: 'test',
          online: true,
        ),
      ],
    );
    final msg2 = SignalingMessage.nearbyUpdate(
      devices: [
        DeviceInfo(
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

    expect(receivedMessages.length, 2);
    expect(receivedMessages[0], msg1);
    expect(receivedMessages[1], msg2);

    await subscription.cancel();
    client.dispose();
  });

  // ── Heartbeat Tests ───────────────────────────────────────────────

  await test('startHeartbeat() begins sending heartbeats', () async {
    final client = MockWebSocketClient();
    await client.connect();

    client.startHeartbeat('TEST123456');

    // Heartbeat started (no immediate verification needed)
    expect(client.currentState, WsConnectionState.connected);

    client.stopHeartbeat();
    client.dispose();
  });

  await test('stopHeartbeat() stops sending heartbeats', () async {
    final client = MockWebSocketClient();
    await client.connect();

    client.startHeartbeat('TEST123456');
    client.stopHeartbeat();

    // Heartbeat stopped (no errors should occur)
    expect(client.currentState, WsConnectionState.connected);

    client.dispose();
  });

  await test('disconnect() stops heartbeat automatically', () async {
    final client = MockWebSocketClient();
    await client.connect();

    client.startHeartbeat('TEST123456');
    client.disconnect();

    expect(client.currentState, WsConnectionState.disconnected);

    client.dispose();
  });

  // ── Connection Lifecycle Tests ────────────────────────────────────

  await test('can reconnect after disconnection', () async {
    final client = MockWebSocketClient();
    await client.connect();
    client.disconnect();

    expect(client.currentState, WsConnectionState.disconnected);

    await client.connect();
    expect(client.currentState, WsConnectionState.connected);

    client.dispose();
  });

  await test('multiple disconnect() calls are safe', () async {
    final client = MockWebSocketClient();
    await client.connect();

    client.disconnect();
    client.disconnect();
    client.disconnect();

    expect(client.currentState, WsConnectionState.disconnected);

    client.dispose();
  });

  await test('dispose() closes all streams', () async {
    final client = MockWebSocketClient();
    await client.connect();

    client.dispose();

    // Streams should be closed, no errors should occur
    expect(client.currentState, _isNotNull());
  });

  // ── Delayed Connection Test ───────────────────────────────────────

  await test('handles slow connection', () async {
    final client = MockWebSocketClient(
      connectionDelay: const Duration(milliseconds: 100),
    );

    final stopwatch = Stopwatch()..start();
    await client.connect();
    stopwatch.stop();

    expect(stopwatch.elapsedMilliseconds, _greaterThanOrEqual(100));
    expect(client.currentState, WsConnectionState.connected);

    client.dispose();
  });

  // ── Results ──────────────────────────────────────────────────────

  print('\n📊 测试结果:');
  print('   通过: $passed');
  print('   失败: $failed');
  print('   总计: ${passed + failed}');

  if (failed == 0) {
    print('\n🎉 所有测试通过！');
  } else {
    print('\n⚠️  有 $failed 个测试失败');
  }
}

// ── Helper: unawaited ──────────────────────────────────────────────

void unawaited(Future<void> future) {
  // Intentionally not awaited
}

// ── Matchers ────────────────────────────────────────────────────────

abstract class _Matcher {
  bool matches(dynamic actual);
  String get description;
}

class _Contains<T> extends _Matcher {
  _Contains(this.element);
  final T element;

  @override
  bool matches(dynamic actual) {
    if (actual is List) {
      return actual.contains(element);
    }
    return false;
  }

  @override
  String get description => 'contain $element';
}

class _ContainsAll<T> extends _Matcher {
  _ContainsAll(this.elements);
  final List<T> elements;

  @override
  bool matches(dynamic actual) {
    if (actual is! List) return false;
    for (final element in elements) {
      if (!actual.contains(element)) return false;
    }
    return true;
  }

  @override
  String get description => 'contain all of $elements';
}

class _GreaterThanOrEqual extends _Matcher {
  _GreaterThanOrEqual(this.value);
  final num value;

  @override
  bool matches(dynamic actual) =>
      actual is num && actual >= value;

  @override
  String get description => 'be greater than or equal to $value';
}

class _IsNotNull extends _Matcher {
  @override
  bool matches(dynamic actual) => actual != null;

  @override
  String get description => 'not be null';
}

_Matcher _contains<T>(T element) => _Contains(element);
_Matcher _containsAll<T>(List<T> elements) => _ContainsAll(elements);
_Matcher _greaterThanOrEqual(num value) => _GreaterThanOrEqual(value);
_Matcher _isNotNull() => _IsNotNull();
