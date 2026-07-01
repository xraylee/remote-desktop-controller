# RDCS 邀请码生成问题诊断报告

**问题**: 点击"生成邀请码"按钮显示 EngineException  
**状态**: 代码已修复，疑似库文件未重新编译/部署  
**日期**: 2026-06-30

---

## 🔍 问题分析

### 走查结果回顾

✅ **正常项**:
1. UI 显示正常
2. 设备代码正常显示（问题 1 已修复）

❌ **异常项**:
3. 生成邀请码出现 EngineException（问题 2 仍存在）
4. UI 交互未严格参照原型图设计（待后续优化）

---

## 🐛 根本原因分析

### 已确认的信息

1. ✅ **代码修改正确**
   - `crates/rdcs-ffi/Cargo.toml` 包含 `rand = "0.8"`
   - `Cargo.lock` 显示 `rand 0.8.6` 已锁定
   - `crates/rdcs-ffi/src/lib.rs` 使用 `rand::thread_rng()` 生成随机码

2. ⚠️ **可能的问题**
   - 库文件未重新编译
   - 旧版本库仍在 Flutter 应用中运行
   - 编译后未正确部署到 `Contents/Frameworks/` 目录

### EngineException 的可能来源

在 `engine_isolate.dart:269-274`：

```dart
Future<String> _sendCommandForString(_CommandType type, [String? payload]) async {
    // ...
    final response = await responsePort.first as List;
    final result = response[0];
    final error = response.length > 1 ? response[1] as String? : null;

    if (error != null) {
        throw EngineException(-1, error);  // ⬅️ 这里抛出异常
    }
    return result as String;
}
```

异常触发条件：
- Rust 侧 `rdcs_generate_invite()` 返回 `nullptr`
- 或 isolate 层捕获到其他错误

### Rust 侧返回 nullptr 的条件

在 `lib.rs:712-721`：

```rust
pub extern "C" fn rdcs_generate_invite(handle: *mut EngineHandle) -> *mut c_char {
    let engine = unsafe { handle.as_ref() };
    let Some(engine) = engine else {
        eprintln!("❌ rdcs_generate_invite: null handle");
        return ptr::null_mut();  // ⬅️ 条件 1
    };
    if engine.shutdown.load(Ordering::SeqCst) {
        eprintln!("❌ rdcs_generate_invite: engine is shutdown");
        return ptr::null_mut();  // ⬅️ 条件 2
    }
    
    // ... rand 代码 ...
}
```

**如果代码已更新但仍返回 nullptr，说明运行的是旧版本库（占位符实现）。**

---

## 🔧 修复步骤

### 方案 A: 强制重新编译和部署（推荐）

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 执行强制重新编译脚本
./rebuild_and_deploy.sh
```

这个脚本会：
1. 🧹 清理旧的编译产物 (`cargo clean -p rdcs-ffi`)
2. 🔨 重新编译库 (`cargo build`)
3. 🗑️ 删除已部署的旧库
4. 📦 复制新库到 Frameworks 目录
5. ✅ 验证文件大小一致性

### 方案 B: 手动操作（如脚本失败）

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 1. 清理
cargo clean -p rdcs-ffi

# 2. 重新编译
cargo build -p rdcs-ffi --features software-encoder

# 3. 删除旧库
rm -f client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib

# 4. 部署新库
cp target/debug/librdcs_core.dylib \
   client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/

# 5. 验证
ls -lh client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
```

### 重启应用（必须步骤）

⚠️ **重要**: 热重启（按 R）不会重新加载 FFI 库，必须完全重启！

```bash
# 1. 在 Flutter 窗口按 'q' 退出

# 2. 重新运行
cd client/flutter
flutter run -d macos
```

---

## 🧪 验证步骤

### 1. 检查编译是否包含 rand

编译完成后，检查符号表：

```bash
nm target/debug/librdcs_core.dylib | grep rand | head -5
```

预期输出应包含 rand 相关符号（可能为空也正常，取决于静态链接）。

### 2. 检查控制台日志

重启应用后，点击"生成邀请码"，观察控制台输出：

**成功的日志**:
```
✅ Generated invite code: 3847
```

**失败的日志**:
```
❌ rdcs_generate_invite: null handle
或
❌ rdcs_generate_invite: engine is shutdown
```

### 3. 验证功能

- ✅ 弹出 AlertDialog 显示 4 位数字
- ✅ 每次点击生成不同的随机数
- ❌ 不应显示 `EngineException`

---

## 📊 诊断工具

### 运行诊断脚本

```bash
cd /Users/lc/Development/source/remote-desktop-controller
./diagnose_invite.sh
```

这个脚本会检查：
- ✓ 源代码修改是否完整
- ✓ Cargo.lock 是否包含 rand
- ✓ 编译产物是否存在
- ✓ 部署的库文件是否最新
- ✓ 文件大小是否一致

---

## 🎯 常见问题

### Q1: 编译后仍然失败

**原因**: 可能是缓存问题或 Cargo 增量编译问题

**解决**:
```bash
cargo clean
cargo build -p rdcs-ffi --features software-encoder
```

### Q2: 文件大小一致但仍然失败

**原因**: 
1. Engine handle 为 null（FFI 引擎未正确初始化）
2. Engine 已 shutdown

**调试**:
- 检查 `main.dart` 是否调用了 `engine.init()`
- 查看控制台是否有 `❌ rdcs_generate_invite:` 日志

### Q3: 符号表检查失败

```bash
nm: error: archive member 'librdcs_core.dylib' with length...
```

**原因**: 库文件损坏

**解决**: 重新编译

---

## 📝 已知限制

### 当前实现

✅ **已实现**:
- 生成随机 4 位邀请码（0000-9999）
- 使用密码学安全的随机数生成器
- 详细的错误日志

⚠️ **未实现**（标记为 TODO）:
- 向信令服务器注册邀请码
- 邀请码过期时间
- 邀请码使用验证

### 后续改进计划

**Phase 3 规划**:
1. 实现信令服务器 API
2. 邀请码注册接口: `POST /api/invite/generate`
3. 邀请码验证接口: `POST /api/invite/verify`
4. 添加过期时间（默认 24 小时）

---

## 📚 相关文件

### 修复相关
- `crates/rdcs-ffi/Cargo.toml` - 添加 rand 依赖
- `crates/rdcs-ffi/src/lib.rs:707-734` - 邀请码生成函数
- `client/flutter/lib/features/home/home_page.dart:199-247` - UI 调用逻辑
- `client/flutter/lib/core/ffi/engine_isolate.dart:203-207` - FFI 桥接

### 工具脚本
- `rebuild_and_deploy.sh` - 强制重新编译和部署
- `diagnose_invite.sh` - 诊断工具
- `test_fixes.sh` - 整体测试脚本

---

## 🤝 反馈

如果执行 `rebuild_and_deploy.sh` 后问题仍然存在，请提供：

1. **控制台完整日志** (从启动到点击按钮)
2. **诊断脚本输出**: `./diagnose_invite.sh > diagnosis.log 2>&1`
3. **库文件信息**:
   ```bash
   ls -lh target/debug/librdcs_core.dylib
   ls -lh client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
   ```
4. **错误截图** (EngineException 对话框)

---

**诊断人**: AI 助手  
**文档版本**: v1.1  
**最后更新**: 2026-06-30
