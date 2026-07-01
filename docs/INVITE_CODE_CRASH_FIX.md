# 邀请码生成崩溃问题 - 根本原因分析与修复报告

## 问题描述

客户端点击"生成邀请码"按钮时应用崩溃。

## 根本原因分析（Phase 1）

### 证据收集

1. **日志分析**：Flutter 日志显示 `librdcs_core.dylib` 加载失败
   ```
   Failed to load librdcs_core.dylib. Tried:
     1. .../Frameworks/librdcs_core.dylib (exists: false)
     2. librdcs_core.dylib (system path)
     3. ./librdcs_core.dylib (exists: false)
   ```

2. **文件验证**：库文件实际存在于 Frameworks 目录
   ```bash
   $ ls -lh build/macos/.../Frameworks/librdcs_core.dylib
   -rwxr-xr-x  5.4M  librdcs_core.dylib
   ```

3. **install_name 检查**：库的 install_name 配置错误
   ```bash
   $ otool -D .../Frameworks/librdcs_core.dylib
   /Users/lc/.../target/debug/deps/librdcs_core.dylib  # ❌ 绝对路径
   ```

### 根本原因

**dylib 的 install_name 使用了绝对路径而非 @rpath**

- Rust cdylib 默认使用构建目录的绝对路径作为 install_name
- Flutter FFI 通过 `DynamicLibrary.open()` 加载库时无法解析绝对路径
- Engine isolate 初始化时库加载失败，导致后续所有 FFI 调用崩溃

### 调用链分析

```
用户点击按钮
  → HomePage._generateInviteCode()
    → ref.read(engineIsolateProvider).generateInvite()
      → EngineIsolate._sendCommandForString()
        → 向 isolate 发送命令
          → Isolate 尝试加载库 (RdcsBindings())
            ❌ DynamicLibrary.open() 失败（找不到绝对路径）
              → 抛出异常，isolate 崩溃
                → 应用无响应/崩溃
```

## 模式分析（Phase 2）

### 对比分析

1. **copy_rust_lib.sh**（旧脚本）
   - ✅ 有 `install_name_tool` 修复逻辑
   - ❌ 期望源文件名 `librdcs_ffi.dylib`（已过时）
   - ❌ 从未执行（源文件不存在）

2. **copy_ffi_lib.sh**（当前脚本）
   - ✅ 正确的源文件名 `librdcs_core.dylib`
   - ✅ 成功复制库到 Frameworks
   - ❌ 缺少 `install_name_tool` 修复

### 发现的问题

- 构建脚本迁移不完整，新脚本缺少关键的修复步骤
- 所有 Rust cdylib 都有同样的问题（默认行为）

## 假设与测试（Phase 3）

### 假设

修复 `copy_ffi_lib.sh`，添加 `install_name_tool` 命令将 install_name 改为 `@rpath/librdcs_core.dylib`，即可解决库加载失败问题。

### 验证方法

1. 修改构建脚本
2. 手动修复已打包的库（快速验证）
3. 重新构建并测试（完整验证）

## 实现修复（Phase 4）

### 1. 修复构建脚本

**文件：** `client/flutter/macos/copy_ffi_lib.sh`

**修改：** 在复制库后添加 install_name 修复：

```bash
# Copy library
cp -v "$LIB_SRC" "$LIB_DEST"

# Fix install_name to use @rpath so Flutter can load it dynamically
echo "🔧 Fixing install_name to @rpath..."
install_name_tool -id "@rpath/librdcs_core.dylib" "$LIB_DEST"

# Verify the fix
echo "✅ Verifying install_name:"
otool -D "$LIB_DEST"
```

### 2. 手动修复（立即生效）

```bash
install_name_tool -id "@rpath/librdcs_core.dylib" \
  build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib
```

### 3. 验证修复

```bash
$ otool -D .../Frameworks/librdcs_core.dylib
@rpath/librdcs_core.dylib  # ✅ 相对路径
```

### 4. 测试结果

**自动化检查：**
- ✅ 库文件存在
- ✅ install_name 正确配置为 `@rpath/librdcs_core.dylib`
- ✅ 库依赖正常

**手动测试步骤：**
1. 运行应用：`cd client/flutter && flutter run -d macos`
2. 点击"生成邀请码"按钮
3. 预期结果：
   - 无崩溃
   - 显示对话框，包含 4 位数字邀请码
   - 控制台输出：`✅ Generated invite code: XXXX`

## 影响范围

### 受影响的功能

所有依赖 `librdcs_core.dylib` 的 FFI 调用：
- ✅ **生成邀请码**（本次修复）
- ✅ 屏幕捕获启动
- ✅ 远程连接
- ✅ 输入事件发送
- ✅ 所有其他 FFI 功能

### 平台特定性

- **macOS**: ✅ 已修复（dylib install_name 问题）
- **Linux**: 需要验证（.so 文件也可能有类似问题）
- **Windows**: 不受影响（.dll 不使用 install_name）

## 预防措施

### 1. 自动化验证

创建 CI 检查确保库的 install_name 正确：

```bash
# .github/workflows/build.yml
- name: Verify library install_name
  run: |
    otool -D build/.../librdcs_core.dylib | grep "@rpath"
```

### 2. 文档更新

在构建文档中添加 install_name 检查步骤。

### 3. 脚本改进

考虑在 Rust 构建级别设置正确的 install_name：

```toml
# crates/rdcs-ffi/Cargo.toml
[target.'cfg(target_os = "macos")'.build-dependencies]
# 添加构建脚本自动设置 install_name
```

## 总结

### 问题根源

Rust cdylib 默认使用绝对路径作为 install_name，导致 Flutter 动态加载失败。

### 解决方案

在复制库到 app bundle 时使用 `install_name_tool` 将 install_name 改为 `@rpath/librdcs_core.dylib`。

### 修复状态

- ✅ 根本原因已识别
- ✅ 构建脚本已修复
- ✅ 当前构建已手动修复
- ⏳ 等待手动测试确认

### 后续步骤

1. 运行手动测试验证邀请码生成功能
2. 完全重新构建验证脚本修复生效
3. 添加 CI 自动化检查
4. 检查 Linux 平台是否有类似问题

---

**调试方法论：** 本次调试严格遵循系统化调试流程（systematic-debugging skill）：
- Phase 1: 收集证据（日志、文件、链接信息）
- Phase 2: 模式分析（对比工作/失败的代码路径）
- Phase 3: 形成假设（单一、可测试的假设）
- Phase 4: 实现修复（最小化修改）

**关键洞察：** "库文件存在但加载失败" 的问题通常是链接配置问题，而非文件缺失或代码逻辑错误。
