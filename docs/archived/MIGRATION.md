# WebRTC 编解码迁移记录

**迁移日期**: 2026-06-28  
**决策文档**: `docs/decisions/WEBRTC_CODEC_INTEGRATION_DECISION.md`  
**迁移方案**: 方案 B（webrtc-rs + 平台原生编解码）

---

## Phase 0: 清理 libwebrtc 残留依赖 ✅ 已完成

### 已移除的文件

| 原路径 | 新路径（已弃用） | 说明 |
|--------|-----------------|------|
| `crates/rdcs-codec/src/webrtc_encoder.rs` | `*.deprecated` | libwebrtc 封装的编码器 |
| `crates/rdcs-codec/src/webrtc_decoder.rs` | `*.deprecated` | libwebrtc 封装的解码器 |
| `crates/rdcs-codec/src/peer_connection.rs` | `*.deprecated` | libwebrtc PeerConnection 封装 |
| `crates/rdcs-session/src/manager.rs` | `*.deprecated` | 依赖 peer_connection 的会话管理器 |

### 已修改的依赖

#### `Cargo.toml`（工作区根）
```diff
- libwebrtc = "0.3"
+ # libwebrtc 已移除 - 迁移至 webrtc-rs + 平台原生编解码（方案 B）
```

#### `crates/rdcs-codec/Cargo.toml`
```diff
- libwebrtc = "0.3"
+ # libwebrtc 已移除 - 迁移至 webrtc-rs + 平台原生编解码（方案 B）
```

#### `crates/rdcs-session/Cargo.toml`
```diff
- libwebrtc = "0.3"
+ # libwebrtc 已移除 - 会话层将直接使用 rdcs-codec 的平台原生编解码器
```

### 已修改的模块导出

#### `crates/rdcs-codec/src/lib.rs`
```diff
- pub mod peer_connection;
- pub mod webrtc_decoder;
- pub mod webrtc_encoder;
+ // peer_connection.rs 已废弃 - 迁移至方案 B（webrtc-rs + 平台原生编解码）
+ // webrtc_decoder.rs 和 webrtc_encoder.rs 已废弃 - 使用 platform/* 模块代替
```

#### `crates/rdcs-session/src/lib.rs`
```diff
- pub mod manager;
+ // manager 模块暂时禁用 - 正在迁移到方案 B
+ // pub mod manager;
```

---

## Phase 1: macOS VideoToolbox 真实编解码器 🚧 待实现

### 现有基础

- ✅ `crates/rdcs-codec/src/platform/videotoolbox.rs` (399 行 FFI 基础)
- ✅ `crates/rdcs-codec/src/platform/mod.rs` (PlatformEncoder/Decoder trait)

### 待完成任务

1. **补全 VideoToolbox FFI 绑定**
   - `VTCompressionSessionCreate` ✅ 已有
   - `VTCompressionSessionEncodeFrame` ✅ 已有
   - `VTCompressionSessionCompleteFrames` ✅ 已有
   - `VTDecompressionSessionCreate` ❌ 缺失
   - `VTDecompressionSessionDecodeFrame` ❌ 缺失

2. **实现编码器**
   - BGRA → NV12 像素格式转换
   - CVPixelBuffer 创建与管理
   - 回调函数处理编码输出
   - H.264 NAL units 提取（Annex B 格式）

3. **实现解码器**
   - H.264 NAL units → CVPixelBuffer
   - NV12 → BGRA 像素格式转换
   - 回调函数处理解码输出

4. **性能验证**
   - 编码延迟 < 10ms
   - 解码延迟 < 10ms
   - CPU 占用 < 15% @ 1080p60fps

---

## Phase 2: RTP/SRTP 集成层 🚧 待实现

### 新增依赖

```toml
[dependencies]
webrtc = "0.11"  # webrtc-rs 协议栈
```

### 待实现模块

```
crates/rdcs-codec/src/
├── rtp/
│   ├── mod.rs           # RTP 打包/解包抽象
│   ├── packetizer.rs    # H.264 → RTP packets (MTU 1200B)
│   ├── depacketizer.rs  # RTP packets → H.264 NAL units
│   └── rtcp.rs          # RTCP 反馈（接收报告/发送报告）
└── srtp/
    ├── mod.rs           # SRTP 加密/解密抽象
    ├── context.rs       # SRTP 会话上下文
    └── dtls.rs          # DTLS-SRTP 密钥协商
```

### 数据流

```
发送端：
  H.264 NAL units (from VideoToolbox)
    → H264Packetizer::packetize() → Vec<RtpPacket>
    → SrtpContext::encrypt() → Vec<u8>
    → rdcs_transport::send()

接收端：
  rdcs_transport::recv() → Vec<u8>
    → SrtpContext::decrypt() → Vec<RtpPacket>
    → H264Depacketizer::depacketize() → Vec<u8> (NAL units)
    → VideoToolbox 解码
```

---

## Phase 3: 与 rdcs-connection/signaling/transport 对接 🚧 待实现

### 集成点

1. **rdcs-signaling**
   - SDP offer/answer 生成（包含编解码器参数）
   - ICE candidate 交换（复用现有实现）

2. **rdcs-connection**
   - ICE 路径选择（复用现有 IceAgent）
   - 候选对连接性检查

3. **rdcs-transport**
   - SRTP 包透传至 NACK/FEC 层
   - 拥塞控制反馈至编码器（动态码率调整）

### 新会话管理器

```rust
// crates/rdcs-session/src/session.rs (新实现)
pub struct RdcsSession {
    // 编解码器
    encoder: Box<dyn PlatformEncoder>,
    decoder: Box<dyn PlatformDecoder>,
    
    // RTP/SRTP
    rtp_packetizer: H264Packetizer,
    rtp_depacketizer: H264Depacketizer,
    srtp_context: SrtpContext,
    
    // 传输层
    transport: TransportChannel,  // rdcs-transport
    connection: IceAgent,         // rdcs-connection
    
    // 信令层
    signaling_tx: mpsc::Sender<WsMessage>,
}
```

---

## Phase 4: 端到端集成测试与验证 🚧 待实现

### 测试场景

1. **单元测试**
   - VideoToolbox 编解码正确性
   - RTP 打包/解包正确性
   - SRTP 加密/解密正确性

2. **集成测试**
   - 本地回环测试（编码 → RTP → SRTP → 解密 → 解码）
   - 真实网络测试（两台机器 P2P）

3. **性能测试**
   - 端到端延迟 < 50ms
   - CPU 占用 < 20% @ 1080p60fps
   - 内存占用 < 100MB

4. **互通性测试**
   - 与标准 WebRTC 端点互通（浏览器/Jitsi）
   - SDP 协商兼容性
   - DTLS-SRTP 握手兼容性

---

## 回滚计划

如果方案 B 遇到无法解决的问题，可考虑：

1. **回滚至 Mock 状态**
   ```bash
   git checkout HEAD -- crates/rdcs-codec/src/webrtc_*.rs
   git checkout HEAD -- crates/rdcs-codec/src/peer_connection.rs
   git checkout HEAD -- crates/rdcs-session/src/manager.rs
   ```

2. **备用方案 C**：自建 RTP/SRTP（仅当 webrtc-rs 不可用时）

---

## 参考资料

- [WebRTC 集成方案决策文档](docs/decisions/WEBRTC_CODEC_INTEGRATION_DECISION.md)
- [webrtc-rs 官方文档](https://github.com/webrtc-rs/webrtc)
- [RustDesk 源码参考](https://github.com/rustdesk/rustdesk)
- [VideoToolbox 官方文档](https://developer.apple.com/documentation/videotoolbox)
