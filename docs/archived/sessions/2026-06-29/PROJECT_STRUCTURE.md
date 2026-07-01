# RDCS 项目结构说明

**更新日期**: 2026-06-28  
**版本**: 2.0

---

## 📁 项目目录结构

```
remote-desktop-controller/
├── README.md                   # 项目主页和简介
├── CHANGELOG.md                # 变更日志
├── CODE_OF_CONDUCT.md          # 行为准则
├── CONTRIBUTING.md             # 贡献指南
├── LICENSE                     # 开源协议 (Apache 2.0)
├── SETUP.md                    # 基本安装指南
├── PROJECT_ORGANIZATION.md     # 旧版项目组织说明（已废弃，参考本文件）
├── PROJECT_STRUCTURE.md        # 本文件 - 项目结构完整说明
│
├── Cargo.toml                  # Rust workspace 配置
├── Cargo.lock                  # Rust 依赖锁定
├── Makefile                    # 项目构建脚本
├── .editorconfig               # 编辑器配置
├── .gitignore                  # Git 忽略规则
├── .env.example                # 环境变量模板
│
├── crates/                     # 🦀 Rust 核心模块
│   ├── rdcs-core/              # 核心类型和接口
│   ├── rdcs-codec/             # 视频编解码
│   ├── rdcs-crypto/            # 加密模块
│   ├── rdcs-platform/          # 跨平台抽象层
│   ├── rdcs-macos/             # macOS 平台实现
│   ├── rdcs-transport/         # 传输层
│   ├── rdcs-relay/             # 中继服务器
│   ├── rdcs-signaling/         # 信令服务器
│   ├── rdcs-connection/        # 连接管理
│   ├── rdcs-session/           # 会话管理
│   ├── rdcs-transfer/          # 数据传输
│   ├── rdcs-nat-test/          # NAT 穿透测试
│   └── rdcs-ffi/               # FFI 接口层
│
├── client/                     # 📱 客户端代码
│   └── flutter/                # Flutter 跨平台客户端
│       ├── lib/                # Dart 源代码
│       ├── ios/                # iOS 平台代码
│       ├── android/            # Android 平台代码
│       ├── macos/              # macOS 平台代码
│       ├── windows/            # Windows 平台代码
│       └── linux/              # Linux 平台代码
│
├── web/                        # 🌐 Web 管理后台
│   └── admin/                  # React + TypeScript 管理控制台
│       ├── src/                # 源代码
│       ├── public/             # 静态资源
│       ├── dist/               # 构建输出
│       └── package.json        # NPM 依赖
│
├── services/                   # 🔧 后端服务（计划中）
│   └── api/                    # Go API 服务
│
├── scripts/                    # 📜 脚本工具
│   ├── README.md               # 脚本使用说明
│   ├── installation/           # 安装脚本
│   │   ├── auto-install-all.sh
│   │   ├── check-and-install.sh
│   │   └── quick-install.sh
│   ├── diagnostics/            # 诊断脚本
│   │   └── diagnose-videotoolbox-crash.sh
│   ├── check-compilation.sh    # 编译检查
│   ├── run-unit-tests.sh       # 单元测试
│   ├── run-local-roundtrip.sh  # 硬件加速回环测试
│   ├── run-local-roundtrip-mock.sh      # Mock 回环测试
│   ├── run-local-roundtrip-openh264.sh  # OpenH264 测试
│   ├── test-hardware-accel-gate.sh      # Feature gate 测试
│   ├── health-check.sh         # 健康检查
│   └── reset-dev.sh            # 重置开发环境
│
├── docs/                       # 📚 项目文档
│   ├── README.md               # 文档索引
│   ├── ROADMAP.md              # 项目路线图
│   ├── DEVELOPMENT.md          # 开发指南
│   ├── research/               # 市场研究
│   │   ├── market-analysis.md
│   │   ├── product-brainstorming.md
│   │   └── product-brainstorming-v2.md
│   ├── specs/                  # 需求规范
│   │   ├── architecture-design.md
│   │   ├── prd-v1.md
│   │   ├── prd-review-report.md
│   │   └── wave-migration-plan.md
│   ├── architecture/           # 架构文档
│   ├── decisions/              # 架构决策记录 (ADR)
│   │   ├── WEBRTC_ARCHITECTURE.md
│   │   ├── WEBRTC_CODEC_INTEGRATION_DECISION.md
│   │   └── WEBRTC_SOLUTION_COMPARISON.md
│   ├── plans/                  # 开发阶段计划
│   │   ├── plan-a-core-engine.md
│   │   ├── plan-b-signaling.md
│   │   ├── plan-c-relay.md
│   │   ├── plan-d-flutter-client.md
│   │   ├── plan-e-web-console.md
│   │   └── plan-f-dev-environment.md
│   ├── progress/               # 进度报告
│   │   ├── NEXT_STEPS.md
│   │   ├── final-project-status.md
│   │   └── (各阶段完成报告)
│   ├── testing/                # 测试文档
│   │   ├── TESTING_GUIDELINES.md
│   │   ├── PHASE1_COMPLETION_REPORT.md
│   │   └── VIDEOTOOLBOX_CRASH_DIAGNOSIS.md
│   ├── installation/           # 安装文档
│   │   ├── README.md
│   │   ├── INSTALLATION_CHECKLIST.md
│   │   ├── APPLE_SILICON_FIX.md
│   │   ├── BEST_MIRRORS.md
│   │   └── CHINA_MIRROR_GUIDE.md
│   ├── reviews/                # 评审文档
│   │   ├── README.md
│   │   ├── PROJECT_REVIEW.md
│   │   ├── WebRTC_Integration_Review.md
│   │   └── architecture-review-report.md
│   ├── archived/               # 归档文档
│   │   └── README.md
│   ├── prototypes/             # 原型代码
│   └── images/                 # 图片和截图
│
├── tests/                      # 🧪 集成测试
├── migrations/                 # 🗄️ 数据库迁移
│   └── postgres/
├── deploy/                     # 🚀 部署配置
│   └── docker/
└── target/                     # 构建输出（已忽略）
```

---

## 🎯 核心模块说明

### Rust Crates (crates/)

| Crate | 功能 | 状态 |
|-------|------|------|
| `rdcs-core` | 核心类型和接口定义 | ✅ 活跃开发 |
| `rdcs-codec` | 视频编解码（H.264/H.265） | ✅ 活跃开发 |
| `rdcs-crypto` | 加密和安全 | 📋 计划中 |
| `rdcs-platform` | 跨平台抽象层 | ✅ 活跃开发 |
| `rdcs-macos` | macOS 屏幕捕获 | ✅ 活跃开发 |
| `rdcs-transport` | 网络传输层 | 📋 计划中 |
| `rdcs-relay` | 中继服务器 | 📋 计划中 |
| `rdcs-signaling` | 信令服务器 | 📋 计划中 |
| `rdcs-connection` | 连接管理 | 📋 计划中 |
| `rdcs-session` | 会话管理 | 📋 计划中 |
| `rdcs-transfer` | 数据传输 | 📋 计划中 |
| `rdcs-nat-test` | NAT 穿透测试 | 📋 计划中 |
| `rdcs-ffi` | FFI 接口层 | 📋 计划中 |

### 客户端 (client/)

- **Flutter 客户端**: 跨平台桌面应用（Windows、macOS、Linux）
  - 使用 Flutter 构建统一 UI
  - 通过 FFI 调用 Rust 核心模块
  - 未来计划支持 iOS/Android

### Web 后台 (web/)

- **Admin Console**: React + TypeScript 管理后台
  - 用户和设备管理
  - 会话监控和录像回放
  - 系统配置和审计日志

### 后端服务 (services/)

- **API 服务**: Go 实现的管理 API（计划中）
  - RESTful API
  - 数据库访问
  - 认证和授权

---

## 📜 脚本工具快速参考

### 开发测试流程

```bash
# 1. 编译检查
./scripts/check-compilation.sh

# 2. 运行单元测试
./scripts/run-unit-tests.sh

# 3. Mock 回环测试（推荐）
./scripts/run-local-roundtrip-mock.sh

# 4. Feature gate 测试
./scripts/test-hardware-accel-gate.sh
```

### 安装和配置

```bash
# 自动安装所有依赖
./scripts/installation/auto-install-all.sh

# 快速安装（中国镜像优化）
./scripts/installation/quick-install.sh
```

详见 [scripts/README.md](scripts/README.md)

---

## 📚 文档导航

### 新手入门
1. [README.md](README.md) - 项目介绍
2. [SETUP.md](SETUP.md) - 安装指南
3. [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) - 开发指南
4. [CONTRIBUTING.md](CONTRIBUTING.md) - 贡献指南

### 了解项目
1. [docs/research/market-analysis.md](docs/research/market-analysis.md) - 市场分析
2. [docs/ROADMAP.md](docs/ROADMAP.md) - 项目路线图
3. [docs/specs/prd-v1.md](docs/specs/prd-v1.md) - 产品需求文档

### 技术深入
1. [docs/specs/architecture-design.md](docs/specs/architecture-design.md) - 架构设计
2. [docs/decisions/](docs/decisions/) - 架构决策记录
3. [docs/testing/TESTING_GUIDELINES.md](docs/testing/TESTING_GUIDELINES.md) - 测试规范

### 快速索引
- 📖 完整文档索引: [docs/README.md](docs/README.md)
- 🔧 脚本使用说明: [scripts/README.md](scripts/README.md)
- 📦 安装文档: [docs/installation/README.md](docs/installation/README.md)
- 🔍 评审报告: [docs/reviews/README.md](docs/reviews/README.md)

---

## 🔄 开发工作流

### 开发新功能

```bash
# 1. 创建特性分支
git checkout -b feature/my-feature

# 2. 开发和测试
# 编辑代码...
./scripts/check-compilation.sh
./scripts/run-unit-tests.sh

# 3. 提交代码
git add .
git commit -m "feat: add my feature"

# 4. 推送和创建 PR
git push origin feature/my-feature
```

### 修复 Bug

```bash
# 1. 创建修复分支
git checkout -b fix/issue-123

# 2. 修复和测试
# 编辑代码...
./scripts/run-unit-tests.sh
./scripts/run-local-roundtrip-mock.sh

# 3. 提交
git commit -m "fix: resolve issue #123"
```

### 发布版本

```bash
# 1. 完整测试
./scripts/check-compilation.sh
./scripts/run-unit-tests.sh
./scripts/run-local-roundtrip-mock.sh
cargo bench

# 2. 更新版本号
# 编辑 Cargo.toml 中的版本号

# 3. 更新 CHANGELOG
# 编辑 CHANGELOG.md

# 4. 标记版本
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

---

## 📊 项目状态

### 当前阶段
- **Phase 1**: 编解码核心功能 ✅ 基本完成
  - Mock 编解码器测试通过
  - VideoToolbox 硬件加速待修复
- **Phase 2**: 网络传输层 📋 规划中

### 近期目标
1. 修复 VideoToolbox FFI 崩溃问题
2. 集成 OpenH264 软件编码器
3. 实现 RTP 网络传输
4. 开发 Flutter 客户端原型

详见 [docs/progress/NEXT_STEPS.md](docs/progress/NEXT_STEPS.md)

---

## 🤝 贡献指南

### 添加新模块

1. 在 `crates/` 下创建新的 crate
   ```bash
   cargo new --lib crates/rdcs-newmodule
   ```

2. 在根 `Cargo.toml` 中添加到 workspace
   ```toml
   [workspace]
   members = [
       "crates/rdcs-newmodule",
       # ...
   ]
   ```

3. 编写代码和测试

4. 更新本文件的模块说明

### 添加文档

1. 根据文档类型放到 `docs/` 下的相应子目录
2. 更新 `docs/README.md` 的索引
3. 在相关文档中添加交叉引用

### 添加脚本

1. 根据用途放到 `scripts/` 下的相应子目录
2. 添加可执行权限: `chmod +x scripts/xxx.sh`
3. 更新 `scripts/README.md`
4. 在本文件中更新脚本列表

---

## 📝 版本历史

### v2.0 - 2026-06-28
- ✅ 重组项目文档结构
- ✅ 整理脚本到 `scripts/` 目录
- ✅ 创建 `docs/installation/`、`docs/reviews/`、`docs/archived/` 子目录
- ✅ 更新 `.gitignore` 添加测试输出文件
- ✅ 完善文档索引和导航

### v1.0 - 2026-06-28 (PROJECT_ORGANIZATION.md)
- ✅ 创建初始项目组织文档
- ✅ 整理测试脚本
- ✅ 创建测试规范文档

---

**维护人**: RDCS Team  
**最后更新**: 2026-06-28
