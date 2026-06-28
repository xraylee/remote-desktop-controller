# 🚀 立即执行：livekit/webrtc-sys 集成

**执行时间**: 现在  
**预计用时**: 5-10 分钟（步骤 1）

---

## 📋 执行前提

在开始之前，请确认：

1. ✅ 你在 macOS 上（Apple Silicon）
2. ❓ Rust 是否已安装？

**检查命令**（在 Mac 终端运行）:
```bash
rustc --version
cargo --version
```

- 如果显示版本号 → 继续下面的步骤
- 如果显示 "command not found" → 先运行 `./install-china-mirror.sh` 安装 Rust

---

## 🎯 立即执行

### 在 Mac 终端运行以下命令：

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 执行集成脚本（带完整验证）
./execute-livekit-integration.sh
```

---

## 📊 脚本会做什么

这个脚本会自动完成以下步骤，每一步都有验证：

### ✅ 步骤 0: 前置检查
- 检查 Rust 是否安装
- 检查 Cargo 镜像配置
- 验证项目目录

### ✅ 步骤 1: 添加 livekit 依赖
- 验证 `Cargo.toml` 已包含 livekit
- 验证 `crates/rdcs-codec/Cargo.toml` 已包含 livekit
- 下载所有依赖（使用 rsproxy.cn 镜像）
- 验证依赖树正确加载

### ✅ 步骤 2: 编译验证
- 编译 `rdcs-codec` 包
- 编译整个 workspace
- 确保没有编译错误

### ✅ 步骤 3: 运行现有测试（基线）
- 运行 `rdcs-codec` 单元测试
- 验证现有功能正常（使用 mock simulator）
- 建立性能基线

---

## 🎉 成功标志

脚本成功后，你会看到：

```
==========================================
✅ 步骤 1 完成：依赖集成成功
==========================================

📊 验收结果：

  ✓ Rust 环境检查通过
  ✓ Cargo.toml 配置正确
  ✓ livekit 依赖下载成功
  ✓ 依赖树验证通过
  ✓ rdcs-codec 编译通过
  ✓ Workspace 编译通过
  ✓ 现有测试通过（基线）

🎯 下一步：
  
  现在可以开始替换编解码器实现...
```

---

## 🚨 如果遇到问题

### 问题 1: "Rust 未安装"

**解决方案**:
```bash
# 先安装 Rust 环境
./install-china-mirror.sh

# 然后重新运行
./execute-livekit-integration.sh
```

### 问题 2: 依赖下载慢或失败

**检查镜像配置**:
```bash
cat .cargo/config.toml
# 应该看到 rsproxy.cn
```

**如果镜像未配置，手动配置**:
```bash
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = 'rsproxy-sparse'

[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"
EOF
```

### 问题 3: 编译错误

**查看详细日志**:
```bash
cargo check -p rdcs-codec --verbose
```

**清理并重试**:
```bash
cargo clean
./execute-livekit-integration.sh
```

---

## 📝 执行后的输出

脚本执行完成后，请将输出结果复制给我，我需要检查：

1. ✅ livekit 依赖是否正确下载
2. ✅ 编译是否通过
3. ✅ 现有测试是否全部通过
4. ✅ 是否有任何警告或错误

---

## 🎯 下一步预告

步骤 1 完成后，我会：

1. **替换 WebRtcEncoder** - 将 simulator 改为 livekit::VideoEncoder
2. **替换 WebRtcDecoder** - 将 simulator 改为 livekit::VideoDecoder
3. **运行集成测试** - 验证真实编解码工作
4. **性能验证** - 测试硬件加速和 CPU 占用
5. **最终验收** - 确认所有指标达标

---

## ⏱️ 时间预期

- **步骤 1**（现在）: 5-10 分钟
- **步骤 2-3**（替换实现）: 我来完成
- **步骤 4-5**（测试验证）: 10-15 分钟（需要你在 Mac 上运行）

**总计**: 约 30-40 分钟完成完整集成

---

**立即开始 → 在 Mac 终端运行**:
```bash
cd /Users/lc/Development/source/remote-desktop-controller
./execute-livekit-integration.sh
```

执行完成后，把输出结果告诉我！
