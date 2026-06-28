// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/theme.dart';

/// Provider controlling whether the floating bar is visible.
///
/// Set to `true` when a session is established (the bar auto-fades
/// after [ControlledFloatingBar.autoHideDuration]). External code
/// (e.g. tray icon tap) can toggle it back on.
final floatingBarVisibleProvider = StateProvider<bool>((ref) => false);

/// Provider holding the name of the current controller for display
/// in the floating bar.
final currentControllerNameProvider = StateProvider<String>((ref) => '');

/// Floating status bar displayed on the controlled side while a
/// remote session is active.
///
/// Appears at the top-center of the screen as a rounded pill with:
/// - An animated blue pulse dot (heartbeat indicator)
/// - A status message: "<controller> is viewing your screen"
/// - A "Disconnect" button
///
/// On first show the bar is visible for [autoHideDuration] seconds,
/// then fades out. The tray icon can re-show it at any time.
class ControlledFloatingBar extends ConsumerStatefulWidget {
  const ControlledFloatingBar({super.key});

  /// Duration the bar stays visible before auto-fading.
  static const Duration autoHideDuration = Duration(seconds: 3);

  /// Duration of the fade-out animation.
  static const Duration fadeDuration = Duration(milliseconds: 500);

  @override
  ConsumerState<ControlledFloatingBar> createState() =>
      _ControlledFloatingBarState();
}

class _ControlledFloatingBarState extends ConsumerState<ControlledFloatingBar>
    with SingleTickerProviderStateMixin {
  Timer? _autoHideTimer;
  bool _visible = false;

  late final AnimationController _pulseController;
  late final Animation<double> _pulseAnimation;

  @override
  void initState() {
    super.initState();

    // Pulse animation for the blue dot — scales between 0.6 and 1.0
    // opacity to simulate a heartbeat.
    _pulseController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 1200),
    )..repeat(reverse: true);
    _pulseAnimation = Tween<double>(begin: 0.4, end: 1.0).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOut),
    );

    _startAutoHide();
  }

  void _startAutoHide() {
    _autoHideTimer?.cancel();
    setState(() => _visible = true);
    ref.read(floatingBarVisibleProvider.notifier).state = true;

    _autoHideTimer = Timer(ControlledFloatingBar.autoHideDuration, () {
      if (mounted) {
        setState(() => _visible = false);
        ref.read(floatingBarVisibleProvider.notifier).state = false;
      }
    });
  }

  /// Called when the user taps the tray icon to re-show the bar.
  void showAgain() {
    _startAutoHide();
  }

  void _onDisconnect() {
    // Signal disconnect — the parent widget / session provider
    // handles the actual disconnection logic.
    setState(() => _visible = false);
    ref.read(floatingBarVisibleProvider.notifier).state = false;
  }

  @override
  void dispose() {
    _autoHideTimer?.cancel();
    _pulseController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final controllerName = ref.watch(currentControllerNameProvider);

    return AnimatedOpacity(
      opacity: _visible ? 1.0 : 0.0,
      duration: ControlledFloatingBar.fadeDuration,
      curve: Curves.easeOut,
      child: AnimatedSlide(
        offset: _visible ? Offset.zero : const Offset(0, -0.3),
        duration: ControlledFloatingBar.fadeDuration,
        curve: Curves.easeOut,
        child: IgnorePointer(
          ignoring: !_visible,
          child: _buildBar(context, controllerName),
        ),
      ),
    );
  }

  Widget _buildBar(BuildContext context, String controllerName) {
    return Center(
      child: Container(
        margin: const EdgeInsets.only(top: 16),
        padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 10),
        decoration: BoxDecoration(
          color: const Color(0xFF1F2937).withOpacity(0.85),
          borderRadius: BorderRadius.circular(28),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withOpacity(0.25),
              blurRadius: 12,
              offset: const Offset(0, 4),
            ),
          ],
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            // ── Animated pulse dot ──────────────────────────────
            AnimatedBuilder(
              animation: _pulseAnimation,
              builder: (context, child) {
                return Opacity(
                  opacity: _pulseAnimation.value,
                  child: child,
                );
              },
              child: Container(
                width: 10,
                height: 10,
                decoration: const BoxDecoration(
                  color: RdcsTheme.primaryLight,
                  shape: BoxShape.circle,
                  boxShadow: [
                    BoxShadow(
                      color: RdcsTheme.primaryLight,
                      blurRadius: 6,
                      spreadRadius: 1,
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(width: 12),

            // ── Status text ─────────────────────────────────────
            Flexible(
              child: Text(
                controllerName.isNotEmpty
                    ? '$controllerName 正在查看您的屏幕'
                    : '远程用户正在查看您的屏幕',
                style: const TextStyle(
                  color: Colors.white,
                  fontSize: 14,
                  fontWeight: FontWeight.w400,
                  decoration: TextDecoration.none,
                ),
                overflow: TextOverflow.ellipsis,
              ),
            ),
            const SizedBox(width: 14),

            // ── Disconnect button ───────────────────────────────
            GestureDetector(
              onTap: _onDisconnect,
              child: Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: 10,
                  vertical: 4,
                ),
                decoration: BoxDecoration(
                  color: RdcsTheme.error.withOpacity(0.15),
                  borderRadius: BorderRadius.circular(14),
                ),
                child: const Text(
                  '断开',
                  style: TextStyle(
                    color: RdcsTheme.error,
                    fontSize: 12,
                    fontWeight: FontWeight.w600,
                    decoration: TextDecoration.none,
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
