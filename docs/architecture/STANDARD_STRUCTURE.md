# Superpowers 标准项目结构

**制定日期**: 2026-06-28  
**框架**: Superpowers 最佳实践  
**版本**: v2.0

---

## 🎯 设计原则

### 1. 清晰性原则（Clarity First）
- 根目录简洁明了
- 文档分类清晰
- 5 分钟内找到需要的信息

### 2. 单一信息源原则（Single Source of Truth）
- 每个主题只有一个权威文档
- 避免重复和冗余
- 保持同步更新

### 3. MVP 优先原则（MVP First）
- 突出当前阶段
- 明确下一步行动
- 隐藏不相关信息

### 4. 垂直切片原则（Vertical Slice）
- 按功能而非层次组织
- 端到端可追溯
- 易于理解和维护

---

## 📁 推荐的项目结构

```
remote-desktop-controller/
│
├── 📄 核心文档（5个）
│   ├── README.md                    # 项目入口（一页纸概览）
│   ├── CHANGELOG.md                 # 变更日志
│   ├── CONTRIBUTING.md              # 贡献指南
│   ├── CODE_OF_CONDUCT.md          # 行为准则
│   └── LICENSE                      # 开源协议
│
├── 🏗️ 核心代码
│   ├── crates/                      # Rust 核心模块
│   │   ├── rdcs-core/              # 核心类型
│   │   ├── rdcs-codec/             # 编解码器
│   │   ├── rdcs-platform/          # 平台抽象
│   │   ├── rdcs-connection/        # 网络连接
│   │   └── ...
│   │
│   ├── client/                      # 客户端（Future）
│   ├── web/                         # Web 后台（Future）
│   └── services/                    # 后端服务（Future）
│
├── 📜 脚本工具
│   ├── scripts/
│   │   ├── README.md               # 脚本使用说明
│   │   ├── installation/           # 安装脚本
│   │   ├── diagnostics/            # 诊断脚本
│   │   └── e2e/                    # E2E 测试脚本 ⭐
│   │       ├── run-all-tests.sh
│   │       ├── test-local-network.sh
│   │       └── test-remote-network.sh
│   │
│   └── Makefile                    # 快捷命令
│
└── 📚 文档系统
    └── docs/
        ├── README.md               # 文档索引（入口）
        │
        ├── 🚀 快速开始（3个核心文档）
        │   ├── QUICK_START.md      # 5 分钟快速开始
        │   ├── INSTALLATION.md     # 安装指南
        │   └── TROUBLESHOOTING.md  # 常见问题
        │
        ├── 🎯 项目管理（4个核心文档）
        │   ├── MVP.md              # MVP 定义 ⭐
        │   ├── CURRENT_PHASE.md    # 当前阶段 ⭐
        │   ├── ROADMAP.md          # 产品路线图
        │   └── PHASES.md           # 阶段历史记录
        │
        ├── 🧪 测试文档（3个核心文档）
        │   ├── TESTING_STRATEGY.md     # 测试策略 ⭐
        │   ├── E2E_TEST_PLAN.md       # E2E 测试方案 ⭐
        │   └── TEST_RESULTS.md        # 最新测试结果
        │
        ├── 🏗️ 架构设计（按功能组织）
        │   ├── decisions/          # ADR（架构决策记录）
        │   │   ├── README.md
        │   │   ├── 001-webrtc-architecture.md
        │   │   ├── 002-codec-choice.md
        │   │   └── ...
        │   │
        │   └── specs/              # 功能规范
        │       ├── README.md
        │       ├── architecture-design.md
        │       └── ...
        │
        ├── 📖 开发指南
        │   ├── DEVELOPMENT.md      # 开发环境设置
        │   ├── CODING_STYLE.md     # 代码规范
        │   └── API.md              # API 文档
        │
        ├── 📦 部署运维（Future）
        │   └── deployment/
        │
        └── 🗄️ 归档
            └── archived/
                ├── README.md       # 归档说明
                ├── sessions/       # 会话记录
                │   └── 2026-06-28/
                └── deprecated/     # 废弃文档
```

---

## 📄 核心文档说明

### 1. README.md（项目入口）⭐ 最重要
**目标**: 5 分钟内让新成员了解项目

**必须包含**:
```markdown
# RDCS - Remote Desktop Control System

## 什么是 RDCS？
[一句话描述]

## 我们在哪里？
✅ Phase 1: 网络连接层（完成）
🔄 Phase 2: 视频传输层（70%）
📋 Phase 3: 输入控制层（计划中）
📋 MVP: 预计 2-3 周

## 快速开始
[最简单的 5 步上手指南]

## 文档导航
- [MVP 定义](docs/MVP.md) - 我们要做什么？
- [当前阶段](docs/CURRENT_PHASE.md) - 我们在哪里？
- [E2E 测试](docs/E2E_TEST_PLAN.md) - 如何验证？
- [开发指南](docs/DEVELOPMENT.md) - 如何贡献？

## 项目结构
[简单的目录树]

## 贡献
[如何贡献]

## License
[协议]
```

### 2. docs/README.md（文档索引）
**目标**: 快速找到需要的文档

**组织方式**:
- 按用户角色组织（新手、开发者、测试者）
- 按任务组织（开始、开发、测试、部署）
- 突出最重要的 3-5 个文档

---

## 🎯 核心文档矩阵

### 必须有的文档（Must Have）

| 文档 | 目的 | 受众 | 更新频率 |
|------|------|------|---------|
| **README.md** | 项目入口 | 所有人 | 重大里程碑 |
| **docs/MVP.md** | MVP 定义 | 所有人 | MVP 前锁定 |
| **docs/CURRENT_PHASE.md** | 当前状态 | 团队 | 每日/每周 |
| **docs/E2E_TEST_PLAN.md** | 测试方案 | 测试者 | Phase 开始 |
| **docs/QUICK_START.md** | 快速开始 | 新手 | Phase 完成 |
| **docs/INSTALLATION.md** | 安装指南 | 所有人 | 依赖变更 |
| **docs/ROADMAP.md** | 产品路线 | PM/用户 | 季度 |

### 应该有的文档（Should Have）

| 文档 | 目的 | 优先级 |
|------|------|--------|
| **docs/PHASES.md** | 阶段历史 | 🟡 中 |
| **docs/TESTING_STRATEGY.md** | 测试策略 | 🟡 中 |
| **docs/DEVELOPMENT.md** | 开发指南 | 🟡 中 |
| **docs/TROUBLESHOOTING.md** | 问题排查 | 🟡 中 |
| **docs/API.md** | API 文档 | 🟢 低 |

### 不需要的文档（Avoid）

- ❌ 多个阶段完成报告（合并到 PHASES.md）
- ❌ 会话总结（归档到 archived/sessions/）
- ❌ 临时测试文档（归档或删除）
- ❌ 重复的安装指南（合并到 INSTALLATION.md）

---

## 📊 文档生命周期

### 活跃文档（Active）
**位置**: `docs/` 主目录  
**状态**: 持续更新  
**示例**: MVP.md, CURRENT_PHASE.md

**维护规则**:
- 每个 Phase 开始时审查
- 重大变更后更新
- 保持准确和最新

### 参考文档（Reference）
**位置**: `docs/decisions/`, `docs/specs/`  
**状态**: 稳定，偶尔更新  
**示例**: ADR, 架构设计

**维护规则**:
- 写入后基本不变
- 新增而非修改
- 作为历史记录

### 归档文档（Archived）
**位置**: `docs/archived/`  
**状态**: 只读  
**示例**: 会话记录、废弃功能

**维护规则**:
- 移入时添加归档说明
- 不再更新
- 定期清理

---

## 🔄 文档工作流

### 创建新文档
```
1. 确定类型（核心/参考/临时）
2. 选择位置（主目录/子目录）
3. 使用模板创建
4. 更新索引（docs/README.md）
5. 添加到 git
```

### 更新文档
```
1. 检查文档是否最新
2. 修改内容
3. 更新"最后更新"日期
4. 提交 git（描述性 commit）
```

### 归档文档
```
1. 移动到 archived/
2. 在原位置添加重定向说明
3. 更新索引移除链接
4. 提交 git
```

---

## 🎨 文档模板

### 核心文档模板
```markdown
# [标题]

**创建日期**: YYYY-MM-DD  
**最后更新**: YYYY-MM-DD  
**状态**: [活跃/稳定/归档]

---

## 🎯 目标

[本文档的目的]

## 📋 内容

[主要内容]

## 📚 相关文档

- [链接 1](path)
- [链接 2](path)

---

**维护人**: [Name]  
**审查周期**: [每日/每周/每月]
```

### ADR（架构决策记录）模板
```markdown
# ADR-XXX: [决策标题]

**状态**: [提议/接受/废弃]  
**日期**: YYYY-MM-DD  
**决策者**: [Name]

## 背景

[为什么需要这个决策？]

## 决策

[我们决定做什么？]

## 备选方案

1. 方案 A: ...
2. 方案 B: ...

## 后果

### 优点
- ...

### 缺点
- ...

## 相关决策

- ADR-XXX: ...
```

---

## 🚀 迁移计划

### 从当前状态到标准结构

#### Phase 1: 清理根目录（30分钟）
```bash
# 只保留 5 个核心文档
keep: README.md, CHANGELOG.md, CONTRIBUTING.md, CODE_OF_CONDUCT.md, LICENSE

# 归档临时文档
mkdir -p docs/archived/sessions/2026-06-28
mv ICE_TEST_*.md docs/archived/sessions/2026-06-28/
mv ROLE_*.md docs/archived/sessions/2026-06-28/
mv SESSION_*.md docs/archived/sessions/2026-06-28/
mv PROJECT_*.md docs/archived/sessions/2026-06-28/
mv ORGANIZATION_*.md docs/archived/sessions/2026-06-28/
```

#### Phase 2: 合并重复文档（1小时）
```bash
# 合并所有 Phase 报告
cat docs/PHASE*.md docs/testing/PHASE*.md > docs/PHASES.md
rm docs/PHASE*.md docs/testing/PHASE*_REPORT.md

# 合并安装文档
cat docs/installation/*.md > docs/INSTALLATION.md
mv docs/installation/* docs/archived/deprecated/

# 创建测试策略
# 从 testing/TESTING_GUIDELINES.md 提取
```

#### Phase 3: 创建核心文档（1小时）
```bash
# 已创建
docs/MVP.md ✅
docs/CURRENT_PHASE.md ✅
docs/E2E_TEST_PLAN.md ✅

# 待创建
docs/QUICK_START.md
docs/TESTING_STRATEGY.md
docs/TROUBLESHOOTING.md
```

#### Phase 4: 更新索引（30分钟）
```bash
# 更新 README.md
# 更新 docs/README.md
# 确保所有链接有效
```

---

## ✅ 验证清单

### 结构验证
- [ ] 根目录只有 5-6 个文档
- [ ] docs/ 有清晰的分类
- [ ] 没有重复文档
- [ ] 所有文档有明确目的

### 内容验证
- [ ] MVP 定义清晰
- [ ] 当前阶段明确
- [ ] E2E 测试方案完整
- [ ] 快速开始指南可用

### 可用性验证
- [ ] 新成员 5 分钟内找到入口
- [ ] 开发者 10 分钟内开始贡献
- [ ] 测试者 15 分钟内运行测试

---

## 📈 持续改进

### 每周检查
- [ ] CURRENT_PHASE.md 是否最新？
- [ ] 有没有新的临时文档需要整理？
- [ ] 链接是否都有效？

### 每个 Phase 检查
- [ ] 更新 MVP.md（如果有变化）
- [ ] 更新 CURRENT_PHASE.md
- [ ] 归档上个 Phase 的临时文档
- [ ] 创建新 Phase 的文档

### 季度检查
- [ ] 审查所有活跃文档
- [ ] 归档不再需要的文档
- [ ] 更新 ROADMAP.md
- [ ] 收集用户反馈

---

## 💡 最佳实践

### DO（应该做）
✅ 保持根目录简洁  
✅ 每个文档有明确目的  
✅ 使用清晰的标题和结构  
✅ 及时更新"最后更新"日期  
✅ 归档而非删除旧文档  
✅ 使用相对链接  
✅ 添加导航面包屑

### DON'T（不应该做）
❌ 在根目录堆积临时文档  
❌ 创建重复的文档  
❌ 让文档过时不更新  
❌ 使用绝对路径链接  
❌ 删除有历史价值的文档  
❌ 过度嵌套目录（> 3 层）  
❌ 使用模糊的文件名

---

**制定人**: AI Assistant  
**制定日期**: 2026-06-28  
**版本**: v2.0  
**状态**: ✅ 待执行
