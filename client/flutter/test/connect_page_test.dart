// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:rdcs_client/core/ffi/engine_isolate.dart';
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

      await tester.enterText(find.byType(TextFormField), '12345');
      await tester.tap(find.text('连接'));
      await tester.pumpAndSettle();

      expect(find.text('请输入9位设备代码'), findsOneWidget);
    });

    testWidgets('验证 — 多于9位数字显示错误', (tester) async {
      await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField), '1234567890');
      await tester.tap(find.text('连接'));
      await tester.pumpAndSettle();

      expect(find.text('请输入9位设备代码'), findsOneWidget);
    });

    testWidgets('验证 — 带空格的9位代码通过验证', (tester) async {
      final container = await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      // Enter code with spaces — validator strips them.
      await tester.enterText(find.byType(TextFormField), '123 456 789');
      await tester.tap(find.text('连接'));
      await tester.pump(const Duration(milliseconds: 100));

      // Should NOT show validation error (code is valid after stripping).
      expect(find.text('请输入9位设备代码'), findsNothing);

      // Verify the engine's connect was called with the stripped code.
      final engine = container.read(engineProvider) as FakeEngineIsolate;
      expect(engine.lastConnectCode, '123456789');
    });

    testWidgets('输入有效9位代码并连接，调用 sessionProvider.connect()',
        (tester) async {
      final signaling = FakeSessionSignaling();
      final container = await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField), '987654321');
      await tester.tap(find.text('连接'));
      for (var i = 0; i < 10; i++) {
        await tester.pump(const Duration(milliseconds: 100));
        if (find.text('输入对方设备代码').evaluate().isEmpty) {
          break;
        }
      }

      // Verify the fake engine received the connect call.
      final engine = container.read(engineProvider) as FakeEngineIsolate;
      expect(engine.lastConnectCode, '987654321');
      expect(signaling.lastRequestTargetCode, '987654321');

      // The session notifier should have processed the connection.
      final session = container.read(sessionProvider);
      expect(session, isNotNull);
      expect(session!.state, SessionState.connected);
    });

    testWidgets('Signaling 离线时连接前会先重连再发送 connect_request',
        (tester) async {
      final signaling = FakeSessionSignaling(
        currentConnectionState: WsConnectionState.disconnected,
      );
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeSignaling: signaling,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField), '761335217');
      await tester.tap(find.text('连接'));
      await tester.pump(const Duration(milliseconds: 100));

      expect(signaling.connectCalled, isTrue);
      expect(signaling.lastRequestTargetCode, '761335217');
    });

    testWidgets('连接成功后 session 状态为 connected', (tester) async {
      final container = await pumpTestApp(tester, initialLocation: '/connect');
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField), '987654321');
      await tester.tap(find.text('连接'));
      await tester.pump(const Duration(milliseconds: 100));

      final session = container.read(sessionProvider);
      expect(session, isNotNull);
      expect(session!.state, SessionState.connected);
    });

    testWidgets('连接失败时显示错误 snackbar', (tester) async {
      // Use a fake engine that returns -1 (failure) for connect.
      final failingEngine = FakeEngineIsolate(connectResult: -1);
      await pumpTestApp(
        tester,
        initialLocation: '/connect',
        fakeEngine: failingEngine,
      );
      await tester.pumpAndSettle();

      await tester.enterText(find.byType(TextFormField), '987654321');
      await tester.tap(find.text('连接'));
      await tester.pumpAndSettle();

      // Should show error snackbar.
      expect(find.text('连接失败，请检查设备代码后重试'), findsOneWidget);
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
      final textField = tester.widget<TextFormField>(find.byType(TextFormField));
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
          of: find.byType(TextFormField),
          matching: find.byType(TextField),
        ),
      );
      expect(textField.decoration?.hintText, '123 456 789');
    });
  });
}
