# WebRTC 编解码迁移 - Phase 0 完成报告

**执行日期**: 2026-06-28  
**任务**: Phase 0 - 清理 libwebrtc 残留依赖  
**状态**: ✅ 已完成  

---

## 📋 执行摘要

成功移除了 RDCS 项目中所有 `libwebrtc` 相关依赖，为迁移到方案 B（webrtc-rs + 平台原生编解码）清理了技术债务。

### 关键指标

- **移除的依赖**: 3 个 Cargo.toml 文件
- **废弃的代码**: 4 个源文件（共 61,045 字节）
- **受影响的 crate**: 2 个（rdcs-codec, rdcs-session）
- **保留的核心模块**: 9 个（adaptive, analyzer, encoder, decoder, pipeline 等）

---

## ✅ 已完成的工作

### 1. 依赖清理

| 文件 | 变更内容 |
|------|---------|
| `Cargo.toml` | 移除 workspace 级别的 `libwebrtc = "0.3"` |
| `crates/rdcs-codec/Cargo.toml` | 移除依赖并添加迁移注释 |
| `crates/rdcs-session/Cargo.toml` | 移除依赖并添加迁移注释 |

### 2. 源文件重构

#### 已废弃文件（重命名为 *.deprecated）

```
crates/rdcs-codec/src/
├── webrtc_encoder.rs.deprecated    ← libwebrtc H.264 编码器封装
├── webrtc_decoder.rs.deprecated    ← libwebrtc H.264 解码器封装  
└── peer_connection.rs.deprecated   ← PeerConnection 生命周期管理

crates/rdcs-session/src/
└── manager.rs.deprecated           ← 依赖 libwebrtc 的会话管理器
```

**为什么不直接删除？**
- 保留作为方案 A 的实现参考
- 保留回滚能力
- 保留状态机设计、错误处理模式等可复用逻辑

#### 更新的模块导出

```rust
// crates/rdcs-codec/src/lib.rs
pub mod adaptive;
pub mod analyzer;
pub mod decoder;
pub mod encoder;
pub mod error;
// pub mod peer_connection;      ← 已禁用
pub mod pipeline;
pub mod platform;
pub mod types;
// pub mod webrtc_decoder;       ← 已禁用
// pub mod webrtc_encoder;       ← 已禁用
```

```rust
// crates/rdcs-session/src/lib.rs
// pub mod manager;              ← 已禁用，待重新实现
```

### 3. 文档更新

新增文档：

1. **[MIGRATION.md](../MIGRATION.md)** - 完整迁移追踪文档
   - Phase 0-4 的详细计划
   - 已移除文件清单
   - 待实现任务列表
   - 回滚计划

2. **[docs/PHASE0_CLEANUP_SUMMARY.md](PHASE0_CLEANUP_SUMMARY.md)** - Phase 0 详细总结
   - 清理操作清单
   - 验证步骤
   - 下一步工作指引

---

## 🏗️ 保留的架构基础

以下模块**未受影响**，继续作为方案 B 的基础：

### rdcs-codec 核心模块

| 模块 | 功能 | 状态 |
|------|------|------|
| `adaptive.rs` | 自适应码率控制 | ✅ 保留 |
| `analyzer.rs` | 内容分析（文本/视频场景） | ✅ 保留 |
| `encoder.rs` | VideoEncoder trait 定义 | ✅ 保留 |
| `decoder.rs` | VideoDecoder trait 定义 | ✅ 保留 |
| `pipeline.rs` | EncodePipeline / DecodePipeline | ✅ 保留 |
| `platform/videotoolbox.rs` | macOS VideoToolbox FFI（399 行） | ✅ 保留 |
| `platform/media_foundation.rs` | Windows Media Foundation | ✅ 保留 |
| `platform/vaapi.rs` | Linux VA-API | ✅ 保留 |
| `types.rs` | 通用类型定义 | ✅ 保留 |

### 其他 crate（完全保留）

- ✅ `rdcs-signaling` - WebSocket 信令服务器
- ✅ `rdcs-connection` - ICE/NAT 穿透
- ✅ `rdcs-transport` - NACK/FEC/拥塞控制
- ✅ `rdcs-platform` - 屏幕捕获
- ✅ `rdcs-crypto` - DTLS/SRTP 加密

---

## 🎯 下一步工作：Phase 1

### 目标

实现 macOS VideoToolbox 真实编解码器，达到以下指标：

- 编码延迟 < 10ms
- 解码延迟 < 10ms  
- CPU 占用 < 15% @ 1080p60fps
- 输出标准 H.264 Annex B 格式

### 待完成任务

#### 1. 补全 VideoToolbox FFI 绑定

```rust
// crates/rdcs-codec/src/platform/videotoolbox.rs

// ✅ 已有（编码器）
extern "C" {
    fn VTCompressionSessionCreate(...);
    fn VTCompressionSessionEncodeFrame(...);
    fn VTCompressionSessionCompleteFrames(...);
}

// ❌ 缺失（解码器）
extern "C" {
    fn VTDecompressionSessionCreate(...);           // 待添加
    fn VTDecompressionSessionDecodeFrame(...);      // 待添加
    fn VTDecompressionSessionFinishDelayedFrames(...); // 待添加
}
```

#### 2. 实现编码器

```rust
impl PlatformEncoder for VideoToolboxEncoder {
    fn encode(&mut self, frame: &CapturedFrame) -> Result<Vec<u8>, CodecError> {
        // 1. BGRA → NV12 像素格式转换
        // 2. 创建 CVPixelBuffer
        // 3. 调用 VTCompressionSessionEncodeFrame
        // 4. 从回调中提取 H.264 NAL units（Annex B 格式）
        // 5. 返回 Vec<u8> (0x00 0x00 0x00 0x01 前缀)
    }
}
```

#### 3. 实现解码器

```rust
impl PlatformDecoder for VideoToolboxDecoder {
    fn decode(&mut self, nal_units: &[u8]) -> Result<DecodedFrame, CodecError> {
        // 1. 解析 H.264 NAL units（Annex B → AVCC 格式转换）
        // 2. 调用 VTDecompressionSessionDecodeFrame
        // 3. 从回调中获取 CVPixelBuffer
        // 4. NV12 → BGRA 像素格式转换
        // 5. 返回 DecodedFrame { data, width, height, stride, pts_us }
    }
}
```

#### 4. 单元测试

```rust
#[test]
fn videotoolbox_encode_decode_roundtrip() {
    let config = EncoderConfig {
        width: 1920,
        height: 1080,
        target_fps: 60,
        target_bitrate_bps: 5_000_000,
        codec: CodecType::H264,
        hardware_accel: true,
        keyframe_interval: 60,
    };
    
    let mut encoder = VideoToolboxEncoder::new(&config).unwrap();
    let mut decoder = VideoToolboxDecoder::new(CodecType::H264).unwrap();
    
    // 创建测试帧（1920x1080 BGRA）
    let frame = create_test_frame(1920, 1080);
    
    // 编码
    let encoded = encoder.encode(&frame).unwrap();
    assert!(!encoded.is_empty());
    assert!(encoded.starts_with(&[0x00, 0x00, 0x00, 0x01])); // Annex B
    
    // 解码
    let decoded = decoder.decode(&encoded).unwrap();
    assert_eq!(decoded.width, 1920);
    assert_eq!(decoded.height, 1080);
    
    // 验证像素数据相似度（允许有损压缩误差）
    let psnr = calculate_psnr(&frame.data, &decoded.data);
    assert!(psnr > 30.0, "PSNR too low: {}", psnr);
}

#[test]
fn videotoolbox_encoder_performance() {
    let mut encoder = VideoToolboxEncoder::new(&config).unwrap();
    let frame = create_test_frame(1920, 1080);
    
    let start = Instant::now();
    for _ in 0..100 {
        encoder.encode(&frame).unwrap();
    }
    let elapsed = start.elapsed();
    
    let avg_encode_time = elapsed / 100;
    assert!(avg_encode_time < Duration::from_millis(10), 
            "编码延迟过高: {:?}", avg_encode_time);
}
```

---

## 📊 迁移进度

```
✅ Phase 0: 清理 libwebrtc 残留依赖              [████████████████████] 100%
🚧 Phase 1: macOS VideoToolbox 真实编解码器      [░░░░░░░░░░░░░░░░░░░░]   0%
⬜ Phase 2: RTP/SRTP 集成层                      [░░░░░░░░░░░░░░░░░░░░]   0%
⬜ Phase 3: 与 rdcs-connection/signaling/transport 对接  [░░░░░░░░░░░░░░░░░░░░]   0%
⬜ Phase 4: 端到端集成测试与验证                 [░░░░░░░░░░░░░░░░░░░░]   0%
```

**总体进度**: 20%（Phase 0 占 1/5 权重）

---

## 🔗 相关文档

- **决策文档**: [docs/decisions/WEBRTC_CODEC_INTEGRATION_DECISION.md](decisions/WEBRTC_CODEC_INTEGRATION_DECISION.md)
- **迁移追踪**: [MIGRATION.md](../MIGRATION.md)
- **Phase 0 详细总结**: [docs/PHASE0_CLEANUP_SUMMARY.md](PHASE0_CLEANUP_SUMMARY.md)

---

## ✅ 验证清单

- [x] 所有 `libwebrtc` 依赖已从 Cargo.toml 移除
- [x] 依赖 libwebrtc 的源文件已重命名为 `*.deprecated`
- [x] 模块导出已更新（禁用废弃模块）
- [x] 核心模块（adaptive, analyzer, encoder, decoder, pipeline）保持完整
- [x] 平台原生编解码器基础代码（videotoolbox.rs 399 行）保留
- [x] 迁移文档已创建（MIGRATION.md + PHASE0_CLEANUP_SUMMARY.md）
- [x] Phase 1 任务清单已明确

---

**完成时间**: 2026-06-28  
**下一阶段**: Phase 1 - macOS VideoToolbox 真实编解码器实现
