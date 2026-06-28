// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';
import 'package:window_manager/window_manager.dart';

import 'core/config/config_provider.dart';
import 'core/theme.dart';
import 'core/tray/tray_service.dart';
import 'features/home/home_page.dart';
import 'features/connect/connect_page.dart';
import 'features/session/session_screen.dart';
import 'features/admin/admin_page.dart';
import 'features/settings/settings_screen.dart';

// ── Dark mode provider ────────────────────────────────────────

/// Reactive boolean that controls light vs dark theme.
/// Persisted in-memory only; toggled via the tray "切换主题" menu.
final darkModeProvider = StateProvider<bool>((ref) => false);

/// Application-level GoRouter configuration.
///
/// Routes:
///   /          → HomePage (device code display + connect button)
///   /connect   → ConnectPage (enter remote device code)
///   /session   → SessionScreen (remote desktop view)
///   /admin     → AdminPage (management dashboard, role-gated)
///   /settings  → SettingsScreen (security, network, general)
final goRouterProvider = Provider<GoRouter>((ref) {
  return GoRouter(
    initialLocation: '/',
    routes: [
      GoRoute(
        path: '/',
        name: 'home',
        builder: (context, state) => const HomePage(),
      ),
      GoRoute(
        path: '/connect',
        name: 'connect',
        builder: (context, state) => const ConnectPage(),
      ),
      GoRoute(
        path: '/session',
        name: 'session',
        builder: (context, state) => const SessionScreen(),
      ),
      GoRoute(
        path: '/admin',
        name: 'admin',
        builder: (context, state) => const AdminPage(),
      ),
      GoRoute(
        path: '/settings',
        name: 'settings',
        builder: (context, state) => const SettingsScreen(),
      ),
    ],
    errorBuilder: (context, state) => Scaffold(
      body: Center(
        child: Text(
          '页面未找到: ${state.uri}',
          style: RdcsTheme.light.textTheme.bodyLarge,
        ),
      ),
    ),
  );
});

/// Root application widget with MaterialApp + GoRouter.
///
/// Also manages window lifecycle events:
///   - On close: minimise to tray instead of quitting.
///   - On first frame: initialise system tray icon.
class RdcsApp extends ConsumerStatefulWidget {
  const RdcsApp({super.key});

  @override
  ConsumerState<RdcsApp> createState() => _RdcsAppState();
}

class _RdcsAppState extends ConsumerState<RdcsApp> with WindowListener {
  /// Tracks whether the user explicitly chose "退出" from the tray
  /// menu. When `false`, a window close event minimises to tray
  /// rather than terminating the process.
  bool _forceQuit = false;

  @override
  void initState() {
    super.initState();
    windowManager.addListener(this);

    // Defer tray initialisation until after the first frame so
    // the Flutter engine is fully ready (required by tray_manager
    // for asset resolution).
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _initTray();
    });
  }

  @override
  void dispose() {
    windowManager.removeListener(this);
    super.dispose();
  }

  // ── Tray initialisation ─────────────────────────────────────────

  Future<void> _initTray() async {
    final tray = ref.read(trayServiceProvider);

    // Wire tray callbacks.
    tray.onToggleTheme = () {
      final current = ref.read(darkModeProvider);
      ref.read(darkModeProvider.notifier).state = !current;
    };

    tray.onQuitRequested = () {
      _handleQuit();
    };

    await tray.init();

    // Honour the startMinimized config preference.
    final config = ref.read(configProvider);
    if (config.general.startMinimized) {
      await tray.hideWindow();
    }
  }

  // ── Quit flow ───────────────────────────────────────────────────

  /// Handles the "退出" tray menu action.
  ///
  /// Sets [_forceQuit] so the WindowListener's `onWindowClose`
  /// allows the window to actually close, then destroys the window.
  void _handleQuit() {
    _forceQuit = true;
    windowManager.close();
  }

  // ── WindowListener overrides ────────────────────────────────────

  @override
  void onWindowClose() {
    if (_forceQuit) {
      // User explicitly chose to exit — let the window close.
      return;
    }

    // Otherwise, minimise to tray instead of quitting.
    final tray = ref.read(trayServiceProvider);
    tray.hideWindow();
  }

  // ── Build ───────────────────────────────────────────────────────

  @override
  Widget build(BuildContext context) {
    final router = ref.watch(goRouterProvider);
    final isDark = ref.watch(darkModeProvider);

    return MaterialApp.router(
      title: 'RDCS 远程桌面',
      debugShowCheckedModeBanner: false,
      theme: isDark ? _buildDarkTheme() : RdcsTheme.light,
      darkTheme: _buildDarkTheme(),
      themeMode: isDark ? ThemeMode.dark : ThemeMode.light,
      routerConfig: router,
    );
  }

  /// Builds a dark variant of the RDCS brand theme.
  ///
  /// Uses the same brand palette but with a dark colour scheme.
  ThemeData _buildDarkTheme() {
    return ThemeData(
      useMaterial3: true,
      brightness: Brightness.dark,
      colorScheme: ColorScheme.fromSeed(
        seedColor: RdcsTheme.primary,
        brightness: Brightness.dark,
        primary: RdcsTheme.primaryLight,
        secondary: RdcsTheme.accent,
      ),
      appBarTheme: const AppBarTheme(
        backgroundColor: Color(0xFF1F2937),
        foregroundColor: Colors.white,
        elevation: 0,
        scrolledUnderElevation: 1,
      ),
      cardTheme: CardTheme(
        color: const Color(0xFF1F2937),
        elevation: 0,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: const BorderSide(color: Color(0xFF374151), width: 1),
        ),
      ),
      elevatedButtonTheme: ElevatedButtonThemeData(
        style: ElevatedButton.styleFrom(
          backgroundColor: RdcsTheme.primaryLight,
          foregroundColor: Colors.white,
          minimumSize: const Size(120, 44),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
          ),
        ),
      ),
      outlinedButtonTheme: OutlinedButtonThemeData(
        style: OutlinedButton.styleFrom(
          foregroundColor: RdcsTheme.primaryLight,
          minimumSize: const Size(120, 44),
          side: const BorderSide(color: RdcsTheme.primaryLight),
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8),
          ),
        ),
      ),
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: const Color(0xFF1F2937),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0xFF374151)),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide: const BorderSide(color: Color(0xFF374151)),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(8),
          borderSide:
              const BorderSide(color: RdcsTheme.primaryLight, width: 2),
        ),
        contentPadding:
            const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      ),
    );
  }
}
