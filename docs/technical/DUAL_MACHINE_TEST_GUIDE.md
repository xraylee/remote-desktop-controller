# 双机测试指南

**日期**: 2026-06-29  
**目标**: 在两台 Mac 上测试完整的远程桌面功能

---

## 🖥️ 机器角色

根据项目 memory 配置：

### Apple Silicon Mac（当前机器）- **主控端** 🎮

**角色**: Controller / Viewer / Client  
**功能**:
- 运行 Flutter 客户端应用
- 查看被控端的屏幕
- 发送鼠标/键盘输入
- 控制远程桌面

**为什么作为主控端**:
- 主力开发机，方便调试
- 更强的性能处理视频解码
- 便于查看日志和排查问题

---

### Intel Mac - **被控端** 🖥️

**角色**: Host / Server / Target  
**功能**:
- 运行屏幕捕获
- 运行视频编码器
- 接收输入事件并注入
- 共享屏幕给主控端

**为什么作为被控端**:
- 辅助测试机
- 验证跨架构兼容性
- 实际的被控场景

---

## 📋 测试前准备

### 1. 环境检查

**两台 Mac 都需要**:

```bash
# 检查 Rust 工具链
rustc --version
cargo --version

# 检查 Flutter SDK
flutter --version
flutter doctor

# 检查权限
# macOS 设置 → 隐私与安全性 → 屏幕录制
# macOS 设置 → 隐私与安全性 → 辅助功能
```

---

### 2. 代码同步

**在 Apple Silicon Mac（主控端）上**:

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 提交当前更改
git add -A
git commit -m "feat: complete video pipeline and input integration"
git push origin main
```

**在 Intel Mac（被控端）上**:

```bash
cd ~/Development/remote-desktop-controller  # 或你的路径

# 拉取最新代码
git pull origin main

# 编译 Rust FFI 库
cargo build --release

# 编译 Flutter 应用
cd client/flutter
flutter build macos --release
```

---

## 🧪 测试步骤

### 阶段 1: 本地回环测试（单机验证）

**目的**: 验证每台机器上的基本功能

#### 在 Apple Silicon Mac 上测试

```bash
cd crates/rdcs-ffi

# 测试视频编码/解码
cargo run --example local_loopback_test --features software-encoder

# 期望输出:
# ✅ 150 frames received in 5 seconds (~30 FPS)
```

#### 在 Intel Mac 上测试

```bash
cd crates/rdcs-ffi

# 测试视频编码/解码
cargo run --example local_loopback_test --features software-encoder

# 期望输出:
# ✅ 150 frames received in 5 seconds (~30 FPS)
```

**验证点**:
- ✅ 两台机器都能成功捕获屏幕
- ✅ 两台机器都能编码/解码视频
- ✅ FPS 都在 25-30 之间
- ✅ 无崩溃或错误

---

### 阶段 2: Flutter UI 测试（单机验证）

#### 在 Apple Silicon Mac 上测试

```bash
cd client/flutter
flutter run -d macos

# 操作步骤:
# 1. 点击"开始捕获"按钮
# 2. 观察是否显示实时画面
# 3. 检查 FPS 和延迟指标
# 4. 尝试鼠标移动和点击
```

**验证点**:
- ✅ Flutter 应用启动成功
- ✅ 显示实时屏幕画面
- ✅ 统计信息正确显示（FPS、延迟）
- ✅ 鼠标输入事件发送成功（查看日志）

---

### 阶段 3: 双机网络连接测试 ⭐

**注意**: 当前实现是本地回环，跳过 WebRTC 网络传输。双机测试需要以下修改之一：

#### 方案 A: 使用 TCP 直连（推荐，快速验证）

**在 Intel Mac（被控端）上运行服务器**:

```bash
cd crates/rdcs-transport
cargo run --example tcp_video_server
```

**在 Apple Silicon Mac（主控端）上运行客户端**:

```bash
# 修改 client IP 地址为 Intel Mac 的局域网 IP
cd crates/rdcs-transport
cargo run --example tcp_video_client -- <intel-mac-ip>:8888
```

#### 方案 B: 完整 WebRTC 连接（复杂，完整功能）

**需要实现**:
1. 信令服务器（交换 SDP offer/answer）
2. STUN/TURN 服务器（NAT 穿透）
3. WebRTC DataChannel 集成

**时间估算**: 2-3 天

---

## 📊 测试场景

### 场景 1: 静态桌面

**操作**: 不动鼠标，观察 30 秒

**期望**:
- FPS: 3-5（帧跳过生效）
- 带宽: ~0.025 MB/s
- 延迟: <50 ms

---

### 场景 2: 文字编辑

**操作**: 打开文本编辑器，输入文字

**期望**:
- FPS: 8-12
- 带宽: ~0.075 MB/s
- 文字清晰可读

---

### 场景 3: 浏览器滚动

**操作**: 在浏览器中滚动网页

**期望**:
- FPS: 18-25
- 带宽: ~0.15 MB/s
- 画面流畅

---

### 场景 4: 视频播放

**操作**: 播放 YouTube 视频

**期望**:
- FPS: 25-30
- 带宽: ~0.25 MB/s
- 画面可辨识（会有压缩）

---

### 场景 5: 输入控制

**操作**: 从主控端控制被控端

**鼠标测试**:
- [ ] 移动鼠标，观察被控端光标跟随
- [ ] 单击，验证点击事件注入
- [ ] 双击，验证双击事件
- [ ] 滚轮，验证滚动事件

**键盘测试**:
- [ ] 输入英文字符
- [ ] 输入中文字符
- [ ] 输入特殊字符（@#$%）
- [ ] 快捷键（Cmd+C, Cmd+V）

---

## 🐛 故障排查

### 问题 1: 无法捕获屏幕

**症状**: 黑屏或白屏

**解决**:
```bash
# 检查屏幕录制权限
# macOS 设置 → 隐私与安全性 → 屏幕录制
# 添加你的应用并重启
```

---

### 问题 2: FPS 过低（<10）

**症状**: 画面卡顿

**排查**:
1. 检查 CPU 占用（Activity Monitor）
2. 检查编码器配置（bitrate）
3. 查看日志中的编码耗时

**优化**:
```rust
// 降低码率
let target_bitrate = 1_000_000; // 1 Mbps

// 降低分辨率
let target_resolution = VideoResolution::HD720;
```

---

### 问题 3: 解码器返回 None

**症状**: "OpenH264 decoder returned None"

**原因**: 编码器没有生成 SPS/PPS

**解决**: 已在视频管道中修复，确保使用最新代码

---

### 问题 4: 输入事件不生效

**症状**: 鼠标/键盘输入无响应

**排查**:
```bash
# 检查辅助功能权限
# macOS 设置 → 隐私与安全性 → 辅助功能
# 添加你的应用并重启

# 查看 FFI 日志
RUST_LOG=debug cargo run
```

---

### 问题 5: 网络连接失败

**症状**: "Connection refused"

**排查**:
1. 检查防火墙设置
2. 确认两台 Mac 在同一局域网
3. ping 测试连通性

```bash
# 在主控端 ping 被控端
ping <intel-mac-ip>

# 检查端口是否打开
nc -zv <intel-mac-ip> 8888
```

---

## 📈 性能基准

### Apple Silicon Mac（M1/M2/M3）

**屏幕捕获**:
- FPS: 30
- CPU: 5-8%
- 内存: 50 MB

**视频编码** (OpenH264):
- 1080p @ 2 Mbps
- CPU: 15-20%
- 延迟: 8-12 ms

**视频解码** (OpenH264):
- 1080p
- CPU: 10-15%
- 延迟: 5-8 ms

---

### Intel Mac（i5/i7/i9）

**屏幕捕获**:
- FPS: 30
- CPU: 8-12%
- 内存: 50 MB

**视频编码** (OpenH264):
- 1080p @ 2 Mbps
- CPU: 20-30%
- 延迟: 12-18 ms

**视频解码** (OpenH264):
- 1080p
- CPU: 15-20%
- 延迟: 8-12 ms

---

## ✅ 验收标准

### MVP 功能完整性

- [ ] **屏幕共享**: 主控端能看到被控端实时画面
- [ ] **视频质量**: 文字清晰，办公场景可用
- [ ] **流畅度**: FPS ≥ 20，延迟 ≤ 100ms
- [ ] **鼠标控制**: 移动、点击、双击、滚轮正常
- [ ] **键盘控制**: 英文输入正常
- [ ] **稳定性**: 持续运行 5 分钟无崩溃
- [ ] **带宽**: 平均 ≤ 2 MB/s

### 跨架构兼容性

- [ ] Apple Silicon → Intel 控制正常
- [ ] Intel → Apple Silicon 控制正常
- [ ] 两种架构性能都可接受

---

## 📝 测试记录模板

```markdown
## 测试日期: 2026-06-29

### 环境信息

**主控端 (Apple Silicon Mac)**:
- 型号: MacBook Pro M1/M2/M3
- macOS 版本: 14.x
- 屏幕分辨率: 3024×1964
- 局域网 IP: 192.168.x.x

**被控端 (Intel Mac)**:
- 型号: MacBook Pro Intel i7
- macOS 版本: 14.x
- 屏幕分辨率: 1920×1080
- 局域网 IP: 192.168.x.x

### 测试结果

| 测试项 | 预期 | 实际 | 状态 | 备注 |
|--------|------|------|------|------|
| 本地回环测试 | 30 FPS | __ FPS | ☐ Pass ☐ Fail | |
| Flutter UI 显示 | 显示画面 | __ | ☐ Pass ☐ Fail | |
| 静态场景带宽 | 0.025 MB/s | __ MB/s | ☐ Pass ☐ Fail | |
| 鼠标移动 | 跟随 | __ | ☐ Pass ☐ Fail | |
| 鼠标点击 | 响应 | __ | ☐ Pass ☐ Fail | |
| 键盘输入 | 正常 | __ | ☐ Pass ☐ Fail | |

### 性能数据

**静态桌面**:
- FPS: __
- 带宽: __ MB/s
- 延迟: __ ms

**文字编辑**:
- FPS: __
- 带宽: __ MB/s
- 延迟: __ ms

**视频播放**:
- FPS: __
- 带宽: __ MB/s
- 延迟: __ ms

### 问题记录

1. 
2. 
3. 

### 总结

- MVP 功能: ☐ 完成 ☐ 未完成
- 跨架构兼容: ☐ 通过 ☐ 失败
- 下一步: 
```

---

## 🚀 后续优化

完成基础测试后，可以考虑：

### 1. WebRTC 集成（2-3 天）
- 实现信令协议
- 集成 ICE 穿透
- 支持真实远程连接

### 2. 性能优化（1-2 天）
- 切换到 VideoToolbox 硬件编码
- 实现自适应码率
- 优化内存占用

### 3. 功能增强（按需）
- 音频传输
- 文件传输
- 剪贴板同步
- 多显示器支持

---

**创建日期**: 2026-06-29  
**维护者**: RDCS Team  
**状态**: Ready for Testing 🎯
