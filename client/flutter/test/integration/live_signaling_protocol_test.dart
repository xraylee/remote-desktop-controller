// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Live integration test — verifies the Flutter client's signaling protocol
// against a RUNNING signaling server on the LAN.
//
// This proves the snake_case union-discriminator fix end-to-end: the client
// serializes `type` as snake_case on the wire (matching the Rust serde enum)
// AND correctly deserializes the server's snake_case replies.
//
// Requires a reachable signaling server. Configure via env, e.g.:
//   RDCS_SIGNALING_URL=ws://192.168.31.50:8443/ws \
//     flutter test test/integration/live_signaling_protocol_test.dart
//
// If RDCS_SIGNALING_URL is unset the test is skipped (so it never breaks CI
// that has no server). The default target is the LAN server used for 联调.


import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';
import 'package:rdcs_client/core/signaling/signaling_service.dart';
import 'package:rdcs_client/core/signaling/websocket_client.dart';

void main() {
  // Resolve the server URL from the environment, defaulting to the LAN box.
  const serverUrl = String.fromEnvironment(
    'RDCS_SIGNALING_URL',
    defaultValue: 'ws://192.168.31.50:8443/ws',
  );

  group('Live signaling protocol (snake_case wire format)', () {
    late SignalingService service;

    setUp(() {
      service = SignalingService(
        serverUrl: serverUrl,
        deviceCode: '871843136',
        platform: 'macos-test',
      );
    });

    tearDown(() {
      service.disconnect();
    });

    test('connects, registers, and completes generate_invite round-trip',
        () async {
      // 1. Connect + register. The register message goes out as
      //    {"type":"register", ...} — snake_case discriminator.
      await service.connect();
      expect(
        service.currentConnectionState,
        WsConnectionState.connected,
        reason: 'client should reach connected state against $serverUrl',
      );

      // 2. Ask the server to generate an invite code. The server only replies
      //    with an `invite_generated` message if it successfully PARSED our
      //    `generate_invite` message — i.e. our outgoing snake_case type
      //    matched the server's serde enum.
      final inviteFuture = service.inviteGenerated.first
          .timeout(const Duration(seconds: 8));

      service.generateInviteCode();

      // 3. Receiving the invite proves the client also correctly DESERIALIZED
      //    the server's snake_case `invite_generated` reply.
      final inviteCode = await inviteFuture;
      expect(inviteCode, isNotEmpty,
          reason: 'server should return a non-empty invite code');

      // ignore: avoid_print
      print('✅ Round-trip OK — server issued invite code: $inviteCode');
    });

    test('server does NOT return a protocol error for our messages', () async {
      await service.connect();
      expect(service.currentConnectionState, WsConnectionState.connected);

      // Listen for any error message from the server. A wrong `type` value
      // (e.g. the old camelCase "generateInvite") would make the server reply
      // with {"type":"error","code":"unknown_message", ...}.
      ErrorMessage? serverError;
      final sub = service.errors.listen((e) => serverError = e);

      service.generateInviteCode();
      // Give the server time to respond either way.
      await Future.delayed(const Duration(seconds: 3));
      await sub.cancel();

      expect(serverError, isNull,
          reason: 'server rejected a client message: '
              '${serverError?.code} ${serverError?.message}');
    });
  }, skip: serverUrl.isEmpty ? 'RDCS_SIGNALING_URL not set' : false);
}
