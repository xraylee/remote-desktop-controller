# Flutter 客户端 WebSocket 集成方案

**日期**: 2026-06-30  
**目标**: 实现与 rdcs-signaling 服务器的 WebSocket 通信

---

## 📋 需求分析

### 服务端协议（已确认）

根据服务端代码分析，信令服务器支持以下消息类型：

#### 客户端 → 服务器

1. **register** - 设备注册
   ```json
   {
     "type": "register",
     "device_code": "DEV-MACOS-001",
     "platform": "macos",
     "version": "0.1.0",
     "team_id": "team-test"  // 可选
   }
   ```

2. **heartbeat** - 心跳保活
   ```json
   {
     "type": "heartbeat",
     "device_code": "DEV-MACOS-001",
     "ts": 1719734400
   }
   ```

3. **connect_request** - 连接请求
   ```json
   {
     "type": "connect_request",
     "from_code": "DEV-MACOS-001",
     "to_code": "DEV-MACOS-002",
     "invite_code": "ABC123"  // 可选
   }
   ```

4. **connect_response** - 连接响应
   ```json
   {
     "type": "connect_response",
     "accepted": true,
     "session_id": "uuid",
     "from_code": "DEV-MACOS-001"
   }
   ```

5. **ice_offer** - ICE Offer
   ```json
   {
     "type": "ice_offer",
     "session_id": "uuid",
     "sdp": "...",
     "candidates": [...]
   }
   ```

6. **ice_answer** - ICE Answer
7. **ice_trickle** - Trickle ICE
8. **relay_request** - 请求中继服务器
9. **generate_invite** - 生成邀请码
10. **use_invite** - 使用邀请码

#### 服务器 → 客户端

1. **nearby_update** - 附近设备更新
   ```json
   {
     "type": "nearby_update",
     "devices": [
       {
         "code": "DEV-MACOS-002",
         "name": "办公室 Mac",
         "platform": "macos",
         "online": true
       }
     ]
   }
   ```

2. **peer_offline** - 对端离线
3. **relay_assigned** - 中继服务器分配
4. **invite_generated** - 邀请码已生成
5. **invite_result** - 邀请码使用结果
6. **error** - 错误响应

---

## 🏗️ 实现架构

### 层次结构

```
lib/core/signaling/
├── models/
│   ├── signaling_message.dart       # 消息类型定义（freezed）
│   ├── device_info.dart             # 设备信息模型
│   └── ice_candidate.dart           # ICE 候选模型
├── websocket_client.dart            # WebSocket 连接管理
├── signaling_service.dart           # 信令服务（业务逻辑）
└── signaling_provider.dart          # Riverpod 状态管理
```

### 关键组件

1. **WebSocketClient** - 底层 WebSocket 连接
   - 连接管理（连接、断开、重连）
   - 消息发送/接收
   - 错误处理
   - 心跳机制

2. **SignalingService** - 业务逻辑层
   - 设备注册
   - 心跳保活
   - 连接协商
   - 消息路由

3. **SignalingProvider** - 状态管理
   - 连接状态
   - 设备列表
   - 会话状态
   - 事件流

---

## 🔧 技术选型

### 依赖包

需要添加到 `pubspec.yaml`:

```yaml
dependencies:
  web_socket_channel: ^3.0.0  # WebSocket 客户端
  freezed_annotation: ^2.4.0   # 已有
  json_annotation: ^4.9.0      # 已有
  rxdart: ^0.28.0              # 事件流管理

dev_dependencies:
  freezed: ^2.5.0              # 已有
  json_serializable: ^6.7.0    # 已有
```

### 状态管理策略

使用 Riverpod 的 `StreamNotifierProvider` 管理 WebSocket 连接状态和消息流。

---

## 📝 实现计划

### Phase 1: 基础 WebSocket 连接（优先）

- [x] 分析需求
- [ ] 添加依赖包
- [ ] 实现消息模型（freezed + json_serializable）
- [ ] 实现 WebSocketClient
- [ ] 实现心跳机制
- [ ] 实现重连逻辑

### Phase 2: 信令服务

- [ ] 实现 SignalingService
- [ ] 设备注册流程
- [ ] 连接协商流程
- [ ] 邀请码机制

### Phase 3: 状态管理

- [ ] 实现 SignalingProvider
- [ ] 集成到主应用
- [ ] UI 绑定

### Phase 4: 测试

- [ ] 单元测试
- [ ] 集成测试
- [ ] 端到端测试

---

## 🎯 核心挑战

### 1. 连接状态管理

**挑战**: WebSocket 连接不稳定，需要优雅处理断线重连

**方案**:
- 实现指数退避重连策略
- 在重连成功后自动重新注册设备
- 维护离线消息队列

### 2. 心跳机制

**挑战**: 服务器要求 60 秒内必须发送心跳

**方案**:
- 使用 `Timer.periodic` 每 30 秒发送心跳
- 监听连接状态，断线时停止心跳
- 重连后立即发送心跳

### 3. 消息序列化

**挑战**: 10+ 种消息类型，手动解析易错

**方案**:
- 使用 `freezed` 生成不可变模型
- 使用 `json_serializable` 自动序列化
- 统一错误处理

### 4. 并发会话管理

**挑战**: 可能同时进行多个连接协商

**方案**:
- 使用 `session_id` 区分不同会话
- 维护会话状态映射表
- 实现超时清理机制

---

## 🚀 快速开始

### 配置服务器地址

在 `ServerConfig` 中配置（已存在）:

```dart
ServerConfig(
  rendezvousUrl: 'ws://127.0.0.1:8443/ws',
  apiUrl: 'http://127.0.0.1:8080',
)
```

### 初始化流程

```dart
// main.dart 中初始化
final signaling = container.read(signalingServiceProvider);
await signaling.connect();
```

### 监听设备列表

```dart
// 在 UI 中使用
ref.watch(nearbyDevicesProvider)
```

---

## 📊 预期效果

### 成功指标

- ✅ WebSocket 连接稳定
- ✅ 设备注册成功率 > 99%
- ✅ 心跳间隔准确（30±2 秒）
- ✅ 重连时间 < 5 秒
- ✅ 消息往返延迟 < 100ms

### 测试场景

1. **正常连接** - 启动后自动连接并注册
2. **网络中断** - 断网 10 秒后恢复，自动重连
3. **服务器重启** - 服务器重启后，客户端自动重连
4. **并发连接** - 同时向 3 台设备发起连接请求
5. **长时间运行** - 运行 24 小时保持连接稳定

---

## 🔍 调试支持

### 日志输出

```dart
// 开启详细日志
SignalingService(
  logLevel: LogLevel.debug,
)
```

### 状态监控

```dart
// 监听连接状态
signalingService.connectionState.listen((state) {
  print('Connection state: $state');
});
```

### 消息追踪

```dart
// 记录所有消息
signalingService.onMessage.listen((msg) {
  print('Received: ${msg.type}');
});
```

---

## 📚 参考资料

- 服务端协议: `crates/rdcs-signaling/src/ws/message.rs`
- 服务端处理器: `crates/rdcs-signaling/src/handlers/`
- 现有配置: `client/flutter/lib/core/config/config_model.dart`

---

**下一步**: 开始实现 Phase 1 - 基础 WebSocket 连接
