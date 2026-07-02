// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0
//
// Single-process UI end-to-end handshake test — NO live server.
//
// The live two-client test (live_two_client_invite_test.dart) proves the
// service layer against a real server, but requires the network. This test
// closes the same loop entirely in-memory and, crucially, drives it through
// the *real production widgets* on both sides:
//
//   • Initiator: ConnectPage → sessionProvider (SessionNotifier.connect),
//     which awaits a matching connect_response before reaching `connected`.
//   • Acceptor:  InvitationHost → ConnectionConfirmDialog → respondToConnection.
//
// Both halves are wired through one LoopbackSignalingBus (test/helpers.dart),
// which reproduces the two server routing facts the UI depends on:
//   1. connect_request → invitation carrying a server-minted session_id.
//   2. connect_response back to the initiator with from_code rewritten to the
//      responder's own device code (so the initiator's fromCode match works).
//
// This is the offline/CI-runnable proof that the two UI halves interoperate —
// the one thing neither the isolated widget tests nor the live test covered.

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';

import 'package:rdcs_client/core/ffi/engine_isolate.dart';
import 'package:rdcs_client/core/signaling/signaling_provider.dart';
import 'package:rdcs_client/features/connect/connect_page.dart';
import 'package:rdcs_client/features/session/invitation_host.dart';
import 'package:rdcs_client/features/session/session_providers.dart';

import '../helpers.dart';

void main() {
  const initiatorCode = '987654321';
  const acceptorCode = '123456789';

  testWidgets(
      'ConnectPage 发起 → InvitationHost 弹窗接受 → 发起方 session 进入 connected（单进程闭环）',
      (tester) async {
    final bus = LoopbackSignalingBus(
      initiatorCode: initiatorCode,
      acceptorCode: acceptorCode,
      sessionId: 'sess-e2e-1',
    );
    addTearDown(bus.dispose);

    final initiatorSignaling = BusInitiatorSignaling(bus);
    final acceptorService = BusAcceptorSignalingService(bus);
    final engine = FakeEngineIsolate();
    final navKey = GlobalKey<NavigatorState>();

    // Single container hosts BOTH halves of the handshake:
    //   - sessionProvider (initiator) speaks through the bus's initiator view.
    //   - invitationsProvider + signalingServiceProvider (acceptor) speak
    //     through the same bus's acceptor view.
    final container = ProviderContainer(overrides: [
      engineProvider.overrideWithValue(engine),
      sessionProvider.overrideWith((ref) {
        return SessionNotifier(ref.watch(engineProvider), initiatorSignaling);
      }),
      invitationsProvider.overrideWith((ref) => bus.invitations),
      signalingServiceProvider.overrideWithValue(acceptorService),
    ]);
    addTearDown(container.dispose);

    final router = GoRouter(
      navigatorKey: navKey,
      initialLocation: '/connect',
      routes: [
        GoRoute(path: '/', builder: (_, __) => const Scaffold(body: Text('home'))),
        GoRoute(path: '/connect', builder: (_, __) => const ConnectPage()),
      ],
    );

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp.router(
          routerConfig: router,
          // InvitationHost mounts app-wide, exactly as production app.dart does.
          builder: (context, child) => InvitationHost(
            navigatorKey: navKey,
            child: child ?? const SizedBox.shrink(),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    // -- Initiator dials the acceptor -------------------------------------
    await tester.enterText(find.byType(TextFormField).first, acceptorCode);
    await tester.tap(find.text('连接'));
    // Let connect() send the request onto the bus and reach its response await.
    await tester.pump();

    // Still connecting; engine.connect must NOT be driven in milestone A.
    expect(container.read(sessionProvider)?.state, SessionState.connecting);
    expect(engine.lastConnectCode, isNull);

    // -- The invitation must have popped the acceptor dialog --------------
    await tester.pump(const Duration(milliseconds: 100)); // let dialog open
    expect(find.text('远程连接请求'), findsOneWidget);
    // Dialog shows the initiator's code (the requester).
    expect(
      find.textContaining(initiatorCode, findRichText: true),
      findsWidgets,
    );

    // -- Acceptor taps 允许 -----------------------------------------------
    await tester.tap(find.widgetWithText(ElevatedButton, '允许'));
    await tester.pump(); // start dialog pop → respondToConnection routes on bus
    await tester.pump(const Duration(milliseconds: 400)); // dialog close anim

    // The response propagates through the bus back to the initiator, whose
    // connect() completes to `connected`.
    await tester.pump(const Duration(milliseconds: 100));

    final session = container.read(sessionProvider);
    expect(session?.state, SessionState.connected,
        reason: 'initiator must reach connected after the acceptor accepts');
    // Media is Milestone B — the mock engine stays untouched.
    expect(engine.lastConnectCode, isNull);

    // Pin the current body-fromCode semantics: InvitationHost sends the
    // *initiator's* code in the body, which the server (and our bus) ignores
    // in favour of the responder's connection identity. Documented, not fixed.
    expect(acceptorService.lastRespondBodyFromCode, initiatorCode);

    // Success feedback surfaced to the initiator.
    expect(find.text('对方已接受，信令握手成功（画面通道将在后续里程碑接入）'),
        findsOneWidget);
  });

  testWidgets(
      'ConnectPage 发起 → 接受方点击拒绝 → 发起方 session 进入 error（单进程闭环）',
      (tester) async {
    final bus = LoopbackSignalingBus(
      initiatorCode: initiatorCode,
      acceptorCode: acceptorCode,
      sessionId: 'sess-e2e-2',
    );
    addTearDown(bus.dispose);

    final initiatorSignaling = BusInitiatorSignaling(bus);
    final acceptorService = BusAcceptorSignalingService(bus);
    final engine = FakeEngineIsolate();
    final navKey = GlobalKey<NavigatorState>();

    final container = ProviderContainer(overrides: [
      engineProvider.overrideWithValue(engine),
      sessionProvider.overrideWith((ref) {
        return SessionNotifier(ref.watch(engineProvider), initiatorSignaling);
      }),
      invitationsProvider.overrideWith((ref) => bus.invitations),
      signalingServiceProvider.overrideWithValue(acceptorService),
    ]);
    addTearDown(container.dispose);

    final router = GoRouter(
      navigatorKey: navKey,
      initialLocation: '/connect',
      routes: [
        GoRoute(path: '/', builder: (_, __) => const Scaffold(body: Text('home'))),
        GoRoute(path: '/connect', builder: (_, __) => const ConnectPage()),
      ],
    );

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp.router(
          routerConfig: router,
          builder: (context, child) => InvitationHost(
            navigatorKey: navKey,
            child: child ?? const SizedBox.shrink(),
          ),
        ),
      ),
    );
    await tester.pumpAndSettle();

    await tester.enterText(find.byType(TextFormField).first, acceptorCode);
    await tester.tap(find.text('连接'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100)); // dialog open

    // Acceptor rejects.
    await tester.tap(find.widgetWithText(OutlinedButton, '拒绝'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400)); // close anim
    await tester.pump(const Duration(milliseconds: 100)); // propagate response

    expect(container.read(sessionProvider)?.state, SessionState.error,
        reason: 'initiator must reach error after the acceptor rejects');
    expect(find.text('连接失败：对方已拒绝、超时或设备离线'), findsOneWidget);
  });
}
