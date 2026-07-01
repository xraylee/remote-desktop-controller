# Flutter FFI 库连接问题修复指南

## 问题概述

Flutter APP 启动时报错 "engine not create"，无法连接到信令服务器。根本原因是 FFI 库名称不匹配。

## 快速修复（5分钟）

### 方案一：运行自动修复脚本（推荐）

```bash
# 在项目根目录执行
./scripts/fix-flutter-ffi.sh
```

脚本会自动：
1. 重新编译 Rust FFI 库（使用正确的名称）
2. 验证库文件名
3. 部署到 Flutter APP bundle（如果已构建）

### 方案二：手动修复

#### 步骤 1: 修改 Cargo.toml（已完成）

文件：`crates/rdcs-ffi/Cargo.toml`

```toml
[lib]
name = "rdcs_core"  # ✅ 已修改
crate-type = ["cdylib", "staticlib", "rlib"]
```

#### 步骤 2: 重新编译库

```bash
cargo build -p rdcs-ffi
```

验证：
```bash
ls -la target/debug/librdcs_core.dylib
# 应该看到 5-6 MB 的文件
```

#### 步骤 3: 重新构建 Flutter APP

```bash
cd client/flutter
flutter clean
flutter run -d macos
```

## 验证修复

### 成功标志

Flutter 启动日志应该显示：

```
flutter: 🔍 Executable path: .../rdcs_client.app/Contents/MacOS/rdcs_client
flutter: 🔍 Trying Frameworks: .../Contents/Frameworks/librdcs_core.dylib
flutter:    Exists: true
flutter: ✅ Loading from: .../Contents/Frameworks/librdcs_core.dylib
```

**没有**看到：
```
Exception: Failed to load librdcs_core.dylib
```

### 功能测试

1. APP 启动正常，无崩溃
2. 可以生成邀请码（Generate Invite）
3. 可以连接到远程设备

## 故障排查

### 问题 1: 库文件仍然是 librdcs_ffi.dylib

**症状**：编译后还是生成 `target/debug/librdcs_ffi.dylib`

**解决方案**：
```bash
# 清理构建缓存
cargo clean -p rdcs-ffi

# 验证 Cargo.toml 修改
grep "name = " crates/rdcs-ffi/Cargo.toml
# 应该看到: name = "rdcs_core"

# 重新编译
cargo build -p rdcs-ffi
```

### 问题 2: Flutter 仍然报错找不到库

**症状**：库文件存在但 Flutter 加载失败

**解决方案**：

1. 确认库文件路径：
   ```bash
   ls -la client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/
   # 应该看到 librdcs_core.dylib
   ```

2. 如果不存在，手动复制：
   ```bash
   mkdir -p client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/
   cp target/debug/librdcs_core.dylib \
      client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/
   ```

3. 清理并重启：
   ```bash
   cd client/flutter
   flutter clean
   flutter run -d macos
   ```

### 问题 3: 编译时找不到 rdcs-ffi

**症状**：`cargo build -p rdcs-ffi` 报错

**解决方案**：
```bash
# 使用完整构建命令
cargo build --package rdcs-ffi

# 或构建整个 workspace
cargo build
```

## 长期方案：自动化部署

为避免每次都手动复制库文件，建议配置 Xcode 构建脚本。

### 配置步骤（可选）

1. 在 Xcode 中打开 `client/flutter/macos/Runner.xcodeproj`

2. 选择 "Runner" target → "Build Phases"

3. 点击 "+" → "New Run Script Phase"

4. 添加脚本：
   ```bash
   cd "$PROJECT_DIR"
   ./copy_ffi_lib.sh
   ```

5. 拖动此 Phase 到 "Embed Frameworks" 之前

这样每次 Flutter 构建时会自动复制库文件。

## 为什么 Web 控制台正常？

Web 控制台是纯 TypeScript/React 应用，不依赖 Rust FFI 库：

```
架构对比：

Web Console:
  浏览器 → WebSocket → 信令服务器 ✅
  (无需 Rust 库)

Flutter APP:
  Dart → FFI → librdcs_core.dylib → Rust Engine → WebSocket → 信令服务器
           ↑
      之前在这里失败（库名不匹配）
```

## 相关文件

- **诊断报告**: `SIGNALING_CONNECTION_DIAGNOSIS.md`
- **修复脚本**: `scripts/fix-flutter-ffi.sh`
- **部署脚本**: `client/flutter/macos/copy_ffi_lib.sh`
- **库配置**: `crates/rdcs-ffi/Cargo.toml`
- **FFI 绑定**: `client/flutter/lib/core/ffi/bindings.dart`

## 需要帮助？

如果问题仍未解决，请提供以下信息：

1. Flutter 启动完整日志
   ```bash
   cd client/flutter
   flutter run -d macos 2>&1 | tee flutter_debug.log
   ```

2. 库文件列表
   ```bash
   ls -la target/debug/*.dylib
   ls -la client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/
   ```

3. Cargo.toml 内容
   ```bash
   cat crates/rdcs-ffi/Cargo.toml
   ```
