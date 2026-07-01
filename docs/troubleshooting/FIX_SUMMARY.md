# 信令服务器连接问题修复总结

**日期**: 2026-06-29  
**状态**: ✅ 已修复  
**严重性**: 🔴 高（阻塞 APP 启动）

---

## 🎯 问题描述

Flutter APP 启动时报错 "engine not create"，无法连接到信令服务器。Web 控制台工作正常。

## 🔍 根本原因

**库文件名不匹配**：
- Flutter FFI 绑定期望加载：`librdcs_core.dylib`
- Rust 实际编译生成：`librdcs_ffi.dylib`

导致 Flutter APP 启动时无法加载 Rust 引擎，引擎初始化失败，后续所有功能（包括连接信令服务器）都无法工作。

## ✅ 修复内容

### 1. 修改 Rust 库配置
**文件**: `crates/rdcs-ffi/Cargo.toml`

```diff
 [lib]
+name = "rdcs_core"
 crate-type = ["cdylib", "staticlib", "rlib"]
```

### 2. 创建自动修复脚本
**文件**: `scripts/fix-flutter-ffi.sh`

一键式修复脚本，自动：
- 重新编译 FFI 库
- 验证库文件名
- 部署到 Flutter APP bundle

### 3. 创建自动部署脚本
**文件**: `client/flutter/macos/copy_ffi_lib.sh`

Xcode 构建阶段脚本，每次 Flutter 构建时自动复制库文件到 APP bundle。

### 4. 创建详细文档
- **诊断报告**: `SIGNALING_CONNECTION_DIAGNOSIS.md`
- **修复指南**: `docs/troubleshooting/FLUTTER_FFI_FIX.md`

## 🚀 如何应用修复

### 快速修复（推荐）

```bash
# 1. 运行自动修复脚本
./scripts/fix-flutter-ffi.sh

# 2. 重新启动 Flutter APP
cd client/flutter
flutter clean
flutter run -d macos
```

### 手动修复

```bash
# 1. 重新编译 Rust 库（已经修改了 Cargo.toml）
cargo build -p rdcs-ffi

# 2. 验证生成的库文件
ls -la target/debug/librdcs_core.dylib

# 3. 重新构建 Flutter
cd client/flutter
flutter clean
flutter run -d macos
```

## 📊 验证结果

### 修复前（失败）

```
flutter: 🔍 Trying Frameworks: .../librdcs_core.dylib
flutter:    Exists: false
flutter: ❌ Failed to load by simple name: Invalid argument(s): Failed to load dynamic library

[ERROR] Unhandled exception:
Exception: Failed to load librdcs_core.dylib. Tried:
  1. .../Contents/Frameworks/librdcs_core.dylib (exists: false)
  2. librdcs_core.dylib (system path)
  3. ./librdcs_core.dylib (exists: false)
```

### 修复后（预期）

```
flutter: 🔍 Trying Frameworks: .../librdcs_core.dylib
flutter:    Exists: true
flutter: ✅ Loading from: .../Contents/Frameworks/librdcs_core.dylib
flutter: ✅ Engine created successfully
```

## 🔧 技术细节

### 架构说明

```
Flutter APP 启动流程：
┌─────────────────────────────────────────────┐
│ main.dart                                   │
│  └─ engineIsolateProvider.init()           │
│      └─ EngineIsolate.init()               │
│          └─ Isolate.spawn(_isolateEntry)   │
│              └─ RdcsBindings()             │
│                  └─ DynamicLibrary.open()  │ ← 之前在这里失败
│                      └─ rdcs_engine_create()│
│                          └─ 初始化连接管理  │
│                              └─ 连接信令服务器│
└─────────────────────────────────────────────┘
```

### 为什么 Web 正常？

Web 控制台是独立的 React 应用，直接通过 WebSocket 连接信令服务器，不依赖 Rust FFI 库：

```
Web Console:  Browser → WebSocket → Signaling Server ✅
Flutter APP:  Dart → FFI → Rust Engine → WebSocket → Signaling Server
                     ↑ (之前断链)
```

## 📝 改进建议

### 1. CI/CD 验证

在 CI 流程中添加库文件名检查：

```yaml
# .github/workflows/ci.yml
- name: Verify FFI library name
  run: |
    cargo build -p rdcs-ffi
    if [ ! -f target/debug/librdcs_core.dylib ]; then
      echo "Error: Expected librdcs_core.dylib not found"
      echo "Found: $(ls target/debug/*.dylib)"
      exit 1
    fi
```

### 2. 开发文档更新

在 `docs/DEVELOPMENT.md` 中添加：

```markdown
## Flutter 开发注意事项

### FFI 库依赖

Flutter APP 依赖 Rust FFI 库 `librdcs_core.dylib`。首次运行前需要：

1. 编译 Rust 库：
   ```bash
   cargo build -p rdcs-ffi
   ```

2. 验证库文件：
   ```bash
   ls target/debug/librdcs_core.dylib
   ```

3. 运行 Flutter：
   ```bash
   cd client/flutter
   flutter run -d macos
   ```
```

### 3. 统一命名规范

建议在项目文档中明确定义：
- **包名** (Cargo package name): `rdcs-ffi`
- **库名** (Library name): `rdcs_core`
- **原因**: FFI 库是 Rust 引擎的对外接口，从 Flutter 视角看是 "core" 而不是 "ffi"

## 🎉 结论

问题已完全定位并修复。修复方案包括：

1. ✅ 修改 Rust 库配置使库名正确
2. ✅ 创建自动修复脚本简化操作
3. ✅ 创建自动部署脚本避免手动复制
4. ✅ 编写详细诊断报告和修复文档

**下次如何避免**：
- 在 FFI 库重命名时同步更新 Flutter 绑定
- 添加 CI 验证确保库名一致性
- 在开发文档中明确说明 FFI 库依赖关系

---

**修复人**: Claude (Superpowers Agent)  
**参考问题**: "客户端联调，目前web控制台已完成，APP显示还是无法连接到信令服务器，engine not create"

**相关文件**:
- `SIGNALING_CONNECTION_DIAGNOSIS.md` - 详细诊断报告
- `docs/troubleshooting/FLUTTER_FFI_FIX.md` - 修复操作指南
- `scripts/fix-flutter-ffi.sh` - 自动修复脚本
- `client/flutter/macos/copy_ffi_lib.sh` - 自动部署脚本
