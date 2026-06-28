// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';

import 'package:flutter/material.dart';

import '../../core/theme.dart';

/// Category of session-end reason, determines the icon and color.
enum SessionEndSeverity {
  /// Normal disconnect — informational icon, blue tint.
  info,

  /// Unexpected loss (network error, crash) — warning icon, amber tint.
  warning,
}

/// Slide-in notification toast shown when a remote session ends.
///
/// Appears from the top of the screen, auto-dismisses after
/// [autoDismissDuration] seconds, and provides a manual "知道了"
/// dismiss button.
///
/// Usage:
/// ```dart
/// SessionEndToast.show(
///   context,
///   reason: '对方已断开',
///   severity: SessionEndSeverity.info,
/// );
/// ```
class SessionEndToast {
  SessionEndToast._();

  /// Duration before the toast auto-dismisses.
  static const Duration autoDismissDuration = Duration(seconds: 4);

  /// Duration of the slide-in / slide-out animation.
  static const Duration animationDuration = Duration(milliseconds: 350);

  /// Shows the toast as an overlay entry and auto-dismisses it.
  ///
  /// The toast slides in from the top, displays [reason] with an
  /// icon determined by [severity], and auto-dismisses after
  /// [autoDismissDuration]. The user can also dismiss it early by
  /// tapping the "知道了" button.
  static void show(
    BuildContext context, {
    required String reason,
    SessionEndSeverity severity = SessionEndSeverity.info,
    VoidCallback? onDismiss,
  }) {
    final overlay = Overlay.of(context);
    late final OverlayEntry entry;

    entry = OverlayEntry(
      builder: (context) => _SessionEndToastOverlay(
        reason: reason,
        severity: severity,
        onDismissed: () {
          if (entry.mounted) {
            entry.remove();
          }
          onDismiss?.call();
        },
      ),
    );

    overlay.insert(entry);
  }
}

/// Internal overlay widget that manages the slide-in animation,
/// auto-dismiss timer, and visual layout for the session-end toast.
class _SessionEndToastOverlay extends StatefulWidget {
  const _SessionEndToastOverlay({
    required this.reason,
    required this.severity,
    required this.onDismissed,
  });

  final String reason;
  final SessionEndSeverity severity;
  final VoidCallback onDismissed;

  @override
  State<_SessionEndToastOverlay> createState() =>
      _SessionEndToastOverlayState();
}

class _SessionEndToastOverlayState extends State<_SessionEndToastOverlay>
    with SingleTickerProviderStateMixin {
  late final AnimationController _animController;
  late final Animation<Offset> _slideAnimation;
  late final Animation<double> _fadeAnimation;
  Timer? _autoDismissTimer;

  @override
  void initState() {
    super.initState();

    _animController = AnimationController(
      vsync: this,
      duration: SessionEndToast.animationDuration,
    );

    _slideAnimation = Tween<Offset>(
      begin: const Offset(0, -1),
      end: Offset.zero,
    ).animate(CurvedAnimation(
      parent: _animController,
      curve: Curves.easeOutCubic,
    ));

    _fadeAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(parent: _animController, curve: Curves.easeOut),
    );

    _animController.forward();

    _autoDismissTimer =
        Timer(SessionEndToast.autoDismissDuration, _dismiss);
  }

  void _dismiss() {
    if (!mounted) return;
    _animController.reverse().then((_) {
      widget.onDismissed();
    });
  }

  @override
  void dispose() {
    _autoDismissTimer?.cancel();
    _animController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isWarning = widget.severity == SessionEndSeverity.warning;
    final accentColor = isWarning ? RdcsTheme.warning : RdcsTheme.info;
    final icon = isWarning
        ? Icons.warning_amber_rounded
        : Icons.info_outline_rounded;

    return Positioned(
      top: 0,
      left: 0,
      right: 0,
      child: SlideTransition(
        position: _slideAnimation,
        child: FadeTransition(
          opacity: _fadeAnimation,
          child: SafeArea(
            bottom: false,
            child: Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
              child: Material(
                color: Colors.transparent,
                child: Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 18,
                    vertical: 14,
                  ),
                  decoration: BoxDecoration(
                    color: Colors.white,
                    borderRadius: BorderRadius.circular(12),
                    border: Border(
                      left: BorderSide(color: accentColor, width: 4),
                    ),
                    boxShadow: [
                      BoxShadow(
                        color: Colors.black.withOpacity(0.12),
                        blurRadius: 16,
                        offset: const Offset(0, 4),
                      ),
                    ],
                  ),
                  child: Row(
                    children: [
                      // ── Severity icon ─────────────────────────
                      Icon(icon, size: 24, color: accentColor),
                      const SizedBox(width: 14),

                      // ── Reason text ──────────────────────────
                      Expanded(
                        child: Text(
                          widget.reason,
                          style: const TextStyle(
                            fontSize: 14,
                            fontWeight: FontWeight.w500,
                            color: RdcsTheme.textPrimary,
                          ),
                        ),
                      ),
                      const SizedBox(width: 12),

                      // ── Dismiss button ────────────────────────
                      GestureDetector(
                        onTap: _dismiss,
                        child: Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 14,
                            vertical: 6,
                          ),
                          decoration: BoxDecoration(
                            color: accentColor.withOpacity(0.08),
                            borderRadius: BorderRadius.circular(16),
                          ),
                          child: Text(
                            '知道了',
                            style: TextStyle(
                              fontSize: 13,
                              fontWeight: FontWeight.w500,
                              color: accentColor,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
