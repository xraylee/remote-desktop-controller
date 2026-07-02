// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';
import 'package:rdcs_client/features/session/session_providers.dart';

import '../../helpers.dart';

void main() {
  test(
      'connect stays connecting until accepted response arrives, then connected; engine.connect NOT called (AC8)',
      () async {
    final engine = FakeEngineIsolate();
    final signaling = FakeSessionSignaling();
    final notifier = SessionNotifier(engine, signaling);

    // Start connect() but don't await yet — it should be waiting for a response.
    final future = notifier.connect('123456789');

    // Give the request-send loop a chance to run and reach the await.
    await Future<void>.delayed(const Duration(milliseconds: 10));
    expect(notifier.state?.state, SessionState.connecting);
    expect(engine.lastConnectCode, isNull); // engine.connect NOT called yet

    // Target accepts.
    signaling.emitConnectResponse(const ConnectResponseMessage(
        accepted: true, sessionId: 'sess-1', fromCode: '123456789'));
    await future;

    expect(notifier.state?.state, SessionState.connected);
    // Milestone A: media not driven, so engine.connect still NOT called.
    expect(engine.lastConnectCode, isNull);
  });

  test('rejected response -> error state, engine.connect NOT called (AC4)',
      () async {
    final engine = FakeEngineIsolate();
    final signaling = FakeSessionSignaling();
    final notifier = SessionNotifier(engine, signaling);

    final future = notifier.connect('123456789');
    await Future<void>.delayed(const Duration(milliseconds: 10));
    signaling.emitConnectResponse(const ConnectResponseMessage(
        accepted: false, sessionId: 'sess-1', fromCode: '123456789'));
    await future;

    expect(notifier.state?.state, SessionState.error);
    expect(engine.lastConnectCode, isNull);
  });
}
