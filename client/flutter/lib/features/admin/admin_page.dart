// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

/// Admin page — management dashboard (role-gated to admin users).
///
/// Displays device list, user permissions, audit logs, and
/// session recordings. Only accessible when the logged-in user
/// has an admin role.
class AdminPage extends ConsumerWidget {
  const AdminPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/'),
        ),
        title: const Text('管理控制台'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '管理控制台',
              style: theme.textTheme.headlineMedium,
            ),
            const SizedBox(height: 24),
            // Placeholder management sections
            Expanded(
              child: GridView.count(
                crossAxisCount: 3,
                mainAxisSpacing: 16,
                crossAxisSpacing: 16,
                childAspectRatio: 2.5,
                children: [
                  _AdminCard(
                    icon: Icons.devices,
                    title: '设备管理',
                    subtitle: '管理已注册设备',
                  ),
                  _AdminCard(
                    icon: Icons.people,
                    title: '用户管理',
                    subtitle: '管理用户权限',
                  ),
                  _AdminCard(
                    icon: Icons.history,
                    title: '审计日志',
                    subtitle: '查看操作记录',
                  ),
                  _AdminCard(
                    icon: Icons.videocam,
                    title: '会话录制',
                    subtitle: '查看历史录像',
                  ),
                  _AdminCard(
                    icon: Icons.settings,
                    title: '系统设置',
                    subtitle: '服务器与网络配置',
                  ),
                  _AdminCard(
                    icon: Icons.analytics,
                    title: '使用统计',
                    subtitle: '并发数与流量监控',
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _AdminCard extends StatelessWidget {
  const _AdminCard({
    required this.icon,
    required this.title,
    required this.subtitle,
  });

  final IconData icon;
  final String title;
  final String subtitle;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(icon, size: 36, color: theme.colorScheme.primary),
            const SizedBox(width: 16),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Text(title, style: theme.textTheme.titleLarge),
                const SizedBox(height: 4),
                Text(subtitle, style: theme.textTheme.bodyMedium),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
