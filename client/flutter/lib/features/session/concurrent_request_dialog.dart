// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/theme.dart';

/// Result returned when the concurrent request dialog closes.
enum ConcurrentRequestResult {
  /// Reject the new request; keep the current session intact.
  rejected,

  /// Disconnect the current controller and accept the new request.
  disconnectAndAccept,

  /// Accept the new request while keeping the current session active.
  acceptBoth,

  /// The dialog was dismissed without an explicit choice.
  dismissed,
}

/// Dialog shown on the controlled side when a new connection request
/// arrives while the device is already being controlled.
///
/// Presents three options:
/// 1. Reject the new request
/// 2. Disconnect the current controller and accept the new one
/// 3. Accept both simultaneously
class ConcurrentRequestDialog extends ConsumerWidget {
  const ConcurrentRequestDialog({
    super.key,
    required this.currentControllerName,
    required this.newRequesterName,
    required this.newRequesterCode,
  });

  /// Display name of the device currently controlling this machine.
  final String currentControllerName;

  /// Display name of the new device requesting to connect.
  final String newRequesterName;

  /// Device code of the new requester.
  final String newRequesterCode;

  /// Shows the dialog and returns a [ConcurrentRequestResult].
  static Future<ConcurrentRequestResult> show(
    BuildContext context, {
    required String currentControllerName,
    required String newRequesterName,
    required String newRequesterCode,
  }) async {
    final result = await showDialog<ConcurrentRequestResult>(
      context: context,
      barrierDismissible: false,
      barrierColor: Colors.black.withOpacity(0.6),
      builder: (_) => ConcurrentRequestDialog(
        currentControllerName: currentControllerName,
        newRequesterName: newRequesterName,
        newRequesterCode: newRequesterCode,
      ),
    );
    return result ?? ConcurrentRequestResult.dismissed;
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final theme = Theme.of(context);

    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 440),
        child: Card(
          elevation: 8,
          shadowColor: Colors.black.withOpacity(0.3),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(16),
          ),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 28, vertical: 24),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // ── Warning icon ────────────────────────────────
                Container(
                  width: 56,
                  height: 56,
                  decoration: BoxDecoration(
                    color: RdcsTheme.warning.withOpacity(0.1),
                    shape: BoxShape.circle,
                  ),
                  child: const Icon(
                    Icons.swap_horiz_rounded,
                    size: 32,
                    color: RdcsTheme.warning,
                  ),
                ),
                const SizedBox(height: 20),

                // ── Title ───────────────────────────────────────
                Text(
                  '并发连接请求',
                  style: theme.textTheme.headlineMedium?.copyWith(
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 16),

                // ── Current session info ────────────────────────
                _SessionInfoRow(
                  icon: Icons.link,
                  iconColor: RdcsTheme.success,
                  label: '当前连接',
                  value: currentControllerName,
                ),
                const SizedBox(height: 8),

                // ── New request info ────────────────────────────
                _SessionInfoRow(
                  icon: Icons.person_add_outlined,
                  iconColor: RdcsTheme.primary,
                  label: '新请求',
                  value: '$newRequesterName（$newRequesterCode）',
                ),
                const SizedBox(height: 16),

                // ── Description ─────────────────────────────────
                Text(
                  '您正在被 $currentControllerName 控制中，'
                  '$newRequesterName 也请求连接。请选择操作：',
                  textAlign: TextAlign.center,
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: RdcsTheme.textSecondary,
                    height: 1.5,
                  ),
                ),
                const SizedBox(height: 24),

                // ── Action buttons ──────────────────────────────
                // Reject new request
                SizedBox(
                  width: double.infinity,
                  child: OutlinedButton(
                    onPressed: () => _close(
                      context,
                      ConcurrentRequestResult.rejected,
                    ),
                    style: OutlinedButton.styleFrom(
                      foregroundColor: RdcsTheme.textSecondary,
                      side: const BorderSide(color: RdcsTheme.divider),
                      minimumSize: const Size(0, 44),
                    ),
                    child: const Text('拒绝新请求'),
                  ),
                ),
                const SizedBox(height: 10),

                // Disconnect current and accept new
                SizedBox(
                  width: double.infinity,
                  child: OutlinedButton.icon(
                    onPressed: () => _close(
                      context,
                      ConcurrentRequestResult.disconnectAndAccept,
                    ),
                    icon: const Icon(Icons.swap_horiz, size: 18),
                    label: const Text('断开当前并允许'),
                    style: OutlinedButton.styleFrom(
                      foregroundColor: RdcsTheme.warning,
                      side: const BorderSide(color: RdcsTheme.warning),
                      minimumSize: const Size(0, 44),
                    ),
                  ),
                ),
                const SizedBox(height: 10),

                // Accept both
                SizedBox(
                  width: double.infinity,
                  child: ElevatedButton.icon(
                    onPressed: () => _close(
                      context,
                      ConcurrentRequestResult.acceptBoth,
                    ),
                    icon: const Icon(Icons.group_add_outlined, size: 18),
                    label: const Text('同时允许'),
                    style: ElevatedButton.styleFrom(
                      backgroundColor: RdcsTheme.primary,
                      foregroundColor: Colors.white,
                      minimumSize: const Size(0, 44),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void _close(BuildContext context, ConcurrentRequestResult result) {
    if (Navigator.of(context).canPop()) {
      Navigator.of(context).pop(result);
    }
  }
}

/// A row displaying session information with an icon, label, and value.
class _SessionInfoRow extends StatelessWidget {
  const _SessionInfoRow({
    required this.icon,
    required this.iconColor,
    required this.label,
    required this.value,
  });

  final IconData icon;
  final Color iconColor;
  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
      decoration: BoxDecoration(
        color: RdcsTheme.background,
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: RdcsTheme.divider),
      ),
      child: Row(
        children: [
          Icon(icon, size: 20, color: iconColor),
          const SizedBox(width: 10),
          Text(
            '$label: ',
            style: theme.textTheme.bodyMedium?.copyWith(
              color: RdcsTheme.textSecondary,
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: theme.textTheme.bodyLarge?.copyWith(
                fontWeight: FontWeight.w500,
                color: RdcsTheme.textPrimary,
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }
}
