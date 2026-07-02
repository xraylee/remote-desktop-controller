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
  });
}
