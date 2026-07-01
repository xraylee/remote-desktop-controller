# 🚀 RDCS 邀请码功能测试 - 快速启动指南

**更新时间**: 2026-06-30 18:35  
**状态**: 准备就绪

---

## ⚡ 三步启动

### Step 1: 启动后端服务

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 使用交互式启动脚本（推荐）
./start_backend.sh

# 或者直接运行部署脚本
./scripts/deployment/deploy_backend.sh
```

**预期输出**:
```
✅ RDCS 后端服务部署成功！

🌐 服务地址:
  • API 服务:       http://localhost:8080
  • 健康检查:       http://localhost:8080/healthz
  • Redis:          localhost:6379
```

---

### Step 2: 启动 Flutter 客户端

```bash
cd client/flutter
flutter run -d macos
```

**预期终端输出**:
```
✅ Configuration initialized. Device code: 123-456-789
```

**预期 UI 显示**:
- 主页显示 9 位设备代码（格式: `XXX-XXX-XXX`）
- 状态显示"设备已就绪"（绿色指示灯）

---

### Step 3: 测试邀请码生成

1. **点击「生成邀请码」按钮**

2. **观察终端输出**:
   ```
   ✅ Generated invite code: 3847
   ```

3. **验证 UI 弹窗**:
   - 标题: "邀请码"
   - 显示: 4 位数字（大字体）
   - 按钮: "复制" 和 "关闭"

4. **测试复制功能**:
   - 点击「复制」按钮
   - SnackBar 显示: "邀请码已复制到剪贴板"
   - 打开文本编辑器，粘贴验证

5. **验证随机性**:
   - 连续生成 5 次
   - 应该看到不同的数字

---

## ✅ 验收清单

### 后端服务 ✓
- [ ] Docker Desktop 已启动
- [ ] 部署脚本执行成功
- [ ] `docker ps` 显示 rdcs-api、rdcs-redis 等容器
- [ ] `curl http://localhost:8080/healthz` 返回成功

### Flutter 客户端 ✓
- [ ] 应用正常启动
- [ ] 终端显示 "Configuration initialized" 日志
- [ ] 主页显示 9 位设备代码（XXX-XXX-XXX）
- [ ] 设备代码可以点击复制

### 邀请码功能 ✓
- [ ] 点击「生成邀请码」按钮无错误
- [ ] 弹出对话框显示 4 位数字
- [ ] 终端显示 `✅ Generated invite code: XXXX`
- [ ] 可以点击「复制」按钮
- [ ] 剪贴板包含正确的邀请码
- [ ] 连续生成 5 次，至少 3 个不同数字

---

## 🔧 故障排查

### 问题 1: 部署脚本报错

**症状**: `PROJECT_ROOT` 路径错误

**解决方案**:
```bash
# 确保从项目根目录运行
cd /Users/lc/Development/source/remote-desktop-controller
./scripts/deployment/deploy_backend.sh
```

---

### 问题 2: Docker 容器启动失败

**症状**: 端口被占用或构建失败

**解决方案**:
```bash
# 停止所有容器
cd deploy/docker
docker compose -f docker-compose.yml -f docker-compose.dev.yml down

# 清理并重新部署
cd /Users/lc/Development/source/remote-desktop-controller
./scripts/deployment/deploy_backend.sh
```

---

### 问题 3: 设备代码不显示

**症状**: 主页显示 `--- --- ---`

**解决方案**:
```bash
# 删除配置文件
rm -rf ~/.rdcs/

# 重启 Flutter 应用
# 配置会自动重新生成
```

---

### 问题 4: 邀请码生成失败

**症状**: SnackBar 显示「生成邀请码失败」

**原因**: FFI 库可能未正确加载

**解决方案**:
```bash
# 1. 检查 FFI 库
ls -lh target/debug/librdcs_core.dylib

# 2. 如果不存在，重新编译
cargo build -p rdcs-ffi

# 3. 重启 Flutter 应用
```

---

## ⚙️ 客户端配置

在 Flutter 客户端的「设置 → 网络」中填入：

| 配置项 | 值 |
|--------|---|
| **信令服务器** | `ws://127.0.0.1:8443` |
| **管理 API** | `http://127.0.0.1:8080` |
| **中继服务器** | （留空） |

---

## 📊 当前实现状态

### ✅ 已实现
- 客户端生成随机 4 位邀请码（0000-9999）
- UI 完整（弹窗、复制、错误提示）
- 配置自动初始化
- 设备代码自动生成

### ⚠️ 当前限制
- 邀请码仅在客户端生成，未注册到服务器
- 无过期时间管理
- 无冲突检测
- 无持久化和历史记录

### 🚧 下一步（Phase 3）
- 服务器端邀请码验证
- Redis 存储，TTL 5 分钟
- 邀请码格式优化（6 位字母数字）

---

## 📚 有用的命令

```bash
# 查看所有容器
docker ps

# 查看 API 日志
docker logs -f rdcs-api

# 查看 Redis 日志
docker logs -f rdcs-redis

# 停止所有服务
cd deploy/docker
docker compose -f docker-compose.yml -f docker-compose.dev.yml down

# 重启服务
cd deploy/docker
docker compose -f docker-compose.yml -f docker-compose.dev.yml restart

# 查看配置文件
cat ~/.rdcs/config.json | jq .
```

---

## 🎯 成功标志

当你完成所有步骤并看到以下结果时，说明联调成功：

1. ✅ Docker 容器全部运行
2. ✅ Flutter 应用正常启动
3. ✅ 主页显示设备代码
4. ✅ 可以生成并复制邀请码
5. ✅ 终端输出 `✅ Generated invite code: XXXX`

---

**完成所有测试后，请勾选验收清单中的所有项目！** ✓
