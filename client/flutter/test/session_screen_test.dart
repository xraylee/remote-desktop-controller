// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:rdcs_client/features/session/session_providers.dart';
import 'helpers.dart';

void main() {
  group('SessionScreen — 会话页面', () {
    // ── Null session (loading) ──────────────────────────────────

    testWidgets('session 为 null 时显示加载动画', (tester) async {
      // No initialSession → sessionProvider is null → loading spinner.
      await pumpTestApp(tester, initialLocation: '/session');
      await tester.pump();

      expect(find.byType(CircularProgressIndicator), findsOneWidget);
    });

    // ── Connecting state ────────────────────────────────────────

    testWidgets('连接中状态显示加载动画和设备代码', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 0,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.connecting,
        ),
      );
      await tester.pump();

      // Should show the connecting view with spinner.
      expect(find.byType(CircularProgressIndicator), findsOneWidget);

      // Device code formatted as XXX-XXX-XXX.
      expect(find.textContaining('987-654-321'), findsOneWidget);
    });

    testWidgets('连接中状态显示返回按钮', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 0,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.connecting,
        ),
      );
      await tester.pump();

      // Back button (arrow_back icon) should be visible in top toolbar.
      expect(find.byIcon(Icons.arrow_back), findsOneWidget);
    });

    // ── Connected state ─────────────────────────────────────────

    testWidgets('已连接状态显示工具栏 — 设备名称', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
          latencyMs: 15,
          fps: 60.0,
          qualityMode: 0,
        ),
      );
      await tester.pump();

      // Device name displayed in top toolbar.
      expect(find.text('办公室电脑'), findsOneWidget);
    });

    testWidgets('已连接状态 — 无设备名称时显示设备代码', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '',
          state: SessionState.connected,
        ),
      );
      await tester.pump();

      // Formatted device code shown when name is empty.
      expect(find.text('987-654-321'), findsOneWidget);
    });

    testWidgets('已连接状态显示延迟指示器', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
          latencyMs: 25,
          fps: 60.0,
        ),
      );
      await tester.pump();

      expect(find.text('25ms'), findsOneWidget);
    });

    testWidgets('已连接状态显示 FPS 指示器', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
          latencyMs: 10,
          fps: 60.0,
        ),
      );
      await tester.pump();

      expect(find.text('60 FPS'), findsOneWidget);
    });

    testWidgets('已连接状态显示画质模式选择器 — 默认 "自动"', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
          qualityMode: 0,
        ),
      );
      await tester.pump();

      expect(find.text('自动'), findsOneWidget);
    });

    testWidgets('画质模式 — "清晰优先" 显示正确', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
          qualityMode: 1,
        ),
      );
      await tester.pump();

      expect(find.text('清晰优先'), findsOneWidget);
    });

    testWidgets('画质模式 — "流畅优先" 显示正确', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
          qualityMode: 2,
        ),
      );
      await tester.pump();

      expect(find.text('流畅优先'), findsOneWidget);
    });

    testWidgets('已连接状态显示底部工具栏按钮', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
        ),
      );
      await tester.pump();

      // Bottom toolbar buttons.
      expect(find.text('键盘'), findsOneWidget);
      expect(find.text('文件传输'), findsOneWidget);
      expect(find.text('消息'), findsOneWidget);
      expect(find.text('隐藏面板'), findsOneWidget);
    });

    testWidgets('已连接状态显示断开连接按钮 (红色)', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
        ),
      );
      await tester.pump();

      // Disconnect button (call_end icon, red).
      expect(find.byIcon(Icons.call_end), findsOneWidget);
    });

    testWidgets('点击断开按钮调用 sessionProvider.disconnect()', (tester) async {
      final container = await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 42,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
        ),
      );
      await tester.pump();

      // Tap the disconnect button.
      await tester.tap(find.byIcon(Icons.call_end));
      await tester.pumpAndSettle();

      // Verify the engine's disconnect was called with the session ID.
      final engine = container.read(engineProvider) as FakeEngineIsolate;
      expect(engine.lastDisconnectSessionId, 42);
    });

    testWidgets('已连接状态显示全屏按钮', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
        ),
      );
      await tester.pump();

      expect(find.byIcon(Icons.fullscreen), findsOneWidget);
    });

    testWidgets('已连接状态显示远程桌面占位区域', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
        ),
      );
      await tester.pump();

      // Video placeholder shows "远程桌面画面" text.
      expect(find.text('远程桌面画面'), findsOneWidget);
    });

    // ── Disconnected state ──────────────────────────────────────

    testWidgets('断开状态显示 "连接已断开" 和返回首页按钮', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.disconnected,
        ),
      );
      // Pump a single frame to render the disconnected view before
      // the listener fires and navigates away.
      await tester.pump();

      expect(find.text('连接已断开'), findsOneWidget);
      expect(find.byIcon(Icons.wifi_off), findsOneWidget);
      expect(find.text('返回首页'), findsOneWidget);
    });

    testWidgets('断开状态触发导航回首页', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.disconnected,
        ),
      );
      // Let the listener navigate back.
      await tester.pumpAndSettle();

      // Should be on the home page now.
      expect(find.text('RDCS 远程桌面'), findsOneWidget);
    });

    // ── Error state ─────────────────────────────────────────────

    testWidgets('错误状态显示 "连接失败" 和设备代码', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 0,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.error,
        ),
      );
      await tester.pump();

      expect(find.text('连接失败'), findsOneWidget);
      expect(find.byIcon(Icons.error_outline), findsOneWidget);
      expect(find.text('返回首页'), findsOneWidget);

      // Should show the formatted device code in the error message.
      expect(find.textContaining('987-654-321'), findsOneWidget);
    });

    testWidgets('错误状态触发导航回首页', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 0,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '987654321',
          state: SessionState.error,
        ),
      );
      await tester.pumpAndSettle();

      expect(find.text('RDCS 远程桌面'), findsOneWidget);
    });

    // ── Background color ────────────────────────────────────────

    testWidgets('会话页面背景为深色', (tester) async {
      await pumpTestApp(
        tester,
        initialLocation: '/session',
        initialSession: const SessionInfo(
          sessionId: 1,
          remoteDeviceCode: '987654321',
          remoteDeviceName: '办公室电脑',
          state: SessionState.connected,
        ),
      );
      await tester.pump();

      final scaffold = tester.widget<Scaffold>(find.byType(Scaffold));
      expect(scaffold.backgroundColor, const Color(0xFF111111));
    });
  });
}
