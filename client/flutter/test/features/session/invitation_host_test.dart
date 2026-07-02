// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';
import 'package:rdcs_client/core/signaling/signaling_provider.dart';
import 'package:rdcs_client/core/signaling/signaling_service.dart';
import 'package:rdcs_client/features/session/invitation_host.dart';

void main() {
  testWidgets(
      'shows accept dialog when an invitation arrives on a non-home page',
      (tester) async {
    final navKey = GlobalKey<NavigatorState>();
    final controller = StreamController<ConnectRequestMessage>.broadcast();
    addTearDown(controller.close);

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          invitationsProvider.overrideWith((ref) => controller.stream),
          // Stand-in service: constructs without connecting. respondToConnection
          // only fires after the dialog closes, which this test does not trigger.
          signalingServiceProvider.overrideWithValue(
            SignalingService(
              serverUrl: 'ws://x',
              deviceCode: 'X',
              platform: 'test',
            ),
          ),
        ],
        child: MaterialApp(
          navigatorKey: navKey,
          home: InvitationHost(
            navigatorKey: navKey,
            child: const Scaffold(body: Text('some-other-page')),
          ),
        ),
      ),
    );

    // Sanity: we are NOT on the home page.
    expect(find.text('some-other-page'), findsOneWidget);

    controller.add(const ConnectRequestMessage(
      fromCode: '761335217',
      toCode: '123456789',
      sessionId: 'sess-1',
    ));
    await tester.pump(); // deliver the stream event
    await tester.pump(const Duration(milliseconds: 50)); // let the dialog open

    expect(find.text('远程连接请求'), findsOneWidget);
    // The requester code is rendered inside a RichText span.
    expect(
      find.textContaining('761335217', findRichText: true),
      findsWidgets,
    );
  });

  testWidgets(
      'accepting the dialog calls respondToConnection with accepted:true',
      (tester) async {
    final navKey = GlobalKey<NavigatorState>();
    final controller = StreamController<ConnectRequestMessage>.broadcast();
    addTearDown(controller.close);
    final fake = _RecordingSignalingService();

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          invitationsProvider.overrideWith((ref) => controller.stream),
          signalingServiceProvider.overrideWithValue(fake),
        ],
        child: MaterialApp(
          navigatorKey: navKey,
          home: InvitationHost(
            navigatorKey: navKey,
            child: const Scaffold(body: Text('some-other-page')),
          ),
        ),
      ),
    );

    controller.add(const ConnectRequestMessage(
      fromCode: '761335217',
      toCode: '123456789',
      sessionId: 'sess-1',
    ));
    await tester.pump(); // deliver the stream event
    await tester.pump(const Duration(milliseconds: 50)); // let the dialog open

    // Tap the accept button ('允许').
    await tester.tap(find.widgetWithText(ElevatedButton, '允许'));
    await tester.pump(); // start the pop
    await tester.pump(const Duration(milliseconds: 300)); // let the dialog close

    expect(fake.accepted, isTrue);
    expect(fake.sessionId, 'sess-1');
    expect(fake.fromCode, '761335217');
  });
}

/// Minimal fake that records the arguments passed to [respondToConnection].
/// All other members are unused by the dialog flow and routed to noSuchMethod.
class _RecordingSignalingService implements SignalingService {
  String? sessionId;
  String? fromCode;
  bool? accepted;

  @override
  void respondToConnection({
    required String sessionId,
    required String fromCode,
    required bool accepted,
  }) {
    this.sessionId = sessionId;
    this.fromCode = fromCode;
    this.accepted = accepted;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      super.noSuchMethod(invocation);
}
