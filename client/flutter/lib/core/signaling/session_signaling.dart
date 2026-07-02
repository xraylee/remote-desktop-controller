// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'websocket_client.dart';

/// Minimal signaling surface needed by session initiation.
abstract interface class SessionSignaling {
  String get deviceCode;
  WsConnectionState get currentConnectionState;

  Future<void> connect();

  void requestConnection(String targetCode, {String? inviteCode});
}
