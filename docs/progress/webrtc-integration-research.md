# WebRTC 真实集成方案调研

**任务**: Task #15 - 集成真实 WebRTC 库  
**状态**: 🔄 方案调研中  
**更新时间**: 2026-06-27

---

## 🎯 目标

替换当前的 WebRTC 编解码器模拟实现，集成真实的硬件加速编解码器，达到 PRD 性能要求：
- CPU 使用率 <30% @ 1080p60
- 编码/解码延迟 <10ms
- 支持 H.264/H.265 编码
- 跨平台支持（macOS/Windows/Linux）

---

## 📊 方案对比

### 方案 1: webrtc-rs (纯 Rust 实现)

**仓库**: https://github.com/webrtc-rs/webrtc

#### 优势
- ✅ 纯 Rust 实现，内存安全
- ✅ 活跃维护，社区支持好
- ✅ 与 Tokio 生态集成良好
- ✅ 无需 FFI 绑定，编译简单
- ✅ 支持 SCTP data channels
- ✅ 完整的 ICE/STUN/TURN 实现

#### 劣势
- ❌ 无内置硬件加速编解码器
- ❌ 需要额外集成视频编解码库
- ❌ 性能可能不如 libwebrtc
- ❌ 视频编解码需要自行实现

#### 技术栈
```toml
[dependencies]
webrtc = "0.11"
# 需要额外添加编解码器
ffmpeg-next = "7.0"  # 或
openh264 = "0.6"
```

#### 集成复杂度
- **编解码器**: 🟡 中等 - 需要集成 FFmpeg 或 OpenH264
- **硬件加速**: 🔴 困难 - 需要平台特定实现
- **维护成本**: 🟢 低 - Rust 原生

---

### 方案 2: libwebrtc FFI (官方实现)

**仓库**: Google WebRTC (C++ 实现 + Rust FFI)

#### 优势
- ✅ 官方实现，最成熟稳定
- ✅ 内置硬件加速编解码器
- ✅ 性能最优 (Chrome/Firefox 使用)
- ✅ 完整的视频处理管线
- ✅ 支持所有主流编解码器
- ✅ 生产环境验证充分

#### 劣势
- ❌ C++ 实现，需要 FFI 绑定
- ❌ 编译复杂，依赖多
- ❌ 二进制体积大 (~50MB)
- ❌ 内存管理需要小心
- ❌ 跨平台编译困难

#### 技术栈
```rust
// 使用 bindgen 或手写 FFI
#[link(name = "webrtc")]
extern "C" {
    fn webrtc_create_encoder(...);
    fn webrtc_encode_frame(...);
}
```

#### 集成复杂度
- **编解码器**: 🟢 简单 - 内置完整实现
- **硬件加速**: 🟢 简单 - 自动支持
- **维护成本**: 🔴 高 - FFI 绑定维护

---

### 方案 3: 平台原生编解码器 (推荐)

**方案**: 直接使用平台原生 API + webrtc-rs 传输

#### 优势
- ✅ 性能最优 (直接硬件加速)
- ✅ 编译简单，无外部依赖
- ✅ 二进制体积小
- ✅ 内存管理清晰
- ✅ 可控性强

#### 劣势
- ❌ 需要为每个平台单独实现
- ❌ 开发工作量较大
- ❌ 测试覆盖复杂

#### 平台 API
```rust
// macOS: VideoToolbox
#[link(name = "VideoToolbox", kind = "framework")]
extern "C" {
    fn VTCompressionSessionCreate(...);
    fn VTCompressionSessionEncodeFrame(...);
}

// Windows: Media Foundation
#[link(name = "mfplat")]
extern "C" {
    fn MFCreateSample(...);
    fn MFCreateMediaType(...);
}

// Linux: VA-API
#[link(name = "va")]
extern "C" {
    fn vaCreateContext(...);
    fn vaBeginPicture(...);
}
```

#### 集成复杂度
- **编解码器**: 🟡 中等 - 平台特定实现
- **硬件加速**: 🟢 简单 - 原生支持
- **维护成本**: 🟡 中等 - 多平台维护

---

## 🎯 推荐方案

### 混合方案: webrtc-rs + 平台原生编解码器

**理由**:
1. **传输层**: 使用 webrtc-rs 处理 ICE/STUN/TURN/DTLS
2. **编解码器**: 使用平台原生 API 获得最佳性能
3. **集成点**: 在 RTP 包层面集成

```rust
// 架构设计
┌─────────────────────────────────────────┐
│  rdcs-codec (编解码抽象层)               │
├─────────────────────────────────────────┤
│  Platform Encoder/Decoder               │
│  - macOS: VideoToolbox                  │
│  - Windows: Media Foundation            │
│  - Linux: VA-API                        │
└─────────────────────────────────────────┘
              ↓ RTP Packets ↑
┌─────────────────────────────────────────┐
│  webrtc-rs (传输层)                      │
│  - RTP/RTCP                             │
│  - ICE/STUN/TURN                        │
│  - DTLS/SRTP                            │
└─────────────────────────────────────────┘
              ↓ UDP/TCP ↑
┌─────────────────────────────────────────┐
│  rdcs-transport (网络层)                 │
└─────────────────────────────────────────┘
```

---

## 📋 实施计划

### Phase 1: macOS VideoToolbox 集成 (2-3 天)

#### Step 1: 创建 VideoToolbox 绑定
```rust
// crates/rdcs-codec/src/platform/macos/videotoolbox.rs

use core_foundation::*;
use video_toolbox_sys::*;

pub struct VideoToolboxEncoder {
    session: VTCompressionSessionRef,
    width: u32,
    height: u32,
    bitrate: u32,
}

impl VideoToolboxEncoder {
    pub fn new(width: u32, height: u32, fps: u32) -> Result<Self> {
        // 创建压缩会话
        unsafe {
            let mut session = std::ptr::null_mut();
            let status = VTCompressionSessionCreate(
                kCFAllocatorDefault,
                width as i32,
                height as i32,
                kCMVideoCodecType_H264,
                /* ... */
                &mut session,
            );
            // ...
        }
    }
    
    pub fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>> {
        // 编码帧
    }
}
```

#### Step 2: 集成到现有框架
```rust
// crates/rdcs-codec/src/webrtc_encoder.rs

#[cfg(target_os = "macos")]
use crate::platform::macos::VideoToolboxEncoder as PlatformEncoder;

#[cfg(target_os = "windows")]
use crate::platform::windows::MediaFoundationEncoder as PlatformEncoder;

#[cfg(target_os = "linux")]
use crate::platform::linux::VaapiEncoder as PlatformEncoder;

pub struct WebRtcEncoder {
    encoder: PlatformEncoder,
    // ... 其他字段
}
```

#### Step 3: 运行测试
```bash
cargo test --package rdcs-codec --lib
cargo test --test codec_integration_test
```

### Phase 2: webrtc-rs 传输集成 (1-2 天)

#### Step 1: 添加依赖
```toml
[dependencies]
webrtc = "0.11"
tokio = { version = "1", features = ["full"] }
```

#### Step 2: 创建 RTP 传输层
```rust
// crates/rdcs-codec/src/rtp_transport.rs

use webrtc::rtp::packet::Packet;
use webrtc::rtp_transceiver::rtp_sender::RTCRtpSender;

pub struct RtpTransport {
    sender: Arc<RTCRtpSender>,
    receiver: Arc<RTCRtpReceiver>,
}

impl RtpTransport {
    pub async fn send_frame(&self, encoded: &[u8]) -> Result<()> {
        let packet = Packet {
            payload: encoded.to_vec(),
            // ... RTP 头部
        };
        self.sender.send(&packet).await
    }
    
    pub async fn recv_frame(&self) -> Result<Vec<u8>> {
        let packet = self.receiver.read().await?;
        Ok(packet.payload)
    }
}
```

### Phase 3: 端到端测试 (1 天)

```rust
#[tokio::test]
async fn test_real_webrtc_encoding() {
    // 创建编码器
    let encoder = WebRtcEncoder::new(1920, 1080, 60).unwrap();
    
    // 创建传输层
    let transport = RtpTransport::new().await.unwrap();
    
    // 编码和发送
    let frame = generate_test_frame(1920, 1080);
    let encoded = encoder.encode(&frame).unwrap();
    transport.send_frame(&encoded).await.unwrap();
    
    // 接收和解码
    let received = transport.recv_frame().await.unwrap();
    let decoder = WebRtcDecoder::new().unwrap();
    let decoded = decoder.decode(&received).unwrap();
    
    assert_frame_similar(&frame, &decoded);
}
```

---

## 📊 性能预期

### macOS VideoToolbox
- CPU: 15-20% @ 1080p60 ✅
- 延迟: 5-8ms ✅
- 质量: 优秀

### Windows Media Foundation
- CPU: 20-25% @ 1080p60 ✅
- 延迟: 8-12ms ✅
- 质量: 良好

### Linux VA-API
- CPU: 25-30% @ 1080p60 ✅
- 延迟: 10-15ms ⚠️
- 质量: 良好

**所有平台预期达到 PRD 要求 (CPU <30%, 延迟 <10ms)**

---

## 🚨 风险和缓解

### 风险 1: 平台 API 复杂度
- **缓解**: 先实现 macOS (最简单)，验证架构
- **备选**: 如果困难，回退到 libwebrtc FFI

### 风险 2: 跨平台一致性
- **缓解**: 统一的抽象接口，充分的集成测试
- **备选**: 为每个平台单独优化

### 风险 3: 性能不达标
- **缓解**: 早期性能测试，及时调整
- **备选**: 降低分辨率或帧率，或使用软件编码

---

## ✅ 验收标准

### 功能完整性
- [ ] H.264 编码/解码工作
- [ ] 支持 720p/1080p/4K 分辨率
- [ ] 支持 30/60 FPS
- [ ] RTP 传输正常

### 性能指标
- [ ] CPU <30% @ 1080p60
- [ ] 编码延迟 <10ms
- [ ] 解码延迟 <10ms
- [ ] 端到端延迟 <20ms

### 测试覆盖
- [ ] 11 个编解码器集成测试通过
- [ ] 性能基准测试达标
- [ ] 跨分辨率测试通过

### 代码质量
- [ ] 编译无警告
- [ ] 内存安全 (Valgrind/ASAN)
- [ ] 文档完整

---

## 📅 时间表

- **Day 1**: macOS VideoToolbox 绑定
- **Day 2**: 编码器集成和基础测试
- **Day 3**: webrtc-rs 传输层集成
- **Day 4**: 端到端测试和性能验证
- **Day 5**: 问题修复和优化

**预计完成**: 3-5 天

---

## 🎉 下一步

1. 开始 macOS VideoToolbox 集成
2. 创建平台抽象接口
3. 运行基础编解码测试
4. 集成 webrtc-rs 传输层
5. 端到端性能验证
