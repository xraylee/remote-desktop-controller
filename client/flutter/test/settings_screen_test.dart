// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:rdcs_client/core/config/config_model.dart';
import 'package:rdcs_client/core/config/config_provider.dart';
import 'helpers.dart';

void main() {
  group('SettingsScreen — 设置页面', () {
    // ── Tab structure ───────────────────────────────────────────

    testWidgets('显示标题 "设置"', (tester) async {
      await pumpTestApp(tester, initialLocation: '/settings');
      await tester.pumpAndSettle();

      expect(find.text('设置'), findsOneWidget);
    });

    testWidgets('有3个标签页', (tester) async {
      await pumpTestApp(tester, initialLocation: '/settings');
      await tester.pumpAndSettle();

      expect(find.text('安全设置'), findsOneWidget);
      expect(find.text('网络设置'), findsOneWidget);
      expect(find.text('通用设置'), findsOneWidget);
    });

    testWidgets('默认显示安全设置标签', (tester) async {
      await pumpTestApp(tester, initialLocation: '/settings');
      await tester.pumpAndSettle();

      // Security tab content should be visible.
      expect(find.text('修改密码'), findsOneWidget);
    });

    testWidgets('点击返回按钮导航到首页', (tester) async {
      await pumpTestApp(tester, initialLocation: '/settings');
      await tester.pumpAndSettle();

      await tester.tap(find.byIcon(Icons.arrow_back));
      await tester.pumpAndSettle();

      expect(find.text('RDCS 远程桌面'), findsOneWidget);
    });

    // ── Security tab ────────────────────────────────────────────

    group('安全设置标签', () {
      testWidgets('显示密码修改表单', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        expect(find.text('修改密码'), findsOneWidget);
        expect(find.text('当前密码'), findsOneWidget);
        expect(find.text('新密码'), findsOneWidget);
        expect(find.text('确认新密码'), findsOneWidget);
      });

      testWidgets('密码验证 — 空当前密码显示错误', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        // Tap the submit button without entering anything.
        // There are two "修改密码" texts: section title and button.
        // Find the FilledButton (submit button).
        await tester.tap(find.widgetWithText(FilledButton, '修改密码'));
        await tester.pumpAndSettle();

        expect(find.text('请输入当前密码'), findsOneWidget);
      });

      testWidgets('密码验证 — 新密码太短显示错误', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        // Fill in old password.
        await tester.enterText(
          find.widgetWithText(TextFormField, '当前密码'),
          'oldpassword',
        );

        // Fill in short new password.
        await tester.enterText(
          find.widgetWithText(TextFormField, '新密码'),
          'short',
        );

        // Fill in confirm password (matching).
        await tester.enterText(
          find.widgetWithText(TextFormField, '确认新密码'),
          'short',
        );

        await tester.tap(find.widgetWithText(FilledButton, '修改密码'));
        await tester.pumpAndSettle();

        expect(find.text('密码长度至少为 8 位'), findsOneWidget);
      });

      testWidgets('密码验证 — 确认密码不一致显示错误', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await tester.enterText(
          find.widgetWithText(TextFormField, '当前密码'),
          'oldpassword',
        );
        await tester.enterText(
          find.widgetWithText(TextFormField, '新密码'),
          'newpassword123',
        );
        await tester.enterText(
          find.widgetWithText(TextFormField, '确认新密码'),
          'differentpassword',
        );

        await tester.tap(find.widgetWithText(FilledButton, '修改密码'));
        await tester.pumpAndSettle();

        expect(find.text('两次输入的密码不一致'), findsOneWidget);
      });

      testWidgets('密码验证 — 所有字段正确时显示成功 snackbar', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await tester.enterText(
          find.widgetWithText(TextFormField, '当前密码'),
          'oldpassword',
        );
        await tester.enterText(
          find.widgetWithText(TextFormField, '新密码'),
          'newpassword123',
        );
        await tester.enterText(
          find.widgetWithText(TextFormField, '确认新密码'),
          'newpassword123',
        );

        await tester.tap(find.widgetWithText(FilledButton, '修改密码'));
        await tester.pumpAndSettle();

        expect(find.text('密码修改成功'), findsOneWidget);
      });

      testWidgets('显示两步验证 (TOTP) 区域', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        expect(find.text('两步验证 (TOTP)'), findsOneWidget);
        expect(find.text('未启用'), findsOneWidget);
        expect(find.text('启用'), findsOneWidget);
      });

      testWidgets('显示设备授权区域', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        expect(find.text('设备授权'), findsOneWidget);
        expect(find.text('允许远程连接'), findsOneWidget);
      });

      testWidgets('"允许远程连接" 开关默认为开启', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        final switchTile = tester.widget<SwitchListTile>(
          find.widgetWithText(SwitchListTile, '允许远程连接'),
        );
        expect(switchTile.value, true);
      });

      testWidgets('切换 "允许远程连接" 开关', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        // Toggle the switch off.
        await tester.tap(find.widgetWithText(SwitchListTile, '允许远程连接'));
        await tester.pumpAndSettle();

        final switchTile = tester.widget<SwitchListTile>(
          find.widgetWithText(SwitchListTile, '允许远程连接'),
        );
        expect(switchTile.value, false);
      });
    });

    // ── Network tab ─────────────────────────────────────────────

    group('网络设置标签', () {
      /// Helper to switch to the Network tab.
      Future<void> switchToNetworkTab(WidgetTester tester) async {
        await tester.tap(find.text('网络设置'));
        await tester.pumpAndSettle();
      }

      testWidgets('显示服务器配置区域', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToNetworkTab(tester);

        expect(find.text('服务器配置'), findsOneWidget);
        expect(find.text('信令服务器 (Rendezvous URL)'), findsOneWidget);
        expect(find.text('中继服务器 (Relay URL)'), findsOneWidget);
        expect(find.text('管理 API (API URL)'), findsOneWidget);
      });

      testWidgets('显示 TLS 加密开关', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToNetworkTab(tester);

        expect(find.text('要求 TLS 加密'), findsOneWidget);
      });

      testWidgets('显示连接质量区域', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToNetworkTab(tester);

        expect(find.text('连接质量'), findsOneWidget);
      });

      testWidgets('编解码器下拉菜单显示4个选项', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToNetworkTab(tester);

        // Tap the dropdown to expand it.
        await tester.tap(find.byType(DropdownButtonFormField<String>).first);
        await tester.pumpAndSettle();

        // Verify all 4 codec options are present.
        expect(find.text('H.264'), findsOneWidget);
        expect(find.text('H.265'), findsOneWidget);
        expect(find.text('VP9'), findsOneWidget);
        expect(find.text('AV1'), findsOneWidget);
      });

      testWidgets('编解码器默认为 H.264', (tester) async {
        await pumpTestApp(
          tester,
          initialLocation: '/settings',
          config: const RdcsConfig(
            quality: QualityConfig(codec: 'h264'),
          ),
        );
        await tester.pumpAndSettle();

        await switchToNetworkTab(tester);

        // The dropdown should show H.264 as the current selection.
        // DropdownButtonFormField displays the selected item's child text.
        expect(find.text('H.264'), findsOneWidget);
      });

      testWidgets('切换编解码器为 VP9', (tester) async {
        await pumpTestApp(
          tester,
          initialLocation: '/settings',
          config: const RdcsConfig(
            quality: QualityConfig(codec: 'h264'),
          ),
        );
        await tester.pumpAndSettle();

        await switchToNetworkTab(tester);

        // Tap the dropdown.
        await tester.tap(find.byType(DropdownButtonFormField<String>).first);
        await tester.pumpAndSettle();

        // Select VP9.
        await tester.tap(find.text('VP9').last);
        await tester.pumpAndSettle();

        // Verify the config was updated.
        final container = ProviderScope.containerOf(
          tester.element(find.byType(SettingsScreen)),
        );
        final config = container.read(configProvider);
        expect(config.quality.codec, 'vp9');
      });

      testWidgets('显示硬件加速开关', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToNetworkTab(tester);

        expect(find.text('硬件加速'), findsOneWidget);
      });

      testWidgets('显示带宽设置区域', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToNetworkTab(tester);

        expect(find.text('带宽设置'), findsOneWidget);
        expect(find.text('码率 (kbps)'), findsOneWidget);
      });

      testWidgets('显示保存服务器配置按钮', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToNetworkTab(tester);

        expect(find.text('保存服务器配置'), findsOneWidget);
      });
    });

    // ── General tab ─────────────────────────────────────────────

    group('通用设置标签', () {
      /// Helper to switch to the General tab.
      Future<void> switchToGeneralTab(WidgetTester tester) async {
        await tester.tap(find.text('通用设置'));
        await tester.pumpAndSettle();
      }

      testWidgets('显示语言选择', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        expect(find.text('语言'), findsOneWidget);
        expect(find.text('显示语言'), findsOneWidget);
      });

      testWidgets('显示启动行为区域', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        expect(find.text('启动行为'), findsOneWidget);
        expect(find.text('启动时最小化'), findsOneWidget);
        expect(find.text('开机自启'), findsOneWidget);
        expect(find.text('断线自动重连'), findsOneWidget);
      });

      testWidgets('切换 "同步剪贴板" 开关更新 configProvider', (tester) async {
        await pumpTestApp(
          tester,
          initialLocation: '/settings',
          config: const RdcsConfig(
            general: GeneralConfig(syncClipboard: true),
          ),
        );
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        // Verify initial state.
        var switchTile = tester.widget<SwitchListTile>(
          find.widgetWithText(SwitchListTile, '同步剪贴板'),
        );
        expect(switchTile.value, true);

        // Toggle off.
        await tester.tap(find.widgetWithText(SwitchListTile, '同步剪贴板'));
        await tester.pumpAndSettle();

        // Verify UI updated.
        switchTile = tester.widget<SwitchListTile>(
          find.widgetWithText(SwitchListTile, '同步剪贴板'),
        );
        expect(switchTile.value, false);

        // Verify configProvider was updated.
        final container = ProviderScope.containerOf(
          tester.element(find.byType(SettingsScreen)),
        );
        final config = container.read(configProvider);
        expect(config.general.syncClipboard, false);
      });

      testWidgets('切换 "断线自动重连" 开关更新 configProvider', (tester) async {
        await pumpTestApp(
          tester,
          initialLocation: '/settings',
          config: const RdcsConfig(
            general: GeneralConfig(autoReconnect: false),
          ),
        );
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        // Toggle on.
        await tester.tap(find.widgetWithText(SwitchListTile, '断线自动重连'));
        await tester.pumpAndSettle();

        final container = ProviderScope.containerOf(
          tester.element(find.byType(SettingsScreen)),
        );
        final config = container.read(configProvider);
        expect(config.general.autoReconnect, true);
      });

      testWidgets('切换 "远程音频" 开关更新 configProvider', (tester) async {
        await pumpTestApp(
          tester,
          initialLocation: '/settings',
          config: const RdcsConfig(
            general: GeneralConfig(enableRemoteAudio: true),
          ),
        );
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        // Toggle off.
        await tester.tap(find.widgetWithText(SwitchListTile, '远程音频'));
        await tester.pumpAndSettle();

        final container = ProviderScope.containerOf(
          tester.element(find.byType(SettingsScreen)),
        );
        final config = container.read(configProvider);
        expect(config.general.enableRemoteAudio, false);
      });

      testWidgets('显示剪贴板区域', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        expect(find.text('剪贴板'), findsOneWidget);
        expect(find.text('同步剪贴板'), findsOneWidget);
      });

      testWidgets('显示音频区域', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        expect(find.text('音频'), findsOneWidget);
        expect(find.text('远程音频'), findsOneWidget);
      });

      testWidgets('显示关于区域', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        expect(find.text('关于'), findsOneWidget);
        expect(find.text('0.1.0'), findsOneWidget);
        expect(find.text('Apache 2.0'), findsOneWidget);
      });

      testWidgets('点击 "检查更新" 按钮显示 snackbar', (tester) async {
        await pumpTestApp(tester, initialLocation: '/settings');
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        await tester.tap(find.text('检查更新'));
        await tester.pumpAndSettle();

        expect(find.text('当前已是最新版本'), findsOneWidget);
      });

      testWidgets('语言下拉菜单显示简体中文选项', (tester) async {
        await pumpTestApp(
          tester,
          initialLocation: '/settings',
          config: const RdcsConfig(
            general: GeneralConfig(locale: 'zh-CN'),
          ),
        );
        await tester.pumpAndSettle();

        await switchToGeneralTab(tester);

        // The dropdown should show 简体中文 as the current selection.
        expect(find.text('简体中文'), findsOneWidget);
      });
    });
  });
}
