# Plan F: 开发环境一键部署

**阶段**: Phase 3 — MVP Development 前置基础设施
**预估总工期**: ~12 天
**目标**: 一条命令启动完整本地开发环境（7 个服务），支持热重载、种子数据、日志聚合，同时为生产部署提供对等配置。

## 跨项目依赖

本项目无外部项目依赖（为其他项目提供基础设施）

---

## 依赖关系图

```
Task 1 (仓库结构)
  ├─► Task 2 (Docker Compose 基础)
  │     ├─► Task 3 (Rust Dockerfile)
  │     ├─► Task 4 (Go API Dockerfile)
  │     ├─► Task 5 (React Web Dockerfile)
  │     ├─► Task 6 (PostgreSQL 初始化)
  │     └─► Task 7 (Redis + MinIO 初始化)
  ├─► Task 8 (Dev Compose 覆盖层)  ← 依赖 Task 2-7
  ├─► Task 9 (Prod Compose + Caddy) ← 依赖 Task 2-7
  └─► Task 10 (Makefile + 文档)     ← 依赖 Task 8-9
```

---

### Task 1: 仓库 Monorepo 结构 (~1d)

**目标**: 建立 Monorepo 目录结构，将信令/中转（Rust）、管理 API（Go）、Web 控制台（React）、客户端引擎（Rust）和部署配置组织到统一仓库中。所有后续 Task 在此骨架上构建。

**依赖**: 无

**文件**:
- `crates/rdcs-signaling/Cargo.toml` — 信令服务 crate，依赖 axum, tokio, redis, serde
- `crates/rdcs-relay/Cargo.toml` — 中转节点 crate，依赖 tokio, libsodium
- `crates/rdcs-core/Cargo.toml` — 核心引擎共享库（连接管理、加密、编解码接口）
- `services/api/go.mod` — Go 管理 API 模块，依赖 chi, pgx, slog
- `web/admin/package.json` — React 管理控制台，依赖 vite, react, tailwind, tanstack-query, zustand
- `client/flutter/pubspec.yaml` — Flutter 客户端 UI 占位（MVP 后续填充）
- `deploy/docker/` — 所有 Dockerfile 和 compose 文件存放目录
- `migrations/postgres/` — SQL 迁移文件目录，按序号命名（001_init_schema.sql）
- `scripts/` — 开发辅助脚本（种子数据、健康检查等）

**关键配置**:
- 根目录 `Cargo.toml` 为 workspace，members 包含 `crates/*`
- `.editorconfig`: 2 空格缩进、UTF-8、LF 换行
- `.gitignore`: 忽略 target/, node_modules/, vendor/, .env, *.log
- `LICENSE`: Apache 2.0 头部模板

**验收标准**:
- [ ] `tree -L 2` 显示 crates/, services/, web/, client/, deploy/, migrations/, scripts/ 七个顶层目录
- [ ] `cargo check --workspace` 通过（空 crate 骨架）
- [ ] `go build ./...` 在 services/api/ 通过（空 main.go）
- [ ] `npm install` 在 web/admin/ 通过（Vite 脚手架）

---

### Task 2: Docker Compose 基础定义 (~1.5d)

**目标**: 编写 docker-compose.yml 基础文件，定义 7 个服务的镜像/构建上下文、网络拓扑和持久化卷。此文件为"基础层"，dev/prod 通过 override 文件叠加差异。

**依赖**: Task 1

**文件**:
- `deploy/docker/docker-compose.yml` — 基础 compose 文件，定义所有服务和共享资源

**关键配置**:
- **signaling**: build context `../../crates/rdcs-signaling`, dockerfile `../../deploy/docker/Dockerfile.signaling`, 端口 8443:8443, 依赖 redis + postgres
- **relay**: build context `../../crates/rdcs-relay`, dockerfile `../../deploy/docker/Dockerfile.relay`, 端口 3478:3478/udp + 49152-49200:49152-49200/udp, 环境变量 RELAY_SLOTS=48
- **api**: build context `../../services/api`, dockerfile `../../deploy/docker/Dockerfile.api`, 端口 8080:8080, 依赖 postgres + redis
- **web**: build context `../../web/admin`, dockerfile `../../deploy/docker/Dockerfile.web`, 端口 3000:3000, 依赖 api
- **postgres**: 镜像 postgres:16-alpine, 端口 5432:5432, 环境变量 POSTGRES_DB=rdcs / POSTGRES_USER=rdcs / POSTGRES_PASSWORD=rdcs_dev, volume pgdata
- **redis**: 镜像 redis:7-alpine, 端口 6379:6379, command 附带 --notify-keyspace-events Ex
- **minio**: 镜像 minio/minio, 端口 9000:9000 + 9001:9001, command server /data --console-address ":9001", volume minio_data
- **networks**: rdcs-net (bridge driver), 所有服务共享
- **volumes**: pgdata (postgres), minio_data (minio), redis_data (redis rdb 持久化)
- 所有服务配置 healthcheck（pg_isready / redis-cli ping / mc ready / curl 端口检查）

**验收标准**:
- [ ] `docker compose config` 验证 YAML 无语法错误
- [ ] `docker compose config --services` 输出 7 个服务名
- [ ] 网络和卷定义正确：1 个网络、3 个命名卷

---

### Task 3: Rust 服务 Dockerfile（信令 + 中转） (~1.5d)

**目标**: 为信令服务和中转节点编写多阶段 Dockerfile。builder 阶段编译 Rust 二进制，runtime 阶段使用最小基础镜像。最终镜像 < 50MB。

**依赖**: Task 2

**文件**:
- `deploy/docker/Dockerfile.signaling` — 信令服务多阶段构建
- `deploy/docker/Dockerfile.relay` — 中转节点多阶段构建

**关键配置**:
- **builder 阶段** (rust:1.80-bookworm):
  - 安装 cmake, pkg-config, libssl-dev (信令需要 openssl)
  - COPY Cargo.lock + Cargo.toml → 先执行 `cargo build --release --bin rdcs-signaling`（依赖缓存层）
  - COPY crates/ → `cargo build --release --bin rdcs-signaling`
  - relay 类似：`cargo build --release --bin rdcs-relay`
- **runtime 阶段** (debian:bookworm-slim):
  - 安装 ca-certificates, libsodium23 (relay) 或 libssl3 (signaling)
  - COPY --from=builder 编译产物到 /usr/local/bin/
  - 非 root 用户运行 (rdcs:rdcs, UID 1000)
  - ENTRYPOINT 直接执行二进制
- **构建参数**: CARGO_PROFILE=release, TARGET=x86_64-unknown-linux-gnu
- **安全**: 不暴露源码，runtime 镜像不含 cargo/gcc

**验收标准**:
- [ ] `docker build -f Dockerfile.signaling` 构建成功，镜像 < 80MB
- [ ] `docker build -f Dockerfile.relay` 构建成功，镜像 < 60MB
- [ ] 容器以非 root 用户运行 (`docker exec whoami` = rdcs)
- [ ] 依赖缓存层生效：修改源码后重新构建不需要重新下载 crate

---

### Task 4: Go API Dockerfile (~1d)

**目标**: 为管理 API 编写多阶段 Dockerfile，生产版使用静态编译，开发版集成 air 热重载工具。

**依赖**: Task 2

**文件**:
- `deploy/docker/Dockerfile.api` — Go API 生产构建
- `deploy/docker/Dockerfile.api.dev` — Go API 开发构建（air 热重载）

**关键配置**:
- **builder 阶段** (golang:1.22-alpine):
  - CGO_ENABLED=0, GOOS=linux
  - `go build -ldflags="-s -w" -o /out/rdcs-api ./cmd/api`
  - 依赖缓存：先 COPY go.mod go.sum → go mod download，再 COPY 源码
- **runtime 阶段** (alpine:3.20):
  - 安装 ca-certificates, tzdata
  - COPY --from=builder /out/rdcs-api
  - 非 root 用户运行 (rdcs:rdcs)
  - EXPOSE 8080, HEALTHCHECK curl localhost:8080/healthz
- **dev 变体** (Dockerfile.api.dev):
  - 基于 golang:1.22-alpine
  - 安装 air (`go install github.com/air-verse/air@latest`)
  - WORKDIR /app, 挂载源码卷
  - CMD air -c .air.toml
  - .air.toml 配置：监听 .go 文件变更，排除 vendor/ 和 _tmp/

**验收标准**:
- [ ] `docker build -f Dockerfile.api` 构建成功，镜像 < 30MB
- [ ] dev 变体启动后，修改 .go 文件自动重新编译（日志显示 "rebuilding..."）
- [ ] `/healthz` 端点返回 200

---

### Task 5: React Web 控制台 Dockerfile (~1d)

**目标**: 为 Web 管理控制台编写 Dockerfile，开发模式使用 Vite dev server（HMR），生产模式使用 nginx 托管静态构建产物。

**依赖**: Task 2

**文件**:
- `deploy/docker/Dockerfile.web` — 生产构建 (nginx)
- `deploy/docker/Dockerfile.web.dev` — 开发构建 (Vite dev server)
- `deploy/docker/nginx.conf` — 生产 nginx 配置

**关键配置**:
- **builder 阶段** (node:20-alpine):
  - npm ci → npm run build → 输出 dist/ 静态文件
  - 环境变量 VITE_API_BASE_URL (build-time 注入)
- **runtime 阶段** (nginx:1.27-alpine):
  - COPY dist/ → /usr/share/nginx/html/
  - COPY nginx.conf → SPA 路由 fallback (try_files → /index.html)
  - EXPOSE 3000 (nginx 监听 3000 替代默认 80)
  - gzip on, 缓存策略：index.html no-cache, assets/ 长期缓存
- **dev 变体** (Dockerfile.web.dev):
  - 基于 node:20-alpine
  - npm install → npm run dev (Vite dev server)
  - EXPOSE 3000, 环境变量 VITE_API_BASE_URL=http://localhost:8080
  - 挂载源码卷实现 HMR

**验收标准**:
- [ ] 生产镜像构建成功，`docker run` 后访问 localhost:3000 返回 HTML
- [ ] dev 变体启动后，修改 .tsx 文件浏览器自动热更新
- [ ] SPA 路由：访问 /devices 不返回 404

---

### Task 6: PostgreSQL 初始化脚本 (~1d)

**目标**: 编写数据库初始化 SQL，创建 6 张核心表和必要索引，并提供种子数据脚本用于开发测试。postgres 容器首次启动时自动执行。

**依赖**: Task 1

**文件**:
- `migrations/postgres/001_init_schema.sql` — 完整 Schema 定义
- `migrations/postgres/002_seed_data.sql` — 开发用种子数据

**关键配置**:
- **001_init_schema.sql** 包含 6 张表:
  - `teams`: id(UUID PK), name, plan(free/basic/pro), max_concurrent(INT), created_at, updated_at
  - `members`: id(UUID PK), team_id(FK→teams), name, email(UNIQUE), role(owner/manager/member), password_hash, totp_secret, totp_enabled, last_login, created_at
  - `devices`: id(UUID PK), team_id(FK→teams), device_code(CHAR(9) UNIQUE), device_name, platform, os_version, client_version, status(online/offline), last_seen, ip_address(INET), created_at
  - `connection_records`: id(UUID PK), team_id(FK), controller_code, controlled_code, path(L1/L2/L3), started_at, ended_at, duration_sec, bytes_transferred, recording_path, created_at
  - `audit_logs`: id(UUID PK), team_id(FK), actor_id(FK→members), action, target_type, target_id, details(JSONB), ip_address, created_at
  - `recordings`: id(UUID PK), connection_id(FK→connection_records), team_id(FK), encryption_key(BYTEA), storage_path, duration_sec, file_size, created_at, expires_at
- **索引**: devices(team_id, status), devices(device_code), connection_records(team_id, started_at DESC), audit_logs(team_id, created_at DESC), recordings(team_id, created_at DESC)
- **002_seed_data.sql**:
  - 1 个测试团队 ("测试团队 Alpha", plan=pro, max_concurrent=10)
  - 3 个成员 (1 owner + 1 manager + 1 member)，密码使用 bcrypt 哈希的 "test123"
  - 5 个设备 (3 macos + 1 windows + 1 linux)，device_code 为 9 位纯数字 CHAR(9)，显示时按三位分组加空格（如 "100 200 301"、"100 200 302"、"100 200 303"、"100 200 304"、"100 200 305"）
  - 3 条连接记录（不同路径 L1/L2/L3）

**验收标准**:
- [ ] postgres 容器启动后 `docker exec psql -U rdcs -d rdcs -c '\dt'` 显示 6 张表
- [ ] 种子数据存在：`SELECT count(*) FROM devices` = 5
- [ ] 索引创建成功：`\di` 显示至少 5 个索引
- [ ] 外键约束生效：插入无效 team_id 报错

---

### Task 7: Redis + MinIO 初始化 (~0.5d)

**目标**: 配置 Redis keyspace 通知（用于设备离线检测）和 MinIO 存储桶（用于会话录制和文件传输）。通过 init 容器或启动脚本自动完成。

**依赖**: Task 2

**文件**:
- `deploy/docker/init-redis.sh` — Redis 初始化脚本（设置 keyspace 通知）
- `deploy/docker/init-minio.sh` — MinIO 初始化脚本（创建存储桶）
- `scripts/mc-alias.sh` — mc CLI 别名配置辅助脚本

**关键配置**:
- **Redis 初始化**:
  - CONFIG SET notify-keyspace-events Ex (E=keyevent, x=expired events)
  - CONFIG SET maxmemory 256mb (开发限制)
  - CONFIG SET maxmemory-policy allkeys-lru
  - 通过 docker compose 的 command 参数直接传入 --notify-keyspace-events Ex
- **MinIO 初始化** (init-minio.sh):
  - 等待 MinIO 就绪 (`mc ready` 循环检查)
  - mc alias set rdcs http://minio:9000 minioadmin minioadmin_dev
  - mc mb rdcs/recordings — 会话录制存储桶
  - mc mb rdcs/file-transfers — 文件传输暂存桶
  - mc mb rdcs/backups — 数据库备份桶（预留）
  - mc anonymous set download rdcs/recordings (只读下载)
  - 设置生命周期规则：file-transfers 对象 7 天自动清理
- **docker compose 集成**: 使用 init-minio 作为一次性 init 容器（profiles: ["init"]），或 entrypoint 脚本中检查并初始化

**验收标准**:
- [ ] `redis-cli CONFIG GET notify-keyspace-events` 返回 "Ex"
- [ ] `mc ls rdcs/` 显示 recordings, file-transfers, backups 三个桶
- [ ] MinIO Console (localhost:9001) 可登录查看桶列表

---

### Task 8: 开发环境 Compose 覆盖层 (~1d)

**目标**: 编写 docker-compose.dev.yml 覆盖基础配置，启用卷挂载热重载、DEBUG 日志、禁用 TLS、暴露调试端口。开发者仅需 `docker compose -f docker-compose.yml -f docker-compose.dev.yml up`。

**依赖**: Task 2, 3, 4, 5, 6, 7

**文件**:
- `deploy/docker/docker-compose.dev.yml` — 开发环境覆盖层
- `deploy/docker/.env.dev` — 开发环境变量默认值

**关键配置**:
- **signaling 覆盖**:
  - build 使用 Dockerfile.signaling.dev (cargo-watch 热重载)
  - volumes: `../../crates/rdcs-signaling/src:/app/src` (源码挂载)
  - environment: RUST_LOG=debug, SKIP_TLS=true, REDIS_URL=redis://redis:6379
- **relay 覆盖**:
  - build 使用 Dockerfile.relay.dev (cargo-watch 热重载)
  - volumes: `../../crates/rdcs-relay/src:/app/src`
  - environment: RUST_LOG=debug, RELAY_MIN_PORT=49152, RELAY_MAX_PORT=49200
- **api 覆盖**:
  - build 使用 Dockerfile.api.dev (air 热重载)
  - volumes: `../../services/api:/app`
  - environment: LOG_LEVEL=debug, DB_HOST=postgres, REDIS_HOST=redis, SKIP_TOTP=true
- **web 覆盖**:
  - build 使用 Dockerfile.web.dev (Vite HMR)
  - volumes: `../../web/admin/src:/app/src`, `../../web/admin/public:/app/public`
  - environment: VITE_API_BASE_URL=http://localhost:8080
- **postgres**: volumes 挂载 `../../migrations/postgres:/docker-entrypoint-initdb.d` 自动执行
- **全局**: restart: "no" (开发环境不自动重启), 日志驱动 json-file 限制 max-size=10m

**验收标准**:
- [ ] `docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d` 启动 7 个服务全部 healthy
- [ ] 修改 crates/rdcs-signaling/src/main.rs 后日志显示重新编译
- [ ] 修改 services/api/main.go 后日志显示 air rebuilding
- [ ] 修改 web/admin/src/App.tsx 后浏览器自动更新
- [ ] 所有服务日志级别为 DEBUG

---

### Task 9: 生产环境 Compose + Caddy (~1.5d)

**目标**: 编写生产级 compose 覆盖层，使用最终镜像（无卷挂载）、INFO 日志、Caddy 自动 TLS、资源限制和安全加固。

**依赖**: Task 2, 3, 4, 5, 6, 7

**文件**:
- `deploy/docker/docker-compose.prod.yml` — 生产环境覆盖层
- `deploy/docker/Caddyfile` — Caddy 反向代理配置
- `deploy/docker/.env.prod.example` — 生产环境变量模板

**关键配置**:
- **Caddy 服务**:
  - 镜像 caddy:2.8-alpine
  - 端口 80:80, 443:443, 443:443/udp (HTTP/3)
  - volumes: Caddyfile 挂载, caddy_data (证书), caddy_config
  - Caddyfile 路由:
    - signal.example.com → reverse_proxy signaling:8443 (WebSocket 升级)
    - api.example.com → reverse_proxy api:8080
    - admin.example.com → reverse_proxy web:3000
    - 自动 HTTPS (Let's Encrypt), HSTS header
- **所有应用服务覆盖**:
  - 移除 volumes 挂载（使用镜像内构建产物）
  - environment: LOG_LEVEL=info, SKIP_TLS=false, SKIP_TOTP=false
  - deploy.resources.limits: signaling (1G mem, 1 CPU), relay (2G mem, 2 CPU), api (512M mem, 0.5 CPU), web (256M mem, 0.25 CPU)
  - restart: unless-stopped
  - read_only: true + tmpfs 挂载 /tmp (安全加固)
  - security_opt: no-new-privileges
- **数据库安全**:
  - postgres 不暴露端口到宿主机（仅内部网络可达）
  - redis 不暴露端口到宿主机
  - minio 仅暴露 Console 端口 9001（API 通过内部网络）
- **.env.prod.example**: DOMAIN, POSTGRES_PASSWORD, MINIO_ROOT_PASSWORD, CADDY_EMAIL 占位

**验收标准**:
- [ ] `docker compose -f docker-compose.yml -f docker-compose.prod.yml config` 验证通过
- [ ] Caddy 容器启动并生成默认证书（自签名或 Let's Encrypt staging）
- [ ] 应用服务容器为 read_only 文件系统
- [ ] postgres/redis 端口不映射到宿主机 (`docker compose ps` 无 5432/6379 外部映射)
- [ ] 资源限制生效 (`docker stats` 显示 limits)

---

### Task 10: Makefile + 开发者文档 (~1.5d)

**目标**: 编写 Makefile 统一封装所有开发操作命令，并在 README 中添加新开发者 5 分钟上手指南。

**依赖**: Task 8, 9

**文件**:
- `Makefile` — 项目根目录，15+ targets
- `docs/DEVELOPMENT.md` — 开发者上手指南
- `scripts/health-check.sh` — 服务健康检查脚本
- `scripts/reset-dev.sh` — 开发环境重置脚本

**关键配置**:
- **Makefile targets** (使用 tab 缩进, 22 个 target):
  - `dev` / `dev-build` / `dev-down` / `dev-logs [service]`: 开发环境启停与日志
  - `prod` / `prod-build` / `prod-down`: 生产环境启停
  - `build-all`: 构建所有服务镜像（不启动）
  - `test-rust` / `test-go` / `test-web` / `test-all`: 单元测试（cargo test / go test / npm test）
  - `lint-rust` / `lint-go` / `lint-web` / `lint-all`: 静态检查（clippy -D warnings / golangci-lint / npm run lint）
  - `db-migrate` / `db-seed` / `db-shell`: 数据库操作
  - `redis-cli` / `minio-console`: 中间件交互式访问
  - `clean`: docker compose down -v 清除所有容器和卷
  - `health`: 执行 health-check.sh 检查 7 个服务状态
  - `reset`: 执行 reset-dev.sh 完全重置开发环境
- **scripts/health-check.sh**: 遍历 7 个服务检查 healthcheck 结果，输出彩色状态表格
- **scripts/reset-dev.sh**: docker compose down -v → 重新执行 dev target
- **docs/DEVELOPMENT.md**: 前置要求 (Docker 24+, Rust 1.80+, Go 1.22+, Node 20+)、5 分钟快速开始 (`make dev` → `make health`)、各服务本地地址表、常见问题排查

**验收标准**:
- [ ] `make dev` 一条命令启动所有服务
- [ ] `make health` 显示 7 个服务状态表格
- [ ] `make test-all` 依次执行三组测试
- [ ] `make lint-all` 依次执行三组 lint
- [ ] `make clean` 清除所有容器和卷
- [ ] `make reset` 完全重置后环境可正常使用
- [ ] 新开发者按 DEVELOPMENT.md 操作可在 5 分钟内看到所有服务运行

---

## 总工期估算

| Task | 工期 | 累计 |
|------|------|------|
| Task 1: 仓库结构 | 1d | 1d |
| Task 2: Compose 基础 | 1.5d | 2.5d |
| Task 3: Rust Dockerfile | 1.5d | 4d |
| Task 4: Go API Dockerfile | 1d | 5d |
| Task 5: React Web Dockerfile | 1d | 6d |
| Task 6: PostgreSQL 初始化 | 1d | 7d |
| Task 7: Redis + MinIO | 0.5d | 7.5d |
| Task 8: Dev 覆盖层 | 1d | 8.5d |
| Task 9: Prod + Caddy | 1.5d | 10d |
| Task 10: Makefile + 文档 | 1.5d | 11.5d |
| **缓冲 (review + 调试)** | **0.5d** | **~12d** |

Task 3/4/5 可并行（独立 Dockerfile），Task 8/9 可并行（独立 compose 覆盖层），并行后实际关键路径约 **8-9 天**。
