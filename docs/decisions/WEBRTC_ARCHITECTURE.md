# WebRTC 架构决策文档

**决策日期**: 2026-06-27  
**决策者**: 核心开发团队  
**状态**: ✅ 已确认

---

## 背景

RDCS 项目需要实现跨平台的视频编解码能力，支持 macOS、Windows、Linux 三个平台。当前使用 mock simulator (`WebRtcEncoderSimulator` / `WebRtcDecoderSimulator`) 作为占位实现，需要选择真实的 WebRTC 库进行集成。

### 关键约束

1. **跨平台复用**: "这些核心能力需要考虑跨平台共用" —— 编解码器核心逻辑必须在三个平台复用，避免重复开发
2. **硬件加速**: CPU 占用 < 30%（PRD 要求），必须使用硬件编解码
3. **生产级质量**: 需要稳定可靠的编解码质量
4. **维护成本**: 优先选择有持续维护的方案

---

## 方案对比

### 方案 A: livekit/webrtc-sys（libwebrtc FFI 封装）⭐ **已选择**

**架构图**:
```
rdcs-codec (跨平台 Rust 抽象层)
    ↓
livekit::VideoEncoder / VideoDecoder (统一 Rust API)
    ↓
libwebrtc (Google C++ 实现)
    ↓
macOS VideoToolbox / Windows DXVA / Linux VA-API
```

**技术细节**:
- **库**: https://github.com/livekit/rust-sdks (livekit-webrtc crate)
- **底层引擎**: Google libwebrtc（Chrome/WebRTC 同款）
- **预编译库大小**: ~80MB（包含所有平台）
- **硬件加速**: ✅ 全平台支持（libwebrtc 已实现）

**优点**:
- ✅ **一份 Rust 代码跨三平台** —— 满足"跨平台复用"核心约束
- ✅ **硬件加速开箱即用** —— libwebrtc 已封装 VideoToolbox/MF/VA-API
- ✅ **生产级质量** —— Google WebRTC 标准实现，Chrome 同款引擎
- ✅ **持续维护** —— LiveKit 商业公司支持，定期更新
- ✅ **行业验证** —— RustDesk（最成功的开源 Rust 远程桌面）使用同方案

**缺点**:
- ⚠️ 预编译库体积较大（~80MB），但对桌面应用可接受
- ⚠️ 首次接入需理解 FFI 层，但 LiveKit 文档完善

**性能预期**:
- CPU 占用: <20%（硬件加速）
- 延迟: 8-15ms（编码）+ 5-10ms（解码）
- 质量: 与 Chrome WebRTC 一致

---

### 方案 B: webrtc-rs（纯 Rust 实现）❌ **已排除**

**架构图**:
```
rdcs-codec
    ↓
webrtc-rs::VideoEncoder (纯软件编解码)
```

**优点**:
- ✅ 纯 Rust，无 C++ 依赖
- ✅ Cargo 直接引入

**缺点（致命）**:
- ❌ **无硬件加速** —— 纯软件编解码，CPU 占用 >50%，不满足 PRD
- ❌ 编解码质量不如 libwebrtc
- ❌ 维护活跃度下降（最近更新频率降低）

**排除原因**: 无法满足 CPU < 30% 的性能要求。

---

### 方案 C: 平台原生各自实现 ❌ **已排除**

**架构图**:
```
rdcs-codec (trait 抽象)
    ↓
rdcs-macos → VideoToolbox 实现
rdcs-windows → Media Foundation 实现
rdcs-linux → VA-API 实现
```

**优点**:
- ✅ 每个平台性能最优

**缺点（致命）**:
- ❌ **违背"跨平台复用"约束** —— 编解码核心逻辑重复 3 份
- ❌ 开发成本 3x（每个平台单独开发）
- ❌ 测试成本 3x（三套测试）
- ❌ 维护成本 3x（bug 修 3 次，新功能开发 3 次）

**排除原因**: 违背项目已有的架构设计原则（`rdcs-codec` 作为跨平台抽象层）。

---

## 最终决策：方案 A（livekit SDK）

### 决策理由

经过全面调研和验证（详见 `WEBRTC_SOLUTION_COMPARISON.md`），确认：

1. **libwebrtc 独立 crate 不存在** ❌
   - Rust 生态中没有维护良好的 libwebrtc 轻量级绑定
   - 自建 FFI 需要 2-4 周工作量

2. **webrtc-rs 无硬件加速** ❌
   - 纯软件编解码，CPU >50%
   - 不满足 PRD 的 <30% 要求

3. **livekit 虽重但可靠** ✅
   - 唯一满足所有技术要求的现成方案
   - 生产级质量，LiveKit 商业公司支持
   - RustDesk 等项目验证可行

4. **依赖较重但可接受** ⚠️
   - ~100+ crates，包含信令/房间功能（虽然不用）
   - 预编译库 ~80MB
   - 桌面应用可接受的代价

### 符合现有架构

当前项目架构：
```
rdcs-codec (跨平台抽象)
    ↓
rdcs-platform (trait 定义)
    ↓
rdcs-macos / rdcs-windows / rdcs-linux (平台特定实现)
```

使用 livekit/webrtc-sys 后：
```
rdcs-codec (跨平台抽象)
    ↓
livekit::VideoEncoder (跨平台统一实现)
    ↓
libwebrtc (自动调用平台硬件加速)
```

**架构改进**：编解码器不再需要平台特定实现，`rdcs-platform` 层专注于其他平台差异（如屏幕捕获、输入事件）。

---

## 集成路径

### 第一阶段：基础集成（3-5 天）

**目标**: 替换 mock simulator，跑通第一帧编解码

**步骤**:

1. **添加依赖** (0.5 天)
   ```toml
   # Cargo.toml
   [workspace.dependencies]
   livekit = "0.5"  # 最新版本
   ```

2. **修改 `rdcs-codec/src/webrtc_encoder.rs`** (1-2 天)
   ```rust
   use livekit::webrtc::{VideoEncoder, VideoFrame, VideoCodec};

   pub struct WebRtcEncoder {
       inner: VideoEncoder,  // LiveKit 提供
       config: EncoderConfig,
   }

   impl WebRtcEncoder {
       pub fn new(config: EncoderConfig) -> Result<Self> {
           let codec = VideoCodec::H264;  // 或 VP8/VP9
           let inner = VideoEncoder::new(codec, config)?;
           Ok(Self { inner, config })
       }

       pub fn encode(&mut self, frame: &FrameBuffer) -> Result<EncodedData> {
           let video_frame = VideoFrame::from_rgba(
               frame.width,
               frame.height,
               frame.data,
           )?;
           self.inner.encode(&video_frame)
       }
   }
   ```

3. **修改 `rdcs-codec/src/webrtc_decoder.rs`** (1-2 天)
   ```rust
   use livekit::webrtc::{VideoDecoder, VideoCodec};

   pub struct WebRtcDecoder {
       inner: VideoDecoder,
   }

   impl WebRtcDecoder {
       pub fn new(codec: VideoCodec) -> Result<Self> {
           let inner = VideoDecoder::new(codec)?;
           Ok(Self { inner })
       }

       pub fn decode(&mut self, data: &EncodedData) -> Result<FrameBuffer> {
           let frame = self.inner.decode(data)?;
           Ok(FrameBuffer::from_video_frame(&frame))
       }
   }
   ```

4. **编写集成测试** (1 天)
   ```rust
   #[test]
   fn test_encode_decode_roundtrip() {
       let config = EncoderConfig::default();
       let mut encoder = WebRtcEncoder::new(config).unwrap();
       let mut decoder = WebRtcDecoder::new(VideoCodec::H264).unwrap();

       let input = create_test_frame(1920, 1080);
       let encoded = encoder.encode(&input).unwrap();
       let decoded = decoder.decode(&encoded).unwrap();

       assert_frame_similarity(&input, &decoded, 0.95);
   }
   ```

5. **验证** (0.5 天)
   - 跑通单元测试
   - 验证硬件加速已启用
   - 测量编解码延迟和 CPU 占用

### 第二阶段：Flutter 集成（3-4 天）

参考 `docs/progress/real-environment-integration-plan.md` Week 1 计划。

---

## 风险评估

### 风险 1: FFI 层复杂度

**风险等级**: 🟡 中  
**缓解措施**:
- LiveKit 文档完善，有示例代码
- Rust FFI 绑定已封装好，无需直接操作 C++ 层
- 可参考 RustDesk 的实现

### 风险 2: 预编译库体积

**风险等级**: 🟢 低  
**影响**: 安装包增加 ~80MB  
**评估**: 对桌面应用可接受（Chrome、Zoom 等都包含 libwebrtc）

### 风险 3: 调试难度

**风险等级**: 🟡 中  
**缓解措施**:
- libwebrtc 有详细日志
- livekit 提供调试工具
- 可逐步验证（先测试静态图像编解码）

---

## 参考资料

### 官方文档
- LiveKit Rust SDK: https://github.com/livekit/rust-sdks
- livekit-webrtc crate: https://docs.rs/livekit-webrtc

### 成功案例
- **RustDesk**: https://github.com/rustdesk/rustdesk
  - 100k+ stars，最成功的开源 Rust 远程桌面
  - 使用 libwebrtc 的 Rust 封装
  - 支持 macOS/Windows/Linux 硬件加速

### 技术对比
- WebRTC 编解码性能对比: https://webrtc.org/architecture/
- libwebrtc 硬件加速文档: https://chromium.googlesource.com/external/webrtc/+/master/docs/native-code/

---

## 决策历史

| 日期 | 事件 | 决策 |
|------|------|------|
| 2026-06-27 | 初始方案调研 | 对比 3 个方案 |
| 2026-06-27 | 跨平台复用约束明确 | 排除方案 B、C |
| 2026-06-27 | 最终决策 | 选择方案 A（livekit/webrtc-sys）|

---

**下一步**: 更新 `docs/progress/codec-integration-status.md` 和 `real-environment-integration-plan.md`
