// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../config/config_provider.dart';
import 'models/signaling_message.dart';
import 'signaling_service.dart';
import 'websocket_client.dart';

/// Provider for the signaling service singleton.
///
/// Automatically initialized with configuration from [configProvider].
final signalingServiceProvider = Provider<SignalingService>((ref) {
  final config = ref.watch(configProvider);
  final serverUrl = config.server.rendezvousUrl.isNotEmpty
      ? config.server.rendezvousUrl
      : 'ws://127.0.0.1:8443/ws'; // Default for development

  final service = SignalingService(
    serverUrl: serverUrl,
    deviceCode: config.deviceCode,
    platform: _getPlatform(),
    version: '0.1.0',
    teamId: null, // TODO: Add team support
  );

  // Cleanup when provider is disposed
  ref.onDispose(() {
    service.dispose();
  });

  return service;
});

/// Provider for WebSocket connection state.
final connectionStateProvider = StreamProvider<WsConnectionState>((ref) {
  final service = ref.watch(signalingServiceProvider);
  return service.connectionState;
});

/// Provider for nearby devices list.
final nearbyDevicesProvider = StreamProvider<List<DeviceInfo>>((ref) {
  final service = ref.watch(signalingServiceProvider);
  return service.nearbyDevices;
});

/// Provider for incoming connection invitations.
final invitationsProvider = StreamProvider<ConnectRequestMessage>((ref) {
  final service = ref.watch(signalingServiceProvider);
  return service.invitations;
});

/// Provider for server error messages.
final signalingErrorsProvider = StreamProvider<ErrorMessage>((ref) {
  final service = ref.watch(signalingServiceProvider);
  return service.errors;
});

/// Auto-connect manager provider.
///
/// Automatically connects to the signaling server when the app starts
/// and maintains the connection.
final signalingAutoConnectProvider = Provider<SignalingAutoConnect>((ref) {
  final service = ref.watch(signalingServiceProvider);
  final autoConnect = SignalingAutoConnect(service);

  // Start connection
  autoConnect.start();

  // Cleanup
  ref.onDispose(() {
    autoConnect.dispose();
  });

  return autoConnect;
});

/// Helper class to manage automatic connection lifecycle.
class SignalingAutoConnect {
  SignalingAutoConnect(this.service);

  final SignalingService service;
  bool _isDisposed = false;

  /// Start the connection and monitor state.
  void start() {
    _connect();

    // Monitor connection state and reconnect if needed
    service.connectionState.listen((state) {
      if (_isDisposed) return;

      if (state == WsConnectionState.error ||
          state == WsConnectionState.disconnected) {
        // Will auto-reconnect via WebSocketClient's logic
        print('🔄 Connection lost, auto-reconnect will trigger');
      } else if (state == WsConnectionState.connected) {
        print('✅ Signaling connection established');
      }
    });
  }

  Future<void> _connect() async {
    try {
      await service.connect();
    } catch (e) {
      print('❌ Failed to connect to signaling server: $e');
    }
  }

  void dispose() {
    _isDisposed = true;
    service.disconnect();
  }
}

/// Get the current platform identifier.
String _getPlatform() {
  // TODO: Use dart:io Platform.operatingSystem in production
  // For now, return a placeholder
  return 'macos'; // Assuming macOS based on your setup
}
