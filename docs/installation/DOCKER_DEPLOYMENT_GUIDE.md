# Docker 部署和调试方案

**日期**: 2026-06-30  
**目标**: 在已安装 Docker Desktop 的 macOS 上部署 RDCS 服务端  
**适用场景**: 本地开发、集成测试、生产部署

---

## 📋 目录

1. [快速开始](#快速开始)
2. [部署架构](#部署架构)
3. [环境准备](#环境准备)
4. [部署模式](#部署模式)
5. [服务配置](#服务配置)
6. [调试方案](#调试方案)
7. [故障排查](#故障排查)
8. [监控和日志](#监控和日志)

---

## 🚀 快速开始

### 最小化部署（仅数据库）

**场景**: 在主机上运行服务，Docker 只提供数据库

```bash
cd /Users/lc/Development/source/remote-desktop-controller/deploy/docker

# 1. 复制环境变量配置
cp .env.example .env

# 2. 启动数据库服务
docker compose -f docker-compose.minimal.yml up -d

# 3. 验证服务状态
docker compose -f docker-compose.minimal.yml ps

# 4. 查看日志
docker compose -f docker-compose.minimal.yml logs -f
```

**运行结果**:
```
✓ PostgreSQL 运行在 localhost:5432
✓ Redis 运行在 localhost:6379
✓ 数据库自动初始化（迁移 + 种子数据）
```

### 完整开发环境

**场景**: 所有服务在 Docker 中运行，支持热重载

```bash
cd /Users/lc/Development/source/remote-desktop-controller/deploy/docker

# 1. 配置环境变量
cp .env.example .env
# 根据需要编辑 .env

# 2. 启动所有服务
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d

# 3. 查看服务状态
docker compose ps

# 4. 查看实时日志
docker compose logs -f
```

**服务端口**:
```
API:       http://localhost:8080
Signaling: ws://localhost:8443
Relay:     udp://localhost:3478
Web:       http://localhost:3000
```

---

## 🏗️ 部署架构

### 服务依赖关系

```
┌─────────────────────────────────────┐
│         Docker Network              │
│           (rdcs-net)                │
│                                     │
│  ┌──────────┐      ┌────────────┐  │
│  │PostgreSQL│◄─────│ API Service│  │
│  │  :5432   │      │   :8080    │  │
│  └──────────┘      └─────┬──────┘  │
│                           │         │
│  ┌──────────┐            │         │
│  │  Redis   │◄───────────┴─────┐   │
│  │  :6379   │◄─────────────┐   │   │
│  └──────────┘              │   │   │
│                            │   │   │
│  ┌──────────┐              │   │   │
│  │Signaling │──────────────┘   │   │
│  │  :8443   │                  │   │
│  └────┬─────┘                  │   │
│       │                        │   │
│       │ 分配端口               │   │
│       ↓                        │   │
│  ┌──────────┐                  │   │
│  │  Relay   │──────────────────┘   │
│  │  :3478+  │                      │
│  └──────────┘                      │
│                                     │
│  ┌──────────┐                      │
│  │   Web    │                      │
│  │  :3000   │                      │
│  └──────────┘                      │
└─────────────────────────────────────┘
        ↑
        │ 端口映射到主机
        ↓
   Docker Desktop
```

### 容器列表

| 容器名 | 镜像 | 端口 | 作用 |
|-------|------|------|------|
| rdcs-postgres | postgres:16-alpine | 5432 | 主数据库 |
| rdcs-redis | redis:7-alpine | 6379 | 缓存/状态 |
| rdcs-api | golang:1.25-alpine | 8080 | REST API |
| rdcs-signaling | rust:1.83-alpine | 8443 | WebRTC 信令 |
| rdcs-relay | rust:1.83-alpine | 3478+ | UDP 中继 |
| rdcs-web | node:22-alpine | 3000 | 管理后台 |

---

## 🔧 环境准备

### 1. 系统要求

```bash
# 检查 Docker 版本
docker --version  # 需要 >= 24.0
docker compose version  # 需要 >= 2.20

# 检查 Docker Desktop 运行状态
docker info
```

**最低配置**:
- CPU: 4 核心
- 内存: 8 GB
- 磁盘: 20 GB 可用空间

**推荐配置**:
- CPU: 8 核心
- 内存: 16 GB
- 磁盘: 50 GB SSD

### 2. 生成 JWT 密钥对

```bash
# 生成 RS256 密钥对
cd deploy/docker

# 生成私钥
openssl genpkey -algorithm RSA -out jwt_private.pem -pkeyopt rsa_keygen_bits:2048

# 提取公钥
openssl rsa -pubout -in jwt_private.pem -out jwt_public.pem

# 转换为环境变量格式（单行，\n 转义）
echo "JWT_PRIVATE_KEY=\"$(awk 'NF {sub(/\r/, ""); printf "%s\\n",$0;}' jwt_private.pem)\""
echo "JWT_PUBLIC_KEY=\"$(awk 'NF {sub(/\r/, ""); printf "%s\\n",$0;}' jwt_public.pem)\""

# 将输出复制到 .env 文件
```

### 3. 配置环境变量

编辑 `deploy/docker/.env`:

```bash
# ===========================================
# 生产环境配置示例
# ===========================================

# 服务端口
SIGNALING_PORT=8443
RELAY_STUN_PORT=3478
API_PORT=8080
WEB_PORT=3000
POSTGRES_PORT=5432
REDIS_PORT=6379

# 数据库配置（生产环境请修改密码）
POSTGRES_PASSWORD=<强密码>
DATABASE_URL=postgres://rdcs:<强密码>@postgres:5432/rdcs?sslmode=disable

# Redis
REDIS_URL=redis://redis:6379

# JWT 密钥（使用上面生成的）
JWT_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----\n...\n-----END PRIVATE KEY-----\n"
JWT_PUBLIC_KEY="-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----\n"

# CORS（根据前端域名配置）
CORS_ORIGINS=http://localhost:3000,http://localhost:8080

# TOTP
TOTP_ISSUER=RDCS
SKIP_TOTP=false  # 生产环境建议启用 2FA

# 日志级别
RUST_LOG=info
LOG_LEVEL=info

# Relay HMAC 密钥（生产环境请修改）
RELAY_HMAC_SECRET=<随机字符串>
```

**生成随机密钥**:
```bash
# RELAY_HMAC_SECRET
openssl rand -base64 32

# POSTGRES_PASSWORD
openssl rand -base64 24
```

---

## 📦 部署模式

### 模式 1: 仅数据库（推荐用于开发）

**用途**: 在主机上调试 Rust/Go 代码，Docker 只提供数据库

**配置文件**: `docker-compose.minimal.yml`

```bash
# 启动
docker compose -f docker-compose.minimal.yml up -d

# 停止
docker compose -f docker-compose.minimal.yml down

# 清理数据（危险操作）
docker compose -f docker-compose.minimal.yml down -v
```

**优势**:
- ✅ 启动速度快
- ✅ 资源占用少
- ✅ IDE 调试友好
- ✅ 代码热重载

**在主机运行服务**:
```bash
# Terminal 1: API 服务
cd services/api
export $(cat ../../deploy/docker/.env | xargs)
go run ./cmd/api

# Terminal 2: Signaling 服务
export $(cat deploy/docker/.env | xargs)
cargo run --bin rdcs-signaling

# Terminal 3: Relay 服务
export $(cat deploy/docker/.env | xargs)
cargo run --bin rdcs-relay -- --hmac-secret "$RELAY_HMAC_SECRET"
```

### 模式 2: 完整开发环境

**用途**: 所有服务在 Docker 中，支持热重载

**配置文件**: `docker-compose.yml` + `docker-compose.dev.yml`

```bash
# 启动
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d

# 查看日志
docker compose logs -f api signaling relay

# 重启单个服务
docker compose restart api

# 停止
docker compose down
```

**热重载机制**:
- **API (Go)**: Air 监听文件变化
- **Signaling/Relay (Rust)**: cargo-watch 自动重新编译
- **Web (React)**: Vite HMR

**源代码挂载**:
```yaml
volumes:
  - ../../services/api:/app           # API 源码
  - ../../crates/rdcs-signaling:/app/crates/rdcs-signaling  # Signaling 源码
  - ../../crates/rdcs-relay:/app/crates/rdcs-relay          # Relay 源码
```

### 模式 3: 生产部署

**用途**: 优化的生产构建，无热重载

**配置文件**: `docker-compose.prod.yml`（需要创建）

```bash
# 构建镜像
docker compose -f docker-compose.prod.yml build

# 启动
docker compose -f docker-compose.prod.yml up -d

# 查看状态
docker compose -f docker-compose.prod.yml ps
```

**生产优化**:
- 多阶段构建（减小镜像体积）
- 健康检查配置
- 资源限制
- 重启策略
- TLS 证书配置

---

## ⚙️ 服务配置

### PostgreSQL

**持久化**:
```yaml
volumes:
  pgdata:
    driver: local
```

**健康检查**:
```yaml
healthcheck:
  test: ["CMD-SHELL", "pg_isready -U rdcs -d rdcs"]
  interval: 10s
  timeout: 5s
  retries: 5
```

**数据迁移**:
```yaml
volumes:
  - ../../migrations/postgres:/docker-entrypoint-initdb.d
```

**备份**:
```bash
# 导出数据库
docker exec rdcs-postgres pg_dump -U rdcs rdcs > backup.sql

# 恢复数据库
docker exec -i rdcs-postgres psql -U rdcs rdcs < backup.sql
```

### Redis

**持久化策略**:
```yaml
command: >
  redis-server
  --notify-keyspace-events Ex      # 键过期通知
  --maxmemory 256mb                # 内存限制
  --maxmemory-policy allkeys-lru   # 淘汰策略
  --appendonly yes                 # AOF 持久化
```

**监控**:
```bash
# 查看 Redis 状态
docker exec rdcs-redis redis-cli INFO

# 查看键空间
docker exec rdcs-redis redis-cli DBSIZE

# 监控命令
docker exec rdcs-redis redis-cli MONITOR
```

### API 服务

**环境变量**:
```yaml
environment:
  API_PORT: 8080
  DATABASE_URL: postgres://rdcs:rdcs_dev@postgres:5432/rdcs?sslmode=disable
  REDIS_URL: redis://redis:6379
  JWT_PRIVATE_KEY: ${JWT_PRIVATE_KEY}
  JWT_PUBLIC_KEY: ${JWT_PUBLIC_KEY}
  CORS_ORIGINS: ${CORS_ORIGINS}
  RATE_LIMIT_RPS: 100
  TOTP_ISSUER: RDCS
```

**健康检查**:
```bash
curl http://localhost:8080/healthz
# 预期: "ok"
```

### Signaling 服务

**环境变量**:
```yaml
environment:
  RUST_LOG: info
  SIGNALING_PORT: 8443
  REDIS_URL: redis://redis:6379
  RELAY_NODES: |
    [
      {
        "addr": "relay",
        "port": 3478,
        "region": "local",
        "max_sessions": 100
      }
    ]
  RELAY_HMAC_SECRET: ${RELAY_HMAC_SECRET}
```

**WebSocket 测试**:
```bash
# 使用 wscat 测试
npm install -g wscat
wscat -c ws://localhost:8443/ws?code=ABC123456
```

### Relay 服务

**端口范围**:
```yaml
ports:
  - "3478:3478/udp"           # 控制端口
  - "49152-49200:49152-49200/udp"  # 媒体端口（示例范围）
```

**健康检查**:
```bash
curl http://localhost:9091/health
# 预期: {"status":"healthy","active_sessions":0,"capacity":8192}
```

**指标**:
```bash
curl http://localhost:9090/metrics
# Prometheus 格式指标
```

---

## 🐛 调试方案

### 1. 日志调试

#### 查看所有服务日志
```bash
docker compose logs -f
```

#### 查看特定服务日志
```bash
# API 服务
docker compose logs -f api

# Signaling 服务（详细日志）
docker compose logs -f signaling

# 最近 100 行日志
docker compose logs --tail=100 relay
```

#### 调整日志级别
编辑 `.env`:
```bash
# 调试级别
RUST_LOG=debug,rdcs_signaling=trace
LOG_LEVEL=debug

# 重启服务
docker compose restart signaling
```

### 2. 容器内调试

#### 进入容器
```bash
# 进入 API 容器
docker exec -it rdcs-api sh

# 进入 PostgreSQL 容器
docker exec -it rdcs-postgres psql -U rdcs -d rdcs
```

#### 检查网络连通性
```bash
# 从 API 容器测试 PostgreSQL 连接
docker exec rdcs-api ping postgres

# 测试 Redis 连接
docker exec rdcs-api sh -c 'apk add redis && redis-cli -h redis ping'
```

#### 检查进程状态
```bash
# 查看容器内进程
docker exec rdcs-signaling ps aux

# 查看端口监听
docker exec rdcs-api netstat -tlnp
```

### 3. 网络调试

#### 检查 Docker 网络
```bash
# 列出网络
docker network ls

# 查看 rdcs-net 详情
docker network inspect rdcs-net

# 查看容器 IP
docker inspect rdcs-api | grep IPAddress
```

#### 端口映射验证
```bash
# 检查主机端口监听
netstat -an | grep LISTEN | grep -E '8080|8443|3478|5432|6379'

# 测试端口连通性
nc -zv localhost 8080  # API
nc -zv localhost 8443  # Signaling
nc -zuv localhost 3478 # Relay (UDP)
```

### 4. 数据库调试

#### 连接数据库
```bash
# 使用 psql
docker exec -it rdcs-postgres psql -U rdcs -d rdcs

# 或从主机连接
psql -h localhost -U rdcs -d rdcs
```

#### 常用 SQL 查询
```sql
-- 查看所有表
\dt

-- 查看表结构
\d members

-- 查询团队
SELECT * FROM teams;

-- 查询成员（带团队信息）
SELECT m.*, t.name as team_name 
FROM members m 
JOIN teams t ON m.team_id = t.id;

-- 查询在线设备
SELECT * FROM devices WHERE status = 'online';

-- 查询最近的连接记录
SELECT * FROM connection_records 
ORDER BY started_at DESC 
LIMIT 10;
```

#### Redis 调试
```bash
# 连接 Redis
docker exec -it rdcs-redis redis-cli

# 查看所有键
KEYS *

# 查看会话数据
GET session:ABC123456

# 查看设备在线状态
TTL device:ABC123456

# 监控实时命令
MONITOR
```

### 5. 性能调试

#### 资源使用监控
```bash
# 实时监控所有容器
docker stats

# 监控特定容器
docker stats rdcs-api rdcs-signaling rdcs-relay
```

#### 慢查询分析
```sql
-- PostgreSQL 慢查询日志
-- 在 postgresql.conf 中启用
log_min_duration_statement = 100  -- 100ms 以上的查询

-- 查看当前活跃查询
SELECT pid, now() - query_start AS duration, query 
FROM pg_stat_activity 
WHERE state = 'active' 
ORDER BY duration DESC;
```

### 6. 集成测试

#### API 端点测试
```bash
# 健康检查
curl http://localhost:8080/healthz

# 登录测试
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@example.com",
    "password": "admin123"
  }'

# 使用 JWT 访问保护资源
TOKEN="<从登录响应获取>"
curl http://localhost:8080/api/v1/teams/TEAM_ID/devices \
  -H "Authorization: Bearer $TOKEN"
```

#### WebSocket 测试
```javascript
// 使用浏览器控制台
const ws = new WebSocket('ws://localhost:8443/ws?code=ABC123456');
ws.onopen = () => console.log('Connected');
ws.onmessage = (e) => console.log('Message:', e.data);
ws.onerror = (e) => console.error('Error:', e);

// 发送测试消息
ws.send(JSON.stringify({
  type: 'offer',
  target: 'DEF789012',
  sdp: '...'
}));
```

---

## 🚨 故障排查

### 问题 1: 容器无法启动

**症状**:
```bash
docker compose up -d
# 某个服务一直重启
```

**排查步骤**:
```bash
# 1. 查看容器状态
docker compose ps

# 2. 查看启动日志
docker compose logs <service-name>

# 3. 检查配置文件语法
docker compose config

# 4. 验证环境变量
docker compose config | grep -A 5 environment
```

**常见原因**:
- ❌ 环境变量未设置或格式错误
- ❌ 端口已被占用
- ❌ 依赖服务未就绪
- ❌ 配置文件路径错误

**解决方案**:
```bash
# 检查端口占用
lsof -i :8080

# 清理并重新创建
docker compose down
docker compose up -d --force-recreate
```

### 问题 2: 数据库连接失败

**症状**:
```
API Error: failed to connect to database: dial tcp 172.18.0.2:5432: connect: connection refused
```

**排查步骤**:
```bash
# 1. 检查 PostgreSQL 是否运行
docker compose ps postgres

# 2. 检查健康状态
docker inspect rdcs-postgres | grep -A 10 Health

# 3. 测试数据库连接
docker exec rdcs-postgres pg_isready -U rdcs

# 4. 查看 PostgreSQL 日志
docker compose logs postgres
```

**常见原因**:
- ❌ PostgreSQL 未完成初始化
- ❌ 数据库凭据错误
- ❌ 网络配置问题

**解决方案**:
```bash
# 等待数据库就绪
docker compose up -d postgres
sleep 10
docker compose up -d api

# 或使用 depends_on + healthcheck
```

### 问题 3: Redis 连接超时

**症状**:
```
Signaling Error: RedisError: I/O error: Connection timed out (os error 60)
```

**排查步骤**:
```bash
# 1. 测试 Redis 连接
docker exec rdcs-redis redis-cli ping

# 2. 从 Signaling 容器测试
docker exec rdcs-signaling sh -c 'apk add redis && redis-cli -h redis ping'

# 3. 检查 Redis URL 配置
docker compose config | grep REDIS_URL
```

**解决方案**:
- 确认 REDIS_URL 格式: `redis://redis:6379`
- 检查网络配置
- 重启 Redis 容器

### 问题 4: WebSocket 连接失败

**症状**:
```
WebSocket connection to 'ws://localhost:8443/ws' failed: Connection closed before receiving a handshake response
```

**排查步骤**:
```bash
# 1. 检查 Signaling 服务状态
docker compose ps signaling

# 2. 查看 Signaling 日志
docker compose logs signaling | grep -i error

# 3. 测试 WebSocket 端点
wscat -c ws://localhost:8443/ws?code=TEST12345

# 4. 检查防火墙/代理配置
```

**常见原因**:
- ❌ 设备代码参数缺失
- ❌ CORS 配置错误
- ❌ TLS 证书问题（生产环境）

### 问题 5: Relay 无法分配端口

**症状**:
```
Relay Error: Failed to bind UDP socket: Address already in use
```

**排查步骤**:
```bash
# 1. 检查端口范围配置
docker compose config | grep -A 5 relay

# 2. 查看 Relay 日志
docker compose logs relay

# 3. 检查端口占用
netstat -an | grep 49152

# 4. 查看 Relay 健康状态
curl http://localhost:9091/health
```

**解决方案**:
- 调整端口范围（RELAY_MIN_PORT/MAX_PORT）
- 释放占用的端口
- 增加容器权限

---

## 📊 监控和日志

### 日志聚合

#### 配置日志驱动
```yaml
# docker-compose.yml
x-logging: &default-logging
  logging:
    driver: json-file
    options:
      max-size: "10m"
      max-file: "3"
      labels: "service,environment"
```

#### 集中日志查看
```bash
# 所有服务日志（带时间戳）
docker compose logs -f --timestamps

# 过滤错误日志
docker compose logs | grep -i error

# 导出日志到文件
docker compose logs > rdcs-logs-$(date +%Y%m%d).log
```

### Prometheus 指标

#### Relay 指标端点
```bash
curl http://localhost:9090/metrics
```

**关键指标**:
```
# 活跃会话数
relay_active_sessions 12

# 转发字节数
relay_bytes_forwarded_total{direction="tx"} 1234567
relay_bytes_forwarded_total{direction="rx"} 7654321

# 错误计数
relay_errors_total{type="auth_failed"} 5
```

#### Grafana 仪表板（可选）

创建 `docker-compose.monitoring.yml`:
```yaml
services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9091:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    networks:
      - rdcs-net

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3001:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    networks:
      - rdcs-net
```

### 健康检查

#### API 健康检查
```bash
#!/bin/bash
# health-check-api.sh

ENDPOINT="http://localhost:8080/healthz"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $ENDPOINT)

if [ $RESPONSE -eq 200 ]; then
  echo "✓ API 健康"
  exit 0
else
  echo "✗ API 异常 (HTTP $RESPONSE)"
  exit 1
fi
```

#### Relay 健康检查
```bash
#!/bin/bash
# health-check-relay.sh

ENDPOINT="http://localhost:9091/health"
RESPONSE=$(curl -s $ENDPOINT | jq -r '.status')

if [ "$RESPONSE" == "healthy" ]; then
  echo "✓ Relay 健康"
  exit 0
else
  echo "✗ Relay 异常"
  exit 1
fi
```

#### 自动健康检查脚本
```bash
#!/bin/bash
# check-all-services.sh

services=("api:8080/healthz" "relay:9091/health")

for service in "${services[@]}"; do
  IFS=':' read -r name endpoint <<< "$service"
  
  if curl -sf "http://localhost:$endpoint" > /dev/null; then
    echo "✓ $name"
  else
    echo "✗ $name"
  fi
done
```

---

## 📝 快速命令参考

### 启动/停止
```bash
# 启动所有服务
docker compose up -d

# 启动特定服务
docker compose up -d api signaling

# 停止所有服务
docker compose down

# 停止并删除数据卷（危险）
docker compose down -v
```

### 查看状态
```bash
# 服务状态
docker compose ps

# 实时日志
docker compose logs -f

# 资源使用
docker stats
```

### 重启服务
```bash
# 重启所有服务
docker compose restart

# 重启特定服务
docker compose restart api

# 重新构建并启动
docker compose up -d --build
```

### 清理
```bash
# 停止并删除容器
docker compose down

# 删除所有 RDCS 相关镜像
docker images | grep rdcs | awk '{print $3}' | xargs docker rmi

# 清理未使用的镜像
docker image prune -a

# 清理所有未使用的资源
docker system prune -a --volumes
```

---

## 🎯 生产部署检查清单

### 部署前
- [ ] 生成强密码和密钥
- [ ] 配置 TLS 证书
- [ ] 调整资源限制
- [ ] 启用健康检查
- [ ] 配置备份策略
- [ ] 设置监控告警

### 部署中
- [ ] 执行数据库迁移
- [ ] 验证环境变量
- [ ] 检查网络配置
- [ ] 测试服务连通性
- [ ] 验证日志输出

### 部署后
- [ ] 负载测试
- [ ] 安全扫描
- [ ] 性能基准测试
- [ ] 故障恢复演练
- [ ] 文档更新

---

## 📞 支持

### 日志文件位置
```
容器内: /var/log/
主机: docker inspect <container> | grep LogPath
```

### 常见问题
参见: [FAQ.md](./FAQ.md)

### 报告问题
创建 Issue 时请包含：
- Docker 版本
- docker-compose.yml 内容
- 相关日志（docker compose logs）
- 环境变量配置（脱敏）
- 错误截图

---

**维护人**: RDCS Team  
**最后更新**: 2026-06-30  
**版本**: 1.0.0
