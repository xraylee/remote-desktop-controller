# RDCS 测试流程（Mock Simulator）

**日期**: 2026-06-27  
**目标**: 验证使用 Mock Simulator 的项目各模块功能

---

## 📋 测试范围

### Phase 1: 编译验证 ✅
- [x] rdcs-codec 编译通过
- [x] 完整 workspace 编译通过
- [x] 依赖解析正确

### Phase 2: 单元测试
- [ ] rdcs-codec 单元测试
- [ ] rdcs-platform 单元测试
- [ ] rdcs-connection 单元测试
- [ ] rdcs-signaling 单元测试
- [ ] rdcs-transport 单元测试

### Phase 3: 集成测试
- [ ] 编解码端到端测试
- [ ] 网络层集成测试
- [ ] 完整流程测试

### Phase 4: 其他模块验证
- [ ] Go API 服务编译
- [ ] Flutter 客户端编译
- [ ] Web 管理后台编译

---

## 🧪 测试步骤

### 步骤 1: 编译验证

```bash
# 1.1 清理环境
cargo clean
rm -f Cargo.lock

# 1.2 编译 rdcs-codec
cargo check -p rdcs-codec

# 1.3 编译完整 workspace
cargo check --workspace

# 1.4 编译 release 版本
cargo build --release -p rdcs-codec
```

**预期结果**: 所有编译通过，无错误

---

### 步骤 2: 运行单元测试

```bash
# 2.1 rdcs-codec 测试
cargo test -p rdcs-codec --lib

# 2.2 rdcs-codec 测试（详细输出）
cargo test -p rdcs-codec --lib -- --nocapture

# 2.3 查看测试覆盖
cargo test -p rdcs-codec --lib -- --test-threads=1 --nocapture

# 2.4 运行特定测试
cargo test -p rdcs-codec test_encoder_basic
cargo test -p rdcs-codec test_decoder_basic
cargo test -p rdcs-codec test_codec_roundtrip
```

**预期结果**: 
- 所有测试通过（使用 Mock Simulator）
- 性能指标在预期范围内
- 日志输出正常

---

### 步骤 3: 检查测试输出

```bash
# 3.1 运行测试并保存日志
cargo test -p rdcs-codec --lib 2>&1 | tee test-output.log

# 3.2 检查是否有警告
cargo test -p rdcs-codec --lib 2>&1 | grep -i "warning"

# 3.3 检查是否有失败
cargo test -p rdcs-codec --lib 2>&1 | grep -i "failed"
```

**预期结果**: 无警告，无失败

---

### 步骤 4: 集成测试

```bash
# 4.1 运行集成测试
cargo test -p rdcs-codec --test '*'

# 4.2 运行编解码端到端测试
cargo test -p rdcs-codec codec_integration_test

# 4.3 长时间压力测试（可选）
cargo test -p rdcs-codec test_long_session -- --ignored
```

**预期结果**: 
- 编解码往返成功
- 数据完整性验证通过
- 性能指标符合 Mock 预期

---

### 步骤 5: 其他模块验证

```bash
# 5.1 检查 rdcs-platform
cargo check -p rdcs-platform
cargo test -p rdcs-platform

# 5.2 检查 rdcs-connection
cargo check -p rdcs-connection
cargo test -p rdcs-connection

# 5.3 检查 rdcs-signaling
cargo check -p rdcs-signaling

# 5.4 检查 rdcs-transport
cargo check -p rdcs-transport
```

**预期结果**: 所有模块编译通过

---

### 步骤 6: Go API 服务

```bash
cd api
go mod tidy
go build ./...
go test ./...
```

**预期结果**: Go 服务编译和测试通过

---

### 步骤 7: Flutter 客户端

```bash
cd client
flutter pub get
flutter analyze
flutter test
```

**预期结果**: Flutter 客户端编译和测试通过

---

### 步骤 8: Web 管理后台

```bash
cd web/admin
npm install
npm run build
npm test
```

**预期结果**: Web 后台编译和测试通过

---

## 📊 测试记录

### 编译验证结果

| 模块 | 状态 | 时间 | 备注 |
|------|------|------|------|
| rdcs-codec | ⏳ | - | 等待测试 |
| rdcs-platform | ⏳ | - | 等待测试 |
| rdcs-connection | ⏳ | - | 等待测试 |
| rdcs-signaling | ⏳ | - | 等待测试 |
| rdcs-transport | ⏳ | - | 等待测试 |
| Workspace | ⏳ | - | 等待测试 |

### 单元测试结果

| 模块 | 总数 | 通过 | 失败 | 忽略 | 备注 |
|------|------|------|------|------|------|
| rdcs-codec | ? | ? | ? | ? | 等待测试 |
| rdcs-platform | ? | ? | ? | ? | 等待测试 |
| rdcs-connection | ? | ? | ? | ? | 等待测试 |

### 集成测试结果

| 测试场景 | 状态 | 时间 | 备注 |
|----------|------|------|------|
| 编解码往返 | ⏳ | - | 等待测试 |
| 多帧序列 | ⏳ | - | 等待测试 |
| 性能基准 | ⏳ | - | 等待测试 |

---

## 🎯 成功标准

### 必须通过 ✅
1. **编译**: 所有 Rust crates 编译通过
2. **单元测试**: rdcs-codec 测试全部通过
3. **Mock 验证**: Mock Simulator 能模拟编解码流程

### 可以失败 ⚠️
1. **性能测试**: Mock 性能指标不代表真实性能
2. **硬件加速**: Mock 不测试硬件加速
3. **真实画面**: Mock 不产生真实视频

---

## 🚨 故障排查

### 编译失败
```bash
# 清理并重试
cargo clean
rm -f Cargo.lock
cargo check --workspace
```

### 测试失败
```bash
# 查看详细日志
RUST_LOG=trace cargo test -p rdcs-codec -- --nocapture

# 运行单个测试
cargo test -p rdcs-codec test_encoder_basic -- --nocapture
```

### 依赖问题
```bash
# 更新依赖
cargo update

# 验证依赖树
cargo tree -p rdcs-codec
```

---

## 📝 测试完成后

### 生成测试报告
```bash
# 运行所有测试并生成报告
cargo test --workspace 2>&1 | tee full-test-report.log

# 统计测试数量
grep -E "test result:" full-test-report.log
```

### 更新文档
- [ ] 更新 `SESSION_REVIEW.md`
- [ ] 记录测试结果
- [ ] 标记通过的模块

---

**开始测试**: 运行 `./verify-mock-simulator.sh`
