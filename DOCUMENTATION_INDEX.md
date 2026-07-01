# 📚 RDCS 文档导航指南

**最后更新:** 2026-07-01  
**目的:** 快速定位项目中的各类文档

---

## 🎯 我想要...

### 快速开始项目
➡️ **[QUICK_START.md](QUICK_START.md)** - 5 分钟快速部署指南  
➡️ **[FLUTTER_START_GUIDE.md](FLUTTER_START_GUIDE.md)** - Flutter 客户端启动指南

### 了解项目概况
➡️ **[README.md](README.md)** - 项目总览、技术栈、路线图  
➡️ **[TODO.md](TODO.md)** - 开发计划和待办事项  
➡️ **[CHANGELOG.md](CHANGELOG.md)** - 版本变更记录

### 贡献代码
➡️ **[CONTRIBUTING.md](CONTRIBUTING.md)** - 贡献者指南和开发流程  
➡️ **[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)** - 社区行为准则  
➡️ **[client/flutter/test/README.md](client/flutter/test/README.md)** - 测试文档和 TDD 流程

### 查看实现文档
➡️ **[docs/implementation/](docs/implementation/)** - 所有实现计划和进度报告  
➡️ **[docs/NEXT_STEPS.md](docs/NEXT_STEPS.md)** - 下一步开发计划

### 查看历史文档
➡️ **[docs/archive/README.md](docs/archive/README.md)** - 归档文档索引  
➡️ **[client/flutter/test/archive/](client/flutter/test/archive/)** - 测试会话归档

---

## 📁 根目录文档清单

### 核心文档 (7 个)

| 文档 | 大小 | 用途 |
|------|------|------|
| **[README.md](README.md)** | 6.5K | 项目总览（中英文） |
| **[QUICK_START.md](QUICK_START.md)** | 4.9K | 快速部署指南 |
| **[FLUTTER_START_GUIDE.md](FLUTTER_START_GUIDE.md)** | 2.4K | Flutter 客户端指南 |
| **[TODO.md](TODO.md)** | 7.5K | 开发计划 |
| **[CHANGELOG.md](CHANGELOG.md)** | 0.6K | 变更日志 |
| **[CONTRIBUTING.md](CONTRIBUTING.md)** | 5.7K | 贡献指南 |
| **[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)** | 5.5K | 行为准则 |

### 辅助文档 (1 个)

| 文档 | 用途 |
|------|------|
| **[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)** | 本文档 - 文档导航 |

---

## 📂 重要目录结构

### docs/ - 项目文档
```
docs/
├── README.md                    # 文档总索引
├── NEXT_STEPS.md                # 下一步计划
├── implementation/              # 实现文档
│   ├── TASK_45_IMPLEMENTATION_PLAN.md
│   └── TASK_45_PROGRESS_REPORT.md
└── archive/                     # 历史归档
    ├── README.md                # 归档索引
    ├── client-websocket/        # WebSocket 集成文档
    ├── invite-code/             # 邀请码功能文档
    ├── server-status/           # 服务器状态文档
    └── test-fixes/              # 测试修复文档
```

### client/flutter/test/ - 测试文档
```
client/flutter/test/
├── README.md                              # 测试总览
├── GREEN_PHASE_FINAL_SUMMARY.md           # GREEN Phase 总结
├── TDD_PROGRESS_SUMMARY.md                # TDD 进度跟踪
├── SIGNALING_SERVICE_VERIFICATION.md      # 组件验证报告
├── WEBSOCKET_CLIENT_VERIFICATION.md       # 组件验证报告
├── verify_config_repository.dart          # 验证脚本
├── verify_signaling_service.dart          # 验证脚本
├── verify_websocket_client.dart           # 验证脚本
└── archive/                               # 测试会话归档
    └── 2026-06-30-tdd-green-phase/
```

---

## 🔍 按主题查找文档

### 部署和运维
- [QUICK_START.md](QUICK_START.md) - 快速部署
- `deploy/docker/` - Docker 配置

### 客户端开发
- [FLUTTER_START_GUIDE.md](FLUTTER_START_GUIDE.md) - Flutter 启动
- [client/flutter/test/README.md](client/flutter/test/README.md) - 测试文档
- `client/flutter/lib/` - Dart 源代码

### 后端开发
- `services/api/` - Go API 服务
- `services/signaling/` - Rust 信令服务
- `crates/` - Rust 核心库

### 测试和质量
- [client/flutter/test/README.md](client/flutter/test/README.md) - 测试总览
- [client/flutter/test/TDD_PROGRESS_SUMMARY.md](client/flutter/test/TDD_PROGRESS_SUMMARY.md) - TDD 进度
- `client/flutter/test/verify_*.dart` - 独立验证脚本

### 历史和归档
- [docs/archive/README.md](docs/archive/README.md) - 归档文档索引
- `docs/archive/client-websocket/` - WebSocket 集成历史
- `docs/archive/invite-code/` - 邀请码功能历史
- `client/flutter/test/archive/` - 测试会话归档

---

## 📊 文档状态

### 活跃维护 ✅
- README.md
- TODO.md
- client/flutter/test/README.md
- client/flutter/test/TDD_PROGRESS_SUMMARY.md

### 定期更新 🔄
- CHANGELOG.md
- docs/NEXT_STEPS.md
- docs/implementation/ 目录

### 归档保存 📦
- docs/archive/ 目录
- client/flutter/test/archive/ 目录

---

## 🚀 快速命令

### 运行测试
```bash
# 进入测试目录
cd client/flutter/test

# 运行验证脚本
dart run verify_config_repository.dart
dart run verify_signaling_service.dart
dart run verify_websocket_client.dart
```

### 查看文档
```bash
# 查看主文档
cat README.md

# 查看测试文档
cat client/flutter/test/README.md

# 查看归档索引
cat docs/archive/README.md
```

---

## 💡 文档规范

### 文件命名
- **大写 + 下划线:** `README.md`, `QUICK_START.md`, `TODO.md`
- **小写 + 连字符:** 归档目录使用 `client-websocket/`, `invite-code/`

### 文档结构
每个主要文档应包含：
1. 标题和简介
2. 目录（如果超过 200 行）
3. 主要内容
4. 相关链接
5. 最后更新日期

### 归档规则
- 临时进度文档 → `docs/archive/`
- 测试会话文档 → `client/flutter/test/archive/`
- 每个归档目录包含 `README.md` 索引

---

## 📞 需要帮助？

- 📖 查看 [CONTRIBUTING.md](CONTRIBUTING.md) 了解如何贡献
- 🐛 在 GitHub Issues 报告问题
- 💬 在 GitHub Discussions 参与讨论

---

**维护者:** RDCS Development Team  
**文档版本:** v1.0  
**最后更新:** 2026-07-01
