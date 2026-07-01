// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../core/config/config_repository.dart';
import '../../core/theme.dart';
import 'session_providers.dart';
import 'video_renderer.dart';

/// Session screen — full-screen remote desktop view with toolbars.
///
/// Displays the remote desktop video feed (placeholder gradient
/// until real video rendering is implemented) with overlay toolbars
/// for quality control, metrics display, and session actions.
class SessionScreen extends ConsumerWidget {
  const SessionScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return const Scaffold(
      backgroundColor: Color(0xFF111111),
      body: _SessionScreenBody(),
    );
  }
}

// ── Body (stateful to manage toolbar visibility) ───────────────

class _SessionScreenBody extends ConsumerStatefulWidget {
  const _SessionScreenBody();

  @override
  ConsumerState<_SessionScreenBody> createState() => _SessionScreenBodyState();
}

class _SessionScreenBodyState extends ConsumerState<_SessionScreenBody> {
  bool _toolbarsVisible = true;

  @override
  Widget build(BuildContext context) {
    final session = ref.watch(sessionProvider);

    // Navigate back when session ends or errors out.
    ref.listen<SessionInfo?>(sessionProvider, (prev, next) {
      if (next == null && prev != null) {
        context.go('/');
      } else if (next != null && next.state == SessionState.disconnected) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('连接已断开'),
            backgroundColor: RdcsTheme.warning,
          ),
        );
        context.go('/');
      } else if (next != null && next.state == SessionState.error) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('连接失败，请检查设备代码后重试'),
            backgroundColor: RdcsTheme.error,
          ),
        );
        context.go('/');
      }
    });

    if (session == null) {
      return const Center(
        child: CircularProgressIndicator(color: Colors.white),
      );
    }

    switch (session.state) {
      case SessionState.connecting:
        return _buildConnectingView(context, session);
      case SessionState.connected:
        return _buildConnectedView(context, session);
      case SessionState.disconnected:
        return _buildDisconnectedView(context, session);
      case SessionState.error:
        return _buildErrorView(context, session);
      case SessionState.idle:
        return const SizedBox.shrink();
    }
  }

  // ── Connecting ────────────────────────────────────────────────

  Widget _buildConnectingView(BuildContext context, SessionInfo session) {
    final code = ConfigRepository.formatDeviceCode(session.remoteDeviceCode);
    return Stack(
      children: [
        const VideoRenderer(),
        Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const CircularProgressIndicator(color: Colors.white),
              const SizedBox(height: 24),
              Text(
                '正在连接 $code ...',
                style: const TextStyle(color: Colors.white, fontSize: 18),
              ),
            ],
          ),
        ),
        Positioned(
          top: 0,
          left: 0,
          right: 0,
          child: _TopToolbar(
            session: session,
            onBack: () => context.go('/'),
            onDisconnect: _onDisconnect,
            onQualityChanged: _onQualityChanged,
          ),
        ),
      ],
    );
  }

  // ── Connected ─────────────────────────────────────────────────

  Widget _buildConnectedView(BuildContext context, SessionInfo session) {
    return Stack(
      children: [
        // Remote desktop video area (placeholder + input capture).
        GestureDetector(
          onTap: _onVideoTap,
          onPanUpdate: _onVideoPanUpdate,
          onDoubleTap: _onVideoDoubleTap,
          child: const _VideoPlaceholder(),
        ),

        // Top toolbar.
        if (_toolbarsVisible)
          Positioned(
            top: 0,
            left: 0,
            right: 0,
            child: _TopToolbar(
              session: session,
              onBack: () => context.go('/'),
              onDisconnect: _onDisconnect,
              onQualityChanged: _onQualityChanged,
            ),
          ),

        // Bottom toolbar.
        if (_toolbarsVisible)
          Positioned(
            bottom: 0,
            left: 0,
            right: 0,
            child: _BottomToolbar(
              onKeyboardPressed: _onKeyboardPressed,
              onFileTransferPressed: _onFileTransferPressed,
              onChatPressed: _onChatPressed,
              onToggleToolbar: _toggleToolbars,
            ),
          ),

        // Tap to toggle toolbars (only visible area outside toolbars).
        if (!_toolbarsVisible)
          Positioned(
            top: 8,
            right: 8,
            child: IconButton(
              icon: const Icon(Icons.visibility, color: Colors.white54),
              onPressed: _toggleToolbars,
            ),
          ),
      ],
    );
  }

  // ── Disconnected ──────────────────────────────────────────────

  Widget _buildDisconnectedView(BuildContext context, SessionInfo session) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.wifi_off, size: 64, color: Colors.white54),
          const SizedBox(height: 16),
          const Text(
            '连接已断开',
            style: TextStyle(color: Colors.white, fontSize: 20),
          ),
          const SizedBox(height: 24),
          ElevatedButton(
            onPressed: () => context.go('/'),
            child: const Text('返回首页'),
          ),
        ],
      ),
    );
  }

  // ── Error ─────────────────────────────────────────────────────

  Widget _buildErrorView(BuildContext context, SessionInfo session) {
    return Center(
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error_outline, size: 64, color: Colors.redAccent),
          const SizedBox(height: 16),
          const Text(
            '连接失败',
            style: TextStyle(color: Colors.white, fontSize: 20),
          ),
          const SizedBox(height: 8),
          Text(
            '无法连接到设备 ${ConfigRepository.formatDeviceCode(session.remoteDeviceCode)}',
            style: const TextStyle(color: Colors.white54, fontSize: 14),
          ),
          const SizedBox(height: 24),
          ElevatedButton(
            onPressed: () => context.go('/'),
            child: const Text('返回首页'),
          ),
        ],
      ),
    );
  }

  // ── Actions ───────────────────────────────────────────────────

  void _onDisconnect() {
    ref.read(sessionProvider.notifier).disconnect();
  }

  void _onQualityChanged(int mode) {
    ref.read(sessionProvider.notifier).setQuality(mode);
  }

  void _toggleToolbars() {
    setState(() {
      _toolbarsVisible = !_toolbarsVisible;
    });
  }

  // ── Input forwarding ──────────────────────────────────────────

  void _onVideoTap() {
    // Toggle toolbars on single tap, also send click event.
    setState(() {
      _toolbarsVisible = !_toolbarsVisible;
    });
    ref.read(sessionProvider.notifier).sendInput(
      jsonEncode({'type': 'mouse', 'action': 'click', 'x': 0, 'y': 0}),
    );
  }

  void _onVideoDoubleTap() {
    ref.read(sessionProvider.notifier).sendInput(
      jsonEncode({'type': 'mouse', 'action': 'double_click', 'x': 0, 'y': 0}),
    );
  }

  void _onVideoPanUpdate(DragUpdateDetails details) {
    ref.read(sessionProvider.notifier).sendInput(
      jsonEncode({
        'type': 'mouse',
        'action': 'move',
        'x': details.localPosition.dx.round(),
        'y': details.localPosition.dy.round(),
      }),
    );
  }

  void _onKeyboardPressed() {
    // TODO: open virtual keyboard overlay or system IME
  }

  void _onFileTransferPressed() {
    // TODO: open file transfer panel
  }

  void _onChatPressed() {
    _showChatDialog(context);
  }

  void _showChatDialog(BuildContext context) {
    final controller = TextEditingController();
    showDialog(
      context: context,
      builder: (dialogContext) => AlertDialog(
        title: const Text('发送消息'),
        content: TextField(
          controller: controller,
          decoration: const InputDecoration(hintText: '输入消息...'),
          autofocus: true,
          onSubmitted: (text) {
            if (text.isNotEmpty) {
              ref.read(sessionProvider.notifier).sendMessage(text);
            }
            Navigator.of(dialogContext).pop();
          },
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(dialogContext).pop(),
            child: const Text('取消'),
          ),
          ElevatedButton(
            onPressed: () {
              if (controller.text.isNotEmpty) {
                ref
                    .read(sessionProvider.notifier)
                    .sendMessage(controller.text);
              }
              Navigator.of(dialogContext).pop();
            },
            child: const Text('发送'),
          ),
        ],
      ),
    );
  }
}

// ── Top toolbar ─────────────────────────────────────────────────

class _TopToolbar extends StatelessWidget {
  const _TopToolbar({
    required this.session,
    required this.onBack,
    required this.onDisconnect,
    required this.onQualityChanged,
  });

  final SessionInfo session;
  final VoidCallback onBack;
  final VoidCallback onDisconnect;
  final ValueChanged<int> onQualityChanged;

  @override
  Widget build(BuildContext context) {
    final code = ConfigRepository.formatDeviceCode(session.remoteDeviceCode);

    return Container(
      padding: EdgeInsets.only(
        top: MediaQuery.of(context).padding.top,
        left: 8,
        right: 8,
        bottom: 4,
      ),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [
            Colors.black.withOpacity(0.7),
            Colors.transparent,
          ],
        ),
      ),
      child: Row(
        children: [
          // Back button.
          IconButton(
            icon: const Icon(Icons.arrow_back, color: Colors.white),
            onPressed: onBack,
            tooltip: '返回',
          ),

          const SizedBox(width: 8),

          // Remote device name / code.
          Expanded(
            child: Text(
              session.remoteDeviceName.isNotEmpty
                  ? session.remoteDeviceName
                  : code,
              style: const TextStyle(
                color: Colors.white,
                fontSize: 15,
                fontWeight: FontWeight.w500,
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),

          // Latency indicator.
          _ToolbarChip(
            icon: Icons.signal_cellular_alt,
            label: '${session.latencyMs}ms',
            color: session.latencyMs < 50
                ? RdcsTheme.success
                : session.latencyMs < 150
                    ? RdcsTheme.warning
                    : RdcsTheme.error,
          ),

          const SizedBox(width: 8),

          // FPS indicator.
          _ToolbarChip(
            icon: Icons.videocam,
            label: '${session.fps.toStringAsFixed(0)} FPS',
            color: Colors.white70,
          ),

          const SizedBox(width: 8),

          // Quality mode selector.
          PopupMenuButton<int>(
            initialValue: session.qualityMode,
            onSelected: onQualityChanged,
            color: Colors.black87,
            tooltip: '画质模式',
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
              decoration: BoxDecoration(
                color: Colors.white.withOpacity(0.12),
                borderRadius: BorderRadius.circular(4),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    qualityModeLabel(session.qualityMode),
                    style: const TextStyle(color: Colors.white, fontSize: 13),
                  ),
                  const SizedBox(width: 4),
                  const Icon(
                    Icons.arrow_drop_down,
                    color: Colors.white70,
                    size: 18,
                  ),
                ],
              ),
            ),
            itemBuilder: (context) => [
              const PopupMenuItem(value: 0, child: Text('自动')),
              const PopupMenuItem(value: 1, child: Text('清晰优先')),
              const PopupMenuItem(value: 2, child: Text('流畅优先')),
            ],
          ),

          const SizedBox(width: 4),

          // Fullscreen toggle.
          IconButton(
            icon: const Icon(Icons.fullscreen, color: Colors.white),
            onPressed: () {
              SystemChrome.setEnabledSystemUIMode(
                SystemUiMode.immersiveSticky,
              );
            },
            tooltip: '全屏',
          ),

          // Disconnect button (red).
          IconButton(
            icon: const Icon(Icons.call_end, color: Colors.redAccent),
            onPressed: onDisconnect,
            tooltip: '断开连接',
          ),
        ],
      ),
    );
  }
}

// ── Bottom toolbar ──────────────────────────────────────────────

class _BottomToolbar extends StatelessWidget {
  const _BottomToolbar({
    required this.onKeyboardPressed,
    required this.onFileTransferPressed,
    required this.onChatPressed,
    required this.onToggleToolbar,
  });

  final VoidCallback onKeyboardPressed;
  final VoidCallback onFileTransferPressed;
  final VoidCallback onChatPressed;
  final VoidCallback onToggleToolbar;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: EdgeInsets.only(
        left: 16,
        right: 16,
        top: 8,
        bottom: MediaQuery.of(context).padding.bottom + 8,
      ),
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.bottomCenter,
          end: Alignment.topCenter,
          colors: [
            Colors.black.withOpacity(0.7),
            Colors.transparent,
          ],
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          _ToolbarButton(
            icon: Icons.keyboard,
            label: '键盘',
            onPressed: onKeyboardPressed,
          ),
          _ToolbarButton(
            icon: Icons.folder_open,
            label: '文件传输',
            onPressed: onFileTransferPressed,
          ),
          _ToolbarButton(
            icon: Icons.chat_bubble_outline,
            label: '消息',
            onPressed: onChatPressed,
          ),
          _ToolbarButton(
            icon: Icons.visibility_off,
            label: '隐藏面板',
            onPressed: onToggleToolbar,
          ),
        ],
      ),
    );
  }
}

// ── Shared toolbar widgets ──────────────────────────────────────

/// A small icon+label button used in the bottom toolbar.
class _ToolbarButton extends StatelessWidget {
  const _ToolbarButton({
    required this.icon,
    required this.label,
    required this.onPressed,
  });

  final IconData icon;
  final String label;
  final VoidCallback onPressed;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onPressed,
      borderRadius: BorderRadius.circular(8),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, color: Colors.white, size: 22),
            const SizedBox(height: 4),
            Text(
              label,
              style: const TextStyle(color: Colors.white70, fontSize: 11),
            ),
          ],
        ),
      ),
    );
  }
}

/// A compact chip with icon + label used in the top toolbar.
class _ToolbarChip extends StatelessWidget {
  const _ToolbarChip({
    required this.icon,
    required this.label,
    required this.color,
  });

  final IconData icon;
  final String label;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
      decoration: BoxDecoration(
        color: Colors.white.withOpacity(0.1),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 14, color: color),
          const SizedBox(width: 4),
          Text(
            label,
            style: TextStyle(color: color, fontSize: 12),
          ),
        ],
      ),
    );
  }
}
