// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';

import 'package:rxdart/rxdart.dart';

import 'models/signaling_message.dart';
import 'session_signaling.dart';
import 'websocket_client.dart';

/// Signaling service for device registration, connection negotiation,
/// and session management.
///
/// This is the main interface for the rest of the application to interact
/// with the signaling server. It wraps [WebSocketClient] and provides
/// high-level methods for common operations.
class SignalingService implements SessionSignaling {
  SignalingService({
    required this.serverUrl,
    required this.deviceCode,
    required this.platform,
    this.version = '0.1.0',
    this.teamId,
  }) : _client = WebSocketClient(serverUrl: serverUrl);

  final String serverUrl;
  final String deviceCode;
  final String platform;
  final String version;
  final String? teamId;

  final WebSocketClient _client;

  /// Nearby devices (updated by nearby_update messages).
  final _nearbyDevicesController = BehaviorSubject<List<DeviceInfo>>.seeded([]);
  Stream<List<DeviceInfo>> get nearbyDevices => _nearbyDevicesController.stream;
  List<DeviceInfo> get currentNearbyDevices => _nearbyDevicesController.value;

  /// Connection state from the underlying WebSocket client.
  Stream<WsConnectionState> get connectionState => _client.state;
  WsConnectionState get currentConnectionState => _client.currentState;

  /// Incoming messages stream (for custom handling).
  Stream<SignalingMessage> get messages => _client.messages;

  /// Session invitations (connect_request from other devices).
  final _invitationsController = StreamController<ConnectRequestMessage>.broadcast();
  Stream<ConnectRequestMessage> get invitations => _invitationsController.stream;

  /// Generated invite codes.
  final _inviteGeneratedController = StreamController<String>.broadcast();
  Stream<String> get inviteGenerated => _inviteGeneratedController.stream;

  /// Relay server assignments.
  final _relayAssignedController = StreamController<RelayAssignedMessage>.broadcast();
  Stream<RelayAssignedMessage> get relayAssigned => _relayAssignedController.stream;

  /// Error messages from the server.
  final _errorsController = StreamController<ErrorMessage>.broadcast();
  Stream<ErrorMessage> get errors => _errorsController.stream;

  StreamSubscription? _messageSubscription;

  // ── Public API ─────────────────────────────────────────────────

  /// Connect to the signaling server and register this device.
  Future<void> connect() async {
    await _client.connect();

    // Listen for incoming messages and route them
    _messageSubscription = _client.messages.listen(_handleMessage);

    // Wait a moment for connection to stabilize
    await Future.delayed(const Duration(milliseconds: 200));

    // Register device
    await register();
  }

  /// Disconnect from the signaling server.
  void disconnect() {
    _client.stopHeartbeat();
    _messageSubscription?.cancel();
    _client.disconnect();
  }

  /// Register this device with the signaling server.
  Future<void> register() async {
    _client.send(SignalingMessage.register(
      deviceCode: deviceCode,
      platform: platform,
      version: version,
      teamId: teamId,
    ));

    // Start heartbeat after registration
    _client.startHeartbeat(deviceCode);

    print('📝 Device registered: $deviceCode');
  }

  /// Request a connection to another device.
  void requestConnection(String targetCode, {String? inviteCode}) {
    _client.send(SignalingMessage.connectRequest(
      fromCode: deviceCode,
      toCode: targetCode,
      inviteCode: inviteCode,
    ));

    print('📞 Connection requested to $targetCode');
  }

  /// Accept or reject a connection request.
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

    print('${accepted ? "✅" : "❌"} Connection ${accepted ? "accepted" : "rejected"}: $sessionId');
  }

  /// Send ICE offer to the peer.
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

    print('🧊 ICE offer sent for session $sessionId');
  }

  /// Send ICE answer to the peer.
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

    print('🧊 ICE answer sent for session $sessionId');
  }

  /// Send trickle ICE candidate to the peer.
  void sendIceCandidate({
    required String sessionId,
    required IceCandidate candidate,
  }) {
    _client.send(SignalingMessage.iceTrickle(
      sessionId: sessionId,
      candidate: candidate,
    ));
  }

  /// Request a relay server for the session.
  void requestRelay({
    required String sessionId,
    String? preferredRegion,
  }) {
    _client.send(SignalingMessage.relayRequest(
      sessionId: sessionId,
      preferredRegion: preferredRegion,
    ));

    print('🌐 Relay server requested for session $sessionId');
  }

  /// Generate an invite code for this device.
  void generateInviteCode() {
    _client.send(SignalingMessage.generateInvite(
      deviceCode: deviceCode,
    ));

    print('🎫 Invite code generation requested');
  }

  /// Use an invite code to connect to a device.
  void useInviteCode(String inviteCode) {
    _client.send(SignalingMessage.useInvite(
      fromCode: deviceCode,
      inviteCode: inviteCode,
    ));

    print('🎫 Using invite code: $inviteCode');
  }

  /// Dispose resources.
  void dispose() {
    _messageSubscription?.cancel();
    _nearbyDevicesController.close();
    _invitationsController.close();
    _inviteGeneratedController.close();
    _relayAssignedController.close();
    _errorsController.close();
    _client.dispose();
  }

  // ── Private Methods ────────────────────────────────────────────

  /// Route incoming messages to appropriate handlers.
  void _handleMessage(SignalingMessage message) {
    message.when(
      // Server → Client messages
      nearbyUpdate: (devices) {
        _nearbyDevicesController.add(devices);
        print('👥 Nearby devices updated: ${devices.length} device(s)');
      },
      peerOffline: (deviceCode, reason) {
        // Remove device from nearby list
        final updated = currentNearbyDevices
            .where((d) => d.code != deviceCode)
            .toList();
        _nearbyDevicesController.add(updated);
        print('📴 Device offline: $deviceCode ($reason)');
      },
      relayAssigned: (sessionId, relayAddr, relayPort, token) {
        _relayAssignedController.add(message as RelayAssignedMessage);
        print('🌐 Relay assigned: $relayAddr:$relayPort');
      },
      inviteGenerated: (inviteCode) {
        _inviteGeneratedController.add(inviteCode);
        print('🎫 Invite code generated: $inviteCode');
      },
      inviteResult: (sessionId, toCode) {
        print('🎫 Invite accepted: session $sessionId with $toCode');
      },
      error: (code, message) {
        _errorsController.add(message as ErrorMessage);
        print('❌ Server error [$code]: $message');
      },

      // Client → Server messages (should not receive, but handle gracefully)
      register: (_, __, ___, ____) => _logUnexpected('register'),
      heartbeat: (_, __) => _logUnexpected('heartbeat'),
      connectRequest: (fromCode, toCode, inviteCode) {
        // This is an incoming connection request
        _invitationsController.add(message as ConnectRequestMessage);
        print('📞 Connection request from $fromCode');
      },
      connectResponse: (_, __, ___) => _logUnexpected('connect_response'),
      iceOffer: (_, __, ___) => _logUnexpected('ice_offer'),
      iceAnswer: (_, __, ___) => _logUnexpected('ice_answer'),
      iceTrickle: (_, __) => _logUnexpected('ice_trickle'),
      relayRequest: (_, __) => _logUnexpected('relay_request'),
      generateInvite: (_) => _logUnexpected('generate_invite'),
      useInvite: (_, __) => _logUnexpected('use_invite'),
    );
  }

  void _logUnexpected(String messageType) {
    print('⚠️  Unexpected message type from server: $messageType');
  }
}
