# rdcs-codec

视频编解码管道，用于 RDCS 远程桌面系统。

## 功能

- **平台原生编解码器**: VideoToolbox (macOS), Media Foundation (Windows), VA-API (Linux 计划中)
- **RTP/SRTP 传输**: 符合 RFC 6184 的 H.264 打包和 SRTP 加密
- **自适应质量控制**: 基于网络状况的码率/分辨率调整
- **内容分析**: 文本/视频场景检测以优化编码参数

## 架构

```text
┌──────────────────────────────────────────────────────────────┐
│                         rdcs-codec                           │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌────────────┐    ┌──────────────┐    ┌──────────────┐    │
│  │  Analyzer  │───▶│   Encoder    │───▶│ Packetizer   │    │
│  │  (Content) │    │  (Platform)  │    │   (RTP)      │    │
│  └────────────┘    └──────────────┘    └──────────────┘    │
│                            │                    │           │
│                            ▼                    ▼           │
│                    ┌──────────────┐    ┌──────────────┐    │
│                    │   Adaptive   │    │     SRTP     │    │
│                    │   Quality    │    │ (Encryption) │    │
│                    └──────────────┘    └──────────────┘    │
│                                                │            │
│                                                ▼            │
│                                        ┌──────────────┐    │
│  ┌────────────┐    ┌──────────────┐   │   Network    │    │
│  │  Renderer  │◀───│   Decoder    │◀──│              │    │
│  │            │    │  (Platform)  │   └──────────────┘    │
│  └────────────┘    └──────────────┘           ▲           │
│                            ▲                   │           │
│                            │           ┌──────────────┐    │
│                    ┌──────────────┐    │     SRTP     │    │
│                    │Depacketizer  │◀───│ (Decryption) │    │
│                    │   (RTP)      │    └──────────────┘    │
│                    └──────────────┘                        │
└──────────────────────────────────────────────────────────────┘
```

## 模块

### `analyzer`
内容分析器，检测文本密集区域和视频场景，用于优化编码参数。

### `encoder` / `decoder`
编解码器特征和平台实现：
- **macOS**: VideoToolbox H.264 硬件加速
- **Windows**: Media Foundation (待实现)
- **Linux**: VA-API (待实现)

### `rtp`
RTP/SRTP 传输层：
- **packetizer**: H.264 Annex B → RTP (RFC 6184)
- **depacketizer**: RTP → H.264 Annex B
- **srtp**: AES-128-GCM 加密/解密

详见 [RTP_INTEGRATION.md](./RTP_INTEGRATION.md)

### `adaptive`
自适应质量控制，根据网络条件调整码率和分辨率。

### `platform`
平台特定实现：
- `platform/macos`: VideoToolbox 编解码器
- `platform/windows`: Media Foundation (待实现)
- `platform/linux`: VA-API (待实现)

### `pipeline`
端到端编解码管道，集成所有模块。

## 使用示例

### 编码和打包

```rust
use rdcs_codec::{
    encoder::VideoEncoder,
    platform::macos::VideoToolboxEncoder,
    rtp::{H264Packetizer, PacketizerConfig, SrtpContext, SrtpConfig},
};

// 1. 创建编码器
let mut encoder = VideoToolboxEncoder::new()?;
encoder.configure(&config)?;

// 2. 创建打包器
let mut packetizer = H264Packetizer::new(PacketizerConfig::default());

// 3. 创建 SRTP 上下文
let srtp = SrtpContext::new(srtp_config).await?;

// 4. 编码和发送
let h264_data = encoder.encode(&frame)?;
let rtp_packets = packetizer.packetize(&h264_data, timestamp)?;

for rtp_packet in rtp_packets {
    let srtp_packet = srtp.encrypt(&rtp_packet).await?;
    network.send(&srtp_packet).await?;
}
```

### 接收和解码

```rust
use rdcs_codec::{
    decoder::VideoDecoder,
    platform::macos::VideoToolboxDecoder,
    rtp::{H264Depacketizer, SrtpContext},
};

// 1. 创建解包器
let mut depacketizer = H264Depacketizer::new();

// 2. 创建 SRTP 上下文
let srtp = SrtpContext::new(srtp_config).await?;

// 3. 创建解码器
let mut decoder = VideoToolboxDecoder::new()?;
decoder.configure(&config)?;

// 4. 接收和解码
while let Some(srtp_packet) = network.recv().await? {
    let rtp_packet = srtp.decrypt(&srtp_packet).await?;
    
    if let Some(h264_data) = depacketizer.depacketize(&rtp_packet)? {
        let frame = decoder.decode(&h264_data)?;
        renderer.display(frame);
    }
}
```

## 测试

```bash
# 单元测试
cargo test -p rdcs-codec

# RTP 集成测试
cargo test -p rdcs-codec --test rtp_integration

# 平台编解码器测试 (macOS)
cargo test -p rdcs-codec platform::macos
```

## 依赖

- `webrtc-srtp`: SRTP 加密/解密
- `webrtc-util`: RTP 工具库
- `bytes`: 高效字节缓冲区
- `tokio`: 异步运行时

平台依赖：
- **macOS**: `core-foundation`
- **Windows**: `windows` crate (Media Foundation)
- **Linux**: VA-API (待添加)

## 性能

### 编码延迟 (macOS M1)
- 1080p @ 60fps: ~5ms (硬件加速)
- 720p @ 30fps: ~3ms

### RTP 打包开销
- MTU 1200: 每包 28 字节开销 (12 RTP + 16 SRTP)
- 大帧分片: 每 1172 字节一包

### 内存使用
- 编码器缓冲区: ~2MB (可配置)
- RTP 重组缓冲区: 自动清理（1000 包阈值）

## 限制

当前实现限制：
- 仅支持 H.264 (H.265/VP9 待实现)
- 无 RTCP 反馈 (NACK/PLI 待实现)
- 无 FEC 前向纠错
- SRTP 密钥需外部提供 (通常来自 DTLS)

## 贡献

欢迎贡献！特别是：
- Windows Media Foundation 实现
- Linux VA-API 实现
- RTCP 反馈支持
- FEC/ARQ 实现

## 许可证

Apache-2.0

---

**维护者**: RDCS Contributors  
**最后更新**: 2026-06-28
