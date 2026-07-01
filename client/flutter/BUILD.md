# RDCS Flutter Client - Build Instructions

## 快速开始

### 方法 1：使用 Makefile（推荐）

```bash
cd client/flutter
make run          # 构建并运行（自动处理 Rust FFI）
make build        # 仅构建
make clean        # 清理
```

### 方法 2：使用 prebuild 脚本

```bash
cd client/flutter
./prebuild.sh     # 构建 Rust FFI 库
flutter run -d macos
```

### 方法 3：手动构建

```bash
# 1. 构建 Rust FFI 库
cargo build -p rdcs-ffi

# 2. 复制到 Flutter app bundle
mkdir -p client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks
cp target/debug/librdcs_core.dylib client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/

# 3. 运行 Flutter
cd client/flutter
flutter run -d macos
```

## 常见问题

### FFI 库加载失败

**错误:** `Failed to load librdcs_core.dylib`

**解决:** 运行 `./prebuild.sh` 或 `make run` 自动复制库

### WebSocket 连接失败

**错误:** `WebSocketChannelException: SocketException`

**原因:** macOS 系统代理干扰

**解决:** 已在代码中禁用代理（`findProxy = (uri) => 'DIRECT'`）

### 网络权限错误

**错误:** `Operation not permitted, errno = 1`

**解决:** 已在 `DebugProfile.entitlements` 中添加 `com.apple.security.network.client`

## 开发工作流

1. 修改 Rust 代码 → `cargo build -p rdcs-ffi`
2. 修改 Flutter 代码 → `r` 热重载
3. 重新加载 FFI → `R` 热重启

## 依赖检查

```bash
make check-deps   # 检查 Flutter、Cargo 等依赖
```
