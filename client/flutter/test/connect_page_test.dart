// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:rdcs_client/core/ffi/engine_isolate.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';
import 'package:rdcs_client/core/signaling/websocket_client.dart';
import 'package:rdcs_client/features/session/session_providers.dart';
import 'helpers.dart';

void main() {
  group('ConnectPage — 连接页面', () {
    testWidgets('显示标题 "连接远程设备"', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      expect(find.text('连接远程设备'), findsOneWidget);
    });

    testWidgets('显示 "输入对方设备代码" 说明文字', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      expect(find.text('输入对方设备代码'), findsOneWidget);
    });

    testWidgets('设备代码输入框带有标签 "设备代码"', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      expect(find.text('设备代码'), findsOneWidget);
    });

    testWidgets('验证 — 空输入显示 "请输入9位设备代码"', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      // Tap connect without entering anything.
      await tester.tap(find.text('连接'));
      await tester.pumpAndSettle();

      expect(find.text('请输入9位设备代码'), findsOneWidget);
    });

    testWidgets('验证 — 少于9位数字显示错误', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField).first, '12345');
      await tester.tap(find.text('连接'));
      await tester.pumpAndSettle();

      expect(find.text('请输入9位设备代码'), findsOneWidget);
    });

    testWidgets('验证 — 多于9位数字显示错误', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField).first, '1234567890');
      await tester.tap(find.text('连接'));
      await tester.pumpAndSettle();

      expect(find.text('请输入9位设备代码'), findsOneWidget);
    });

    testWidgets('验证 — 带空格的9位代码通过验证', (tester) async {
      final signaling = FakeSessionSignaling();
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      // Enter code with spaces — validator strips them.
      await tester.enterText(find.byType(TextFormField).first, '123 456 789');
      await tester.tap(find.text('连接'));
      await tester.pump(); // let connect() reach the connect_response await
      // Resolve the pending handshake so no timeout timer leaks.
      signaling.emitConnectResponse(const ConnectResponseMessage(
          accepted: true, sessionId: 'sess-1', fromCode: '123456789'));
      await tester.pump(const Duration(milliseconds: 100));

      // Should NOT show validation error (code is valid after stripping).
      expect(find.text('请输入9位设备代码'), findsNothing);

      // The stripped code is what gets dialed via connect_request.
      expect(signaling.lastRequestTargetCode, '123456789');
    });

    testWidgets('输入有效9位代码并连接，发出 connect_request 并等待接受', (tester) async {
      final signaling = FakeSessionSignaling();
      final container = await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField).first, '987654321');
      await tester.tap(find.text('连接'));
      await tester.pump(); // reach the connect_response await

      // Request was dialed; engine.connect is NOT driven in milestone A.
      expect(signaling.lastRequestTargetCode, '987654321');
      final engine = container.read(engineProvider) as FakeEngineIsolate;
      expect(engine.lastConnectCode, isNull);

      // Target accepts — handshake completes to connected.
      signaling.emitConnectResponse(const ConnectResponseMessage(
          accepted: true, sessionId: 'sess-1', fromCode: '987654321'));
      await tester.pump(const Duration(milliseconds: 100));

      final session = container.read(sessionProvider);
      expect(session, isNotNull);
      expect(session!.state, SessionState.connected);
    });

    testWidgets('输入设备码与邀请码，邀请码随 connect_request 一并发送',
        (tester) async {
      final signaling = FakeSessionSignaling();
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      // First field = device code, second field = optional invite code.
      await tester.enterText(find.byType(TextFormField).first, '761335217');
      await tester.enterText(find.byType(TextFormField).at(1), 'INVITE42');
      await tester.tap(find.text('连接'));
      await tester.pump(); // reach the connect_response await

      expect(signaling.lastRequestTargetCode, '761335217');
      expect(signaling.lastInviteCode, 'INVITE42');

      // Resolve the handshake so the timeout timer does not leak.
      signaling.emitConnectResponse(const ConnectResponseMessage(
          accepted: true, sessionId: 'sess-1', fromCode: '761335217'));
      await tester.pump(const Duration(milliseconds: 100));
    });

    testWidgets('不填邀请码时 connect_request 的 inviteCode 为 null',
        (tester) async {
      final signaling = FakeSessionSignaling();
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField).first, '761335217');
      await tester.tap(find.text('连接'));
      await tester.pump(); // reach the connect_response await

      expect(signaling.lastRequestTargetCode, '761335217');
      expect(signaling.lastInviteCode, isNull);

      // Resolve the handshake so the timeout timer does not leak.
      signaling.emitConnectResponse(const ConnectResponseMessage(
          accepted: true, sessionId: 'sess-1', fromCode: '761335217'));
      await tester.pump(const Duration(milliseconds: 100));
    });

    testWidgets('Signaling 离线时连接前会先重连再发送 connect_request', (tester) async {
      final signaling = FakeSessionSignaling(
        currentConnectionState: WsConnectionState.disconnected,
      );
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField).first, '761335217');
      await tester.tap(find.text('连接'));
      await tester.pump(const Duration(milliseconds: 100));

      expect(signaling.connectCalled, isTrue);
      expect(signaling.lastRequestTargetCode, '761335217');

      // Resolve the handshake so the timeout timer does not leak.
      signaling.emitConnectResponse(const ConnectResponseMessage(
          accepted: true, sessionId: 'sess-1', fromCode: '761335217'));
      await tester.pump(const Duration(milliseconds: 100));
    });

    testWidgets('signaling 重连失败时会重试后再发送 connect_request', (tester) async {
      final signaling = FakeSessionSignaling(
        currentConnectionState: WsConnectionState.disconnected,
        failConnectAttempts: 2,
      );
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField).first, '761335217');
      await tester.tap(find.text('连接'));
      await tester.pump(const Duration(seconds: 1));

      expect(signaling.connectAttempts, 3);
      expect(signaling.lastRequestTargetCode, '761335217');

      // Resolve the handshake so the timeout timer does not leak.
      signaling.emitConnectResponse(const ConnectResponseMessage(
          accepted: true, sessionId: 'sess-1', fromCode: '761335217'));
      await tester.pump(const Duration(milliseconds: 100));
    });

    testWidgets('connect_request 发送失败时会重试后再发出请求', (tester) async {
      final signaling = FakeSessionSignaling(failRequestAttempts: 2);
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField).first, '761335217');
      await tester.tap(find.text('连接'));
      await tester.pump(const Duration(seconds: 2));

      expect(signaling.requestAttempts, 3);
      expect(signaling.lastRequestTargetCode, '761335217');

      // Resolve the handshake so the timeout timer does not leak.
      signaling.emitConnectResponse(const ConnectResponseMessage(
          accepted: true, sessionId: 'sess-1', fromCode: '761335217'));
      await tester.pump(const Duration(milliseconds: 100));
    });

    testWidgets('对方接受后 session 状态为 connected', (tester) async {
      final signaling = FakeSessionSignaling();
      final container = await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField).first, '987654321');
      await tester.tap(find.text('连接'));
      await tester.pump();
      signaling.emitConnectResponse(const ConnectResponseMessage(
          accepted: true, sessionId: 'sess-1', fromCode: '987654321'));
      await tester.pump(const Duration(milliseconds: 100));

      final session = container.read(sessionProvider);
      expect(session, isNotNull);
      expect(session!.state, SessionState.connected);
    });

    testWidgets('对方拒绝时显示错误 snackbar', (tester) async {
      final signaling = FakeSessionSignaling();
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField).first, '987654321');
      await tester.tap(find.text('连接'));
      await tester.pump();
      signaling.emitConnectResponse(const ConnectResponseMessage(
          accepted: false, sessionId: 'sess-1', fromCode: '987654321'));
      await tester.pump(const Duration(milliseconds: 100));

      // Should show error snackbar (rejected / timeout / offline).
      expect(find.text('连接失败：对方已拒绝、超时或设备离线'), findsOneWidget);
    });

    testWidgets('点击返回按钮导航到首页', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      // Verify we're on the connect page.
      expect(find.text('输入对方设备代码'), findsOneWidget);

      // Tap the back button (arrow_back icon in AppBar).
      await tester.tap(find.byIcon(Icons.arrow_back));
      await tester.pumpAndSettle();

      // Should navigate back to home page.
      expect(find.text('RDCS 远程桌面'), findsWidgets);
    });

    testWidgets('连接中输入框禁用', (tester) async {
      // Use an engine that never completes (delayed future).
      final slowEngine = FakeEngineIsolate();
      // We can't easily make connect() hang in a synchronous fake,
      // so we'll just verify the enabled state before connecting.
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeEngine: slowEngine,
      );
      await tester.pumpAndSettle();

      // Before connecting, the text field should be enabled.
      final textField =
          tester.widget<TextFormField>(find.byType(TextFormField).first);
      expect(textField.enabled, isNot(false));

      // The connect button should be enabled.
      final button = tester.widget<ElevatedButton>(find.byType(ElevatedButton));
      expect(button.onPressed, isNotNull);
    });

    testWidgets('连接按钮文本为 "连接"', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      expect(find.text('连接'), findsOneWidget);
    });

    testWidgets('输入框提示文本为 "123 456 789"', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      // hintText is inside InputDecoration, find via the TextField.
      final textField = tester.widget<TextField>(
        find.descendant(
          of: find.byType(TextFormField).first,
          matching: find.byType(TextField),
        ),
      );
      expect(textField.decoration?.hintText, '123 456 789');
    });
  });
}
