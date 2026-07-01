# 🚀 Flutter APP 启动指南

**问题**: FFI 库文件未找到  
**状态**: ✅ 已修复（库已复制到正确位置）  
**需要**: 重新启动 Flutter 应用

---

## 立即执行

在当前 Flutter 窗口按 `q` 退出，然后重新运行：

```bash
# 在项目根目录
cd /Users/lc/Development/source/remote-desktop-controller/client/flutter

# 重新启动
flutter run -d macos
```

**或者使用热重启**：在当前 Flutter 窗口按 `R`（大写 R）

---

## 预期成功日志

```
flutter: ✅ Loading from: .../Contents/Frameworks/librdcs_core.dylib
flutter: ✅ Engine created successfully
flutter: ✅ Connected to signaling server
```

---

## 已完成的修复

1. ✅ FFI 库名称配置（`rdcs_core`）
2. ✅ FFI 库编译（5.3 MB）
3. ✅ 库文件复制到应用包
4. ⏳ 需要重新启动应用

---

## 库文件位置

**源文件**: `/Users/lc/Development/source/remote-desktop-controller/target/debug/librdcs_core.dylib`

**目标位置**: `/Users/lc/Development/source/remote-desktop-controller/client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib`

✅ **已验证存在** (5.3M)

---

## 自动化脚本（下次使用）

下次重新构建时，使用快速启动脚本：

```bash
# 在项目根目录
./quick-start-flutter.sh
```

这个脚本会自动：
1. 检查 FFI 库
2. 构建 Flutter 应用
3. 复制库文件
4. 验证

---

## 故障排查

如果重启后仍然失败：

```bash
# 1. 清理构建
cd /Users/lc/Development/source/remote-desktop-controller/client/flutter
flutter clean

# 2. 重新构建
flutter build macos --debug

# 3. 复制库文件
mkdir -p build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks
cp ../../target/debug/librdcs_core.dylib build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/

# 4. 运行
flutter run -d macos
```

---

## Xcode 自动化（推荐）

为了避免每次都手动复制，请在 Xcode 中添加构建阶段：

1. 打开 `client/flutter/macos/Runner.xcworkspace`
2. 选择 Runner target
3. Build Phases → + → New Run Script Phase
4. 脚本内容：

```bash
"$PROJECT_DIR/copy_ffi_lib.sh"
```

5. 将此 phase 拖到 "Embed Frameworks" 之前

---

**当前状态**: ✅ 库已复制，等待应用重启

**下一步**: 按 `q` 退出，然后 `flutter run -d macos`
