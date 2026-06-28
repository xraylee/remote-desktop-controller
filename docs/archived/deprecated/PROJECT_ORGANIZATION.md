# RDCS 项目文件组织说明

**日期**: 2026-06-28  
**版本**: 1.0

---

## 📁 项目结构

```
remote-desktop-controller/
├── crates/              # Rust 核心模块
├── api/                 # Go API 服务
├── client/              # Flutter 客户端
├── web/                 # Web 管理后台
├── scripts/             # 测试和工具脚本
├── docs/                # 文档
└── target/              # 编译输出（.gitignore）
```

---

## 🔧 scripts/ 目录

### 编译和测试脚本

```
scripts/
├── check-compilation.sh        # ✅ 整体编译检查
├── run-unit-tests.sh          # ✅ 单元测试
├── test-hardware-accel-gate.sh # ✅ Feature gate 测试
├── run-local-roundtrip-mock.sh # ✅ Mock 回环测试
└── run-local-roundtrip.sh     # ⚠️  硬件加速回环测试（有问题）
```

### 诊断脚本

```
scripts/diagnostics/
└── diagnose-videotoolbox-crash.sh  # 🔍 VideoToolbox 崩溃诊断
```

### 使用方法

```bash
# 1. 编译检查（第一步）
cd /Users/lc/Development/source/remote-desktop-controller
./scripts/check-compilation.sh

# 2. 单元测试（第二步）
./scripts/run-unit-tests.sh

# 3. Mock 回环测试（第三步）
./scripts/run-local-roundtrip-mock.sh

# 4. Feature gate 测试（验证硬件隔离）
./scripts/test-hardware-accel-gate.sh

# 5. 硬件加速测试（当前有问题，暂不运行）
# ./scripts/run-local-roundtrip.sh

# 6. 诊断 VideoToolbox 崩溃（可选）
# ./scripts/diagnostics/diagnose-videotoolbox-crash.sh
```

---

## 📚 docs/testing/ 目录

### 测试文档

```
docs/testing/
├── TESTING_GUIDELINES.md           # ✅ 测试规范和流程
├── PHASE1_COMPLETION_REPORT.md     # ✅ Phase 1 完成报告
└── VIDEOTOOLBOX_CRASH_DIAGNOSIS.md # 🔍 VideoToolbox 诊断记录
```

### 阅读顺序

1. **TESTING_GUIDELINES.md** - 了解测试规范
2. **PHASE1_COMPLETION_REPORT.md** - 查看 Phase 1 成果
3. **VIDEOTOOLBOX_CRASH_DIAGNOSIS.md** - 了解当前问题

---

## 📋 当前状态

### ✅ 已完成

1. **Feature Gate 配置**
   - `hardware-accel` feature 隔离硬件依赖
   - 默认测试不触发 VideoToolbox

2. **Mock 回环测试**
   - 端到端流程验证
   - 数据完整性验证
   - 所有测试通过

3. **项目文件整理**
   - 脚本移至 `scripts/`
   - 文档移至 `docs/testing/`
   - 结构清晰规范

### 🚧 待解决

1. **VideoToolbox FFI 崩溃** (Task #24)
   - 状态: 🔴 待修复
   - 优先级: P1（非阻塞）
   - 见: `docs/testing/VIDEOTOOLBOX_CRASH_DIAGNOSIS.md`

### 🎯 下一步

根据测试规范，推荐两个方向：

**方向 A: 继续修复 VideoToolbox**
```bash
# 运行诊断脚本
./scripts/diagnostics/diagnose-videotoolbox-crash.sh

# 分析 lldb_output.log
# 定位崩溃原因
# 实施修复
```

**方向 B: 先用软件编码器（推荐）**
- 集成 `openh264` 或 `x264`
- 快速推进 Phase 2（网络传输）
- 后续再优化硬件加速

---

## 🔄 标准测试流程

### 开发新功能后

```bash
# 1. 编译检查
./scripts/check-compilation.sh

# 2. 单元测试
./scripts/run-unit-tests.sh

# 3. Mock 回环测试
./scripts/run-local-roundtrip-mock.sh

# 4. 提交代码
git add .
git commit -m "功能描述"
```

### 发布版本前

```bash
# 1. 完整测试流程
./scripts/check-compilation.sh
./scripts/run-unit-tests.sh
./scripts/run-local-roundtrip-mock.sh

# 2. 硬件加速测试（可选）
./scripts/run-local-roundtrip.sh

# 3. 性能测试
cargo bench

# 4. 生成文档
cargo doc --workspace --no-deps
```

---

## 📝 更新日志

### 2026-06-28
- ✅ 创建规范的目录结构
- ✅ 整理测试脚本到 `scripts/`
- ✅ 整理文档到 `docs/testing/`
- ✅ 创建编译检查脚本
- ✅ 创建单元测试脚本
- ✅ 编写测试规范文档

---

## 🤝 贡献指南

### 添加新测试

1. 单元测试 → `crates/*/src/**/*.rs` 的 `#[cfg(test)]` 模块
2. 集成测试 → `crates/*/tests/*.rs`
3. 示例测试 → `crates/*/examples/*.rs`

### 添加新脚本

1. 测试脚本 → `scripts/`
2. 诊断脚本 → `scripts/diagnostics/`
3. 设置可执行权限: `chmod +x scripts/*.sh`

### 更新文档

1. 测试文档 → `docs/testing/`
2. API 文档 → `cargo doc`
3. 用户文档 → `docs/`

---

**维护人**: AI Assistant  
**最后更新**: 2026-06-28
