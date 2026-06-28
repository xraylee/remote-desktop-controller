# RDCS 测试规范流程

**版本**: 1.0  
**日期**: 2026-06-28

---

## 📋 目录

1. [测试分类](#测试分类)
2. [测试环境](#测试环境)
3. [测试流程](#测试流程)
4. [文件组织](#文件组织)
5. [问题诊断流程](#问题诊断流程)

---

## 测试分类

### 1. 单元测试 (Unit Tests)

**目的**: 测试单个函数/模块的正确性

**位置**: `crates/*/src/**/*.rs` 中的 `#[cfg(test)]` 模块

**运行**:
```bash
# 测试单个 crate
cargo test -p rdcs-codec --lib

# 测试所有 crates
cargo test --workspace --lib
```

**规范**:
- ✅ 使用 Mock 依赖
- ✅ 快速执行（< 1秒）
- ✅ 不依赖外部资源
- ✅ 可并行运行

### 2. 集成测试 (Integration Tests)

**目的**: 测试多个模块协同工作

**位置**: `crates/*/tests/*.rs`

**运行**:
```bash
# 运行集成测试
cargo test -p rdcs-codec --test '*'

# 运行特定集成测试
cargo test -p rdcs-codec --test encoder_integration
```

**规范**:
- ✅ 测试真实的模块交互
- ✅ 可使用 Mock 或真实依赖
- ✅ 执行时间 < 10秒

### 3. 示例测试 (Example Tests)

**目的**: 端到端验证完整功能

**位置**: `crates/*/examples/*.rs`

**运行**:
```bash
# 运行 example
cargo run -p rdcs-codec --example local_roundtrip_mock
```

**规范**:
- ✅ 演示真实使用场景
- ✅ 可作为文档参考
- ✅ 提供详细输出

### 4. 性能测试 (Benchmark Tests)

**目的**: 测量性能指标

**位置**: `benches/*.rs`

**运行**:
```bash
cargo bench
```

**规范**:
- ✅ 测量关键路径性能
- ✅ 与基准对比
- ✅ 生成性能报告

---

## 测试环境

### Mock 环境

**使用场景**:
- 单元测试
- CI/CD 流水线
- 无硬件依赖的测试

**特点**:
- ✅ 快速
- ✅ 稳定
- ✅ 可预测

**配置**:
```bash
# 默认使用 Mock
cargo test -p rdcs-codec
```

### 硬件加速环境

**使用场景**:
- 性能验证
- 真实场景测试
- 手动测试

**特点**:
- ⚠️ 需要硬件支持
- ⚠️ 需要系统权限
- ⚠️ 可能不稳定

**配置**:
```bash
# 启用硬件加速
cargo test -p rdcs-codec --features hardware-accel
```

---

## 测试流程

### Phase 1: 编译验证

**目标**: 确保代码可以编译

```bash
# 检查所有 crates
./scripts/check-compilation.sh
```

**验收标准**:
- [ ] 所有 crates 编译通过
- [ ] 无编译警告（或已确认可忽略）
- [ ] 依赖解析正确

### Phase 2: 单元测试

**目标**: 验证各模块功能正确

```bash
# 运行所有单元测试
./scripts/run-unit-tests.sh
```

**验收标准**:
- [ ] 所有单元测试通过
- [ ] 代码覆盖率 > 70%
- [ ] 无测试跳过（除非有明确原因）

### Phase 3: 集成测试

**目标**: 验证模块协同工作

```bash
# 运行集成测试
./scripts/run-integration-tests.sh
```

**验收标准**:
- [ ] 关键路径测试通过
- [ ] Mock 集成测试通过
- [ ] 硬件测试通过（可选）

### Phase 4: 端到端测试

**目标**: 验证完整功能流程

```bash
# 运行端到端测试
./scripts/run-e2e-tests.sh
```

**验收标准**:
- [ ] MVP 场景可运行
- [ ] 性能指标达标
- [ ] 用户体验流畅

---

## 文件组织

### 测试脚本目录

```
scripts/
├── check-compilation.sh          # 编译检查
├── run-unit-tests.sh             # 单元测试
├── run-integration-tests.sh      # 集成测试
├── run-e2e-tests.sh              # 端到端测试
├── test-hardware-accel-gate.sh   # Feature gate 测试
├── run-local-roundtrip-mock.sh   # Mock 回环测试
└── run-local-roundtrip.sh        # 硬件加速回环测试
```

### 诊断脚本目录

```
scripts/diagnostics/
├── diagnose-videotoolbox-crash.sh  # VideoToolbox 崩溃诊断
├── diagnose-network-issues.sh      # 网络问题诊断
└── diagnose-performance.sh         # 性能问题诊断
```

### 文档目录

```
docs/testing/
├── TESTING_GUIDE.md                # 测试指南
├── PHASE1_COMPLETION_REPORT.md     # Phase 1 报告
├── VIDEOTOOLBOX_CRASH_DIAGNOSIS.md # VideoToolbox 诊断
└── TEST_RESULTS.md                 # 测试结果记录
```

---

## 问题诊断流程

### 1. 编译失败

**步骤**:
1. 清理构建缓存: `cargo clean`
2. 更新依赖: `cargo update`
3. 检查 Rust 版本: `rustc --version`
4. 查看完整错误: `cargo build 2>&1 | tee build.log`

**常见原因**:
- 依赖版本冲突
- 缺少系统库
- Rust 版本不兼容

### 2. 测试失败

**步骤**:
1. 单独运行失败的测试: `cargo test <test_name> -- --nocapture`
2. 启用日志: `RUST_LOG=debug cargo test`
3. 检查测试环境: Mock vs 硬件
4. 查看测试代码是否有假设

**常见原因**:
- Mock 配置不正确
- 测试数据不完整
- 环境依赖缺失

### 3. 运行时崩溃 (SIGSEGV)

**步骤**:
1. 使用调试器: `lldb target/debug/example`
2. 启用 backtrace: `RUST_BACKTRACE=full`
3. 检查 FFI 边界
4. 验证内存安全

**常见原因**:
- 空指针解引用
- 数组越界访问
- FFI 调用错误
- 线程安全问题

### 4. 性能问题

**步骤**:
1. 使用 profiler: `cargo flamegraph`
2. 检查关键路径
3. 测量各阶段耗时
4. 对比预期性能

**常见原因**:
- 算法复杂度高
- 不必要的内存拷贝
- 锁竞争
- I/O 阻塞

---

## 测试报告模板

### 测试执行报告

```markdown
# 测试执行报告

**日期**: YYYY-MM-DD
**测试人**: 
**版本**: 

## 测试环境

- 操作系统: 
- Rust 版本: 
- 硬件: 

## 测试结果

### 编译检查
- [ ] 通过
- [ ] 失败: 

### 单元测试
- [ ] 通过 (X/Y)
- [ ] 失败: 

### 集成测试
- [ ] 通过 (X/Y)
- [ ] 失败: 

### 端到端测试
- [ ] 通过
- [ ] 失败: 

## 问题记录

| ID | 问题描述 | 严重程度 | 状态 |
|----|---------|---------|------|
| 1  |         | P0/P1/P2 | 开放/修复中/已修复 |

## 性能指标

| 指标 | 目标 | 实际 | 状态 |
|-----|------|------|------|
| 编码延迟 | < 50ms | X ms | ✅/❌ |
| 解码延迟 | < 50ms | X ms | ✅/❌ |

## 建议

1. 
2. 
```

---

## 最佳实践

### DO ✅

1. **测试前先编译检查**
   ```bash
   cargo check --workspace
   ```

2. **使用 Mock 优先**
   ```bash
   cargo test  # 默认 Mock
   ```

3. **记录测试结果**
   - 保存日志文件
   - 更新测试文档

4. **渐进式测试**
   - 先单元测试
   - 再集成测试
   - 最后端到端测试

5. **隔离问题**
   - 单独测试失败的模块
   - 创建最小复现用例

### DON'T ❌

1. **不要跳过失败测试**
   - 必须修复或明确标记 `#[ignore]`

2. **不要在 CI 中运行硬件测试**
   - 硬件测试仅在本地手动运行

3. **不要忽略警告**
   - 警告可能隐藏潜在问题

4. **不要混合 Mock 和真实依赖**
   - 明确区分测试环境

5. **不要在测试中使用随机数**
   - 使用固定种子确保可重复性

---

## 附录

### A. 测试覆盖率

使用 `tarpaulin` 生成覆盖率报告:

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --workspace --out Html
```

### B. 性能基准

使用 `criterion` 运行基准测试:

```bash
cargo bench --workspace
```

### C. 内存检查

使用 `valgrind` (Linux) 或 `instruments` (macOS):

```bash
# macOS
instruments -t Leaks target/debug/example

# Linux
valgrind --leak-check=full target/debug/example
```

---

**维护**: 随测试需求更新  
**审核**: 每次重大变更后审核
