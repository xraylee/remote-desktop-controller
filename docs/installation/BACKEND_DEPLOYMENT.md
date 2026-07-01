# RDCS 后端服务部署指南

**日期**: 2026-06-29  
**状态**: 生产就绪

---

## 📋 系统要求

### 必需软件
- **Docker Desktop** ≥ 20.10
  - macOS: [下载](https://www.docker.com/products/docker-desktop)
  - 需要至少 4 GB RAM 分配给 Docker
- **Docker Compose** ≥ 2.0（包含在 Docker Desktop 中）

### 系统资源
- **最小配置**:
  - CPU: 2 核心
  - 内存: 4 GB
  - 磁盘: 5 GB 可用空间

- **推荐配置**:
  - CPU: 4 核心
  - 内存: 8 GB
  - 磁盘: 20 GB 可用空间

---

## 🚀 快速开始

### 步骤 1: 启动 Docker Desktop

确保 Docker Desktop 已启动并运行：

```bash
# 验证 Docker 状态
docker info
```

### 步骤 2: 部署后端服务

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 一键部署所有服务
./deploy_backend.sh
```

**预期输出**:
```
=== RDCS 后端服务部署 ===

[1/6] 检查 Docker...
✅ Docker 已就绪

[2/6] 配置环境变量...
✅ .env 已创建（使用默认配置）

[3/6] 清理旧容器...
✅ 清理完成

[4/6] 构建 Docker 镜像（可能需要几分钟）...
✅ 镜像构建成功

[5/6] 启动所有服务...
✅ 服务已启动

[6/6] 等待服务就绪...
✅ PostgreSQL 已就绪
✅ API 服务已就绪

╔════════════════════════════════════════════════════╗
║         ✅ RDCS 后端服务部署成功！                 ║
╚════════════════════════════════════════════════════╝
```

### 步骤 3: 验证服务

```bash
# 检查服务状态
docker ps

# 测试 API 健康检查
curl http://localhost:8080/healthz
# 预期输出: ok
```

---

## 🌐 服务地址

| 服务 | 地址 | 说明 |
|------|------|------|
| **API 服务** | http://localhost:8080 | RESTful API + WebSocket |
| **健康检查** | http://localhost:8080/healthz | 服务状态检查 |
| **Web 管理台** | http://localhost:3000 | 管理控制台 |
| **MinIO 控制台** | http://localhost:9001 | 对象存储管理 |
| **PostgreSQL** | localhost:5432 | 数据库 |
| **Redis** | localhost:6379 | 缓存服务 |

---

## 📦 服务架构

```
┌─────────────────────────────────────────────────┐
│              Docker Network (rdcs-net)          │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌──────────────┐      ┌──────────────┐        │
│  │   Signaling  │      │    Relay     │        │
│  │   (Rust)     │      │   (Rust)     │        │
│  │   :8443      │      │   :3478      │        │
│  └──────┬───────┘      └──────┬───────┘        │
│         │                     │                 │
│  ┌──────▼──────────────────────▼───────┐       │
│  │         API Server (Go)             │       │
│  │              :8080                   │       │
│  └──────┬─────────────┬─────────────┬──┘       │
│         │             │             │           │
│  ┌──────▼──────┐ ┌───▼─────┐ ┌────▼────┐      │
│  │  PostgreSQL │ │  Redis  │ │  MinIO  │      │
│  │    :5432    │ │  :6379  │ │  :9000  │      │
│  └─────────────┘ └─────────┘ └─────────┘      │
│                                                 │
└─────────────────────────────────────────────────┘
```

### 服务说明

1. **Signaling Server (信令服务)**
   - WebSocket 信令交换
   - 设备注册和管理
   - Offer/Answer/ICE 候选中继

2. **Relay Server (中继服务)**
   - STUN/TURN 服务器
   - UDP 媒体中继
   - NAT 穿透支持

3. **API Server (API 服务)**
   - RESTful API
   - WebSocket Hub
   - 用户认证
   - 会话管理

4. **PostgreSQL (数据库)**
   - 用户数据
   - 设备信息
   - 会话记录
   - 审计日志

5. **Redis (缓存)**
   - 会话状态
   - 实时数据
   - 发布/订阅

6. **MinIO (对象存储)**
   - 会话录制
   - 文件传输
   - 备份存储

---

## 📝 常用命令

### 查看日志

```bash
# 查看所有服务日志
./logs_backend.sh

# 查看特定服务日志
./logs_backend.sh api          # API 服务
./logs_backend.sh signaling    # 信令服务
./logs_backend.sh postgres     # 数据库
```

### 管理服务

```bash
# 停止服务（保留数据）
./stop_backend.sh

# 重启服务
./deploy_backend.sh

# 完全清理（删除所有数据）
cd deploy/docker
docker compose -f docker-compose.yml -f docker-compose.dev.yml down -v
```

### 手动操作

```bash
cd deploy/docker

# 启动服务
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d

# 停止服务
docker compose -f docker-compose.yml -f docker-compose.dev.yml down

# 查看状态
docker compose -f docker-compose.yml -f docker-compose.dev.yml ps

# 重启单个服务
docker compose -f docker-compose.yml -f docker-compose.dev.yml restart api
```

---

## 🔧 配置说明

### 环境变量

配置文件位于 `deploy/docker/.env`，主要配置项：

```bash
# 端口配置
API_PORT=8080
SIGNALING_PORT=8443
POSTGRES_PORT=5432

# 数据库
POSTGRES_PASSWORD=rdcs_dev

# 日志级别
RUST_LOG=info
LOG_LEVEL=info
```

### 修改配置

1. 编辑 `.env` 文件
2. 重启服务: `./deploy_backend.sh`

---

## 🔍 故障排查

### 问题 1: Docker 未启动

**症状**: `Cannot connect to the Docker daemon`

**解决**:
```bash
# 启动 Docker Desktop
open -a Docker

# 等待几秒后验证
docker info
```

---

### 问题 2: 端口被占用

**症状**: `port is already allocated`

**解决**:
```bash
# 查看端口占用
lsof -i :8080
lsof -i :5432

# 停止占用的进程或修改 .env 中的端口
```

---

### 问题 3: API 服务无法启动

**症状**: API 健康检查失败

**解决**:
```bash
# 查看详细日志
docker logs rdcs-api

# 常见原因：
# 1. 数据库未就绪 → 等待 30 秒重试
# 2. 配置错误 → 检查 .env 文件
# 3. 构建失败 → 重新构建: docker compose build api
```

---

### 问题 4: 数据库连接失败

**症状**: `connection refused` 或 `authentication failed`

**解决**:
```bash
# 检查 PostgreSQL 状态
docker exec rdcs-postgres pg_isready -U rdcs -d rdcs

# 查看数据库日志
docker logs rdcs-postgres

# 重置数据库
docker compose down
docker volume rm docker_pgdata
./deploy_backend.sh
```

---

## 📊 数据库管理

### 连接数据库

```bash
# 使用 psql
docker exec -it rdcs-postgres psql -U rdcs -d rdcs

# 使用 GUI 工具（DBeaver, TablePlus 等）
Host: localhost
Port: 5432
Database: rdcs
User: rdcs
Password: rdcs_dev
```

### 查看表结构

```sql
-- 列出所有表
\dt

-- 查看表结构
\d devices
\d connections
\d audit_logs
```

### 备份和恢复

```bash
# 备份数据库
docker exec rdcs-postgres pg_dump -U rdcs rdcs > backup_$(date +%Y%m%d).sql

# 恢复数据库
docker exec -i rdcs-postgres psql -U rdcs rdcs < backup_20260629.sql
```

---

## 🔐 安全建议

### 开发环境
✅ 当前配置适合开发和测试

### 生产环境
需要修改以下配置：

1. **修改默认密码**
   ```bash
   # 生成强密码
   openssl rand -base64 32
   
   # 更新 .env
   POSTGRES_PASSWORD=<强密码>
   MINIO_ROOT_PASSWORD=<强密码>
   JWT_SECRET=<强密钥>
   ```

2. **启用 HTTPS**
   - 配置 TLS 证书
   - 修改 `docker-compose.prod.yml`

3. **限制网络访问**
   - 使用防火墙
   - 限制数据库端口仅内部访问

4. **启用认证**
   ```bash
   SKIP_TOTP=false
   ```

---

## 📈 监控

### 健康检查

```bash
# API 服务
curl http://localhost:8080/healthz

# PostgreSQL
docker exec rdcs-postgres pg_isready -U rdcs -d rdcs

# Redis
docker exec rdcs-redis redis-cli ping
```

### 资源使用

```bash
# 查看容器资源
docker stats

# 查看磁盘使用
docker system df
```

---

## 🎯 下一步

1. ✅ 后端服务已部署
2. ⏳ 配置 Flutter 客户端连接到服务器
3. ⏳ 在两台 Mac 上测试远程连接

详见 [客户端配置指南](CLIENT_CONFIGURATION.md)

---

**维护者**: RDCS Team  
**最后更新**: 2026-06-29  
**状态**: 生产就绪 ✅
