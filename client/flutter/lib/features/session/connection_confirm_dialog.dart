// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/theme.dart';

/// Result returned when the connection confirm dialog closes.
enum ConnectionConfirmResult {
  /// The user accepted the connection request.
  accepted,

  /// The user explicitly rejected the request.
  rejected,

  /// The request timed out without a user response.
  timedOut,
}

/// A pending connection request that can be queued.
class ConnectionRequest {
  const ConnectionRequest({
    required this.requesterName,
    required this.requesterCode,
  });

  /// Display name of the device requesting to connect.
  final String requesterName;

  /// Device code of the requester.
  final String requesterCode;
}

/// Provider tracking the number of queued connection requests.
final pendingRequestCountProvider = StateProvider<int>((ref) => 0);

/// Modal dialog shown on the controlled side when a remote device
/// requests to establish a connection.
///
/// Displays the requester's name and device code, a countdown timer,
/// and accept/reject buttons. Auto-rejects when the timer expires.
/// If additional requests arrive while the dialog is open, a badge
/// shows the queued count.
class ConnectionConfirmDialog extends ConsumerStatefulWidget {
  const ConnectionConfirmDialog({
    super.key,
    required this.requesterName,
    required this.requesterCode,
    this.requestTimeoutSeconds = 30,
    this.queuedCount = 0,
  });

  /// Display name of the device requesting to connect.
  final String requesterName;

  /// Device code of the requester.
  final String requesterCode;

  /// Seconds before the dialog auto-rejects. Defaults to 30.
  final int requestTimeoutSeconds;

  /// Number of additional requests queued behind this one.
  final int queuedCount;

  /// Shows the dialog and returns a [ConnectionConfirmResult].
  ///
  /// Returns [ConnectionConfirmResult.timedOut] if the countdown
  /// expires without user interaction.
  static Future<ConnectionConfirmResult> show(
    BuildContext context, {
    required String requesterName,
    required String requesterCode,
    int requestTimeoutSeconds = 30,
    int queuedCount = 0,
  }) async {
    final result = await showDialog<ConnectionConfirmResult>(
      context: context,
      barrierDismissible: false,
      barrierColor: Colors.black.withOpacity(0.6),
      builder: (_) => ConnectionConfirmDialog(
        requesterName: requesterName,
        requesterCode: requesterCode,
        requestTimeoutSeconds: requestTimeoutSeconds,
        queuedCount: queuedCount,
      ),
    );
    return result ?? ConnectionConfirmResult.timedOut;
  }

  @override
  ConsumerState<ConnectionConfirmDialog> createState() =>
      _ConnectionConfirmDialogState();
}

class _ConnectionConfirmDialogState
    extends ConsumerState<ConnectionConfirmDialog>
    with SingleTickerProviderStateMixin {
  late int _remainingSeconds;
  Timer? _countdownTimer;
  late final AnimationController _pulseController;
  late final Animation<double> _pulseAnimation;

  @override
  void initState() {
    super.initState();
    _remainingSeconds = widget.requestTimeoutSeconds;

    // Pulse animation for the shield icon.
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1500),
    )..repeat(reverse: true);
    _pulseAnimation = Tween<double>(begin: 1.0, end: 1.15).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOut),
    );

    _startCountdown();
  }

  void _startCountdown() {
    _countdownTimer = Timer.periodic(const Duration(seconds: 1), (timer) {
      if (!mounted) {
        timer.cancel();
        return;
      }
      setState(() {
        _remainingSeconds--;
      });
      if (_remainingSeconds <= 0) {
        timer.cancel();
        _close(ConnectionConfirmResult.timedOut);
      }
    });
  }

  void _close(ConnectionConfirmResult result) {
    if (Navigator.of(context).canPop()) {
      Navigator.of(context).pop(result);
    }
  }

  @override
  void dispose() {
    _countdownTimer?.cancel();
    _pulseController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Center(
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 420),
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
                // ── Shield icon with pulse ──────────────────────
                AnimatedBuilder(
                  animation: _pulseAnimation,
                  builder: (context, child) {
                    return Transform.scale(
                      scale: _pulseAnimation.value,
                      child: child,
                    );
                  },
                  child: Container(
                    width: 64,
                    height: 64,
                    decoration: BoxDecoration(
                      color: RdcsTheme.primary.withOpacity(0.1),
                      shape: BoxShape.circle,
                    ),
                    child: const Icon(
                      Icons.shield_outlined,
                      size: 36,
                      color: RdcsTheme.primary,
                    ),
                  ),
                ),
                const SizedBox(height: 20),

                // ── Title ───────────────────────────────────────
                Text(
                  '远程连接请求',
                  style: theme.textTheme.headlineMedium?.copyWith(
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 12),

                // ── Description ─────────────────────────────────
                RichText(
                  textAlign: TextAlign.center,
                  text: TextSpan(
                    style: theme.textTheme.bodyLarge?.copyWith(
                      color: RdcsTheme.textSecondary,
                      height: 1.5,
                    ),
                    children: [
                      const TextSpan(text: '设备 '),
                      TextSpan(
                        text: widget.requesterName,
                        style: const TextStyle(
                          fontWeight: FontWeight.w600,
                          color: RdcsTheme.textPrimary,
                        ),
                      ),
                      const TextSpan(text: '\n（代码: '),
                      TextSpan(
                        text: widget.requesterCode,
                        style: const TextStyle(
                          fontWeight: FontWeight.w500,
                          color: RdcsTheme.primary,
                          fontFeatures: [FontFeature.tabularFigures()],
                        ),
                      ),
                      const TextSpan(text: '）\n请求控制您的电脑'),
                    ],
                  ),
                ),
                const SizedBox(height: 20),

                // ── Countdown timer ─────────────────────────────
                _CountdownIndicator(
                  remainingSeconds: _remainingSeconds,
                  totalSeconds: widget.requestTimeoutSeconds,
                ),
                const SizedBox(height: 8),

                // ── Queued requests badge ───────────────────────
                if (widget.queuedCount > 0)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 12),
                    child: Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 12,
                        vertical: 6,
                      ),
                      decoration: BoxDecoration(
                        color: RdcsTheme.warning.withOpacity(0.1),
                        borderRadius: BorderRadius.circular(20),
                      ),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: [
                          Icon(
                            Icons.queue_outlined,
                            size: 16,
                            color: RdcsTheme.warning,
                          ),
                          const SizedBox(width: 6),
                          Text(
                            '还有 ${widget.queuedCount} 个连接请求等待处理',
                            style: theme.textTheme.bodyMedium?.copyWith(
                              color: RdcsTheme.warning,
                              fontWeight: FontWeight.w500,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),

                // ── Action buttons ──────────────────────────────
                const SizedBox(height: 4),
                Row(
                  children: [
                    Expanded(
                      child: OutlinedButton(
                        onPressed: () =>
                            _close(ConnectionConfirmResult.rejected),
                        style: OutlinedButton.styleFrom(
                          foregroundColor: RdcsTheme.textSecondary,
                          side: const BorderSide(
                            color: RdcsTheme.divider,
                          ),
                          minimumSize: const Size(0, 44),
                        ),
                        child: const Text('拒绝'),
                      ),
                    ),
                    const SizedBox(width: 16),
                    Expanded(
                      child: ElevatedButton(
                        onPressed: () =>
                            _close(ConnectionConfirmResult.accepted),
                        style: ElevatedButton.styleFrom(
                          backgroundColor: RdcsTheme.primary,
                          foregroundColor: Colors.white,
                          minimumSize: const Size(0, 44),
                        ),
                        child: const Text('允许'),
                      ),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// Circular countdown indicator showing remaining time both
/// numerically and as a progress ring.
class _CountdownIndicator extends StatelessWidget {
  const _CountdownIndicator({
    required this.remainingSeconds,
    required this.totalSeconds,
  });

  final int remainingSeconds;
  final int totalSeconds;

  @override
  Widget build(BuildContext context) {
    final progress = totalSeconds > 0 ? remainingSeconds / totalSeconds : 0.0;
    final isUrgent = remainingSeconds <= 10;

    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        SizedBox(
          width: 24,
          height: 24,
          child: CircularProgressIndicator(
            value: progress,
            strokeWidth: 2.5,
            backgroundColor: RdcsTheme.divider,
            color: isUrgent ? RdcsTheme.error : RdcsTheme.primary,
          ),
        ),
        const SizedBox(width: 10),
        Text(
          '${remainingSeconds}s',
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w600,
            color: isUrgent ? RdcsTheme.error : RdcsTheme.textSecondary,
            fontFeatures: const [FontFeature.tabularFigures()],
          ),
        ),
      ],
    );
  }
}
