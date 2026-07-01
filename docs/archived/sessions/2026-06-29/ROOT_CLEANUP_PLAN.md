# 根目录文档清理计划

**日期**: 2026-06-29  
**当前状态**: 根目录 25 个文档文件  
**目标状态**: 根目录 6 个核心文档  
**依据**: docs/STANDARD_STRUCTURE.md (Superpowers 规范)

---

## 📊 文档分类结果

### ✅ 保留在根目录（6个）

| 文件 | 原因 |
|------|------|
| README.md | ✅ 项目入口，必须保留 |
| CHANGELOG.md | ✅ 变更日志，标准文件 |
| CONTRIBUTING.md | ✅ 贡献指南，标准文件 |
| CODE_OF_CONDUCT.md | ✅ 行为准则，标准文件 |
| LICENSE | ✅ 开源协议，标准文件 |
| TODO.md | ✅ 任务清单，根目录常见 |

### 📁 移动到 docs/testing/（7个）

| 文件 | 目标位置 | 原因 |
|------|----------|------|
| QUICK_TEST_GUIDE.md | docs/testing/ | 测试指南 |
| TEST_EXECUTION_SUMMARY.md | docs/testing/ | 测试摘要 |
| WEB_ADMIN_TEST_COMPLETE.md | docs/testing/ | Web 测试报告 |
| WEB_SERVICE_TEST_COMPLETE.md | docs/testing/ | 服务测试报告 |
| WEB_SERVICE_TEST_SUMMARY.md | docs/testing/ | 服务测试摘要 |
| VERIFICATION_CHECKLIST.md | docs/testing/ | 验证清单 |
| COMPILE_VERIFICATION.md | docs/testing/ | 编译验证 |

### 📁 移动到 docs/installation/（1个）

| 文件 | 目标位置 | 原因 |
|------|----------|------|
| SETUP.md | docs/installation/ | 环境配置指南 |

### 📁 移动到 docs/troubleshooting/（2个）

| 文件 | 目标位置 | 原因 |
|------|----------|------|
| SIGNALING_CONNECTION_DIAGNOSIS.md | docs/troubleshooting/ | 问题诊断 |
| FIX_SUMMARY.md | docs/troubleshooting/ | 修复总结 |

### 📁 移动到 docs/localization/（1个）

| 文件 | 目标位置 | 原因 |
|------|----------|------|
| WEB_ADMIN_UI_LOCALIZATION.md | docs/localization/ | UI 本地化 |

### 🗄️ 归档到 docs/archived/sessions/2026-06-29/（7个）

| 文件 | 原因 |
|------|------|
| CLEANUP_PLAN.md | 临时清理计划 |
| COMPLETION_SUMMARY.md | 会话总结 |
| PHASE4.1_STATUS.md | 阶段状态快照 |
| PROJECT_STATUS_2026-06-29.md | 项目状态快照 |
| PROJECT_STRUCTURE.md | 结构说明（已有 docs/STANDARD_STRUCTURE.md） |
| QUICK_REFERENCE.md | 临时参考卡片 |
| STRUCTURE_REORGANIZATION_REPORT.md | 临时分析报告 |

### 🗑️ 删除（2个临时文件）

| 文件 | 原因 |
|------|------|
| README_FIX.txt | 临时修复说明，已修复完成 |
| test-signaling-report.txt | 临时测试报告，内容已整合到其他文档 |

---

## 🚀 执行步骤

### Step 1: 创建必要的目录
```bash
mkdir -p docs/testing
mkdir -p docs/installation
mkdir -p docs/troubleshooting
mkdir -p docs/localization
mkdir -p docs/archived/sessions/2026-06-29
```

### Step 2: 移动测试文档
```bash
mv QUICK_TEST_GUIDE.md docs/testing/
mv TEST_EXECUTION_SUMMARY.md docs/testing/
mv WEB_ADMIN_TEST_COMPLETE.md docs/testing/
mv WEB_SERVICE_TEST_COMPLETE.md docs/testing/
mv WEB_SERVICE_TEST_SUMMARY.md docs/testing/
mv VERIFICATION_CHECKLIST.md docs/testing/
mv COMPILE_VERIFICATION.md docs/testing/
```

### Step 3: 移动其他分类文档
```bash
mv SETUP.md docs/installation/
mv SIGNALING_CONNECTION_DIAGNOSIS.md docs/troubleshooting/
mv FIX_SUMMARY.md docs/troubleshooting/
mv WEB_ADMIN_UI_LOCALIZATION.md docs/localization/
```

### Step 4: 归档会话文档
```bash
mv CLEANUP_PLAN.md docs/archived/sessions/2026-06-29/
mv COMPLETION_SUMMARY.md docs/archived/sessions/2026-06-29/
mv PHASE4.1_STATUS.md docs/archived/sessions/2026-06-29/
mv PROJECT_STATUS_2026-06-29.md docs/archived/sessions/2026-06-29/
mv PROJECT_STRUCTURE.md docs/archived/sessions/2026-06-29/
mv QUICK_REFERENCE.md docs/archived/sessions/2026-06-29/
mv STRUCTURE_REORGANIZATION_REPORT.md docs/archived/sessions/2026-06-29/
```

### Step 5: 删除临时文件
```bash
rm README_FIX.txt
rm test-signaling-report.txt
```

### Step 6: 验证结果
```bash
ls -1 *.md | wc -l  # 应该输出 6
```

---

## ✅ 验证清单

- [ ] 根目录只剩 6 个 .md 文件
- [ ] 所有测试文档在 docs/testing/
- [ ] 配置文档在 docs/installation/
- [ ] 问题诊断在 docs/troubleshooting/
- [ ] 临时文档已归档
- [ ] 临时文件已删除
- [ ] 更新 docs/README.md 索引

---

## 📝 后续工作

1. 更新 docs/README.md，添加新分类的索引
2. 检查所有文档内的链接，确保路径正确
3. 更新 memory 中的项目结构记录
