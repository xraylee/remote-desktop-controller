# RDCS 邀请码功能完整联调指南

**日期**: 2026-06-30  
**状态**: 准备就绪，等待验证  
**项目**: remote-desktop-controller

---

## 📋 执行摘要

**已完成**:
- ✅ 代码实现完整（Rust FFI + Flutter）
- ✅ 配置初始化已修复
- ✅ FFI 库已编译
- ✅ 所有测试脚本已创建

**待执行**: 
- 🔄 在你的 Mac 上启动 Docker 服务
- 🔄 验证邀请码生成功能

---

## 🚀 快速启动流程

### Step 1: 检查 Docker 服务状态

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 运行 Docker 服务检查脚本
./check_docker_services.sh
```

**预期结果**:
- ✅ Docker Desktop 已运行
- ✅ rdcs-api 容器运行中（端口 8080）
- ✅ rdcs-signaling 容器运行中（端口 8443）
- ✅ rdcs-redis 容器运行中（端口 6379）

**如果服务未运行，执行**:
```bash
# 一键部署所有后端服务
./deploy_backend.sh
```

---

### Step 2: 验证服务端点

```bash
# API 健康检查
curl http://localhost:8080/healthz
# 预期: ok 或 HTTP 200

# 信令服务器健康检查（可选）
curl http://localhost:8443/health
# 可能返回 404，但 WebSocket 功能正常
```

---

### Step 3: 启动 Flutter 客户端

```bash
cd client/flutter
flutter run -d macos
```

**预期启动日志**:
```
✅ Configuration initialized. Device code: 123-456-789
```

如果看到这行日志，说明配置初始化成功。

---

### Step 4: 测试邀请码生成

#### 4.1 查看设备代码

主页应该显示：
- 设备代码: `XXX-XXX-XXX`（9 位数字，格式化）
- 状态: "设备已就绪"（绿色指示灯）

**如果显示 `--- --- ---`**:
- ❌ 配置初始化失败
- 检查终端是否有错误
- 查看 `~/.rdcs/config.json` 是否生成

#### 4.2 生成邀请码

1. 点击「生成邀请码」按钮
2. 观察终端输出

**成功的终端输出**:
```
✅ Generated invite code: 3847
```

**成功的 UI 表现**:
- 弹出对话框，标题「邀请码」
- 显示 4 位随机数字（如 `3847`）
- 有「复制」和「关闭」按钮

#### 4.3 测试复制功能

1. 点击「复制」按钮
2. 对话框自动关闭
3. SnackBar 显示「邀请码已复制到剪贴板」
4. 打开任意文本编辑器，粘贴验证

#### 4.4 验证随机性

连续点击「生成邀请码」5 次，应该看到不同的数字。

---

## 🔧 故障排查

### 问题 1: Docker 服务未运行

**症状**:
```
❌ 后端服务未部署
```

**解决方案**:
```bash
# 启动 Docker Desktop（通过应用程序启动）

# 部署后端服务
./deploy_backend.sh

# 验证
docker ps
# 应该看到 rdcs-api, rdcs-signaling, rdcs-redis 等容器
```

---

### 问题 2: FFI 库加载失败

**症状**:
```
Error: Unable to load dynamic library 'librdcs_core.dylib'
```

**解决方案**:
```bash
# 检查 FFI 库
ls -lh target/debug/librdcs_core.dylib

# 如果不存在，重新编译
cargo build -p rdcs-ffi

# 验证库名称
grep 'name = "rdcs_core"' crates/rdcs-ffi/Cargo.toml
```

---

### 问题 3: 设备代码不显示

**症状**: 主页显示 `--- --- ---`

**解决方案**:
```bash
# 1. 检查配置文件
cat ~/.rdcs/config.json

# 2. 如果文件不存在或 deviceCode 为空，删除并重启
rm -rf ~/.rdcs/
# 重新启动 Flutter 应用

# 3. 检查终端是否有 "Configuration initialized" 日志
```

---

### 问题 4: 邀请码生成失败

**症状**:
- SnackBar 显示「生成邀请码失败」
- 或终端显示 `❌ rdcs_generate_invite: null handle`

**解决方案**:
```bash
# 检查 Rust 代码实现
grep -A 10 "pub extern \"C\" fn rdcs_generate_invite" crates/rdcs-ffi/src/lib.rs

# 应该看到:
# use rand::Rng;
# let code = format!("{:04}", rng.gen_range(0..10000));

# 如果是旧代码（返回 "0000"），重新编译
cargo build -p rdcs-ffi
```

---

## 📊 验收清单

### 后端服务

- [ ] Docker Desktop 已启动
- [ ] `docker ps` 显示至少 3 个 rdcs 容器
- [ ] `curl http://localhost:8080/healthz` 返回成功
- [ ] 端口 8080、8443、6379 被监听

### Flutter 客户端

- [ ] 应用正常启动，无崩溃
- [ ] 终端显示 "Configuration initialized" 日志
- [ ] 主页显示 9 位设备代码（格式 XXX-XXX-XXX）
- [ ] 可以点击设备代码复制

### 邀请码功能

- [ ] 点击「生成邀请码」按钮无错误
- [ ] 弹出对话框显示 4 位数字
- [ ] 终端显示 `✅ Generated invite code: XXXX`
- [ ] 可以点击「复制」按钮
- [ ] 剪贴板包含正确的邀请码
- [ ] 连续生成 5 次，至少 3 个不同数字

---

## 🎯 客户端配置参考

在 Flutter 客户端的「设置 → 网络」选项卡中填入：

| 配置项 | 值 | 说明 |
|--------|---|------|
| **信令服务器** | `ws://127.0.0.1:8443` | WebSocket 连接地址 |
| **中继服务器** | （留空） | Phase 3 待实现 |
| **管理 API** | `http://127.0.0.1:8080` | RESTful API 地址 |

---

## 📚 相关文档

- [INVITE_CODE_TEST_GUIDE.md](INVITE_CODE_TEST_GUIDE.md) - 详细测试指南
- [INVITE_CODE_COMPLETION.md](INVITE_CODE_COMPLETION.md) - 完成总结
- [docs/BACKEND_DEPLOYMENT.md](docs/BACKEND_DEPLOYMENT.md) - 后端部署指南

---

## 🔗 有用的脚本

所有脚本都在项目根目录：

```bash
# Docker 服务检查（推荐使用）
./check_docker_services.sh

# 详细服务检查（非 Docker 环境）
./check_services_detailed.sh

# 邀请码功能测试
./test_invite_code.sh

# 启动所有服务（非 Docker）
./start_services.sh

# 停止所有服务（非 Docker）
./stop_services.sh
```

---

## 💡 重要提示

### 1. 当前实现状态

✅ **已实现**:
- 客户端生成随机 4 位邀请码
- UI 完整（弹窗、复制、错误提示）
- 配置初始化自动化

⚠️ **当前限制**:
- 邀请码仅在客户端生成，未注册到服务器
- 无过期时间
- 无冲突检测
- 无持久化

🚧 **未来计划（Phase 3）**:
- 服务器端邀请码验证
- Redis 存储，TTL 5 分钟
- 邀请码格式优化（6 位字母数字）

### 2. 项目架构

```
客户端生成邀请码 (当前实现)
    ↓
Flutter UI 点击按钮
    ↓
FFI 调用 Rust: rdcs_generate_invite()
    ↓
返回 4 位随机数字 (0000-9999)
    ↓
显示在弹窗中

服务器端验证 (Phase 3 计划)
    ↓
POST /api/invite/generate
    ↓
存储到 Redis (key: invite:1234, TTL: 300s)
    ↓
对方输入邀请码时验证
```

---

## ✅ 下一步操作

**立即执行**:

1. 在终端运行：
   ```bash
   cd /Users/lc/Development/source/remote-desktop-controller
   ./check_docker_services.sh
   ```

2. 如果服务未运行：
   ```bash
   ./deploy_backend.sh
   ```

3. 启动 Flutter 客户端：
   ```bash
   cd client/flutter
   flutter run -d macos
   ```

4. 测试邀请码生成功能

5. 完成验收清单

---

**问题反馈**: 如遇到问题，请查看对应章节的故障排查指南

**完成标志**: 所有验收清单项打勾 ✅
