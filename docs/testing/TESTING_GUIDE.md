# RDCS 测试指南

**日期**: 2026-06-29  
**状态**: 开发测试中

---

## 📋 当前部署状态

### 后端服务（本地开发模式）

| 服务 | 状态 | 地址 | 说明 |
|------|------|------|------|
| PostgreSQL | ✅ Docker | `localhost:5432` | 数据库 |
| Redis | ✅ Docker | `localhost:6379` | 缓存 |
| Go API | ✅ 本地 | `http://localhost:8080` | REST API + WebSocket |
| Web 管理台 | ✅ 本地 | `http://localhost:3000` | React 管理控制台 |
| Rust 信令服务 | ❌ 未部署 | - | WebRTC 信令 |
| Rust 中继服务 | ❌ 未部署 | - | STUN/TURN 中继 |

### Flutter 客户端

| 组件 | 状态 | 说明 |
|------|------|------|
| UI 界面 | ✅ 正常 | 主页、设置页可用 |
| FFI 库 | ✅ 已加载 | `librdcs_core.dylib` |
| 设置功能 | ✅ 可用 | 网络配置、质量设置等 |
| 邀请码生成 | 🔄 测试中 | 返回硬编码 "0000" |

---

## 🚀 快速启动指南

### 1. 启动数据库服务

```bash
cd /Users/lc/Development/source/remote-desktop-controller/deploy/docker
docker compose -f docker-compose.minimal.yml up -d
```

验证：
```bash
docker ps
# 应该看到 rdcs-postgres 和 rdcs-redis 运行中
```

### 2. 启动 Go API 服务

```bash
cd /Users/lc/Development/source/remote-desktop-controller/services/api
go run cmd/api/main.go
```

验证：
```bash
curl http://localhost:8080/healthz
# 应该返回: ok
```

### 3. 启动 Web 管理台（可选）

```bash
cd /Users/lc/Development/source/remote-desktop-controller/web/admin
npm install  # 首次运行需要
npm run dev
```

访问：`http://localhost:3000`

### 4. 启动 Flutter 客户端

```bash
cd /Users/lc/Development/source/remote-desktop-controller/client/flutter

# 编译 Rust FFI 库（首次或代码更新后）
cd ../..
cargo build --release --lib -p rdcs-ffi

# 复制库文件
cp target/release/librdcs_ffi.dylib \
   client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib

# 运行 Flutter 应用
cd client/flutter
flutter run
```

---

## 🔑 默认测试账号

### Web 管理台登录

| 角色 | 邮箱 | 密码 | 权限 |
|------|------|------|------|
| 管理员 | `admin@rdcs-test.local` | `test123` | Owner（完全权限） |
| 经理 | `manager@rdcs-test.local` | `test123` | Manager（管理权限） |
| 成员 | `member@rdcs-test.local` | `test123` | Member（普通权限） |

### 测试设备代码

数据库中预置了 5 个测试设备：

| 设备代码 | 设备名称 | 平台 | 状态 |
|---------|---------|------|------|
| 100200301 | MacBook-Pro-张三 | macOS 15.0 | online |
| 100200302 | MacBook-Air-李四 | macOS 14.5 | online |
| 100200303 | iMac-会议室A | macOS 15.0 | offline |
| 100200304 | DESKTOP-王五 | Windows 11 | online |
| 100200305 | ubuntu-server-01 | Ubuntu 24.04 | offline |

---

## 🔧 Flutter 客户端配置

### 服务器地址配置

1. 启动 Flutter 应用
2. 点击右上角 ⚙️ 设置图标
3. 切换到"网络设置"标签
4. 配置服务器地址：

```
管理 API (API URL):        http://localhost:8080
信令服务器 (Rendezvous URL): 留空（未部署）
中继服务器 (Relay URL):      留空（未部署）
```

5. 点击"保存服务器配置"

### FFI 库问题排查

如果应用黑屏或提示 `Failed to load librdcs_core.dylib`：

```bash
# 1. 检查库文件是否存在
ls -lh client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib

# 2. 如果不存在，重新复制
cp target/release/librdcs_ffi.dylib \
   client/flutter/build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks/librdcs_core.dylib

# 3. 重启 Flutter 应用（在运行中的终端按 'R' 或重新 flutter run）
```

**注意**：`flutter clean` 会删除这个文件，需要重新复制！

---

## 🧪 功能测试清单

### 基础功能

- [ ] Web 管理台登录
  - [ ] 使用 admin@rdcs-test.local 登录
  - [ ] 查看设备列表
  - [ ] 查看连接记录

- [ ] Flutter 客户端启动
  - [ ] 应用正常显示主页
  - [ ] 显示设备代码（自动生成的 9 位数字）
  - [ ] 设置页面可以打开

- [ ] 服务器配置
  - [ ] 在 Flutter 设置中配置 API 地址
  - [ ] 配置保存到 `~/.rdcs/config.json`
  - [ ] 重启应用后配置保持

### 邀请码功能（当前测试中）

- [🔄] 生成邀请码
  - [ ] 点击"生成邀请码"按钮
  - [ ] 弹出对话框显示邀请码
  - [ ] 当前返回硬编码 "0000"（待实现真实逻辑）
  - [ ] 复制功能正常

### 双机连接测试（待测试）

- [ ] Apple Silicon Mac 作为控制端
  - [ ] 生成邀请码
  - [ ] 建立连接
  
- [ ] Intel Mac 作为被控端
  - [ ] 输入邀请码
  - [ ] 接受连接请求
  - [ ] 显示远程桌面画面

- [ ] 输入控制
  - [ ] 鼠标移动
  - [ ] 鼠标点击
  - [ ] 键盘输入
  - [ ] 滚轮滚动

---

## 🐛 已知问题

### 1. 邀请码生成返回硬编码值

**现象**：点击"生成邀请码"返回 "0000"

**原因**：Rust FFI 函数 `rdcs_generate_invite` 当前返回硬编码值（第 722 行）

**位置**：`crates/rdcs-ffi/src/lib.rs:722`

```rust
// TODO: Wire to invite code service (generates 4-digit code, stores in signaling)
let code = "0000"; // Placeholder
```

**待修复**：需要实现真实的邀请码生成逻辑，连接到信令服务器

### 2. Docker 完整构建失败

**现象**：`./deploy_backend.sh` 构建失败

**原因**：
- Go 版本要求不匹配（已修复：1.25 → 1.23）
- Rust 版本要求不匹配（已修复：1.80 → 1.83）
- FFI 库路径问题

**临时方案**：使用最小化部署（仅数据库）+ 本地运行 API 服务

### 3. Web 管理台 TypeScript 导出错误

**现象**：`No matching export in "src/api/client.ts" for import "apiClient"`

**状态**：✅ 已修复（添加命名导出）

### 4. Flutter hot restart 后 FFI 库丢失

**现象**：`flutter clean` 或重新构建后应用黑屏

**原因**：库文件被清理，需要重新复制

**解决**：重新执行复制命令（见上文"FFI 库问题排查"）

---

## 📝 测试日志

### 2026-06-29

- ✅ 部署 PostgreSQL + Redis (Docker)
- ✅ 启动 Go API 服务 (本地)
- ✅ 修复 go.mod 版本要求 (1.25 → 1.23)
- ✅ 编译 Rust FFI 库 (`librdcs_ffi.dylib`)
- ✅ 修复 Flutter FFI 库加载路径
- ✅ Flutter 主页添加设置入口按钮
- ✅ 修复 Web 管理台 TypeScript 导出错误
- ✅ 确认默认测试账号可用
- 🔄 测试邀请码生成功能（返回硬编码值）

---

## 🔗 相关文档

- [后端部署指南](BACKEND_DEPLOYMENT.md)
- [开发指南](DEVELOPMENT.md)
- [架构文档](ARCHITECTURE.md)

---

**维护者**: RDCS Team  
**最后更新**: 2026-06-29
