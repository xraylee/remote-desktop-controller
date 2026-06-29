# 客户端打包流程和多设备联调测试方案

**制定日期**: 2026-06-28  
**当前状态**: Phase 2（95%），客户端应用未创建  
**目标**: 制定完整的打包、部署和联调方案

---

## 📦 当前项目架构分析

### 现有模块（库 Crates）
```
crates/
├── rdcs-platform       # 平台抽象（屏幕捕获等）
├── rdcs-codec          # 编解码器
├── rdcs-display        # SDL2 显示 ⭐ NEW
├── rdcs-connection     # ICE P2P 连接
├── rdcs-transport      # 网络传输
├── rdcs-session        # 会话管理
├── rdcs-signaling      # 信令服务
└── ... (其他基础库)
```

### 🔴 缺失组件

**客户端应用程序** (未创建):
- ❌ `rdcs-agent` - 被控端应用（运行在 Intel Mac）
- ❌ `rdcs-controller` - 主控端应用（运行在 Apple Silicon Mac）
- ❌ `rdcs-cli` - 命令行工具
- ❌ `rdcs-gui` - 图形界面（未来）

---

## 🏗️ 客户端应用架构设计

### 方案 A: 双端架构（推荐 MVP）⭐

```
┌─────────────────────────────────────────────┐
│   rdcs-agent (被控端/Server)                 │
│   运行在: Intel Mac (被控机器)                │
├─────────────────────────────────────────────┤
│  • 屏幕捕获 (rdcs-platform)                   │
│  • 视频编码 (rdcs-codec)                      │
│  • 输入注入 (rdcs-platform) [Phase 3]        │
│  • ICE 连接建立 (rdcs-connection)             │
│  • 数据发送 (rdcs-transport)                  │
│  • 会话管理 (rdcs-session)                    │
└─────────────────────────────────────────────┘
              ↓ 网络 (ICE/DTLS) ↓
┌─────────────────────────────────────────────┐
│   rdcs-controller (主控端/Client)            │
│   运行在: Apple Silicon Mac (控制机器)        │
├─────────────────────────────────────────────┤
│  • 视频解码 (rdcs-codec)                      │
│  • 视频显示 (rdcs-display) ⭐                │
│  • 输入捕获 (键盘/鼠标) [Phase 3]             │
│  • ICE 连接建立 (rdcs-connection)             │
│  • 数据接收 (rdcs-transport)                  │
│  • 会话管理 (rdcs-session)                    │
└─────────────────────────────────────────────┘
```

### 方案 B: 统一客户端（长期）

一个应用程序同时支持控制端和被控端角色：
```
rdcs-desktop (统一客户端)
├── 模式 1: Agent 模式（被控端）
├── 模式 2: Controller 模式（主控端）
└── 模式 3: 双向模式（互相控制）
```

**MVP 推荐**: 先实现方案 A（简单、清晰），Phase 4 再考虑统一。

---

## 📦 客户端打包流程设计

### Phase 1: 基础打包（当前阶段）

#### 步骤 1: 创建应用程序 Crates

**1.1 创建 `rdcs-agent` (被控端)**

```bash
# 创建 agent 应用
cd crates
cargo new rdcs-agent --bin

# 目录结构
crates/rdcs-agent/
├── Cargo.toml           # 依赖所有需要的库
├── src/
│   ├── main.rs         # 主入口
│   ├── capture.rs      # 捕获线程
│   ├── encoder.rs      # 编码线程
│   ├── network.rs      # 网络线程
│   └── config.rs       # 配置管理
└── README.md
```

**Cargo.toml 依赖**:
```toml
[dependencies]
rdcs-platform = { path = "../rdcs-platform" }
rdcs-codec = { path = "../rdcs-codec", features = ["software-encoder"] }
rdcs-connection = { path = "../rdcs-connection" }
rdcs-transport = { path = "../rdcs-transport" }
rdcs-session = { path = "../rdcs-session" }
tokio = { workspace = true }
tracing = { workspace = true }
clap = { version = "4", features = ["derive"] }
```

**1.2 创建 `rdcs-controller` (主控端)**

```bash
# 创建 controller 应用
cd crates
cargo new rdcs-controller --bin

# 目录结构
crates/rdcs-controller/
├── Cargo.toml
├── src/
│   ├── main.rs         # 主入口
│   ├── decoder.rs      # 解码线程
│   ├── display.rs      # 显示线程
│   ├── network.rs      # 网络线程
│   └── config.rs       # 配置管理
└── README.md
```

**Cargo.toml 依赖**:
```toml
[dependencies]
rdcs-display = { path = "../rdcs-display" }
rdcs-codec = { path = "../rdcs-codec", features = ["software-encoder"] }
rdcs-connection = { path = "../rdcs-connection" }
rdcs-transport = { path = "../rdcs-transport" }
rdcs-session = { path = "../rdcs-session" }
tokio = { workspace = true }
tracing = { workspace = true }
clap = { version = "4", features = ["derive"] }
```

#### 步骤 2: 编译二进制文件

**2.1 开发模式编译**
```bash
# 编译 agent（被控端）
cargo build -p rdcs-agent

# 编译 controller（主控端）
cargo build -p rdcs-controller

# 输出位置
target/debug/rdcs-agent
target/debug/rdcs-controller
```

**2.2 Release 模式编译**
```bash
# Release 编译（优化）
cargo build -p rdcs-agent --release
cargo build -p rdcs-controller --release

# 输出位置
target/release/rdcs-agent
target/release/rdcs-controller
```

**2.3 跨平台编译（交叉编译）**

```bash
# 在 Apple Silicon Mac 上编译 Intel 版本
rustup target add x86_64-apple-darwin
cargo build -p rdcs-agent --release --target x86_64-apple-darwin

# 在 Intel Mac 上编译 ARM 版本
rustup target add aarch64-apple-darwin
cargo build -p rdcs-controller --release --target aarch64-apple-darwin
```

#### 步骤 3: 依赖打包

**macOS 特殊处理 - SDL2 依赖**:

```bash
# 方案 A: 静态链接（推荐）
# Cargo.toml 中使用
sdl2 = { version = "0.37", features = ["bundled", "static-link"] }

# 方案 B: 动态链接 + 依赖打包
# 需要将 SDL2.framework 打包到 .app bundle 中
```

**依赖检查脚本**:
```bash
#!/bin/bash
# scripts/check-deps.sh

echo "检查二进制依赖..."
otool -L target/release/rdcs-controller

# 预期输出（静态链接）:
# /usr/lib/libSystem.B.dylib
# /System/Library/Frameworks/...（系统框架）
# 无第三方动态库依赖
```

#### 步骤 4: macOS App Bundle 打包

**4.1 创建 .app 结构**

```bash
# scripts/package-macos.sh

#!/bin/bash
set -e

APP_NAME="RDCS Controller"
BUNDLE_ID="dev.rdcs.controller"
VERSION="0.1.0"

# 创建 .app 目录结构
mkdir -p "dist/$APP_NAME.app/Contents/"{MacOS,Resources}

# 拷贝二进制文件
cp target/release/rdcs-controller "dist/$APP_NAME.app/Contents/MacOS/"

# 创建 Info.plist
cat > "dist/$APP_NAME.app/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>rdcs-controller</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# 添加图标（可选）
# cp assets/icon.icns "dist/$APP_NAME.app/Contents/Resources/"

echo "✅ App bundle created: dist/$APP_NAME.app"
```

**4.2 代码签名（可选，发布时需要）**

```bash
# 开发者签名
codesign --force --deep --sign "Developer ID Application: Your Name" \
    "dist/$APP_NAME.app"

# 验证签名
codesign --verify --deep --strict "dist/$APP_NAME.app"
spctl -a -v "dist/$APP_NAME.app"
```

**4.3 创建 DMG 安装包（可选）**

```bash
# 使用 create-dmg 工具
brew install create-dmg

create-dmg \
  --volname "RDCS Controller" \
  --window-pos 200 120 \
  --window-size 800 400 \
  --icon-size 100 \
  --app-drop-link 600 185 \
  "dist/RDCS-Controller-$VERSION.dmg" \
  "dist/$APP_NAME.app"
```

---

## 🧪 多设备联调测试方案

### 测试环境设置

#### 环境 1: 同一局域网（最简单）⭐

**拓扑**:
```
Apple Silicon Mac (Controller)  ←→  Intel Mac (Agent)
    192.168.1.100                      192.168.1.101
            ↓                                ↓
        同一路由器 (192.168.1.1)
```

**优点**:
- ✅ 无需公网 IP
- ✅ 低延迟
- ✅ 调试简单

**测试场景**:
- 基础连接测试
- 性能基准测试
- 功能完整性测试

#### 环境 2: 跨子网（中等）

**拓扑**:
```
Apple Silicon Mac           Intel Mac
  (Subnet A)                (Subnet B)
  10.0.1.100                10.0.2.100
      ↓                         ↓
  Gateway A  ←→ Router ←→  Gateway B
```

**测试要点**:
- ICE NAT 穿透
- STUN 服务器配置
- 防火墙规则

#### 环境 3: 公网跨网（最复杂）

**拓扑**:
```
Home Network              Office Network
Apple Silicon Mac         Intel Mac
  (NAT: Type 3)            (NAT: Type 2)
      ↓                         ↓
  Home Router  ←→ Internet ←→  Office Router
      ↓                         ↓
  STUN Server              TURN Relay Server
```

**测试要点**:
- TURN 中继服务
- 高延迟场景
- 带宽限制测试

---

### 联调测试流程（详细步骤）

#### Phase 1: 本机测试（开发阶段）

**目标**: 验证核心功能

```bash
# Terminal 1: 运行 agent（模拟被控端）
cargo run -p rdcs-agent -- \
    --mode server \
    --port 8000

# Terminal 2: 运行 controller（模拟主控端）
cargo run -p rdcs-controller -- \
    --connect localhost:8000
```

**验收标准**:
- [ ] 能建立连接
- [ ] 能看到视频流
- [ ] 延迟 < 100ms

#### Phase 2: 局域网双机测试（集成阶段）⭐

**准备工作**:

```bash
# 在两台 Mac 上都执行
git clone <repo-url>
cd remote-desktop-controller

# 检查网络连接
ping <对方IP>

# 检查防火墙（macOS）
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --getglobalstate
```

**Step 1: 部署 Agent（Intel Mac）**

```bash
# 在 Intel Mac 上
cd /path/to/remote-desktop-controller

# 编译 agent
cargo build -p rdcs-agent --release

# 运行 agent
./target/release/rdcs-agent \
    --bind 0.0.0.0:8000 \
    --device-id "intel-mac-001" \
    --log-level debug

# 预期输出
# [INFO] RDCS Agent v0.1.0
# [INFO] Device ID: intel-mac-001
# [INFO] Listening on 0.0.0.0:8000
# [INFO] Screen capture initialized (1920x1080)
# [INFO] Encoder: OpenH264 H.264
# [INFO] Waiting for connections...
```

**Step 2: 部署 Controller（Apple Silicon Mac）**

```bash
# 在 Apple Silicon Mac 上
cd /Users/lc/Development/source/remote-desktop-controller

# 编译 controller
cargo build -p rdcs-controller --release

# 运行 controller
./target/release/rdcs-controller \
    --connect <Intel-Mac-IP>:8000 \
    --device-id "arm-mac-001" \
    --log-level debug

# 预期输出
# [INFO] RDCS Controller v0.1.0
# [INFO] Device ID: arm-mac-001
# [INFO] Connecting to intel-mac-001 at <IP>:8000...
# [INFO] ICE gathering started
# [INFO] ICE connection established (P2P)
# [INFO] DTLS handshake complete
# [INFO] Video stream started (1920x1080 @ 30fps)
# [INFO] Display window opened
```

**Step 3: 验证连接**

```bash
# 在 Intel Mac 上检查连接状态
# Terminal 输出应显示
# [INFO] New connection from <Apple-Silicon-IP>
# [INFO] ICE negotiation complete
# [INFO] Streaming video: Frame 30 (15.2ms)
```

**Step 4: 性能监控**

```bash
# 在 Controller 端查看实时统计
# 应该显示类似输出：
# ┌─────────────────────────────────────┐
# │ RDCS Performance Monitor            │
# ├─────────────────────────────────────┤
# │ FPS:          29.8                  │
# │ Latency:      42.5ms                │
# │ - Network:    10.2ms                │
# │ - Decode:     18.3ms                │
# │ - Display:    14.0ms                │
# │ Bitrate:      1.8 Mbps              │
# │ CPU (Agent):  45%                   │
# │ CPU (Ctrl):   38%                   │
# └─────────────────────────────────────┘
```

#### Phase 3: 跨架构自动化测试

**测试矩阵**:

| Agent (被控端) | Controller (主控端) | 网络 | 状态 |
|---------------|-------------------|------|------|
| Intel Mac | Apple Silicon Mac | LAN | ✅ 优先 |
| Apple Silicon Mac | Intel Mac | LAN | 🔄 反向测试 |
| Intel Mac | Intel Mac | LAN | 🔄 同架构 |
| ARM Mac | ARM Mac | LAN | 🔄 同架构 |

**自动化脚本**:

```bash
#!/bin/bash
# scripts/e2e-test.sh

# 配置
AGENT_HOST="192.168.1.101"   # Intel Mac
CTRL_HOST="192.168.1.100"    # Apple Silicon Mac
DURATION=60                   # 测试时长（秒）

echo "========================================="
echo "🧪 RDCS E2E 自动化测试"
echo "========================================="
echo ""

# 1. SSH 到 Intel Mac 启动 agent
echo "1️⃣  Starting agent on Intel Mac..."
ssh user@$AGENT_HOST "cd /path/to/rdcs && \
    ./target/release/rdcs-agent \
    --bind 0.0.0.0:8000 \
    --device-id intel-mac-001 &"

sleep 5

# 2. 本地启动 controller
echo "2️⃣  Starting controller..."
./target/release/rdcs-controller \
    --connect $AGENT_HOST:8000 \
    --duration $DURATION \
    --headless \
    --metrics-output test-results.json

# 3. 收集结果
echo "3️⃣  Collecting test results..."
cat test-results.json | jq '
{
  "avg_fps": .fps.avg,
  "avg_latency_ms": .latency.avg,
  "max_latency_ms": .latency.max,
  "packet_loss": .network.packet_loss,
  "test_passed": (.fps.avg >= 24 and .latency.avg < 100)
}'

# 4. 清理
echo "4️⃣  Cleaning up..."
ssh user@$AGENT_HOST "pkill rdcs-agent"

echo "✅ Test complete"
```

---

## 📊 测试验收标准

### 功能测试

| 测试项 | 验收标准 | 优先级 |
|--------|---------|--------|
| 连接建立 | < 5秒建立连接 | P0 |
| 视频显示 | 能看到清晰画面 | P0 |
| 帧率 | >= 24 FPS | P0 |
| 延迟 | < 100ms (LAN) | P0 |
| 稳定性 | 10分钟无断连 | P1 |
| 跨架构 | ARM ↔ Intel 正常 | P1 |
| 输入控制 | 鼠标键盘响应 | P2 (Phase 3) |

### 性能测试

**场景 1: 同一局域网**
```
目标性能:
  • 延迟: < 50ms
  • 帧率: 30 FPS
  • CPU (Agent): < 50%
  • CPU (Controller): < 40%
  • 带宽: < 5 Mbps
```

**场景 2: 跨子网**
```
目标性能:
  • 延迟: < 100ms
  • 帧率: >= 24 FPS
  • 丢包率: < 1%
```

**场景 3: 公网（TURN 中继）**
```
目标性能:
  • 延迟: < 300ms
  • 帧率: >= 20 FPS
  • 丢包率: < 5%
```

---

## 🚀 实施路线图

### Week 1: 应用程序骨架（当前）

- [ ] 创建 `rdcs-agent` crate
- [ ] 创建 `rdcs-controller` crate
- [ ] 实现基础 main.rs
- [ ] 命令行参数解析
- [ ] 配置文件支持

### Week 2: 集成现有模块

- [ ] Agent: 集成屏幕捕获
- [ ] Agent: 集成编码器
- [ ] Controller: 集成解码器
- [ ] Controller: 集成显示窗口 ✅
- [ ] 网络连接集成

### Week 3: 本地测试

- [ ] 单机回环测试
- [ ] 跨进程测试
- [ ] 性能基准测试
- [ ] 日志和监控

### Week 4: 多设备联调

- [ ] 局域网双机测试 ⭐
- [ ] 跨架构测试
- [ ] 自动化测试脚本
- [ ] 问题修复和优化

### Week 5: 打包和部署

- [ ] macOS .app bundle
- [ ] 依赖打包
- [ ] 安装脚本
- [ ] 用户文档

---

## 📝 待创建的文件清单

### 应用程序代码

```
crates/rdcs-agent/
├── Cargo.toml
├── src/
│   ├── main.rs          (主入口 ~200 行)
│   ├── app.rs           (应用主逻辑 ~300 行)
│   ├── capture.rs       (捕获线程 ~150 行)
│   ├── encoder.rs       (编码线程 ~150 行)
│   ├── network.rs       (网络线程 ~200 行)
│   ├── config.rs        (配置管理 ~100 行)
│   └── cli.rs           (命令行参数 ~80 行)
└── README.md

crates/rdcs-controller/
├── Cargo.toml
├── src/
│   ├── main.rs          (主入口 ~200 行)
│   ├── app.rs           (应用主逻辑 ~300 行)
│   ├── decoder.rs       (解码线程 ~150 行)
│   ├── display.rs       (显示线程 ~150 行)
│   ├── network.rs       (网络线程 ~200 行)
│   ├── config.rs        (配置管理 ~100 行)
│   └── cli.rs           (命令行参数 ~80 行)
└── README.md
```

**预计代码量**: ~2400 行

### 测试和部署脚本

```
scripts/
├── build-agent.sh          (编译 agent)
├── build-controller.sh     (编译 controller)
├── package-macos.sh        (打包 .app bundle)
├── e2e-test.sh            (自动化 E2E 测试)
├── deploy-intel.sh        (部署到 Intel Mac)
├── deploy-arm.sh          (部署到 ARM Mac)
└── check-deps.sh          (检查依赖)
```

### 文档

```
docs/
├── CLIENT_PACKAGING.md     (本文档)
├── DEPLOYMENT_GUIDE.md     (部署指南)
├── E2E_TEST_RESULTS.md     (测试结果模板)
└── TROUBLESHOOTING.md      (问题排查)
```

---

## 💡 最佳实践建议

### 1. 版本管理

```toml
# 在 workspace Cargo.toml 中统一版本
[workspace.package]
version = "0.1.0"

# 每次发布更新版本号
# 0.1.0 -> 0.1.1 (bug fix)
# 0.1.0 -> 0.2.0 (new feature)
# 0.1.0 -> 1.0.0 (MVP 完成)
```

### 2. 配置文件

```toml
# config/default.toml
[agent]
device_id = "auto"  # 自动生成
bind_address = "0.0.0.0:8000"
log_level = "info"

[video]
resolution = "1920x1080"
fps = 30
bitrate = 2000000  # 2 Mbps

[network]
ice_servers = [
    "stun:stun.l.google.com:19302"
]
```

### 3. 日志策略

```rust
// 使用结构化日志
tracing::info!(
    device_id = %config.device_id,
    resolution = %resolution,
    "Agent started"
);

// 保存日志文件
// ~/Library/Logs/RDCS/agent.log
// ~/Library/Logs/RDCS/controller.log
```

### 4. 错误处理

```rust
// 友好的用户错误提示
match start_agent(&config) {
    Ok(_) => println!("✅ Agent started successfully"),
    Err(e) => {
        eprintln!("❌ Failed to start agent: ", e);
        eprintln!("💡 Try running with --help for usage");
        std::process::exit(1);
    }
}
```

---

## 🎯 下一步行动

### 立即可做（本周）

1. **创建应用程序骨架**
   ```bash
   cargo new crates/rdcs-agent --bin
   cargo new crates/rdcs-controller --bin
   ```

2. **实现基础 CLI**
   - 命令行参数解析
   - 配置文件加载
   - 日志初始化

3. **集成现有模块**
   - Agent: 屏幕捕获 + 编码
   - Controller: 解码 + 显示 ✅

### 下周可做

4. **本地测试**
   ```bash
   # 单机回环测试
   ./scripts/local-test.sh
   ```

5. **双机联调准备**
   - 在 Intel Mac 上部署代码
   - 配置网络环境
   - 编写测试脚本

---

**制定人**: AI Assistant  
**制定日期**: 2026-06-28  
**预计完成**: 2026-07-19（3 周）  
**优先级**: 🔴 高（Phase 2 完成的必要条件）
