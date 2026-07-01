// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/config/config_provider.dart';
import '../../core/signaling/signaling_provider.dart';
import '../../core/signaling/websocket_client.dart' as ws;

/// Debug page for testing WebSocket signaling connection.
class SignalingDebugPage extends ConsumerWidget {
  const SignalingDebugPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final config = ref.watch(configProvider);
    final connectionState = ref.watch(connectionStateProvider);
    final nearbyDevices = ref.watch(nearbyDevicesProvider);
    final service = ref.watch(signalingServiceProvider);

    return Scaffold(
      appBar: AppBar(
        title: const Text('信令服务调试'),
        actions: [
          // Refresh button
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: () {
              service.disconnect();
              Future.delayed(const Duration(milliseconds: 500), () {
                service.connect();
              });
            },
            tooltip: '重新连接',
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: ListView(
          children: [
            // Configuration section
            _buildSection(
              title: '配置信息',
              children: [
                _buildInfoRow('设备代码', config.deviceCode),
                _buildInfoRow('设备名称', config.deviceName.isEmpty ? '未设置' : config.deviceName),
                _buildInfoRow('服务器地址', config.server.rendezvousUrl.isEmpty
                    ? 'ws://127.0.0.1:8443/ws (默认)'
                    : config.server.rendezvousUrl),
              ],
            ),

            const SizedBox(height: 24),

            // Connection state section
            _buildSection(
              title: '连接状态',
              children: [
                connectionState.when(
                  data: (state) => _buildConnectionState(state),
                  loading: () => const CircularProgressIndicator(),
                  error: (error, stack) => Text('错误: $error'),
                ),
              ],
            ),

            const SizedBox(height: 24),

            // Nearby devices section
            _buildSection(
              title: '附近设备',
              children: [
                nearbyDevices.when(
                  data: (devices) {
                    if (devices.isEmpty) {
                      return const Text('暂无在线设备');
                    }
                    return Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: devices.map((device) {
                        return Card(
                          child: ListTile(
                            leading: Icon(
                              device.online ? Icons.circle : Icons.circle_outlined,
                              color: device.online ? Colors.green : Colors.grey,
                            ),
                            title: Text(device.name),
                            subtitle: Text('${device.code} · ${device.platform}'),
                            trailing: device.online
                                ? const Chip(label: Text('在线'))
                                : const Chip(label: Text('离线')),
                          ),
                        );
                      }).toList(),
                    );
                  },
                  loading: () => const CircularProgressIndicator(),
                  error: (error, stack) => Text('错误: $error'),
                ),
              ],
            ),

            const SizedBox(height: 24),

            // Test actions section
            _buildSection(
              title: '测试操作',
              children: [
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    ElevatedButton.icon(
                      icon: const Icon(Icons.login),
                      label: const Text('重新注册'),
                      onPressed: () => service.register(),
                    ),
                    ElevatedButton.icon(
                      icon: const Icon(Icons.favorite),
                      label: const Text('发送心跳'),
                      onPressed: () {
                        service.register(); // Triggers heartbeat
                      },
                    ),
                    ElevatedButton.icon(
                      icon: const Icon(Icons.qr_code),
                      label: const Text('生成邀请码'),
                      onPressed: () {
                        service.generateInviteCode();
                        _showSnackBar(context, '已请求生成邀请码');
                      },
                    ),
                  ],
                ),
              ],
            ),

            const SizedBox(height: 24),

            // Server messages log
            _buildSection(
              title: '服务器消息',
              children: [
                const Text(
                  '监听服务器推送的消息（nearby_update, peer_offline 等）',
                  style: TextStyle(color: Colors.grey),
                ),
                const SizedBox(height: 8),
                StreamBuilder(
                  stream: service.messages,
                  builder: (context, snapshot) {
                    if (snapshot.hasData) {
                      final message = snapshot.data!;
                      return Text(
                        '最新消息: ${message.toString().substring(0, 100)}...',
                        style: const TextStyle(fontSize: 12),
                      );
                    }
                    return const Text('等待消息...');
                  },
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSection({
    required String title,
    required List<Widget> children,
  }) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              title,
              style: const TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.bold,
              ),
            ),
            const Divider(),
            ...children,
          ],
        ),
      ),
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4.0),
      child: Row(
        children: [
          SizedBox(
            width: 100,
            child: Text(
              '$label:',
              style: const TextStyle(fontWeight: FontWeight.w500),
            ),
          ),
          Expanded(
            child: Text(value),
          ),
        ],
      ),
    );
  }

  Widget _buildConnectionState(ws.WsConnectionState state) {
    IconData icon;
    Color color;
    String text;

    switch (state) {
      case ws.WsConnectionState.connected:
        icon = Icons.check_circle;
        color = Colors.green;
        text = '已连接';
        break;
      case ws.WsConnectionState.connecting:
        icon = Icons.sync;
        color = Colors.orange;
        text = '连接中...';
        break;
      case ws.WsConnectionState.reconnecting:
        icon = Icons.sync_problem;
        color = Colors.orange;
        text = '重连中...';
        break;
      case ws.WsConnectionState.disconnected:
        icon = Icons.cancel;
        color = Colors.grey;
        text = '已断开';
        break;
      case ws.WsConnectionState.error:
        icon = Icons.error;
        color = Colors.red;
        text = '错误';
        break;
    }

    return Row(
      children: [
        Icon(icon, color: color, size: 32),
        const SizedBox(width: 12),
        Text(
          text,
          style: TextStyle(
            fontSize: 18,
            fontWeight: FontWeight.bold,
            color: color,
          ),
        ),
      ],
    );
  }

  void _showSnackBar(BuildContext context, String message) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(message)),
    );
  }
}
