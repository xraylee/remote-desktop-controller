<div align="center">

# Remote Desktop Control System (RDCS)

[English](#english) | [中文](#中文)

**A self-hosted, enterprise-grade remote desktop solution for small and medium businesses.**

[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-planning-orange.svg)](docs/ROADMAP.md)

</div>

---

## English

### What is RDCS?

RDCS (Remote Desktop Control System) is an open-source, self-hosted remote desktop solution designed specifically for small and medium businesses (SMBs). It combines the ease of use of commercial products like TeamViewer with the data sovereignty of self-hosted solutions like RustDesk.

### Why RDCS?

Based on our [in-depth analysis of 9 mainstream remote desktop products](docs/research/market-analysis.md), we identified a gap in the market:

| Problem | Current Solutions | RDCS Approach |
|---------|-----------------|---------------|
| **Commercial tools are expensive** | TeamViewer: $49/user/month | Concurrent session pricing — 80% cost reduction for a 50-person company |
| **Open-source tools are hard to deploy** | RustDesk requires Linux + Docker knowledge | One-line Docker deployment + web-based setup wizard |
| **Free tiers are crippled** | 720P/30FPS with ads and time limits | Free tier: 1080P/60FPS, no ads, no time limits |
| **Data goes through third parties** | All commercial solutions route data through their servers | 100% self-hosted — data never leaves your infrastructure |
| **Compliance is enterprise-only** | Only ScreenConnect ($30/session/month) offers video audit | Built-in session recording + audit logs on your own storage |

### Key Features (Planned)

- **One-command deployment** — `curl | bash` installs everything including TLS certificates
- **Progressive UI** — Employees see a simple "Connect" button; admins get a full management dashboard
- **Device-code auth** — No account registration required for end users
- **End-to-end encryption** — AES-256 with NaCl (XSalsa20-Poly1305)
- **Privacy screen** — Black out the remote display during sessions
- **Session recording** — Automatic encrypted recording stored on your server
- **Compliance audit** — Full operation logs with PDF/CSV export
- **Cross-platform** — Windows, macOS, Linux clients; iOS/Android planned

### Tech Stack (Planned)

| Component | Technology |
|-----------|-----------|
| Relay/Signaling Server | Rust (high-performance media relay) |
| Management API | Go |
| Desktop Client | Rust + Flutter (cross-platform UI) |
| Web Admin Console | React + TypeScript |
| Database | PostgreSQL + Redis |
| File Storage | MinIO / S3-compatible |
| Deployment | Docker Compose |

### Quick Start

> The project is currently in the planning phase. The commands below are planned, not yet available.

```bash
# Planned: One-line deployment
curl -fsSL https://install.rdcs.dev | bash

# Or manually with Docker Compose
git clone https://github.com/your-org/rdcs.git
cd rdcs
docker compose up -d
```

### Project Status

See our [Roadmap](docs/ROADMAP.md) for detailed milestones and timelines.

| Phase | Status | Timeline |
|-------|--------|---------|
| Market Research & Analysis | ✅ Complete | Q2 2026 |
| Product Brainstorming | ✅ Complete | Q2 2026 |
| Requirements & Architecture Design | 🔄 In Progress | Q3 2026 |
| MVP Development | 📋 Planned | Q3-Q4 2026 |
| Beta Testing | 📋 Planned | Q1 2027 |

### Documentation

- [Project Structure](PROJECT_STRUCTURE.md) — Complete project organization and directory structure
- [Market Analysis Report](docs/research/market-analysis.md) — In-depth comparison of 9 remote desktop products across 8 dimensions
- [Product Brainstorming](docs/research/product-brainstorming.md) — 28 creative ideas evaluated through SCAMPER, 5-Why, and other frameworks
- [Architecture Design](docs/specs/architecture-design.md) — Technical architecture documents
- [Documentation Index](docs/README.md) — Complete documentation navigation

### Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### License

This project is licensed under the [Apache License 2.0](LICENSE).

---

## 中文

### RDCS 是什么？

RDCS（远程桌面控制系统）是一款面向中小企业的开源自托管远程桌面解决方案。它将 TeamViewer 等商业产品的易用性与 RustDesk 等自托管方案的数据自主权相结合。

### 为什么做 RDCS？

基于我们对 [9 款主流远程桌面软件的深度分析](docs/research/market-analysis.md)，我们发现了市场空白：

- **商业软件太贵** — TeamViewer 按用户收费 $49/月，50 人公司年费约 21 万
- **开源方案太难** — RustDesk 部署需要 Linux + Docker 知识
- **免费版太弱** — 向日葵/ToDesk 免费版限 720P/30FPS 且限时限次
- **数据不安全** — 所有商业方案都经过第三方服务器
- **合规门槛高** — 只有 ScreenConnect 提供视频审计，但价格 $30/会话/月

### 核心设计理念

1. **一行部署** — `curl | bash` 完成全部安装，30 分钟从零到用
2. **渐进式 UI** — 员工看到连接按钮，管理员看到完整管理面板
3. **设备码免注册** — 终端用户无需注册账号，打开即用
4. **数据自主可控** — 100% 自托管，数据不离开企业基础设施

### 项目状态

当前处于规划阶段。已完成市场分析报告和产品脑暴，正在进行需求定义和技术架构设计。

详见 [项目路线图](docs/ROADMAP.md)。

### 文档导航

- [项目结构说明](PROJECT_STRUCTURE.md) — 完整的项目组织和目录结构
- [市场分析报告](docs/research/market-analysis.md) — 9 款产品 8 大维度 60+ 功能项深度对比
- [产品脑暴记录](docs/research/product-brainstorming.md) — 28 个创意方案，SCAMPER/5Why/竞品启发等多框架发散
- [架构设计](docs/specs/architecture-design.md) — 技术架构文档
- [文档索引](docs/README.md) — 完整文档导航

### 开源协议

本项目采用 [Apache License 2.0](LICENSE) 开源协议。
