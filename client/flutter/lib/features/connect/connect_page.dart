// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../session/session_providers.dart';

/// Connect page — enter a remote device code to initiate a session.
class ConnectPage extends ConsumerStatefulWidget {
  const ConnectPage({super.key});

  @override
  ConsumerState<ConnectPage> createState() => _ConnectPageState();
}

class _ConnectPageState extends ConsumerState<ConnectPage> {
  final _codeController = TextEditingController();
  final _inviteController = TextEditingController();
  final _formKey = GlobalKey<FormState>();
  bool _isConnecting = false;

  @override
  void dispose() {
    _codeController.dispose();
    _inviteController.dispose();
    super.dispose();
  }

  Future<void> _onConnect() async {
    if (!(_formKey.currentState?.validate() ?? false)) return;

    setState(() {
      _isConnecting = true;
    });

    final code = _codeController.text.replaceAll(RegExp(r'\s'), '');
    final invite = _inviteController.text.trim();

    try {
      await ref.read(sessionProvider.notifier).connect(
            code,
            inviteCode: invite.isEmpty ? null : invite,
          );

      // Check state after connect attempt.
      final session = ref.read(sessionProvider);
      if (!mounted) return;

      if (session != null && session.state == SessionState.connected) {
        context.go('/session');
      } else if (session != null && session.state == SessionState.error) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('连接失败，请检查设备代码后重试'),
            backgroundColor: Color(0xFFEF4444),
          ),
        );
      } else {
        // Still connecting (waiting for server confirmation) — navigate
        // to session screen which will display the connecting state.
        context.go('/session');
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('连接出错: $e'),
            backgroundColor: const Color(0xFFEF4444),
          ),
        );
      }
    } finally {
      if (mounted) {
        setState(() {
          _isConnecting = false;
        });
      }
    }
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
        title: const Text('连接远程设备'),
      ),
      body: Center(
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 400),
          child: Padding(
            padding: const EdgeInsets.all(24),
            child: Form(
              key: _formKey,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Text(
                    '输入对方设备代码',
                    style: theme.textTheme.titleLarge,
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 24),
                  TextFormField(
                    controller: _codeController,
                    decoration: const InputDecoration(
                      labelText: '设备代码',
                      hintText: '123 456 789',
                    ),
                    keyboardType: TextInputType.number,
                    textAlign: TextAlign.center,
                    style: const TextStyle(
                      fontSize: 22,
                      letterSpacing: 3,
                    ),
                    enabled: !_isConnecting,
                    validator: (value) {
                      final digits =
                          value?.replaceAll(RegExp(r'\s'), '') ?? '';
                      if (digits.length != 9) {
                        return '请输入9位设备代码';
                      }
                      return null;
                    },
                  ),
                  const SizedBox(height: 16),
                  TextFormField(
                    controller: _inviteController,
                    decoration: const InputDecoration(
                      labelText: '邀请码(选填)',
                      hintText: '对方分享的一次性口令',
                    ),
                    textAlign: TextAlign.center,
                    enabled: !_isConnecting,
                  ),
                  const SizedBox(height: 24),
                  ElevatedButton(
                    onPressed: _isConnecting ? null : _onConnect,
                    child: _isConnecting
                        ? const SizedBox(
                            height: 20,
                            width: 20,
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
          ),
        ),
      ),
    );
  }
}
