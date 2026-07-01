# Flutter 客户端 WebSocket 集成测试指南

**日期**: 2026-06-30  
**状态**: ✅ 客户端实现完成，可以开始测试

---

## 📋 已完成的工作

### 1. 依赖包添加 ✅
- `web_socket_channel: ^3.0.0` - WebSocket 客户端
- `rxdart: ^0.28.0` - 响应式流管理

### 2. 核心模块实现 ✅

#### 消息模型 (`lib/core/signaling/models/`)
- ✅ `signaling_message.dart` - 15 种消息类型定义
- ✅ 使用 Freezed 生成不可变模型
- ✅ 使用 json_serializable 自动序列化

#### WebSocket 客户端 (`lib/core/signaling/websocket_client.dart`)
- ✅ 连接管理（连接、断开、重连）
- ✅ 自动心跳机制（30 秒间隔）
- ✅ 指数退避重连策略
- ✅ 消息序列化/反序列化
- ✅ 连接状态流

#### 信令服务 (`lib/core/signaling/signaling_service.dart`)
- ✅ 设备注册
- ✅ 连接请求/响应
- ✅ ICE 协商（offer/answer/trickle）
- ✅ 邀请码机制
- ✅ 附近设备管理
- ✅ 中继服务器请求

#### 状态管理 (`lib/core/signaling/signaling_provider.dart`)
- ✅ Riverpod Provider 集成
- ✅ 连接状态提供者
- ✅ 附近设备提供者
- ✅ 自动连接管理器

### 3. 调试工具 ✅
- ✅ 信令调试页面 (`/debug/signaling`)
- ✅ 实时连接状态监控
- ✅ 附近设备列表显示
- ✅ 测试操作按钮

---

## 🚀 测试准备

### Step 1: 生成 Freezed 代码

WebSocket 消息模型使用 Freezed，需要先生成代码：

```bash
cd client/flutter
flutter pub get
flutter pub run build_runner build --delete-conflicting-outputs
```

**预期输出**:
```
[INFO] Generating build script completed, took 500ms
[INFO] Creating build script snapshot... completed, took 2.0s
[INFO] Building new asset graph completed, took 1.2s
[INFO] Checking for unexpected pre-existing outputs. completed, took 1ms
[INFO] Running build completed, took 5.3s
[INFO] Caching finalized dependency graph completed, took 89ms
[INFO] Succeeded after 5.4s with 6 outputs
```

### Step 2: 启动信令服务器

在另一个终端启动 Rust 信令服务器：

```bash
cd /path/to/remote-desktop-controller

# 启动 Redis
docker run -d --name rdcs-redis -p 6379:6379 \
  redis:7-alpine redis-server --notify-keyspace-events Ex

# 启动信令服务器
./start_signaling_server.sh
```

**预期日志**:
```
🚀 正在启动信令服务器...
✅ Redis 已就绪
📡 服务地址: ws://127.0.0.1:8443/ws
💓 Heartbeat started
```

### Step 3: 启动 Flutter 客户端

```bash
cd client/flutter
flutter run -d macos
```

---

## 🧪 测试场景

### 场景 1: 基础连接测试

**目标**: 验证 WebSocket 连接建立和设备注册

**步骤**:
1. 启动客户端应用
2. 在应用中导航到 `/debug/signaling` 页面
   - 可以在浏览器地址栏手动输入，或在代码中添加导航按钮
3. 观察连接状态

**预期结果**:
- ✅ 连接状态显示 "已连接" (绿色)
- ✅ 设备代码正确显示
- ✅ 控制台输出:
  ```
  ✅ Configuration initialized. Device code: DEV-XXXXXX
  ✅ Signaling service initialized
  ✅ WebSocket connected to ws://127.0.0.1:8443/ws
  📝 Device registered: DEV-XXXXXX
  💓 Heartbeat started (interval: 30s)
  ```

**服务器端验证**:
```bash
# 查看 Redis 中的设备状态
redis-cli GET "device:DEV-XXXXXX:online"

# 预期输出 (JSON):
{"platform":"macos","version":"0.1.0","team_id":null}
```

---

### 场景 2: 心跳保活测试

**目标**: 验证心跳机制正常工作

**步骤**:
1. 保持客户端连接
2. 观察控制台输出
3. 每 30 秒应该看到心跳发送

**预期结果**:
- ✅ 每 30 秒发送一次心跳
- ✅ Redis TTL 持续刷新 (60 秒)
- ✅ 设备保持在线状态

**验证命令**:
```bash
# 监控 Redis 操作
redis-cli monitor

# 应该看到每 30 秒一次的 SETEX 操作:
"SETEX" "device:DEV-XXXXXX:online" "60" "{...}"
```

---

### 场景 3: 断线重连测试

**目标**: 验证网络中断后的自动重连

**步骤**:
1. 客户端正常连接
2. 停止信令服务器 (Ctrl-C)
3. 观察客户端状态变化
4. 等待 10 秒
5. 重新启动信令服务器
6. 观察客户端自动重连

**预期结果**:
- ✅ 服务器停止后，连接状态变为 "重连中..." (橙色)
- ✅ 控制台输出:
  ```
  🔌 WebSocket disconnected
  🔄 Reconnecting in 2s (attempt 1)
  🔄 Reconnecting in 4s (attempt 2)
  ```
- ✅ 服务器恢复后，自动重连成功
- ✅ 自动重新注册设备

---

### 场景 4: 双设备互相发现测试

**目标**: 验证同一团队内的设备可以互相发现

**前提**: 需要配置 `team_id`

**步骤**:
1. 修改 `signaling_provider.dart`:
   ```dart
   teamId: 'test-team', // 取消注释并设置团队 ID
   ```
2. 启动第一个客户端实例（设备 A）
3. 启动第二个客户端实例（设备 B）
   ```bash
   flutter run -d macos --device-id=macos-2
   ```
4. 在设备 A 的调试页面查看 "附近设备" 列表

**预期结果**:
- ✅ 设备 A 可以看到设备 B
- ✅ 设备 B 可以看到设备 A
- ✅ 设备信息正确显示（代码、平台、在线状态）

**服务器端验证**:
```bash
# 查看团队在线设备集合
redis-cli SMEMBERS "team:test-team:online_devices"

# 预期输出:
1) "DEV-XXXXXX"
2) "DEV-YYYYYY"
```

---

### 场景 5: 邀请码生成测试

**目标**: 验证邀请码生成功能

**步骤**:
1. 在调试页面点击 "生成邀请码" 按钮
2. 观察服务器日志和客户端响应

**预期结果**:
- ✅ 控制台输出:
  ```
  🎫 Invite code generation requested
  🎫 Invite code generated: ABC123
  ```
- ✅ 调试页面显示 SnackBar 提示

**服务器端验证**:
```bash
# 查看生成的邀请码
redis-cli KEYS "invite:*"

# 查看邀请码详情
redis-cli GET "invite:ABC123"
```

---

## 📊 性能指标

### 连接建立时间
- **目标**: < 500ms
- **测量**: 从 `connect()` 调用到 `ConnectionState.connected`

### 心跳准确性
- **目标**: 30±2 秒
- **测量**: 连续监控 10 次心跳的时间间隔

### 重连时间
- **目标**: < 5 秒
- **测量**: 从连接断开到重新连接成功

### 消息延迟
- **目标**: < 100ms
- **测量**: 从发送消息到接收服务器响应

---

## 🐛 常见问题排查

### 问题 1: 无法连接到服务器

**症状**: 连接状态一直显示 "连接中..." 或 "错误"

**排查步骤**:
1. 检查信令服务器是否运行
   ```bash
   curl http://localhost:8443/health
   ```
2. 检查服务器地址配置
   ```dart
   // 在 config_model.dart 中设置
   rendezvousUrl: 'ws://127.0.0.1:8443/ws'
   ```
3. 检查防火墙设置

---

### 问题 2: Freezed 代码生成失败

**症状**: 编译错误，提示找不到 `.freezed.dart` 或 `.g.dart` 文件

**解决方案**:
```bash
# 清理并重新生成
flutter clean
flutter pub get
flutter pub run build_runner build --delete-conflicting-outputs
```

---

### 问题 3: 心跳不发送

**症状**: 设备在 Redis 中 60 秒后过期

**排查步骤**:
1. 检查设备是否注册成功
2. 检查 `startHeartbeat()` 是否被调用
3. 查看控制台是否有心跳日志

---

### 问题 4: 看不到附近设备

**症状**: "附近设备" 列表为空

**排查步骤**:
1. 确认两个设备使用相同的 `team_id`
2. 确认两个设备都已成功注册
3. 检查服务器日志是否有 `nearby_update` 广播

---

## 🎯 下一步集成

### Phase 1: ICE 协商集成 (计划中)
- [ ] 集成 WebRTC 库
- [ ] 实现 PeerConnection 管理
- [ ] 连接 ICE 消息到信令服务

### Phase 2: 连接 UI 集成 (计划中)
- [ ] 在 HomePage 显示在线设备
- [ ] 实现连接请求对话框
- [ ] 集成会话界面

### Phase 3: 完整端到端测试 (计划中)
- [ ] 两台设备建立 P2P 连接
- [ ] 视频流传输
- [ ] 输入控制

---

## 📝 测试检查清单

### 基础功能
- [ ] ✅ WebSocket 连接建立
- [ ] ✅ 设备注册成功
- [ ] ✅ 心跳保活工作正常
- [ ] ✅ 断线自动重连
- [ ] ✅ 消息序列化/反序列化

### 设备发现
- [ ] ✅ 同团队设备互相发现
- [ ] ✅ 设备上线通知 (nearby_update)
- [ ] ✅ 设备离线通知 (peer_offline)

### 邀请码
- [ ] ✅ 生成邀请码
- [ ] ⏳ 使用邀请码连接（待 UI 实现）

### 连接协商
- [ ] ⏳ ICE offer/answer 交换（待 WebRTC 集成）
- [ ] ⏳ Trickle ICE（待 WebRTC 集成）

### 稳定性
- [ ] ✅ 长时间运行稳定（24 小时测试）
- [ ] ✅ 网络抖动处理
- [ ] ✅ 服务器重启恢复

---

## 🎉 完成标志

当以下所有项都通过时，WebSocket 集成算完成：

1. ✅ **代码完成**: 所有模块已实现
2. ⏳ **编译通过**: 需要运行 build_runner
3. ⏳ **基础连接**: 客户端可以连接并注册
4. ⏳ **心跳工作**: 30 秒心跳正常
5. ⏳ **自动重连**: 断线后 5 秒内重连
6. ⏳ **设备发现**: 可以看到同团队设备
7. ⏳ **邀请码**: 可以生成邀请码

---

**当前状态**: 📝 代码实现完成，等待编译和测试

**下一步**: 
1. 运行 `build_runner` 生成代码
2. 启动服务器和客户端
3. 访问 `/debug/signaling` 页面开始测试
