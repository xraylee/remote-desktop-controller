# 项目组织变更记录

**日期**: 2026-06-28  
**版本**: v2.0

---

## 📌 重要变更

### 文档位置变更

以下文档已从根目录移动到新的分类目录：

#### 移至 docs/installation/
- `INSTALLATION_CHECKLIST.md` → `docs/installation/INSTALLATION_CHECKLIST.md`
- `INSTALLATION_REPORT.md` → `docs/installation/INSTALLATION_REPORT.md`
- `INSTALL_STATUS.md` → `docs/installation/INSTALL_STATUS.md`
- `APPLE_SILICON_FIX.md` → `docs/installation/APPLE_SILICON_FIX.md`
- `BEST_MIRRORS.md` → `docs/installation/BEST_MIRRORS.md`
- `CHINA_MIRROR_GUIDE.md` → `docs/installation/CHINA_MIRROR_GUIDE.md`
- `FLUTTER_SPEED_GUIDE.md` → `docs/installation/FLUTTER_SPEED_GUIDE.md`

#### 移至 docs/reviews/
- `PROJECT_REVIEW.md` → `docs/reviews/PROJECT_REVIEW.md`
- `SESSION_REVIEW.md` → `docs/reviews/SESSION_REVIEW.md`
- `SUPERPOWERS_ASSESSMENT.md` → `docs/reviews/SUPERPOWERS_ASSESSMENT.md`
- `AGENTS.md` → `docs/reviews/AGENTS.md`
- `WebRTC_Integration_Review.md` → `docs/reviews/WebRTC_Integration_Review.md`

#### 移至 docs/archived/
- `EXECUTE_NOW.md` → `docs/archived/EXECUTE_NOW.md`
- `RUN_THIS_ON_YOUR_MAC.md` → `docs/archived/RUN_THIS_ON_YOUR_MAC.md`
- `TEST_PLAN.md` → `docs/archived/TEST_PLAN.md`
- `MIGRATION.md` → `docs/archived/MIGRATION.md`
- `livekit_integration_plan.md` → `docs/archived/livekit_integration_plan.md`

---

## 🆕 新增文档

### 项目结构文档
- **PROJECT_STRUCTURE.md** - 完整的项目组织和目录结构说明
  - 替代旧的 `PROJECT_ORGANIZATION.md`
  - 包含更详细的模块说明
  - 包含开发工作流指南

### 导航索引文档
- **docs/installation/README.md** - 安装文档索引
- **docs/reviews/README.md** - 评审文档索引
- **docs/archived/README.md** - 归档文档说明
- **scripts/README.md** - 脚本使用说明

### 整理报告
- **PROJECT_ORGANIZATION_SUMMARY.md** - 本次整理的详细总结
- **ORGANIZATION_CHANGES.md** - 本文件，变更记录

---

## 📖 如何查找文档

### 快速入口
1. **项目概览** → 查看 [README.md](README.md)
2. **项目结构** → 查看 [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)
3. **文档索引** → 查看 [docs/README.md](docs/README.md)
4. **安装指南** → 查看 [SETUP.md](SETUP.md) 或 [docs/installation/](docs/installation/)
5. **脚本工具** → 查看 [scripts/README.md](scripts/README.md)

### 文档分类
- **研究与规划**: `docs/research/`, `docs/specs/`, `docs/plans/`
- **架构与设计**: `docs/architecture/`, `docs/decisions/`
- **开发与测试**: `docs/testing/`, `docs/progress/`
- **安装与部署**: `docs/installation/`, `SETUP.md`
- **评审与总结**: `docs/reviews/`
- **归档内容**: `docs/archived/`

---

## 🔗 更新的链接

如果你有书签或文档引用了旧的文件路径，请更新为新路径：

### 示例
```markdown
# 旧路径
[安装清单](INSTALLATION_CHECKLIST.md)

# 新路径
[安装清单](docs/installation/INSTALLATION_CHECKLIST.md)
```

---

## ⚠️ 废弃说明

### PROJECT_ORGANIZATION.md
- **状态**: 已废弃（但保留在仓库中供参考）
- **替代**: [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)
- **原因**: 新文档更全面，包含完整的项目结构和工作流

---

## 💡 贡献指南

### 添加新文档时
1. 根据文档类型，放到 `docs/` 下的相应子目录
2. 更新该目录的 `README.md` 索引
3. 如果是重要文档，也更新 `docs/README.md` 的快速链接表
4. 在相关文档中添加交叉引用

### 文档分类参考
- **安装/配置指南** → `docs/installation/`
- **设计文档/PRD** → `docs/specs/`
- **架构决策** → `docs/decisions/`
- **进度报告** → `docs/progress/`
- **测试文档** → `docs/testing/`
- **评审报告** → `docs/reviews/`
- **过时/完成的临时文档** → `docs/archived/`

---

## 📅 变更历史

### 2026-06-28 - v2.0 重大重组
- 创建 docs/ 子目录分类系统
- 移动 17 个根目录文档到分类目录
- 创建多个导航索引文档
- 规范化脚本组织
- 创建完整的项目结构文档

---

**维护**: 请在进行文档结构变更时更新本文件  
**最后更新**: 2026-06-28
