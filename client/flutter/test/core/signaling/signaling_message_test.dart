// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';

void main() {
  group('SignalingMessage JSON protocol', () {
    test('connectRequest serializes type as snake_case', () {
      final json = SignalingMessage.connectRequest(
        fromCode: '871843136',
        toCode: '761335217',
      ).toJson();

      expect(json['type'], 'connect_request');
      expect(json['from_code'], '871843136');
      expect(json['to_code'], '761335217');
    });

    test('useInvite serializes type as snake_case', () {
      final json = SignalingMessage.useInvite(
        fromCode: '871843136',
        inviteCode: 'INVITE789',
      ).toJson();

      expect(json['type'], 'use_invite');
      expect(json['from_code'], '871843136');
      expect(json['invite_code'], 'INVITE789');
    });

    test('error message deserializes and routes via when() without cast error',
        () {
      // Regression: the server's error reply must survive when()-based routing.
      // Previously `error: (code, message) => message as ErrorMessage` crashed
      // because when() passes the field String, not the union object.
      final msg = SignalingMessage.fromJson({
        'type': 'error',
        'code': 'device_offline',
        'message': 'device 761335217 is offline',
      });

      final routed = msg.when(
        register: (_, __, ___, ____) => 'register',
        heartbeat: (_, __) => 'heartbeat',
        connectRequest: (_, __, ___, ____) => 'connect_request',
        connectResponse: (_, __, ___) => 'connect_response',
        iceOffer: (_, __, ___) => 'ice_offer',
        iceAnswer: (_, __, ___) => 'ice_answer',
        iceTrickle: (_, __) => 'ice_trickle',
        relayRequest: (_, __) => 'relay_request',
        relayAssigned: (_, __, ___, ____) => 'relay_assigned',
        peerOffline: (_, __) => 'peer_offline',
        nearbyUpdate: (_) => 'nearby_update',
        generateInvite: (_) => 'generate_invite',
        useInvite: (_, __) => 'use_invite',
        inviteGenerated: (_) => 'invite_generated',
        inviteResult: (_, __) => 'invite_result',
        // The callback receives the code + message String fields directly.
        error: (code, message) => 'error:$code:$message',
      );

      expect(routed, 'error:device_offline:device 761335217 is offline');
    });

    test('connect_request with session_id round-trips', () {
      const msg = SignalingMessage.connectRequest(
        fromCode: '761335217',
        toCode: '123456789',
        sessionId: 'sess-xyz',
        inviteCode: null,
      );
      final json = msg.toJson();
      expect(json['session_id'], 'sess-xyz');
      final parsed = SignalingMessage.fromJson(json);
      expect(parsed, msg);
    });

    test('connect_request without session_id has null session_id', () {
      const msg = SignalingMessage.connectRequest(
        fromCode: '761335217',
        toCode: '123456789',
      );
      final json = msg.toJson();
      expect(json['session_id'], isNull);
    });
  });
}
