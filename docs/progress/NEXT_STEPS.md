# 🎯 RDCS 下一步执行计划

**更新时间**: 2026-06-27  
**当前阶段**: WebRTC 集成实施  
**项目完成度**: 90% → 开始最后 10%

---

## ✅ 今日已完成

1. **WebRTC 架构决策** ✅
   - 完成三个方案深度对比
   - 确定使用 livekit/webrtc-sys
   - 编写架构决策文档：`docs/decisions/WEBRTC_ARCHITECTURE.md`

2. **文档同步更新** ✅
   - 更新 `codec-integration-status.md`：记录方案决策和集成路径
   - 更新 `real-environment-integration-plan.md`：调整时间表和风险评估
   - 创建 `NEXT_STEPS.md`：明确接下来的执行步骤

3. **风险缓解** ✅
   - WebRTC 性能风险从 🔴 高降为 🟢 低
   - 跨平台复用架构约束已满足
   - 有 RustDesk 成功案例背书

---

## 🚀 立即开始：livekit/webrtc-sys 集成

### 第 1 步：添加依赖（预计 30 分钟）

**操作**:
```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 添加 livekit 依赖
cargo add livekit --version 0.5

# 验证编译
cargo check --workspace
```

**预期结果**:
- `Cargo.toml` 中添加 `livekit = "0.5"`
- 依赖下载成功（使用 rsproxy.cn 镜像加速）
- 编译无错误

**验证命令**:
```bash
cargo tree | grep livekit
# 应显示 livekit 及其依赖树
```

---

### 第 2 步：替换编码器（预计 1-2 天）

**目标文件**: `crates/rdcs-codec/src/webrtc_encoder.rs`

**当前实现**（使用 simulator）:
```rust
pub struct WebRtcEncoder {
    config: EncoderConfig,
    frame_count: u64,
    metrics: EncoderMetrics,
}
```

**目标实现**（使用 livekit）:
```rust
use livekit::webrtc::{VideoEncoder, VideoFrame, VideoCodec};

pub struct WebRtcEncoder {
    inner: VideoEncoder,  // LiveKit 真实编码器
    config: EncoderConfig,
    frame_count: u64,
    metrics: EncoderMetrics,  // 保持现有指标接口
}

impl WebRtcEncoder {
    pub fn new(config: EncoderConfig) -> Result<Self> {
        // 配置编码器
        let codec = VideoCodec::H264;
        let inner = VideoEncoder::new(codec, &config)?;
        
        Ok(Self {
            inner,
            config,
            frame_count: 0,
            metrics: EncoderMetrics::default(),
        })
    }

    pub fn encode(&mut self, frame: &FrameBuffer) -> Result<EncodedData> {
        let start = Instant::now();
        
        // 转换帧格式
        let video_frame = VideoFrame::from_rgba(
            frame.width,
            frame.height,
            frame.data,
        )?;
        
        // 真实编码
        let encoded = self.inner.encode(&video_frame)?;
        
        // 更新性能指标
        let elapsed = start.elapsed().as_micros() as u64;
        self.metrics.update(elapsed, encoded.len() as u64);
        self.frame_count += 1;
        
        Ok(encoded.into())
    }
}
```

**验证测试**:
```bash
cargo test --package rdcs-codec --lib webrtc_encoder
```

---

### 第 3 步：替换解码器（预计 1-2 天）

**目标文件**: `crates/rdcs-codec/src/webrtc_decoder.rs`

**目标实现**:
```rust
use livekit::webrtc::{VideoDecoder, VideoCodec};

pub struct WebRtcDecoder {
    inner: VideoDecoder,
    metrics: DecoderMetrics,
}

impl WebRtcDecoder {
    pub fn new(codec: VideoCodec) -> Result<Self> {
        let inner = VideoDecoder::new(codec)?;
        Ok(Self {
            inner,
            metrics: DecoderMetrics::default(),
        })
    }

    pub fn decode(&mut self, data: &EncodedData) -> Result<FrameBuffer> {
        let start = Instant::now();
        
        // 真实解码
        let frame = self.inner.decode(data.as_ref())?;
        
        // 转换为 FrameBuffer
        let buffer = FrameBuffer::from_video_frame(&frame)?;
        
        // 更新指标
        let elapsed = start.elapsed().as_micros() as u64;
        self.metrics.update(elapsed, data.len() as u64);
        
        Ok(buffer)
    }
}
```

**验证测试**:
```bash
cargo test --package rdcs-codec --lib webrtc_decoder
```

---

### 第 4 步：集成测试（预计 0.5 天）

**运行完整测试套件**:
```bash
# 单元测试
cargo test --package rdcs-codec

# 集成测试
cargo test --test codec_integration_test

# 性能测试（release 模式）
cargo test --release test_encoding_performance_1080p_60fps -- --nocapture
```

**预期结果**:
- ✅ 所有 11 个编解码器测试通过
- ✅ 编码时间 < 5ms（平均）
- ✅ 解码时间 < 3ms（平均）
- ✅ 压缩率 > 50:1

**性能基准**:
```
test_encode_decode_roundtrip_single_frame ... ok
  ✓ 1920x1080 BGRA → H.264 → BGRA
  Original: 8,294,400 bytes
  Encoded:  ~80,000 bytes (压缩率 ~100:1)
  Decoded:  8,294,400 bytes

test_encoding_performance_1080p_60fps ... ok
  ✓ 120 frames @ 1080p60
  Average FPS: 60+
  Average encode time: 2-4 ms
  Estimated CPU: 15-25%
  ✓ Meets PRD requirements
```

---

### 第 5 步：硬件加速验证（预计 0.5 天）

**macOS VideoToolbox 验证**:

```bash
# 运行编解码器测试，开启详细日志
RUST_LOG=rdcs_codec=debug cargo test --release test_encoding_performance_1080p_60fps -- --nocapture
```

**检查要点**:
1. 日志中应出现 VideoToolbox 相关信息
2. CPU 占用应显著低于软件编码（<25% vs >50%）
3. 编码延迟应稳定在 2-5ms

**性能对比**:
| 指标 | Mock Simulator | livekit (软件) | livekit (硬件) | 目标 |
|------|----------------|----------------|----------------|------|
| CPU占用 | 模拟值 | >50% | <20% | <30% |
| 编码延迟 | 模拟值 | 10-15ms | 2-5ms | <10ms |
| 解码延迟 | 模拟值 | 5-8ms | 1-3ms | <5ms |
| 压缩率 | 模拟值 | 80:1 | 100:1 | >50:1 |

---

## 📋 本周剩余任务

### Day 1（今天）
- [x] 完成 WebRTC 方案决策
- [x] 更新所有相关文档
- [ ] 添加 livekit 依赖
- [ ] 验证依赖编译成功
- [ ] 阅读 LiveKit 文档

### Day 2
- [ ] 开始替换 WebRtcEncoder
- [ ] 实现基础编码逻辑
- [ ] 运行单元测试

### Day 3
- [ ] 完成 WebRtcEncoder 替换
- [ ] 开始替换 WebRtcDecoder
- [ ] 实现基础解码逻辑

### Day 4
- [ ] 完成 WebRtcDecoder 替换
- [ ] 运行所有集成测试
- [ ] 硬件加速验证

### Day 5
- [ ] 性能测试和优化
- [ ] 编写集成报告
- [ ] 准备 Flutter 渲染接口

---

## 🎯 验收标准

### 功能完整性
- [ ] livekit 依赖成功集成
- [ ] WebRtcEncoder 替换完成
- [ ] WebRtcDecoder 替换完成
- [ ] 所有现有测试通过（11个）
- [ ] 性能指标接口保持不变

### 性能达标
- [ ] 编码延迟 < 5ms（平均）
- [ ] 解码延迟 < 3ms（平均）
- [ ] CPU 占用 < 25%（1080p60）
- [ ] 压缩率 > 80:1
- [ ] 60 FPS 无丢帧

### 质量保障
- [ ] 单元测试覆盖率保持 >90%
- [ ] 无编译警告
- [ ] 无内存泄漏
- [ ] 日志清晰完整

---

## 📚 参考资料

### 官方文档
- LiveKit Rust SDK: https://github.com/livekit/rust-sdks
- LiveKit 文档: https://docs.livekit.io/rust/
- livekit-webrtc API: https://docs.rs/livekit-webrtc/

### 成功案例
- RustDesk: https://github.com/rustdesk/rustdesk
  - 查看 `src/codec/` 目录了解 libwebrtc 集成方式

### 内部文档
- 架构决策: `docs/decisions/WEBRTC_ARCHITECTURE.md`
- 编解码器状态: `docs/progress/codec-integration-status.md`
- 集成计划: `docs/progress/real-environment-integration-plan.md`

---

## 💡 技巧提示

### 调试技巧
```bash
# 1. 详细日志
RUST_LOG=rdcs_codec=trace cargo test -- --nocapture

# 2. 单个测试
cargo test test_encode_decode_roundtrip_single_frame -- --nocapture

# 3. 性能分析
cargo test --release test_encoding_performance -- --nocapture

# 4. 编译检查
cargo check --workspace --all-features
```

### 常见问题

**Q: livekit 依赖下载慢？**
```bash
# 确认 Rust 镜像已配置
cat .cargo/config.toml
# 应显示 rsproxy.cn
```

**Q: 编译错误？**
```bash
# 清理并重新编译
cargo clean
cargo build --package rdcs-codec
```

**Q: 测试失败？**
```bash
# 逐步验证
cargo test --package rdcs-codec --lib  # 先跑单元测试
cargo test --test codec_integration_test  # 再跑集成测试
```

---

## 🚦 里程碑

### 里程碑 1: 依赖集成（Day 1）
- [ ] livekit 依赖添加成功
- [ ] 编译通过
- [ ] 文档阅读完成

### 里程碑 2: 编码器替换（Day 2-3）
- [ ] WebRtcEncoder 实现完成
- [ ] 编码器单元测试通过
- [ ] 基础编码功能验证

### 里程碑 3: 解码器替换（Day 3-4）
- [ ] WebRtcDecoder 实现完成
- [ ] 解码器单元测试通过
- [ ] 端到端编解码验证

### 里程碑 4: 集成验证（Day 4-5）
- [ ] 所有测试通过
- [ ] 性能达标
- [ ] 硬件加速验证
- [ ] 文档更新完成

---

## ✅ 成功标志

**当以下命令全部成功时，集成完成**:

```bash
# 1. 编译成功
cargo build --workspace --release

# 2. 测试全通过
cargo test --workspace

# 3. 性能达标
cargo test --release test_encoding_performance_1080p_60fps -- --nocapture
# 输出应显示: "✓ Meets PRD requirements"

# 4. 无警告
cargo clippy --workspace -- -D warnings

# 5. 格式检查
cargo fmt --check
```

---

**下一步**: 立即开始第 1 步 —— 添加 livekit 依赖

**预计完成时间**: 本周五（5 个工作日）

**信心指数**: 🟢 高（方案成熟，文档完善，有成功案例）
