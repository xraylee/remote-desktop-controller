# RDCS 技术架构设计文档

**版本**：v1.0
**日期**：2026-06-25
**状态**：草稿 — 待审核
**方案选择**：方案 A — Flutter + Rust + libwebrtc

---

## 1. 系统架构总览

### 1.1 技术栈选型

| 组件 | 技术选型 | 选型理由 |
|------|---------|---------|
| 客户端 UI | Flutter (Dart) | 一套代码覆盖 macOS/Windows/Linux/iOS/Android，自绘引擎保证 UI 一致性 |
| 客户端核心引擎 | Rust (librdcs_core) | 内存安全、零成本抽象、C ABI 兼容 FFI、tokio 异步运行时 |
| FFI 桥接 | dart:ffi + Rust C ABI | Rust 导出 C 函数 → Dart DynamicLibrary 加载；ReceivePort 异步回调 |
| 信令服务 | Rust (tokio + axum) | 高并发 WebSocket 长连接，与核心引擎共享 Rust 生态 |
| 管理 API | Go (chi router) | CRUD 操作开发效率高，生态成熟（ORM、中间件） |
| Web 控制台 | React + TypeScript + Tailwind | SPA 组件生态丰富，TanStack Query 数据管理 |
| 数据库 | PostgreSQL | 关系型数据（用户、设备、审计日志） |
| 缓存 | Redis | 在线状态、会话缓存、Pub/Sub 消息 |
| 文件存储 | MinIO (S3 兼容) | 会话录制文件、文件传输暂存 |
| 反向代理 | Caddy | 自动 Let's Encrypt TLS，零配置 HTTPS |
| 视频传输 | WebRTC (libwebrtc) | PRD 强制要求；内置 SCTP、拥塞控制、NAT 穿透 |
| 加密 | NaCl (libsodium) | XSalsa20-Poly1305 AEAD + X25519 密钥交换 |
| 部署 | Docker Compose | MVP 官方运营；V2.0 支持客户自部署 |

### 1.2 系统组件图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Flutter UI Layer (Dart)                             │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────┐  ┌──────────┐     │
│  │  主界面       │  │  远程会话界面 │  │ 被控端界面  │  │ 设置面板 │     │
│  └──────┬───────┘  └──────┬───────┘  └──────┬─────┘  └────┬─────┘     │
│         └─────────────────┼─────────────────┼──────────────┘           │
│              Riverpod State Management                                 │
└───────────────────────────┼────────────────────────────────────────────┘
                            │ Dart FFI
┌───────────────────────────┼────────────────────────────────────────────┐
│              Bridge Layer (Dart FFI ↔ Rust FFI)                        │
│  bridge.rs (12 个 C ABI 函数)  +  bridge.dart (DynamicLibrary 封装)    │
└───────────────────────────┼────────────────────────────────────────────┘
                            │
┌───────────────────────────┼────────────────────────────────────────────┐
│              Core Engine (Rust) — librdcs_core                         │
│  ┌────────────┐ ┌────────────┐ ┌────────────┐ ┌───────────────┐      │
│  │ Connection │ │  Codec     │ │  Crypto    │ │   Transport   │      │
│  │  Manager   │ │  Pipeline  │ │   Layer    │ │   Layer       │      │
│  └────────────┘ └────────────┘ └────────────┘ └───────────────┘      │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │              Platform Abstraction Layer (Rust trait)             │  │
│  │  ScreenCapture · InputInjector · AudioCapture · SystemNotify    │  │
│  └─────────────────────────────────────────────────────────────────┘  │
└───────────────────────────┬────────────────────────────────────────────┘
                            │
┌───────────────────────────┼────────────────────────────────────────────┐
│              Platform Implementations (per-OS Rust crate)              │
│  rdcs-macos (MVP)  ·  rdcs-windows (V1.1)  ·  rdcs-linux (V1.1)      │
│  rdcs-ios (V2.0)   ·  rdcs-android (V2.0)                             │
└────────────────────────────────────────────────────────────────────────┘

                            │ Network
   ┌────────────────────────┼────────────────────────┐
   │                        │                        │
┌──▼──────────┐   ┌────────▼────────┐   ┌──────────▼──────────┐
│  信令服务    │   │   中转节点       │   │   Web 控制台         │
│  (Rust/Axum) │   │  (Rust/Tokio)   │   │  React + Go API     │
│              │   │                 │   │                     │
│ · 设备注册   │   │ · 零知识 UDP 转发│   │ · 仪表盘            │
│ · ICE 协商   │   │ · 负载均衡      │   │ · 设备/成员管理      │
│ · 在线状态   │   │ · 健康监控      │   │ · 连接记录/审计      │
│ · 邀请码验证 │   │                 │   │ · 会话录制管理       │
└──────┬───────┘   └─────────────────┘   └──────────┬──────────┘
       │                                            │
  ┌────▼────┐  ┌──────────┐  ┌───────────────┐     │
  │  Redis  │  │PostgreSQL│  │ MinIO / S3    │◄────┘
  └─────────┘  └──────────┘  └───────────────┘
```

### 1.3 三层连接架构

| 层级 | 场景 | 连接方式 | 延迟预期 | 成本 |
|------|------|---------|---------|------|
| **L1** | 同一局域网 | mDNS/Bonjour 自动发现 + TCP/UDP 直连 | < 5ms | 零 |
| **L2** | 跨网络，非对称 NAT | STUN/ICE UDP 打洞 | < 30ms | 零 |
| **L3** | 跨网络，对称 NAT 或穿透失败 | 官方中继节点 UDP 转发 | < 100ms | 运营承担 |

客户端在 500ms 内自动选择最优路径：L1 → L2 → L3 依次尝试。用户侧仅显示"已连接"。

---

## 2. 客户端架构

### 2.1 四层分层

| 层级 | 模块 | 职责 | 跨平台复用率 |
|------|------|------|-------------|
| **UI 层** | Flutter Widgets | 主界面、远程会话界面、被控端界面、设置面板 | 100% |
| | 状态管理 (Riverpod) | ConnectionState, SessionState, DeviceStore | 100% |
| **Bridge 层** | bridge.rs | C ABI 导出 12 个函数 | 100% |
| | bridge.dart | DynamicLibrary 加载 + ReceivePort 事件流 | 100% |
| **Core 层** | Connection Manager | NAT 探测、ICE 协商、路径选择、自动重连 | ~95% |
| | Codec Pipeline | 屏幕捕获接口、内容分析、自适应编码/解码 | ~70%* |
| | Crypto Layer | X25519 密钥交换、XSalsa20-Poly1305 加密 | 100% |
| | Transport Layer | 帧分包/重组、GCC 拥塞控制、NACK 重传、FEC | ~98% |
| | File / Clipboard | 文件分块传输、断点续传、剪贴板双向同步 | ~90% |
| **Platform 层** | 屏幕捕获 | macOS: ScreenCaptureKit · Win: DXGI · Linux: PipeWire | 0% (每平台独立) |
| | 输入注入 | macOS: CGEvent · Win: SendInput · Linux: uinput | 0% |
| | 系统集成 | 菜单栏/托盘、通知、剪贴板监听、自启动 | 0% |

*编解码逻辑跨平台共享，仅屏幕捕获入口因平台 API 不同而独立。

### 2.2 FFI 接口契约

| FFI 函数 | 参数 | 返回值 | 说明 |
|---------|------|--------|------|
| `rdcs_engine_create` | config_json | *mut EngineHandle | 创建核心引擎实例 |
| `rdcs_engine_destroy` | handle | void | 释放引擎及所有资源 |
| `rdcs_start_capture` | handle, config_json | i32 | 启动屏幕捕获 |
| `rdcs_stop_capture` | handle | i32 | 停止屏幕捕获 |
| `rdcs_connect` | handle, target_code | i32 | 向目标设备发起连接 |
| `rdcs_disconnect` | handle, session_id | i32 | 断开指定会话 |
| `rdcs_send_input` | handle, session_id, event_json | i32 | 发送鼠标/键盘事件 |
| `rdcs_send_file` | handle, session_id, path, dest | i32 | 发起文件传输 |
| `rdcs_send_message` | handle, session_id, text | i32 | 发送聊天消息 |
| `rdcs_set_quality` | handle, session_id, mode | i32 | 切换画质模式 |
| `rdcs_generate_invite` | handle | *const c_char | 生成 4 位邀请码 |
| `rdcs_register_callback` | handle, event_id, dart_fn_ptr | i32 | 注册 Dart 回调 |

### 2.3 Rust → Dart 事件流

| 事件 | 触发时机 | 载荷 |
|------|---------|------|
| `FrameReady` | 新帧解码完成 | session_id, texture_id, width, height |
| `ConnectionRequest` | 收到连接请求 | from_device_code, from_name |
| `ConnectionEstablished` | 连接建立成功 | session_id, path (L1/L2/L3), latency_ms |
| `ConnectionLost` | 连接断开 | session_id, reason |
| `ConnectionRestored` | 重连成功 | session_id, new_path, latency_ms |
| `InputReceived` | 收到远程输入（被控端） | session_id, event_type, data |
| `FileTransferProgress` | 文件传输进度 | transfer_id, progress_pct, speed |
| `FileTransferComplete` | 文件传输完成 | transfer_id, dest_path, success |
| `ChatMessage` | 收到聊天消息 | session_id, text, timestamp |
| `QualityChanged` | 画质自动调整 | session_id, new_mode, reason |
| `NearbyDeviceFound` | 发现局域网设备 | device_code, name, platform |
| `NearbyDeviceLost` | 局域网设备离线 | device_code |

### 2.4 并发模型

```
Flutter (Dart)                              Rust Core Engine
┌─────────────────────────┐                ┌─────────────────────────────────┐
│  Main Isolate (UI)      │   FFI calls    │  API Thread (sync)              │
│  · 渲染 UI              │───────────────►│  处理 FFI 调用，分发任务         │
│  · 用户交互             │                │                                 │
│                         │                │  Tokio Runtime                  │
│  Engine Isolate         │   ReceivePort  │  ┌────────┐ ┌────────┐         │
│  · 接收 Rust 事件       │◄───────────────│  │Conn Mgr│ │Capture │         │
│  · Stream<EngineEvent>  │   dart_post_   │  └────────┘ └────────┘         │
│  · 更新 Riverpod State  │   cobject      │  ┌────────┐ ┌────────┐         │
│                         │                │  │Codec   │ │Transport│         │
│  File Transfer Isolate  │                │  └────────┘ └────────┘         │
│  · 大文件分块读写       │                │  ┌────────┐ ┌────────┐         │
│  · 不阻塞 UI 线程       │                │  │Crypto  │ │File Xfer│         │
│                         │                │  └────────┘ └────────┘         │
└─────────────────────────┘                └─────────────────────────────────┘
```

### 2.5 帧数据处理管线

```
被控端                                      控制端
ScreenCapture → ContentAnalyzer → Encoder → Encrypt → Packetize
                                                          │
                                              Network (L1/L2/L3)
                                                          │
Reassemble ← Decrypt ← Decoder ← FrameRenderer ← Receive
```

- **内容分析**：检测场景类型（文字/图像/视频），动态调整编码参数
- **自适应编码**：文字场景 5fps 高清晰度，视频场景 30-120fps 流畅优先
- **性能反馈环**：延迟升高 → 自动降码率 → 恢复后逐步提升

### 2.6 被控端会话体验

| 阶段 | 表现 | 屏幕捕获排除 |
|------|------|-------------|
| 收到请求 | 居中弹窗：控制方名称 + [拒绝] [允许] + 30s 倒计时 | — |
| 会话中 | 顶部浮动状态条：● 张工 正在查看 [断开连接] | ✅ 排除 |
| 远程光标 | 带名称标签的光标（如"张工"）跟随控制方鼠标 | ✅ 排除 |
| 会话结束 | Toast: "张工 已断开连接，时长 12 分钟"，3s 消失 | ✅ 排除 |
| 菜单栏常驻 | 蓝色脉冲图标，点击展开：连接方信息 + 断开按钮 | — |

### 2.7 客户端设置面板

客户端设置面板通过主界面底部"管理控制台 →"旁或菜单栏/托盘图标入口打开，提供以下配置项：

**服务器配置**（开发调试核心）：

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| 信令服务地址 | `wss://signal.example.com` | WebSocket 信令服务地址，开发时改为 `ws://localhost:8443` |
| 中转服务地址 | `自动选择` | 默认由信令服务分配最优节点；可手动指定为 `localhost:3478` 用于本地调试 |
| API 服务地址 | `https://api.example.com` | 管理 API 地址，开发时改为 `http://localhost:8080` |
| 连接模式 | `自动` | 可选：`自动`（L1→L2→L3 依次尝试）/ `仅局域网` / `强制中转` |

> **本地调试典型配置**：信令 → `ws://localhost:8443`，中转 → `localhost:3478`，API → `http://localhost:8080`。客户端连接本地 Docker Compose 启动的全套服务，无需访问云端。

**画质设置**：

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| 画质模式 | `自动` | 可选：`自动`（内容感知）/ `清晰优先` / `流畅优先` |
| 最大分辨率 | `1080P` | 可选：720P / 1080P / 2K / 4K（付费版） |
| 最大帧率 | `60fps` | 可选：30 / 60 / 120fps（付费版） |
| 硬件加速 | `开启` | 利用 GPU 编解码，降低 CPU 占用 |

**通用设置**：

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| 开机自启动 | `关闭` | 系统启动时自动运行客户端 |
| 关闭时最小化到托盘 | `开启` | 关闭窗口 ≠ 退出，保持在线 |
| 连接确认模式 | `每次询问` | 可选：`每次询问` / `自动接受同团队` / `自动接受全部` |
| 提示音 | `开启` | 连接请求/断开时播放提示音 |
| 遥测数据 | `关闭` | 匿名使用数据统计，默认关闭（opt-in） |
| 语言 | `跟随系统` | 可选：简体中文 / English |

**配置文件存储**：

```
~/.rdcs/config.json
{
  "server": {
    "signaling_url": "wss://signal.example.com",
    "relay_address": "auto",
    "api_url": "https://api.example.com",
    "connection_mode": "auto"
  },
  "quality": {
    "mode": "auto",
    "max_resolution": "1080p",
    "max_fps": 60,
    "hardware_accel": true
  },
  "general": {
    "autostart": false,
    "minimize_to_tray": true,
    "confirm_mode": "ask",
    "sound_enabled": true,
    "telemetry": false,
    "language": "system"
  }
}
```

---

## 3. 信令服务设计

### 3.1 WebSocket 消息协议

| 消息类型 | 方向 | 载荷 | 说明 |
|---------|------|------|------|
| `register` | C→S | {device_code, platform, version, team_id?} | 设备注册上线 |
| `heartbeat` | C→S | {device_code, ts} | 每 30s 心跳，刷新 Redis TTL |
| `connect_request` | C→S→C | {from_code, to_code, invite_code?} | 发起连接请求 |
| `connect_response` | C→S→C | {accepted, session_id, from_code} | 确认/拒绝连接 |
| `ice_offer` | C→S→C | {session_id, sdp, candidates[]} | SDP offer + ICE candidates |
| `ice_answer` | C→S→C | {session_id, sdp, candidates[]} | SDP answer + ICE candidates |
| `relay_request` | C→S | {session_id, preferred_region?} | P2P 失败后请求中转 |
| `relay_assigned` | S→C | {session_id, relay_addr, relay_port, token} | 分配中转节点 |
| `peer_offline` | S→C | {device_code, reason} | 对端离线通知 |
| `nearby_update` | S→C | {devices: [{code, name, platform, online}]} | 附近设备变更 |

### 3.2 REST API（管理接口）

| 方法 | 路径 | 说明 | 鉴权 |
|------|------|------|------|
| POST | /auth/login | 登录（email + password + TOTP） | 无 → 返回 JWT |
| POST | /auth/totp/setup | 初始化 TOTP | JWT (负责人) |
| GET | /teams/:id/devices | 获取设备列表 | JWT (负责人) |
| POST | /teams/:id/invite | 生成邀请链接（24h） | JWT (负责人) |
| DELETE | /teams/:id/devices/:code | 移除设备 | JWT (负责人) |
| GET | /teams/:id/sessions | 连接记录查询 | JWT (负责人) |
| GET | /teams/:id/audit | 审计日志查询 | JWT (负责人) |
| PUT | /teams/:id/members/:id | 更新成员角色/权限 | JWT (负责人) |
| GET | /teams/:id/recordings | 会话录制列表 | JWT (负责人) |

### 3.3 Redis 数据模型

```
# 设备在线状态 (TTL 60s, 心跳刷新)
SET  device:{code}:online   {ip, platform, version, connected_at}  EX 60

# 团队在线设备集合
SADD team:{team_id}:online_devices  {code}

# 活跃会话
HSET session:{session_id}  controller={code}  controlled={code}
                           path={L1|L2|L3}    started_at={ts}

# 邀请码 (TTL 600s, 一次性使用后立即 DEL)
SET  invite:{code}  {device_code, team_id, created_at}  EX 600

# Pub/Sub
SUBSCRIBE  team:{team_id}:events     # 团队内设备上下线通知
SUBSCRIBE  device:{code}:signals     # 设备专属信令消息

# Keyspace 通知（监听 key 过期 = 设备离线）
CONFIG SET notify-keyspace-events Ex
SUBSCRIBE __keyevent@0__:expired
```

### 3.4 水平扩展

- **粘性路由**：按 team_id 哈希路由到固定信令实例（同团队设备在同一实例）
- **跨实例消息**：Redis Pub/Sub 做实例间消息桥接
- **无状态设计**：信令实例无本地状态，所有状态在 Redis/PG，可随时扩缩容

---

## 4. 中转节点设计

### 4.1 核心原则

- **零知识**：仅转发加密数据包，不持有解密密钥，不解析载荷
- **高性能**：零拷贝转发（sendmsg/recvmsg），纯 UDP
- **无状态**：仅维护 Slot 映射表，重启不丢失会话（信令可重新分配）

### 4.2 中转协议

| 阶段 | 流程 | 说明 |
|------|------|------|
| 分配 | 客户端→信令: relay_request → 信令选节点 → 返回 relay_addr + token | Token 一次性，30s 有效 |
| 握手 | 双方→Relay: ALLOCATE(session_id, token) → 分配 Slot | Token 验证 + UDP 端口对分配 |
| 转发 | A→Relay→B / B→Relay→A，双向 UDP | 零拷贝，仅替换包头源地址 |
| 心跳 | 双方每 15s 发 KEEPALIVE | 30s 无心跳 → Slot 回收 |
| 释放 | RELEASE 或超时 → Slot 回收 → 通知信令 | 信令记录会话结束 |

### 4.3 节点选择算法

```
1. 获取所有健康节点列表（心跳 < 30s）
2. 过滤负载 > 80% 的节点
3. 评分: score = 0.4/latency_A + 0.4/latency_B + 0.2/load
4. 选择 score 最高的节点
5. 生成一次性 token (HMAC-SHA256, 30s)
```

### 4.4 MVP 云端部署

| 区域 | 云厂商 | 规格 | 预估容量 | 月成本 |
|------|--------|------|---------|--------|
| 华东 (上海) | 阿里云 | 4C8G, 100Mbps | ~200 并发 | ~800 元 |
| 华南 (广州) | 腾讯云 | 4C8G, 100Mbps | ~200 并发 | ~800 元 |
| **合计** | — | 2 节点 | **~400 并发** | **~1,600 元/月** |

### 4.5 本地开发一键部署

为开发和调试提供完整的本地环境，一条命令启动所有服务：

```bash
# 一键启动本地开发环境
docker compose -f docker-compose.dev.yml up -d
```

**docker-compose.dev.yml 包含的服务**：

| 服务 | 镜像/构建 | 端口映射 | 说明 |
|------|----------|---------|------|
| signaling | 本地构建 (Rust) | 8443 (WSS) | 信令服务，开发模式跳过 TLS |
| relay | 本地构建 (Rust) | 3478 (UDP), 49152-49200 (Relay Slots) | 单节点中转，本地网络零延迟 |
| api | 本地构建 (Go) | 8080 (HTTP) | 管理 API，热重载模式 |
| web | 本地构建 (Node) | 3000 (HTTP) | Web 控制台，Vite dev server |
| postgres | postgres:16-alpine | 5432 | 开发数据库，自动初始化 Schema |
| redis | redis:7-alpine | 6379 | 开发缓存，启用 keyspace 通知 |
| minio | minio/minio | 9000/9001 | 本地 S3 兼容存储 |

**开发辅助功能**：

- **热重载**：Rust (cargo watch) + Go (air) + React (Vite HMR)
- **数据库自动迁移**：容器启动时执行 `migrations/` 目录
- **日志聚合**：所有服务日志输出到 `./logs/` 目录，支持 `docker compose logs -f`
- **种子数据**：首次启动自动创建测试团队 + 3 个测试设备
- **调试模式**：信令服务记录所有 WebSocket 消息到 `./logs/signaling-debug.log`

**本地部署与云端部署的差异**：

| 项目 | 本地开发 | 云端生产 |
|------|---------|---------|
| TLS | 关闭（HTTP/WS） | Caddy 自动 Let's Encrypt |
| 中转节点 | 单节点 (localhost) | 多节点 + 负载均衡 |
| 认证 | 可跳过 TOTP | 强制 TOTP |
| 数据库 | 单实例，无备份 | 主从复制 + 自动备份 |
| 日志级别 | DEBUG | INFO |
| 连接限速 | 不限速 | 免费版 720P/30fps |

### 4.6 自部署方案（V2.0 规划）

面向有数据主权需求的企业客户，提供与云端版功能一致的私有部署方案：

```bash
# 客户服务器一键部署
curl -fsSL https://install.example.com | bash

# 或手动 Docker Compose
docker compose -f docker-compose.prod.yml up -d
```

- 所有数据存储在客户服务器，不上传任何数据到官方
- 可选择关闭遥测数据采集
- 提供 Web 管理界面配置许可密钥和系统参数

---

## 5. Web 控制台 · 安全架构 · 数据模型

### 5.1 Web 控制台

- **前端**：React + TypeScript + Tailwind CSS + TanStack Query
- **状态管理**：Zustand (AuthStore / UIStore / WSStore)
- **API 通信**：REST (HTTPS) + WebSocket (实时事件推送)
- **页面**：仪表盘 · 设备管理 · 连接记录 · 成员管理 · 设置

### 5.2 安全架构

**端到端加密 (E2E)**：
- 密钥交换：X25519 ECDH，每会话独立密钥对
- 会话加密：XSalsa20-Poly1305 AEAD
- 前向保密：临时密钥，会话结束即销毁
- 中转零知识：Relay 仅转发密文

**传输安全**：
- 信令/API 通道：TLS 1.3（Caddy 自动证书）
- JWT：RS256 签名，15 分钟过期 + Refresh Token
- 设备认证：设备码 + 随机 nonce 签名

**审计合规**：
- 连接审计：双方、时间、时长、路径
- 操作审计：所有管理操作
- 文件传输审计：文件名、大小、方向
- 会话录制：AES-256 加密存储
- 保留策略：免费 30 天 / 付费 365 天

**防滥用**：
- 邀请码：5 次错误 → 锁定 30 分钟
- 设备码：单 IP 每小时 ≤10 次连接请求
- API：100 请求/分钟/IP
- 登录：5 次失败 → 锁定 15 分钟 + 强制 TOTP
- WebSocket：单设备仅允许 1 个活跃连接

### 5.3 PostgreSQL 数据模型

```sql
-- 团队
CREATE TABLE teams (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(100) NOT NULL,
    plan            VARCHAR(20) DEFAULT 'free',  -- free/basic/pro/unlimited
    max_concurrent  INTEGER DEFAULT 5,
    created_at      TIMESTAMPTZ DEFAULT now(),
    updated_at      TIMESTAMPTZ DEFAULT now()
);

-- 成员
CREATE TABLE members (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id         UUID REFERENCES teams(id),
    name            VARCHAR(100) NOT NULL,
    email           VARCHAR(255) UNIQUE,
    role            VARCHAR(20) DEFAULT 'member',  -- owner/manager/member
    password_hash   VARCHAR(255),
    totp_secret     VARCHAR(255),
    totp_enabled    BOOLEAN DEFAULT false,
    last_login      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ DEFAULT now()
);

-- 设备
CREATE TABLE devices (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id         UUID REFERENCES teams(id),
    device_code     CHAR(9) UNIQUE NOT NULL,
    device_name     VARCHAR(100),
    platform        VARCHAR(20),  -- macos/windows/linux/ios/android
    os_version      VARCHAR(50),
    client_version  VARCHAR(20),
    status          VARCHAR(20) DEFAULT 'offline',  -- online/offline
    last_seen       TIMESTAMPTZ,
    ip_address      INET,
    created_at      TIMESTAMPTZ DEFAULT now()
);

-- 连接记录
CREATE TABLE connection_records (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id             UUID REFERENCES teams(id),
    controller_code     CHAR(9) NOT NULL,
    controlled_code     CHAR(9) NOT NULL,
    path                VARCHAR(5),  -- L1/L2/L3
    started_at          TIMESTAMPTZ NOT NULL,
    ended_at            TIMESTAMPTZ,
    duration_sec        INTEGER,
    bytes_transferred   BIGINT DEFAULT 0,
    recording_path      VARCHAR(500),
    created_at          TIMESTAMPTZ DEFAULT now()
);

-- 审计日志
CREATE TABLE audit_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id         UUID REFERENCES teams(id),
    actor_id        UUID REFERENCES members(id),
    action          VARCHAR(50) NOT NULL,  -- device_remove/permission_change/...
    target_type     VARCHAR(50),
    target_id       VARCHAR(100),
    details         JSONB,
    ip_address      INET,
    created_at      TIMESTAMPTZ DEFAULT now()
);

-- 会话录制
CREATE TABLE recordings (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    connection_id   UUID REFERENCES connection_records(id),
    team_id         UUID REFERENCES teams(id),
    encryption_key  BYTEA NOT NULL,  -- AES-256 key, encrypted with team master key
    storage_path    VARCHAR(500) NOT NULL,
    duration_sec    INTEGER,
    file_size       BIGINT,
    created_at      TIMESTAMPTZ DEFAULT now(),
    expires_at      TIMESTAMPTZ
);

-- 索引
CREATE INDEX idx_devices_team_status ON devices(team_id, status);
CREATE INDEX idx_devices_code ON devices(device_code);
CREATE INDEX idx_conn_team_time ON connection_records(team_id, started_at DESC);
CREATE INDEX idx_conn_controller ON connection_records(controller_code, started_at DESC);
CREATE INDEX idx_audit_team_time ON audit_logs(team_id, created_at DESC);
CREATE INDEX idx_recordings_team_time ON recordings(team_id, created_at DESC);
```

---

## 6. 错误处理与降级策略

| 场景 | 检测方式 | 自动处理 | 用户感知 |
|------|---------|---------|---------|
| 网络丢包 | NACK 超时 / FEC 修复失败 | NACK 重传 → FEC 冗余增加 → 降低码率 | 画面短暂模糊，自动恢复 |
| 带宽骤降 | GCC 带宽估计下降 | 自动降级：分辨率↓ → 帧率↓ → 画质↓ | 黄色横幅"网络不稳定，已优化画质" |
| P2P 穿透失败 | ICE 候选全部失败（3s 超时） | 自动切换到 L3 中转 | 无感知切换，延迟可能略增 |
| 中转节点故障 | 心跳丢失 / 连接超时 | 自动切换到备用中转节点 | 短暂中断（<2s），自动恢复 |
| WebSocket 断开 | onclose 事件 | 指数退避重连（1s→30s，最多 10 次） | "正在重连…"遮罩 + 倒计时 |
| 文件传输中断 | 传输超时 / 连接断开 | 记录断点，连接恢复后续传 | 进度条暂停 → 自动恢复 |
| 屏幕捕获权限丢失 | 捕获帧全黑 / API 报错 | 停止捕获，通知对端 | "请重新授权屏幕录制权限" |
| 设备离线 | Redis TTL 过期 (60s) | keyspace 通知 → 广播离线 | 附近设备列表灰显/消失 |
| 离线后重连 | WebSocket 断开检测 | 指数退避 → 重注册 → 广播上线 | "● 在线"恢复 |
| 会话中全断 | 信令 + 数据通道均断 | 全屏遮罩 → 重连信令 → 重走连接流程 | "连接已断开，正在重连…" |

---

## 7. 平台发布计划

| 平台 | MVP | V1.1 | V2.0 |
|------|-----|------|------|
| macOS | ✅ 完整客户端 | — | — |
| Windows | — | ✅ 完整客户端 | — |
| Linux 桌面 | — | ✅ 完整客户端 | — |
| 微信小程序 | — | ✅ 控制端（查看） | ✅ 完整控制 |
| Web 浏览器 | — | ✅ 控制端 | — |
| iOS | — | — | ✅ 查看+简单控制 |
| Android | — | — | ✅ 查看+简单控制 |

**跨平台复用**：核心引擎 ~88% 复用，新增平台仅需 ~2,200 行适配代码。

---

## 8. 排期建议

| 阶段 | 内容 | 预估 | 依赖 | 风险 |
|------|------|------|------|------|
| Phase 1 | 核心引擎（连接 + 捕获 + 编解码） | 8 周 | 无 | 编解码性能不达标 |
| Phase 2 | 信令服务 + 中转节点部署 | 4 周 | P1 | 中转成本超预算 |
| Phase 3 | macOS 客户端 UI | 4 周 | P1 | 平台兼容性 |
| Phase 4 | 端到端联调 + 测试 | 3 周 | P2+P3 | NAT 穿透成功率 |
| Phase 5 | Web 控制台 | 4 周 | P2 | — |
| **MVP 合计** | | **~19 周 (5 个月)** | | 并行可压缩至 **4 个月** |
| Phase 6 | Windows 客户端 | 4 周 | P4 | Windows 兼容性 |
| Phase 7 | Linux 客户端 | 4 周 | P4 | Wayland/X11 兼容 |
| Phase 8 | 微信小程序 | 3 周 | P4 | 微信审核政策 |

**并行策略**：P3 与 P2 并行（Mock 接口联调），P5 与 P3/P4 并行，P6-P8 互相并行。

---

## 9. 风险矩阵

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| P2P 穿透成功率 <60% | 中 | 高 | 提前部署多区域中继，优化 ICE 策略 |
| 编解码 CPU 占用过高 | 中 | 中 | GPU 硬件加速 + 编码降级方案 |
| 中转带宽成本超预算 | 高 | 高 | 免费限速 720P + 付费覆盖 + 高效编码 + 成本模型验证 |
| WebRTC 弱网表现差 | 中 | 中 | SVC 自适应码率 + FEC + NACK 重传 |
| 微信审核不通过 | 中 | 中 | 同步开发钉钉/飞书版本 |
| 跨平台捕获兼容性 | 低 | 中 | 核心层抽象差异，各平台独立实现 |
| macOS 权限策略变化 | 低 | 高 | 关注 WWDC，预留适配方案 |

---

## 附录 A：免费版 vs 付费版

| 功能 | 免费版 | 付费版 |
|------|--------|--------|
| P2P 直连画质 | 1080P / 60FPS（不限速） | 4K / 120FPS |
| 中转画质 | 720P / 30FPS | 不限速 |
| 并发连接数 | 5 个 | 不限 |
| 会话录制 | ❌ | ✅ (AES-256 加密) |
| 会话录制回放 | ❌ | ✅ |
| 邀请码 | ✅ | ✅ |
| 文字聊天 | ✅ | ✅ |
| 文件传输 | ✅ | ✅ |
| Web 控制台 | 基础 | 完整 |
| 连接记录保留 | 30 天 | 365 天 |
| TOTP 双因素 | ❌ | ✅ |
| 管理操作审计 | ❌ | ✅ |

**定价参考**（待市场验证）：免费 → 基础 2,980 元/年 (10 并发) → 专业 5,980 元/年 (20 并发) → 无限 12,800 元/年

---

## 附录 B：非功能需求汇总

| 类别 | 要求 | 验收标准 |
|------|------|---------|
| 性能 | 局域网延迟 | < 10ms (L1) |
| 性能 | 局域网帧率 | ≥ 120fps (L1, 1080P) |
| 性能 | 中转延迟 | < 100ms (L3) |
| 性能 | 连接建立 | < 500ms |
| 性能 | 内存占用 | < 150MB 空闲 / < 300MB 会话中 |
| 性能 | CPU 占用 | < 5% 空闲 / < 30% 1080P 会话 |
| 安全 | 传输加密 | 端到端加密（中转节点不可见） |
| 安全 | 信令/API | TLS 1.3 |
| 安全 | 中转 | 零知识（不解密） |
| 安全 | 双因素认证 | TOTP (负责人登录) |
| 安全 | 录制存储 | AES-256 加密 |
| 安全 | 防暴力破解 | 5 次错误锁 30 分钟 |
| 安全 | 连接确认 | 被控端必须明确确认 |
| 安全 | 审计日志 | 所有管理操作记录 |
| 安全 | 数据隐私 | 遥测需 opt-in |
| 兼容性 | macOS | 14.0+ (Sonoma/Sequoia) |
| 兼容性 | Windows | 10 21H2+ / 11 |
| 可用性 | 中转 SLA | > 99.5% (月度) |
| 可用性 | 断线重连 | 自动重连成功率 > 90% |
