// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:tray_manager/tray_manager.dart';
import 'package:window_manager/window_manager.dart';

/// Represents the current connection status shown in the tray icon.
enum TrayStatus {
  /// No active session.
  idle,

  /// Attempting to establish a connection.
  connecting,

  /// This machine is controlling a remote device.
  controlling,

  /// A remote user is controlling this machine.
  beingControlled,
}

/// Recent connection entry displayed in the tray submenu.
class RecentConnection {
  const RecentConnection({
    required this.deviceCode,
    required this.deviceName,
  });

  final String deviceCode;
  final String deviceName;
}

/// Manages the system tray icon, context menu, and basic window
/// show/hide behaviour for the RDCS desktop client.
///
/// Lifecycle:
///   1. Call [init] once after `windowManager.ensureInitialized()`.
///   2. Call [updateStatus] whenever the connection state changes.
///   3. Call [dispose] (or let the Riverpod provider dispose it)
///      when the application exits.
class TrayService with TrayListener {
  TrayService();

  TrayStatus _status = TrayStatus.idle;
  bool _initialized = false;

  /// Cache of recent connections shown in the tray submenu.
  /// Updated externally by calling [setRecentConnections].
  List<RecentConnection> _recentConnections = const [];

  /// The icon path used for the default (idle) tray icon.
  ///
  /// On macOS the tray icon is a template image — the OS applies
  /// the correct colour automatically.  On other platforms the
  /// icon should contain its own colour information.
  static String get _iconPath {
    // Bundled as a Flutter asset; tray_manager requires an
    // absolute path so we resolve from the executable directory.
    final exeDir = File(Platform.resolvedExecutable).parent.path;

    if (Platform.isMacOS) {
      // macOS .app bundle: assets live inside Contents/Frameworks
      // or we can reference the asset via the data directory.
      // Using a relative path from the bundle works reliably.
      return 'assets/tray_icon.png';
    }

    return '$exeDir/data/flutter_assets/assets/tray_icon.png';
  }

  // ── Public API ──────────────────────────────────────────────────

  /// Initialises the system tray icon and registers event listeners.
  ///
  /// Must be called after `windowManager.ensureInitialized()` and
  /// after Flutter rendering is available (i.e. after `runApp`).
  Future<void> init() async {
    if (_initialized) return;

    trayManager.addListener(this);
    await trayManager.setIcon(_iconPath);
    await _rebuildMenu();

    _initialized = true;
  }

  /// Updates the tray icon to reflect the current connection status.
  ///
  /// The icon path stays the same; only the tooltip changes.
  /// On platforms that support badge overlays (macOS) a visual
  /// indicator is added for [TrayStatus.controlling] and
  /// [TrayStatus.beingControlled].
  Future<void> updateStatus(TrayStatus status) async {
    if (!_initialized) return;
    _status = status;
    await trayManager.setToolTip(_tooltipForStatus(status));
    await _rebuildMenu();
  }

  /// Replaces the list of recent connections shown in the tray
  /// context menu under "最近连接".
  Future<void> setRecentConnections(List<RecentConnection> connections) async {
    if (!_initialized) return;
    _recentConnections = connections;
    await _rebuildMenu();
  }

  /// Shows (focuses) the main application window.
  Future<void> showWindow() async {
    await windowManager.show();
    await windowManager.focus();
  }

  /// Hides the main application window (minimises to tray).
  Future<void> hideWindow() async {
    await windowManager.hide();
  }

  /// Releases tray resources and unregisters listeners.
  Future<void> dispose() async {
    if (!_initialized) return;
    trayManager.removeListener(this);
    await trayManager.destroy();
    _initialized = false;
  }

  // ── TrayListener callbacks ─────────────────────────────────────

  @override
  void onTrayIconMouseDown() {
    // Left-click toggles window visibility.
    _toggleWindow();
  }

  @override
  void onTrayIconRightMouseDown() {
    // Right-click shows the context menu (handled natively by
    // tray_manager when a context menu is set).
    trayManager.popUpContextMenu();
  }

  @override
  void onTrayMenuItemClick(MenuItem menuItem) {
    _handleMenuClick(menuItem.key);
  }

  // ── Internal ────────────────────────────────────────────────────

  /// Toggles window visibility: show if hidden, hide if visible.
  Future<void> _toggleWindow() async {
    final visible = await windowManager.isVisible();
    if (visible) {
      await hideWindow();
    } else {
      await showWindow();
    }
  }

  /// Rebuilds the tray context menu based on current state.
  Future<void> _rebuildMenu() async {
    final recentItems = _buildRecentSubmenu();

    final menu = Menu(
      items: [
        MenuItem(
          key: 'show',
          label: '打开 RDCS',
        ),
        MenuItem(
          key: 'recent',
          label: '最近连接',
          disabled: _recentConnections.isEmpty,
          submenu: recentItems.isNotEmpty ? Menu(items: recentItems) : null,
        ),
        MenuItem.separator(),
        MenuItem(
          key: 'toggle_theme',
          label: '切换主题',
        ),
        MenuItem.separator(),
        MenuItem(
          key: 'status',
          label: _statusLabel(),
          disabled: true,
        ),
        MenuItem.separator(),
        MenuItem(
          key: 'quit',
          label: '退出',
        ),
      ],
    );

    await trayManager.setContextMenu(menu);
  }

  /// Builds submenu items for recent connections.
  List<MenuItem> _buildRecentSubmenu() {
    if (_recentConnections.isEmpty) {
      return [
        const MenuItem(
          key: 'recent_empty',
          label: '无',
          disabled: true,
        ),
      ];
    }

    return _recentConnections.map((conn) {
      final label = conn.deviceName.isNotEmpty
          ? '${conn.deviceName} (${conn.deviceCode})'
          : conn.deviceCode;
      return MenuItem(
        key: 'recent_${conn.deviceCode}',
        label: label,
      );
    }).toList();
  }

  /// Routes a menu item click to the appropriate action.
  void _handleMenuClick(String? key) {
    if (key == null) return;

    switch (key) {
      case 'show':
        showWindow();

      case 'toggle_theme':
        // Theme toggle is handled by the app layer via Riverpod.
        // The TrayService emits intent through the callback so
        // the app can react without this service depending on
        // the theme module directly.
        onToggleTheme?.call();

      case 'quit':
        onQuitRequested?.call();

      default:
        // Recent connection items: key format is "recent_<deviceCode>"
        if (key.startsWith('recent_')) {
          final code = key.substring('recent_'.length);
          onRecentConnectionSelected?.call(code);
        }
    }
  }

  /// Returns a human-readable status label for the current tray status.
  String _statusLabel() {
    switch (_status) {
      case TrayStatus.idle:
        return '状态：空闲';
      case TrayStatus.connecting:
        return '状态：连接中…';
      case TrayStatus.controlling:
        return '状态：正在控制远程设备';
      case TrayStatus.beingControlled:
        return '状态：被远程控制中';
    }
  }

  /// Returns the tooltip text for a given tray status.
  String _tooltipForStatus(TrayStatus status) {
    switch (status) {
      case TrayStatus.idle:
        return 'RDCS 远程桌面 - 空闲';
      case TrayStatus.connecting:
        return 'RDCS 远程桌面 - 连接中…';
      case TrayStatus.controlling:
        return 'RDCS 远程桌面 - 正在控制远程设备';
      case TrayStatus.beingControlled:
        return 'RDCS 远程桌面 - 被远程控制中';
    }
  }

  // ── Callbacks set by the app layer ──────────────────────────────

  /// Called when the user selects "切换主题" from the tray menu.
  /// The app layer sets this to toggle between light and dark themes.
  void Function()? onToggleTheme;

  /// Called when the user selects "退出" from the tray menu.
  /// The app layer should show a confirmation dialog if there is
  /// an active session, then call `windowManager.destroy()` and
  /// exit the process.
  void Function()? onQuitRequested;

  /// Called when the user clicks a recent connection in the tray
  /// submenu. The argument is the device code string.
  void Function(String deviceCode)? onRecentConnectionSelected;
}

// ── Riverpod provider ──────────────────────────────────────────

/// Provides the [TrayService] singleton.
///
/// Initialise at app startup after the window manager:
/// ```dart
/// await ref.read(trayServiceProvider).init();
/// ```
final trayServiceProvider = Provider<TrayService>((ref) {
  final service = TrayService();
  ref.onDispose(() => service.dispose());
  return service;
});
