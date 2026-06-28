// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../core/config/config_model.dart';
import '../../core/config/config_provider.dart';

/// Settings screen with three tabbed sections: Security, Network, General.
///
/// All changes are persisted immediately via [configProvider].
class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen>
    with SingleTickerProviderStateMixin {
  late final TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 3, vsync: this);
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => context.go('/'),
        ),
        title: const Text('设置'),
        bottom: TabBar(
          controller: _tabController,
          tabs: const [
            Tab(icon: Icon(Icons.shield_outlined), text: '安全设置'),
            Tab(icon: Icon(Icons.wifi), text: '网络设置'),
            Tab(icon: Icon(Icons.settings_outlined), text: '通用设置'),
          ],
          labelColor: theme.colorScheme.primary,
          unselectedLabelColor: theme.colorScheme.onSurfaceVariant,
          indicatorColor: theme.colorScheme.primary,
        ),
      ),
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 600),
          child: TabBarView(
            controller: _tabController,
            children: const [
              _SecuritySettingsTab(),
              _NetworkSettingsTab(),
              _GeneralSettingsTab(),
            ],
          ),
        ),
      ),
    );
  }
}

// =============================================================================
// Tab 1: Security Settings
// =============================================================================

class _SecuritySettingsTab extends ConsumerStatefulWidget {
  const _SecuritySettingsTab();

  @override
  ConsumerState<_SecuritySettingsTab> createState() =>
      _SecuritySettingsTabState();
}

class _SecuritySettingsTabState extends ConsumerState<_SecuritySettingsTab> {
  final _formKey = GlobalKey<FormState>();
  final _oldPasswordController = TextEditingController();
  final _newPasswordController = TextEditingController();
  final _confirmPasswordController = TextEditingController();

  bool _showOldPassword = false;
  bool _showNewPassword = false;
  bool _showConfirmPassword = false;
  bool _totpEnabled = false;
  bool _showTotpQr = false;
  bool _allowRemoteConnection = true;
  double _autoRejectTimeout = 60;

  @override
  void dispose() {
    _oldPasswordController.dispose();
    _newPasswordController.dispose();
    _confirmPasswordController.dispose();
    super.dispose();
  }

  void _changePassword() {
    if (_formKey.currentState!.validate()) {
      // TODO: Call API to change password.
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('密码修改成功')),
      );
      _oldPasswordController.clear();
      _newPasswordController.clear();
      _confirmPasswordController.clear();
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Form(
        key: _formKey,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // ── Change password section ──────────────────────────
            Text('修改密码', style: theme.textTheme.titleLarge),
            const SizedBox(height: 16),
            TextFormField(
              controller: _oldPasswordController,
              obscureText: !_showOldPassword,
              decoration: InputDecoration(
                labelText: '当前密码',
                suffixIcon: IconButton(
                  icon: Icon(
                    _showOldPassword ? Icons.visibility_off : Icons.visibility,
                  ),
                  onPressed: () =>
                      setState(() => _showOldPassword = !_showOldPassword),
                ),
              ),
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return '请输入当前密码';
                }
                return null;
              },
            ),
            const SizedBox(height: 16),
            TextFormField(
              controller: _newPasswordController,
              obscureText: !_showNewPassword,
              decoration: InputDecoration(
                labelText: '新密码',
                suffixIcon: IconButton(
                  icon: Icon(
                    _showNewPassword ? Icons.visibility_off : Icons.visibility,
                  ),
                  onPressed: () =>
                      setState(() => _showNewPassword = !_showNewPassword),
                ),
              ),
              validator: (value) {
                if (value == null || value.isEmpty) {
                  return '请输入新密码';
                }
                if (value.length < 8) {
                  return '密码长度至少为 8 位';
                }
                return null;
              },
            ),
            const SizedBox(height: 16),
            TextFormField(
              controller: _confirmPasswordController,
              obscureText: !_showConfirmPassword,
              decoration: InputDecoration(
                labelText: '确认新密码',
                suffixIcon: IconButton(
                  icon: Icon(
                    _showConfirmPassword
                        ? Icons.visibility_off
                        : Icons.visibility,
                  ),
                  onPressed: () => setState(
                      () => _showConfirmPassword = !_showConfirmPassword),
                ),
              ),
              validator: (value) {
                if (value != _newPasswordController.text) {
                  return '两次输入的密码不一致';
                }
                return null;
              },
            ),
            const SizedBox(height: 16),
            Align(
              alignment: Alignment.centerRight,
              child: FilledButton(
                onPressed: _changePassword,
                child: const Text('修改密码'),
              ),
            ),

            const SizedBox(height: 32),
            const Divider(),
            const SizedBox(height: 24),

            // ── TOTP two-factor authentication ─────────────────
            Text('两步验证 (TOTP)', style: theme.textTheme.titleLarge),
            const SizedBox(height: 12),
            Row(
              children: [
                Icon(
                  _totpEnabled ? Icons.check_circle : Icons.cancel,
                  color: _totpEnabled
                      ? Theme.of(context).colorScheme.primary
                      : theme.colorScheme.error,
                  size: 20,
                ),
                const SizedBox(width: 8),
                Text(
                  _totpEnabled ? '已启用' : '未启用',
                  style: theme.textTheme.bodyLarge,
                ),
                const Spacer(),
                OutlinedButton(
                  onPressed: () {
                    setState(() {
                      if (_totpEnabled) {
                        _totpEnabled = false;
                        _showTotpQr = false;
                      } else {
                        _showTotpQr = true;
                      }
                    });
                  },
                  child: Text(_totpEnabled ? '禁用' : '启用'),
                ),
              ],
            ),
            if (_showTotpQr && !_totpEnabled) ...[
              const SizedBox(height: 16),
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Column(
                    children: [
                      // QR code placeholder
                      Container(
                        width: 180,
                        height: 180,
                        decoration: BoxDecoration(
                          color: theme.colorScheme.surfaceContainerHighest,
                          borderRadius: BorderRadius.circular(12),
                          border: Border.all(color: theme.colorScheme.outline),
                        ),
                        child: Center(
                          child: Column(
                            mainAxisAlignment: MainAxisAlignment.center,
                            children: [
                              Icon(
                                Icons.qr_code_2,
                                size: 80,
                                color: theme.colorScheme.onSurfaceVariant,
                              ),
                              const SizedBox(height: 8),
                              Text(
                                'QR Code',
                                style: theme.textTheme.bodySmall,
                              ),
                            ],
                          ),
                        ),
                      ),
                      const SizedBox(height: 16),
                      Text(
                        '使用身份验证器应用扫描二维码',
                        style: theme.textTheme.bodyMedium,
                      ),
                      const SizedBox(height: 12),
                      FilledButton(
                        onPressed: () {
                          setState(() {
                            _totpEnabled = true;
                            _showTotpQr = false;
                          });
                          ScaffoldMessenger.of(context).showSnackBar(
                            const SnackBar(content: Text('两步验证已启用')),
                          );
                        },
                        child: const Text('确认启用'),
                      ),
                    ],
                  ),
                ),
              ),
            ],

            const SizedBox(height: 32),
            const Divider(),
            const SizedBox(height: 24),

            // ── Device authorization ───────────────────────────
            Text('设备授权', style: theme.textTheme.titleLarge),
            const SizedBox(height: 12),
            SwitchListTile(
              title: const Text('允许远程连接'),
              subtitle: const Text('关闭后其他设备无法连接到本机'),
              value: _allowRemoteConnection,
              onChanged: (value) {
                setState(() => _allowRemoteConnection = value);
              },
              contentPadding: EdgeInsets.zero,
            ),
            const SizedBox(height: 8),
            Text(
              '自动拒绝超时: ${_autoRejectTimeout.round()} 秒',
              style: theme.textTheme.bodyMedium,
            ),
            Slider(
              value: _autoRejectTimeout,
              min: 10,
              max: 300,
              divisions: 29,
              label: '${_autoRejectTimeout.round()} 秒',
              onChanged: (value) {
                setState(() => _autoRejectTimeout = value);
              },
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('10 秒', style: theme.textTheme.bodySmall),
                Text('300 秒', style: theme.textTheme.bodySmall),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

// =============================================================================
// Tab 2: Network Settings
// =============================================================================

class _NetworkSettingsTab extends ConsumerStatefulWidget {
  const _NetworkSettingsTab();

  @override
  ConsumerState<_NetworkSettingsTab> createState() =>
      _NetworkSettingsTabState();
}

class _NetworkSettingsTabState extends ConsumerState<_NetworkSettingsTab> {
  final _formKey = GlobalKey<FormState>();
  late final TextEditingController _rendezvousController;
  late final TextEditingController _relayController;
  late final TextEditingController _apiController;
  late final TextEditingController _bitrateController;

  late bool _requireTls;
  late String _codec;
  late double _maxFps;
  late bool _hardwareAccel;

  static const _codecLabels = {
    'h264': 'H.264',
    'h265': 'H.265',
    'vp9': 'VP9',
    'av1': 'AV1',
  };

  @override
  void initState() {
    super.initState();
    final config = ref.read(configProvider);
    _rendezvousController =
        TextEditingController(text: config.server.rendezvousUrl);
    _relayController = TextEditingController(text: config.server.relayUrl);
    _apiController = TextEditingController(text: config.server.apiUrl);
    _bitrateController = TextEditingController(
      text: config.quality.bitrateKbps == 0
          ? ''
          : config.quality.bitrateKbps.toString(),
    );
    _requireTls = config.server.requireTls;
    _codec = config.quality.codec;
    _maxFps = config.quality.maxFps.toDouble();
    _hardwareAccel = config.quality.hardwareAccel;
  }

  @override
  void dispose() {
    _rendezvousController.dispose();
    _relayController.dispose();
    _apiController.dispose();
    _bitrateController.dispose();
    super.dispose();
  }

  String? _validateUrl(String? value) {
    if (value == null || value.isEmpty) return null; // Optional field.
    final uri = Uri.tryParse(value);
    if (uri == null || !uri.hasScheme || !uri.hasAuthority) {
      return '请输入有效的 URL (例如 https://example.com)';
    }
    return null;
  }

  void _saveServer() {
    if (!_formKey.currentState!.validate()) return;
    ref.read(configProvider.notifier).updateServer(ServerConfig(
          rendezvousUrl: _rendezvousController.text.trim(),
          relayUrl: _relayController.text.trim(),
          apiUrl: _apiController.text.trim(),
          requireTls: _requireTls,
        ));
  }

  void _saveQuality() {
    final bitrate = int.tryParse(_bitrateController.text.trim()) ?? 0;
    ref.read(configProvider.notifier).updateQuality(QualityConfig(
          codec: _codec,
          maxFps: _maxFps.round(),
          bitrateKbps: bitrate < 0 ? 0 : bitrate,
          hardwareAccel: _hardwareAccel,
        ));
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Form(
        key: _formKey,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // ── Server configuration ───────────────────────────
            Text('服务器配置', style: theme.textTheme.titleLarge),
            const SizedBox(height: 16),
            TextFormField(
              controller: _rendezvousController,
              decoration: const InputDecoration(
                labelText: '信令服务器 (Rendezvous URL)',
                hintText: 'https://rendezvous.example.com',
              ),
              validator: _validateUrl,
              onFieldSubmitted: (_) => _saveServer(),
            ),
            const SizedBox(height: 16),
            TextFormField(
              controller: _relayController,
              decoration: const InputDecoration(
                labelText: '中继服务器 (Relay URL)',
                hintText: 'https://relay.example.com',
              ),
              validator: _validateUrl,
              onFieldSubmitted: (_) => _saveServer(),
            ),
            const SizedBox(height: 16),
            TextFormField(
              controller: _apiController,
              decoration: const InputDecoration(
                labelText: '管理 API (API URL)',
                hintText: 'https://api.example.com',
              ),
              validator: _validateUrl,
              onFieldSubmitted: (_) => _saveServer(),
            ),
            const SizedBox(height: 16),
            SwitchListTile(
              title: const Text('要求 TLS 加密'),
              subtitle: const Text('所有连接必须使用 TLS 加密通道'),
              value: _requireTls,
              onChanged: (value) {
                setState(() => _requireTls = value);
                _saveServer();
              },
              contentPadding: EdgeInsets.zero,
            ),
            const SizedBox(height: 8),
            Align(
              alignment: Alignment.centerRight,
              child: OutlinedButton(
                onPressed: _saveServer,
                child: const Text('保存服务器配置'),
              ),
            ),

            const SizedBox(height: 32),
            const Divider(),
            const SizedBox(height: 24),

            // ── Connection quality ─────────────────────────────
            Text('连接质量', style: theme.textTheme.titleLarge),
            const SizedBox(height: 16),

            // Codec dropdown
            DropdownButtonFormField<String>(
              value: _codec,
              decoration: const InputDecoration(labelText: '视频编码'),
              items: _codecLabels.entries
                  .map((e) => DropdownMenuItem(
                        value: e.key,
                        child: Text(e.value),
                      ))
                  .toList(),
              onChanged: (value) {
                if (value != null) {
                  setState(() => _codec = value);
                  _saveQuality();
                }
              },
            ),
            const SizedBox(height: 20),

            // Max FPS slider
            Text(
              '最大帧率: ${_maxFps.round()} FPS',
              style: theme.textTheme.bodyMedium,
            ),
            Slider(
              value: _maxFps,
              min: 30,
              max: 120,
              divisions: 3,
              label: '${_maxFps.round()} FPS',
              onChanged: (value) {
                setState(() => _maxFps = value);
              },
              onChangeEnd: (_) => _saveQuality(),
            ),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('30', style: theme.textTheme.bodySmall),
                Text('60', style: theme.textTheme.bodySmall),
                Text('90', style: theme.textTheme.bodySmall),
                Text('120', style: theme.textTheme.bodySmall),
              ],
            ),
            const SizedBox(height: 12),

            // Hardware acceleration toggle
            SwitchListTile(
              title: const Text('硬件加速'),
              subtitle: const Text('使用 GPU 加速编解码 (如可用)'),
              value: _hardwareAccel,
              onChanged: (value) {
                setState(() => _hardwareAccel = value);
                _saveQuality();
              },
              contentPadding: EdgeInsets.zero,
            ),

            const SizedBox(height: 24),
            const Divider(),
            const SizedBox(height: 24),

            // ── Bandwidth ──────────────────────────────────────
            Text('带宽设置', style: theme.textTheme.titleLarge),
            const SizedBox(height: 16),
            TextFormField(
              controller: _bitrateController,
              decoration: const InputDecoration(
                labelText: '码率 (kbps)',
                hintText: '0 = 自动',
                suffixText: 'kbps',
              ),
              keyboardType: TextInputType.number,
              inputFormatters: [FilteringTextInputFormatter.digitsOnly],
              onFieldSubmitted: (_) => _saveQuality(),
            ),
            const SizedBox(height: 8),
            Text(
              '设为 0 将根据网络状况自动调整码率',
              style: theme.textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }
}

// =============================================================================
// Tab 3: General Settings
// =============================================================================

class _GeneralSettingsTab extends ConsumerStatefulWidget {
  const _GeneralSettingsTab();

  @override
  ConsumerState<_GeneralSettingsTab> createState() =>
      _GeneralSettingsTabState();
}

class _GeneralSettingsTabState extends ConsumerState<_GeneralSettingsTab> {
  late String _locale;
  late bool _startMinimized;
  late bool _autoStart;
  late bool _autoReconnect;
  late bool _syncClipboard;
  late bool _enableRemoteAudio;

  static const _localeLabels = {
    'zh-CN': '简体中文',
    'en-US': 'English',
  };

  @override
  void initState() {
    super.initState();
    final config = ref.read(configProvider);
    _locale = config.general.locale;
    _startMinimized = config.general.startMinimized;
    _autoReconnect = config.general.autoReconnect;
    _syncClipboard = config.general.syncClipboard;
    _enableRemoteAudio = config.general.enableRemoteAudio;
    // Auto-start is a system-level setting; default to false.
    _autoStart = false;
  }

  void _saveGeneral() {
    ref.read(configProvider.notifier).updateGeneral(GeneralConfig(
          locale: _locale,
          startMinimized: _startMinimized,
          autoReconnect: _autoReconnect,
          syncClipboard: _syncClipboard,
          enableRemoteAudio: _enableRemoteAudio,
        ));
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(24),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // ── Language ─────────────────────────────────────────
          Text('语言', style: theme.textTheme.titleLarge),
          const SizedBox(height: 16),
          DropdownButtonFormField<String>(
            value: _locale,
            decoration: const InputDecoration(labelText: '显示语言'),
            items: _localeLabels.entries
                .map((e) => DropdownMenuItem(
                      value: e.key,
                      child: Text(e.value),
                    ))
                .toList(),
            onChanged: (value) {
              if (value != null) {
                setState(() => _locale = value);
                _saveGeneral();
              }
            },
          ),

          const SizedBox(height: 32),
          const Divider(),
          const SizedBox(height: 24),

          // ── Startup behavior ─────────────────────────────────
          Text('启动行为', style: theme.textTheme.titleLarge),
          const SizedBox(height: 12),
          SwitchListTile(
            title: const Text('启动时最小化'),
            subtitle: const Text('启动后直接最小化到系统托盘'),
            value: _startMinimized,
            onChanged: (value) {
              setState(() => _startMinimized = value);
              _saveGeneral();
            },
            contentPadding: EdgeInsets.zero,
          ),
          SwitchListTile(
            title: const Text('开机自启'),
            subtitle: const Text('系统启动时自动运行客户端'),
            value: _autoStart,
            onChanged: (value) {
              setState(() => _autoStart = value);
              // Auto-start is OS-level; would call platform channel.
            },
            contentPadding: EdgeInsets.zero,
          ),
          SwitchListTile(
            title: const Text('断线自动重连'),
            subtitle: const Text('网络恢复后自动重新连接'),
            value: _autoReconnect,
            onChanged: (value) {
              setState(() => _autoReconnect = value);
              _saveGeneral();
            },
            contentPadding: EdgeInsets.zero,
          ),

          const SizedBox(height: 32),
          const Divider(),
          const SizedBox(height: 24),

          // ── Clipboard ────────────────────────────────────────
          Text('剪贴板', style: theme.textTheme.titleLarge),
          const SizedBox(height: 12),
          SwitchListTile(
            title: const Text('同步剪贴板'),
            subtitle: const Text('在本地和远程设备之间同步剪贴板内容'),
            value: _syncClipboard,
            onChanged: (value) {
              setState(() => _syncClipboard = value);
              _saveGeneral();
            },
            contentPadding: EdgeInsets.zero,
          ),

          const SizedBox(height: 32),
          const Divider(),
          const SizedBox(height: 24),

          // ── Audio ────────────────────────────────────────────
          Text('音频', style: theme.textTheme.titleLarge),
          const SizedBox(height: 12),
          SwitchListTile(
            title: const Text('远程音频'),
            subtitle: const Text('播放远程设备的音频'),
            value: _enableRemoteAudio,
            onChanged: (value) {
              setState(() => _enableRemoteAudio = value);
              _saveGeneral();
            },
            contentPadding: EdgeInsets.zero,
          ),

          const SizedBox(height: 32),
          const Divider(),
          const SizedBox(height: 24),

          // ── About ────────────────────────────────────────────
          Text('关于', style: theme.textTheme.titleLarge),
          const SizedBox(height: 12),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _AboutRow(label: '版本', value: '0.1.0'),
                  const SizedBox(height: 8),
                  _AboutRow(label: '许可证', value: 'Apache 2.0'),
                  const SizedBox(height: 8),
                  _AboutRow(label: '项目主页', value: 'github.com/rdcs'),
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          SizedBox(
            width: double.infinity,
            child: OutlinedButton.icon(
              onPressed: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  const SnackBar(content: Text('当前已是最新版本')),
                );
              },
              icon: const Icon(Icons.system_update),
              label: const Text('检查更新'),
            ),
          ),
        ],
      ),
    );
  }
}

/// Simple key-value row used in the About card.
class _AboutRow extends StatelessWidget {
  const _AboutRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Row(
      children: [
        SizedBox(
          width: 80,
          child: Text(label, style: theme.textTheme.bodyMedium),
        ),
        Expanded(
          child: Text(value, style: theme.textTheme.bodyLarge),
        ),
      ],
    );
  }
}
