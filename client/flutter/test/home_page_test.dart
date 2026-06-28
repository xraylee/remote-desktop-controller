// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'helpers.dart';

void main() {
  group('HomePage — 首页', () {
    testWidgets('显示格式化的设备代码 (XXX-XXX-XXX)', (tester) async {
      await pumpTestApp(
        tester,
        config: const RdcsConfig(deviceCode: '123456789'),
      );
      await tester.pumpAndSettle();

      // The device code should be formatted with dashes.
      expect(find.text('123-456-789'), findsOneWidget);
    });

    testWidgets('设备代码为空时显示占位符 --- --- ---', (tester) async {
      await pumpTestApp(
        tester,
        config: const RdcsConfig(deviceCode: ''),
      );
      await tester.pumpAndSettle();

      expect(find.text('--- --- ---'), findsOneWidget);
    });

    testWidgets('显示应用标题 "RDCS 远程桌面"', (tester) async {
      await pumpTestApp(tester);
      await tester.pumpAndSettle();

      expect(find.text('RDCS 远程桌面'), findsOneWidget);
    });

    testWidgets('显示 "设备已就绪" 状态（无活跃会话时）', (tester) async {
      await pumpTestApp(tester);
      await tester.pumpAndSettle();

      expect(find.text('设备已就绪'), findsOneWidget);
    });

    testWidgets('点击设备代码复制到剪贴板并显示 snackbar', (tester) async {
      await pumpTestApp(
        tester,
        config: const RdcsConfig(deviceCode: '123456789'),
      );
      await tester.pumpAndSettle();

      // Tap the device code container (finds the GestureDetector wrapping it).
      await tester.tap(find.text('123-456-789'));
      await tester.pump();

      // Verify snackbar appears with the correct message.
      expect(find.text('设备代码已复制到剪贴板'), findsOneWidget);

      // Verify clipboard contents.
      final clipboardData = await Clipboard.getData(Clipboard.kTextPlain);
      expect(clipboardData?.text, '123456789');
    });

    testWidgets('设备代码为空时点击不显示 snackbar', (tester) async {
      await pumpTestApp(
        tester,
        config: const RdcsConfig(deviceCode: ''),
      );
      await tester.pumpAndSettle();

      // Tap the placeholder text.
      await tester.tap(find.text('--- --- ---'));
      await tester.pump();

      // No snackbar should appear.
      expect(find.text('设备代码已复制到剪贴板'), findsNothing);
    });

    testWidgets('有设备代码时显示 "点击代码可复制" 提示', (tester) async {
      await pumpTestApp(
        tester,
        config: const RdcsConfig(deviceCode: '123456789'),
      );
      await tester.pumpAndSettle();

      expect(find.text('点击代码可复制'), findsOneWidget);
    });

    testWidgets('无设备代码时不显示 "点击代码可复制" 提示', (tester) async {
      await pumpTestApp(
        tester,
        config: const RdcsConfig(deviceCode: ''),
      );
      await tester.pumpAndSettle();

      expect(find.text('点击代码可复制'), findsNothing);
    });

    testWidgets('点击 "连接远程设备" 按钮导航到 /connect', (tester) async {
      await pumpTestApp(tester);
      await tester.pumpAndSettle();

      // Tap the connect button.
      await tester.tap(find.text('连接远程设备'));
      await tester.pumpAndSettle();

      // Should navigate to the connect page.
      expect(find.text('连接远程设备'), findsWidgets); // AppBar title on connect page
      expect(find.text('输入对方设备代码'), findsOneWidget);
    });

    testWidgets('有设备代码时显示复制图标', (tester) async {
      await pumpTestApp(
        tester,
        config: const RdcsConfig(deviceCode: '123456789'),
      );
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.copy), findsOneWidget);
    });

    testWidgets('无设备代码时不显示复制图标', (tester) async {
      await pumpTestApp(
        tester,
        config: const RdcsConfig(deviceCode: ''),
      );
      await tester.pumpAndSettle();

      expect(find.byIcon(Icons.copy), findsNothing);
    });

    testWidgets('显示 "生成邀请码" 按钮', (tester) async {
      await pumpTestApp(tester);
      await tester.pumpAndSettle();

      expect(find.text('生成邀请码'), findsOneWidget);
    });

    testWidgets('连接中状态显示 "正在连接..."', (tester) async {
      await pumpTestApp(
        tester,
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.connecting,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('正在连接...'), findsOneWidget);
    });

    testWidgets('已连接状态显示 "已连接"', (tester) async {
      await pumpTestApp(
        tester,
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.connected,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('已连接'), findsOneWidget);
    });

    testWidgets('断开状态显示 "已断开"', (tester) async {
      await pumpTestApp(
        tester,
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.disconnected,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('已断开'), findsOneWidget);
    });

    testWidgets('错误状态显示 "连接失败"', (tester) async {
      await pumpTestApp(
        tester,
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.error,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('连接失败'), findsOneWidget);
    });
  });
}
