# 信令服务器连接问题诊断报告

**日期**: 2026-06-29  
**问题**: Flutter APP 无法连接到信令服务器，报错 "engine not create"，而 Web 控制台正常工作

---

## 🔍 问题根源分析

### 核心问题
Flutter APP 启动时无法加载 Rust FFI 引擎库，导致引擎初始化失败。

### 技术细节

#### 1. **库文件名不匹配** (主要问题)
- **期望**: `librdcs_core.dylib`
- **实际**: `librdcs_ffi.dylib`
- **位置**: `target/debug/librdcs_ffi.dylib`

**证据** (flutter_output.log):
```
Exception: Failed to load librdcs_core.dylib. Tried:
  1. .../Contents/Frameworks/librdcs_core.dylib (exists: false)
  2. librdcs_core.dylib (system path)
  3. ./librdcs_core.dylib (exists: false)
```

**Flutter FFI 绑定代码** (`client/flutter/lib/core/ffi/bindings.dart:69`):
```dart
return DynamicLibrary.open('librdcs_core.dylib');
```

**实际库名称** (`crates/rdcs-ffi/Cargo.toml`):
```toml
[lib]
name = "rdcs_ffi"
crate-type = ["cdylib"]
```

#### 2. **库文件未复制到 APP Bundle**
Flutter APP 在以下路径查找库文件（均失败）:
1. `rdcs_client.app/Contents/Frameworks/librdcs_core.dylib` ❌
2. 系统库路径 ❌  
3. 当前工作目录 ❌

实际库文件位于: `target/debug/librdcs_ffi.dylib` (未被复制)

#### 3. **Web 控制台为何正常？**
Web 控制台不依赖 Rust 引擎库，它是纯 TypeScript/React 应用，直接通过 WebSocket 连接到信令服务器。

**架构对比**:
```
Web Console:
  Browser → WebSocket (ws://localhost:8443/ws) → Signaling Server ✅

Flutter APP:
  Dart → FFI → librdcs_ffi.dylib → Rust Engine → WebSocket → Signaling Server ❌
           ↑
      加载失败，流程中断
```

---

## 📊 系统状态检查

### 信令服务器状态
```bash
$ pgrep -f rdcs-signaling
1
2
```
✅ **信令服务器正在运行**

### 健康检查
```bash
$ curl http://localhost:8443/health
(连接失败)
```
⚠️ **HTTP 健康检查失败** - 可能是服务器配置问题（次要问题）

### 库文件验证
```bash
$ ls -la target/debug/*.dylib
-rwx------ 1 user user 5462592 Jun 29 05:01 target/debug/librdcs_ffi.dylib
```
✅ **Rust FFI 库已编译**

---

## 🛠️ 修复方案

### 方案 A: 修改库名称（推荐）
**优点**: 符合项目架构，代码语义清晰  
**工作量**: 低

**步骤**:
1. 修改 `crates/rdcs-ffi/Cargo.toml`:
   ```toml
   [lib]
   name = "rdcs_core"  # 改为 rdcs_core
   crate-type = ["cdylib"]
   ```

2. 重新编译:
   ```bash
   cargo build -p rdcs-ffi
   ```

3. 验证生成的库文件名:
   ```bash
   ls target/debug/librdcs_core.dylib
   ```

### 方案 B: 修改 Flutter 绑定
**优点**: 无需重新编译 Rust 代码  
**缺点**: 代码语义混乱（库名为 ffi 但引用为 core）

**步骤**:
1. 修改 `client/flutter/lib/core/ffi/bindings.dart`:
   ```dart
   // 第 69 行
   return DynamicLibrary.open('librdcs_ffi.dylib');  // 改为 rdcs_ffi
   ```

### 方案 C: 配置构建脚本自动复制
**目标**: 自动将库文件复制到 APP Bundle

**步骤**:
1. 创建 `client/flutter/macos/copy_lib.sh`:
   ```bash
   #!/bin/bash
   LIB_SRC="../../target/debug/librdcs_ffi.dylib"
   LIB_DST="$BUILT_PRODUCTS_DIR/$PRODUCT_NAME.app/Contents/Frameworks/"
   mkdir -p "$LIB_DST"
   cp "$LIB_SRC" "$LIB_DST/librdcs_core.dylib"
   ```

2. 在 Xcode 项目中添加 Build Phase (Run Script):
   ```bash
   cd "$PROJECT_DIR"
   ./copy_lib.sh
   ```

---

## ✅ 推荐实施方案

### 第一步: 统一库名称 (方案 A)
```bash
# 1. 修改 Cargo.toml
cd crates/rdcs-ffi
# 编辑 Cargo.toml，将 name = "rdcs_ffi" 改为 name = "rdcs_core"

# 2. 重新编译
cargo build -p rdcs-ffi

# 3. 验证
ls ../../target/debug/librdcs_core.dylib
```

### 第二步: 配置自动部署 (方案 C)
为了避免手动复制，建议添加构建脚本自动部署库文件。

### 第三步: 验证修复
```bash
# 1. 清理 Flutter 构建缓存
cd client/flutter
flutter clean

# 2. 重新运行 APP
flutter run -d macos

# 3. 检查日志，应该看到:
# ✅ Loading from: .../Contents/Frameworks/librdcs_core.dylib
```

---

## 🔧 次要问题修复 (可选)

### 信令服务器健康检查失败
可能原因：
1. 服务器监听在 `0.0.0.0:8443` 但未正确响应 HTTP 请求
2. Redis 连接问题
3. 缺少 `RDCS_HMAC_SECRET` 环境变量

**调试步骤**:
```bash
# 1. 检查信令服务器日志
journalctl -u rdcs-signaling -f

# 2. 检查环境变量
env | grep RDCS

# 3. 手动测试 WebSocket
wscat -c ws://localhost:8443/ws
```

---

## 📝 后续建议

### 1. 改进文档
在 `docs/DEVELOPMENT.md` 中添加 FFI 库部署说明。

### 2. 自动化测试
添加 CI 步骤验证库文件名一致性:
```bash
# .github/workflows/ci.yml
- name: Verify FFI library name
  run: |
    test -f target/debug/librdcs_core.dylib || \
      (echo "Error: librdcs_core.dylib not found" && exit 1)
```

### 3. 开发脚本
创建一键部署脚本 `scripts/deploy-flutter-lib.sh`:
```bash
#!/bin/bash
set -e
echo "Building Rust FFI library..."
cargo build -p rdcs-ffi
echo "Copying to Flutter app bundle..."
cp target/debug/librdcs_core.dylib \
   client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/
echo "✅ Deployment complete"
```

---

## 🎯 总结

| 问题 | 严重性 | 状态 |
|------|--------|------|
| 库文件名不匹配 | 🔴 严重 | 已定位 |
| 库文件未部署到 APP Bundle | 🔴 严重 | 已定位 |
| 信令服务器健康检查失败 | 🟡 次要 | 待调查 |

**关键操作**: 修改 `rdcs-ffi` 的 Cargo.toml，将库名从 `rdcs_ffi` 改为 `rdcs_core`，然后重新编译。

---

**诊断人**: Claude (Superpowers Agent)  
**参考文档**: 
- `client/flutter/lib/core/ffi/bindings.dart` (L69)
- `crates/rdcs-ffi/Cargo.toml`
- `client/flutter/flutter_output.log`
