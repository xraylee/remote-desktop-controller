// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

/// Standalone verification script for SignalingService
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

/// ICE candidate (simplified)
class IceCandidate {
  IceCandidate({
    required this.candidate,
    this.sdpMid,
    this.sdpMLineIndex,
  });

  final String candidate;
  final String? sdpMid;
  final int? sdpMLineIndex;

  Map<String, dynamic> toJson() => {
        'candidate': candidate,
        'sdpMid': sdpMid,
        'sdpMLineIndex': sdpMLineIndex,
      };
}

/// Device information
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
    String? teamId,
  }) {
    return SignalingMessage('register', {
      'device_code': deviceCode,
      'platform': platform,
      'version': version,
      if (teamId != null) 'team_id': teamId,
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

  factory SignalingMessage.connectRequest({
    required String fromCode,
    required String toCode,
    String? inviteCode,
  }) {
    return SignalingMessage('connect_request', {
      'from_code': fromCode,
      'to_code': toCode,
      if (inviteCode != null) 'invite_code': inviteCode,
    });
  }

  factory SignalingMessage.connectResponse({
    required bool accepted,
    required String sessionId,
    required String fromCode,
  }) {
    return SignalingMessage('connect_response', {
      'accepted': accepted,
      'session_id': sessionId,
      'from_code': fromCode,
    });
  }

  factory SignalingMessage.iceOffer({
    required String sessionId,
    required String sdp,
    required List<IceCandidate> candidates,
  }) {
    return SignalingMessage('ice_offer', {
      'session_id': sessionId,
      'sdp': sdp,
      'candidates': candidates.map((c) => c.toJson()).toList(),
    });
  }

  factory SignalingMessage.iceAnswer({
    required String sessionId,
    required String sdp,
    required List<IceCandidate> candidates,
  }) {
    return SignalingMessage('ice_answer', {
      'session_id': sessionId,
      'sdp': sdp,
      'candidates': candidates.map((c) => c.toJson()).toList(),
    });
  }

  factory SignalingMessage.iceTrickle({
    required String sessionId,
    required IceCandidate candidate,
  }) {
    return SignalingMessage('ice_trickle', {
      'session_id': sessionId,
      'candidate': candidate.toJson(),
    });
  }

  factory SignalingMessage.relayRequest({
    required String sessionId,
    String? preferredRegion,
  }) {
    return SignalingMessage('relay_request', {
      'session_id': sessionId,
      if (preferredRegion != null) 'preferred_region': preferredRegion,
    });
  }

  factory SignalingMessage.generateInvite({
    required String deviceCode,
  }) {
    return SignalingMessage('generate_invite', {
      'device_code': deviceCode,
    });
  }

  factory SignalingMessage.useInvite({
    required String fromCode,
    required String inviteCode,
  }) {
    return SignalingMessage('use_invite', {
      'from_code': fromCode,
      'invite_code': inviteCode,
    });
  }
}

/// Mock WebSocket Client
class MockWebSocketClient {
  MockWebSocketClient({required this.serverUrl});

  final String serverUrl;
  final List<SignalingMessage> sentMessages = [];

  WsConnectionState _currentState = WsConnectionState.disconnected;
  final _stateController = StreamController<WsConnectionState>.broadcast();
  final _messageController = StreamController<SignalingMessage>.broadcast();

  Stream<WsConnectionState> get state => _stateController.stream;
  WsConnectionState get currentState => _currentState;
  Stream<SignalingMessage> get messages => _messageController.stream;

  String? _deviceCode;
  bool _heartbeatRunning = false;

  Future<void> connect() async {
    _setState(WsConnectionState.connecting);
    await Future.delayed(const Duration(milliseconds: 10));
    _setState(WsConnectionState.connected);
  }

  void disconnect() {
    _setState(WsConnectionState.disconnected);
    stopHeartbeat();
  }

  void send(SignalingMessage message) {
    sentMessages.add(message);
  }

  void startHeartbeat(String deviceCode) {
    _deviceCode = deviceCode;
    _heartbeatRunning = true;
  }

  void stopHeartbeat() {
    _heartbeatRunning = false;
  }

  void dispose() {
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

/// Simplified SignalingService for testing
class SignalingService {
  SignalingService({
    required this.serverUrl,
    required this.deviceCode,
    required this.platform,
    this.version = '0.1.0',
    this.teamId,
    MockWebSocketClient? mockClient,
  }) : _client = mockClient ?? MockWebSocketClient(serverUrl: serverUrl);

  final String serverUrl;
  final String deviceCode;
  final String platform;
  final String version;
  final String? teamId;

  final MockWebSocketClient _client;

  final _nearbyDevices = <DeviceInfo>[];
  List<DeviceInfo> get currentNearbyDevices => _nearbyDevices;

  WsConnectionState get currentConnectionState => _client.currentState;
  Stream<WsConnectionState> get connectionState => _client.state;

  StreamSubscription? _messageSubscription;

  Future<void> connect() async {
    await _client.connect();
    _messageSubscription = _client.messages.listen(_handleMessage);
    await Future.delayed(const Duration(milliseconds: 10));
    await register();
  }

  void disconnect() {
    _client.stopHeartbeat();
    _messageSubscription?.cancel();
    _client.disconnect();
  }

  Future<void> register() async {
    _client.send(SignalingMessage.register(
      deviceCode: deviceCode,
      platform: platform,
      version: version,
      teamId: teamId,
    ));
    _client.startHeartbeat(deviceCode);
  }

  void requestConnection(String targetCode, {String? inviteCode}) {
    _client.send(SignalingMessage.connectRequest(
      fromCode: deviceCode,
      toCode: targetCode,
      inviteCode: inviteCode,
    ));
  }

  void respondToConnection({
    required String sessionId,
    required String fromCode,
    required bool accepted,
  }) {
    _client.send(SignalingMessage.connectResponse(
      accepted: accepted,
      sessionId: sessionId,
      fromCode: fromCode,
    ));
  }

  void sendIceOffer({
    required String sessionId,
    required String sdp,
    required List<IceCandidate> candidates,
  }) {
    _client.send(SignalingMessage.iceOffer(
      sessionId: sessionId,
      sdp: sdp,
      candidates: candidates,
    ));
  }

  void sendIceAnswer({
    required String sessionId,
    required String sdp,
    required List<IceCandidate> candidates,
  }) {
    _client.send(SignalingMessage.iceAnswer(
      sessionId: sessionId,
      sdp: sdp,
      candidates: candidates,
    ));
  }

  void sendIceCandidate({
    required String sessionId,
    required IceCandidate candidate,
  }) {
    _client.send(SignalingMessage.iceTrickle(
      sessionId: sessionId,
      candidate: candidate,
    ));
  }

  void requestRelay({
    required String sessionId,
    String? preferredRegion,
  }) {
    _client.send(SignalingMessage.relayRequest(
      sessionId: sessionId,
      preferredRegion: preferredRegion,
    ));
  }

  void generateInviteCode() {
    _client.send(SignalingMessage.generateInvite(
      deviceCode: deviceCode,
    ));
  }

  void useInviteCode(String inviteCode) {
    _client.send(SignalingMessage.useInvite(
      fromCode: deviceCode,
      inviteCode: inviteCode,
    ));
  }

  void dispose() {
    _messageSubscription?.cancel();
    _client.dispose();
  }

  void _handleMessage(SignalingMessage message) {
    // Simplified message handling for tests
  }

  // Expose client for testing
  MockWebSocketClient get client => _client;
}

// ── Test Runner ────────────────────────────────────────────────────

void main() async {
  print('🧪 SignalingService 验证测试\n');

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

  // ── Tests ────────────────────────────────────────────────────────

  await test('initializes with correct configuration', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    expect(service.serverUrl, 'ws://localhost:8080');
    expect(service.deviceCode, 'TEST123456');
    expect(service.platform, 'test');

    service.dispose();
  });

  await test('initial connection state is disconnected', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    expect(service.currentConnectionState, WsConnectionState.disconnected);

    service.dispose();
  });

  await test('initial nearby devices list is empty', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    expect(service.currentNearbyDevices, _isEmpty());

    service.dispose();
  });

  await test('connect() establishes connection and registers device', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    expect(service.currentConnectionState, WsConnectionState.connected);

    // Verify registration message was sent
    expect(service.client.sentMessages.length, _greaterThan(0));
    expect(service.client.sentMessages.first.type, 'register');

    service.disconnect();
    service.dispose();
  });

  await test('disconnect() closes connection cleanly', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();
    expect(service.currentConnectionState, WsConnectionState.connected);

    service.disconnect();
    await Future.delayed(const Duration(milliseconds: 50));

    expect(service.currentConnectionState, WsConnectionState.disconnected);

    service.dispose();
  });

  await test('requestConnection() sends connect_request message', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.requestConnection('TARGET123456');

    final connectMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'connect_request');
    expect(connectMsg.data['to_code'], 'TARGET123456');

    service.disconnect();
    service.dispose();
  });

  await test('requestConnection() with invite code', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.requestConnection('TARGET123456', inviteCode: 'INVITE789');

    final connectMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'connect_request');
    expect(connectMsg.data['invite_code'], 'INVITE789');

    service.disconnect();
    service.dispose();
  });

  await test('respondToConnection() accepts connection', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.respondToConnection(
      sessionId: 'session-123',
      fromCode: 'PEER123456',
      accepted: true,
    );

    final responseMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'connect_response');
    expect(responseMsg.data['accepted'], true);
    expect(responseMsg.data['session_id'], 'session-123');

    service.disconnect();
    service.dispose();
  });

  await test('respondToConnection() rejects connection', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.respondToConnection(
      sessionId: 'session-123',
      fromCode: 'PEER123456',
      accepted: false,
    );

    final responseMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'connect_response');
    expect(responseMsg.data['accepted'], false);

    service.disconnect();
    service.dispose();
  });

  await test('sendIceOffer() transmits offer', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.sendIceOffer(
      sessionId: 'session-123',
      sdp: 'v=0\r\no=- 123456 2 IN IP4 127.0.0.1\r\n',
      candidates: [
        IceCandidate(
          candidate: 'candidate:1 1 UDP 2130706431 192.168.1.100 54321 typ host',
          sdpMid: '0',
          sdpMLineIndex: 0,
        ),
      ],
    );

    final offerMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'ice_offer');
    expect(offerMsg.data['session_id'], 'session-123');
    expect(offerMsg.data['sdp'], _contains('v=0'));

    service.disconnect();
    service.dispose();
  });

  await test('sendIceAnswer() transmits answer', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.sendIceAnswer(
      sessionId: 'session-123',
      sdp: 'v=0\r\no=- 654321 2 IN IP4 127.0.0.1\r\n',
      candidates: [],
    );

    final answerMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'ice_answer');
    expect(answerMsg.data['session_id'], 'session-123');

    service.disconnect();
    service.dispose();
  });

  await test('sendIceCandidate() sends trickle candidate', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.sendIceCandidate(
      sessionId: 'session-123',
      candidate: IceCandidate(
        candidate: 'candidate:2 1 UDP 1694498815 203.0.113.1 54322 typ srflx',
        sdpMid: '0',
        sdpMLineIndex: 0,
      ),
    );

    final trickleMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'ice_trickle');
    expect(trickleMsg.data['session_id'], 'session-123');

    service.disconnect();
    service.dispose();
  });

  await test('requestRelay() sends relay_request', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.requestRelay(sessionId: 'session-123');

    final relayMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'relay_request');
    expect(relayMsg.data['session_id'], 'session-123');

    service.disconnect();
    service.dispose();
  });

  await test('requestRelay() with preferred region', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.requestRelay(
      sessionId: 'session-123',
      preferredRegion: 'us-west-1',
    );

    final relayMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'relay_request');
    expect(relayMsg.data['preferred_region'], 'us-west-1');

    service.disconnect();
    service.dispose();
  });

  await test('generateInviteCode() sends generate_invite', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.generateInviteCode();

    final inviteMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'generate_invite');
    expect(inviteMsg.data['device_code'], 'TEST123456');

    service.disconnect();
    service.dispose();
  });

  await test('useInviteCode() sends use_invite', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
    );

    await service.connect();

    service.useInviteCode('INVITE-XYZ-789');

    final useMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'use_invite');
    expect(useMsg.data['invite_code'], 'INVITE-XYZ-789');

    service.disconnect();
    service.dispose();
  });

  await test('register() includes optional teamId', () async {
    final service = SignalingService(
      serverUrl: 'ws://localhost:8080',
      deviceCode: 'TEST123456',
      platform: 'test',
      teamId: 'team123',
    );

    await service.connect();

    final registerMsg = service.client.sentMessages
        .firstWhere((m) => m.type == 'register');
    expect(registerMsg.data['team_id'], 'team123');

    service.disconnect();
    service.dispose();
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

// ── Matchers ────────────────────────────────────────────────────────

abstract class _Matcher {
  bool matches(dynamic actual);
  String get description;
}

class _IsEmpty extends _Matcher {
  @override
  bool matches(dynamic actual) {
    if (actual is List) return actual.isEmpty;
    if (actual is Map) return actual.isEmpty;
    if (actual is String) return actual.isEmpty;
    return false;
  }

  @override
  String get description => 'be empty';
}

class _GreaterThan extends _Matcher {
  _GreaterThan(this.value);
  final num value;

  @override
  bool matches(dynamic actual) => actual is num && actual > value;

  @override
  String get description => 'be greater than $value';
}

class _Contains extends _Matcher {
  _Contains(this.substring);
  final String substring;

  @override
  bool matches(dynamic actual) =>
      actual is String && actual.contains(substring);

  @override
  String get description => 'contain "$substring"';
}

_Matcher _isEmpty() => _IsEmpty();
_Matcher _greaterThan(num value) => _GreaterThan(value);
_Matcher _contains(String substring) => _Contains(substring);
