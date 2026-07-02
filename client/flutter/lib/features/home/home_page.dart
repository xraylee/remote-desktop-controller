// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../core/config/config_provider.dart';
import '../../core/config/config_repository.dart';
import '../../core/ffi/engine_isolate.dart';
import '../../core/ffi/engine_providers.dart';
import '../../core/signaling/signaling_provider.dart';
import '../../core/signaling/websocket_client.dart';
import '../../core/theme.dart';
import '../session/session_providers.dart';

/// Home page — displays device code and provides a connect button.
///
/// This is the primary screen employees see when launching the client.
/// The device code is assigned by the admin in the management console.
class HomePage extends ConsumerWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);
    final config = ref.watch(configProvider);
    final session = ref.watch(sessionProvider);

    final deviceCode = config.deviceCode;
    final formattedCode = deviceCode.isNotEmpty
        ? ConfigRepository.formatDeviceCode(deviceCode)
        : '--- --- ---';

    return Scaffold(
      appBar: AppBar(
        title: const Text('RDCS 远程桌面'),
        actions: [
          IconButton(
            icon: const Icon(Icons.settings_outlined),
            onPressed: () {
              print('Settings button pressed'); // Debug
              context.go('/settings');
            },
            tooltip: '设置',
          ),
        ],
      ),
      body: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            // App logo
            Icon(
              Icons.desktop_windows_outlined,
              size: 80,
              color: theme.colorScheme.primary,
            ),
            const SizedBox(height: 24),

            Text(
              'RDCS 远程桌面',
              style: theme.textTheme.headlineLarge,
            ),
            const SizedBox(height: 8),

            // Session status indicator
            _buildSessionStatus(theme, session),
            const SizedBox(height: 32),

            // Device code display
            GestureDetector(
              onTap: () {
                if (deviceCode.isNotEmpty) {
                  Clipboard.setData(ClipboardData(text: deviceCode));
                  ScaffoldMessenger.of(context).showSnackBar(
                    const SnackBar(
                      content: Text('设备代码已复制到剪贴板'),
                      duration: Duration(seconds: 2),
                    ),
                  );
                }
              },
              child: Container(
                padding:
                    const EdgeInsets.symmetric(horizontal: 32, vertical: 16),
                decoration: BoxDecoration(
                  border:
                      Border.all(color: theme.colorScheme.primary, width: 2),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Text(
                      formattedCode,
                      style: theme.textTheme.headlineMedium?.copyWith(
                        letterSpacing: 4,
                        fontFeatures: const [FontFeature.tabularFigures()],
                      ),
                    ),
                    if (deviceCode.isNotEmpty) ...[
                      const SizedBox(width: 12),
                      Icon(Icons.copy,
                          size: 18, color: theme.colorScheme.primary),
                    ],
                  ],
                ),
              ),
            ),
            const SizedBox(height: 8),

            if (deviceCode.isNotEmpty)
              Text(
                '点击代码可复制',
                style: theme.textTheme.bodySmall?.copyWith(
                  color: RdcsTheme.textSecondary,
                ),
              ),
            const SizedBox(height: 32),

            // Action buttons
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                ElevatedButton.icon(
                  onPressed: () => _showInviteCodeDialog(context, ref),
                  icon: const Icon(Icons.link),
                  label: const Text('连接远程设备'),
                ),
                const SizedBox(width: 16),
                // Temporarily change to test settings navigation
                OutlinedButton.icon(
                  onPressed: () => context.go('/settings'),
                  icon: const Icon(Icons.settings),
                  label: const Text('测试设置'),
                ),
                const SizedBox(width: 16),
                OutlinedButton.icon(
                  onPressed: () => _generateInviteCode(context, ref),
                  icon: const Icon(Icons.person_add_outlined),
                  label: const Text('生成邀请码'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSessionStatus(ThemeData theme, SessionInfo? session) {
    if (session == null) {
      return Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 8,
            height: 8,
            decoration: const BoxDecoration(
              color: RdcsTheme.success,
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 8),
          Text(
            '设备已就绪',
            style: theme.textTheme.bodyMedium,
          ),
        ],
      );
    }

    final (String label, Color color) = switch (session.state) {
      SessionState.connecting => ('正在连接...', RdcsTheme.info),
      SessionState.connected => ('已连接', RdcsTheme.success),
      SessionState.disconnected => ('已断开', RdcsTheme.warning),
      SessionState.error => ('连接失败', RdcsTheme.error),
      SessionState.idle => ('设备已就绪', RdcsTheme.success),
    };

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: color,
            shape: BoxShape.circle,
          ),
        ),
        const SizedBox(width: 8),
        Text(label, style: theme.textTheme.bodyMedium),
      ],
    );
  }

  Future<void> _generateInviteCode(BuildContext context, WidgetRef ref) async {
    try {
      final engine = ref.read(engineIsolateProvider);
      final code = await engine.generateInvite();
      if (context.mounted) {
        showDialog(
          context: context,
          builder: (dialogContext) => AlertDialog(
            title: const Text('邀请码'),
            content: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  code,
                  style: const TextStyle(fontSize: 28, letterSpacing: 4),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 12),
                Text(
                  '将邀请码分享给对方，对方输入后即可连接',
                  style: Theme.of(context).textTheme.bodyMedium,
                  textAlign: TextAlign.center,
                ),
              ],
            ),
            actions: [
              TextButton(
                onPressed: () {
                  Clipboard.setData(ClipboardData(text: code));
                  Navigator.of(dialogContext).pop();
                },
                child: const Text('复制并关闭'),
              ),
              ElevatedButton(
                onPressed: () => Navigator.of(dialogContext).pop(),
                child: const Text('关闭'),
              ),
            ],
          ),
        );
      }
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('生成邀请码失败: $e')),
        );
      }
    }
  }

  Future<void> _showInviteCodeDialog(
      BuildContext context, WidgetRef ref) async {
    final controller = TextEditingController();
    final formKey = GlobalKey<FormState>();
    var isSubmitting = false;

    await showDialog<void>(
      context: context,
      barrierDismissible: !isSubmitting,
      builder: (dialogContext) => StatefulBuilder(
        builder: (context, setDialogState) => AlertDialog(
          title: const Text('输入邀请码'),
          content: Form(
            key: formKey,
            child: TextFormField(
              controller: controller,
              autofocus: true,
              enabled: !isSubmitting,
              textCapitalization: TextCapitalization.characters,
              decoration: const InputDecoration(
                labelText: '邀请码',
                hintText: '请输入对方分享的邀请码',
              ),
              validator: (value) {
                if ((value ?? '').trim().isEmpty) {
                  return '请输入邀请码';
                }
                return null;
              },
              onFieldSubmitted: (_) {
                if (!isSubmitting) {
                  _submitInviteCode(
                    dialogContext,
                    ref,
                    formKey,
                    controller.text,
                    setDialogState,
                    (value) => isSubmitting = value,
                  );
                }
              },
            ),
          ),
          actions: [
            TextButton(
              onPressed:
                  isSubmitting ? null : () => Navigator.of(dialogContext).pop(),
              child: const Text('取消'),
            ),
            ElevatedButton(
              onPressed: isSubmitting
                  ? null
                  : () => _submitInviteCode(
                        dialogContext,
                        ref,
                        formKey,
                        controller.text,
                        setDialogState,
                        (value) => isSubmitting = value,
                      ),
              child: isSubmitting
                  ? const SizedBox(
                      height: 18,
                      width: 18,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: Colors.white,
                      ),
                    )
                  : const Text('连接'),
            ),
          ],
        ),
      ),
    );

    controller.dispose();
  }

  Future<void> _submitInviteCode(
    BuildContext dialogContext,
    WidgetRef ref,
    GlobalKey<FormState> formKey,
    String inviteCode,
    StateSetter setDialogState,
    ValueChanged<bool> setSubmitting,
  ) async {
    if (!(formKey.currentState?.validate() ?? false)) return;

    setDialogState(() => setSubmitting(true));

    try {
      final service = ref.read(signalingServiceProvider);
      if (service.currentConnectionState != WsConnectionState.connected) {
        await service.connect();
      }
      service.useInviteCode(inviteCode.trim());

      if (dialogContext.mounted) {
        Navigator.of(dialogContext).pop();
        ScaffoldMessenger.of(dialogContext).showSnackBar(
          const SnackBar(content: Text('已发送连接请求')),
        );
      }
    } catch (e) {
      if (dialogContext.mounted) {
        ScaffoldMessenger.of(dialogContext).showSnackBar(
          SnackBar(content: Text('连接出错: $e')),
        );
      }
    } finally {
      if (dialogContext.mounted) {
        setDialogState(() => setSubmitting(false));
      }
    }
  }
}
