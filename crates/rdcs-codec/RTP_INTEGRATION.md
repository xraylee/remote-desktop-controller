# RTP/SRTP Integration Documentation

## 概述

本文档描述 `rdcs-codec` 中的 RTP/SRTP 集成层，用于通过 WebRTC 安全传输 H.264 视频流。

## 架构

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Sender Side                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  VideoToolbox Encoder                                           │
│         ↓                                                       │
│  H.264 NAL Units (Annex B)                                      │
│         ↓                                                       │
│  H264Packetizer (RFC 6184)                                      │
│    • Single NAL Unit Mode                                       │
│    • FU-A Fragmentation                                         │
│         ↓                                                       │
│  RTP Packets (unencrypted)                                      │
│         ↓                                                       │
│  SrtpContext::encrypt()                                         │
│    • AES-128-GCM                                                │
│    • Authentication Tag                                         │
│         ↓                                                       │
│  SRTP Packets (encrypted)                                       │
│         ↓                                                       │
│  Network (UDP via WebRTC DataChannel)                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                       Receiver Side                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Network (UDP via WebRTC DataChannel)                           │
│         ↓                                                       │
│  SRTP Packets (encrypted)                                       │
│         ↓                                                       │
│  SrtpContext::decrypt()                                         │
│    • Verify Auth Tag                                            │
│    • Replay Protection                                          │
│         ↓                                                       │
│  RTP Packets (decrypted)                                        │
│         ↓                                                       │
│  H264Depacketizer                                               │
│    • Fragment Reassembly                                        │
│    • Loss Detection                                             │
│         ↓                                                       │
│  H.264 NAL Units (Annex B)                                      │
│         ↓                                                       │
│  VideoToolbox Decoder                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 模块组成

### 1. RTP Header (`rtp/mod.rs`)

标准 RTP 固定头（12 字节）：

```rust
pub struct RtpHeader {
    pub version: u8,          // 必须为 2
    pub padding: bool,
    pub extension: bool,
    pub csrc_count: u8,
    pub marker: bool,         // 帧结束标记
    pub payload_type: u8,     // 96-127 (动态类型)
    pub sequence_number: u16, // 递增序列号
    pub timestamp: u32,       // 90kHz 时钟
    pub ssrc: u32,            // 同步源标识符
}
```

### 2. H.264 RTP Packetizer (`rtp/packetizer.rs`)

将 H.264 Annex B 格式转换为 RTP 包。

#### 特性

- **Single NAL Unit Mode**: 小 NAL 单元直接打包
- **FU-A Fragmentation**: 大 NAL 单元分片（RFC 6184 §5.8）
- **MTU 配置**: 默认 1200 字节（适配典型网络）
- **统计信息**: 包计数、分片计数、关键帧计数

#### 使用示例

```rust
use rdcs_codec::rtp::{H264Packetizer, PacketizerConfig};

let config = PacketizerConfig {
    mtu: 1200,
    clock_rate: 90000,
    ssrc: 0x12345678,
    payload_type: 96,
};

let mut packetizer = H264Packetizer::new(config);

// 输入：H.264 Annex B 数据
let annex_b = vec![
    0x00, 0x00, 0x00, 0x01, // Start code
    0x65, // IDR NAL header
    // ... payload
];

// 输出：RTP 包列表
let rtp_packets = packetizer.packetize(&annex_b, timestamp)?;

for packet in rtp_packets {
    // 发送到网络或 SRTP 层
}
```

#### NAL Unit 类型

```rust
pub enum NalUnitType {
    NonIdrSlice = 1,  // P/B 帧
    IdrSlice = 5,     // 关键帧
    Sps = 7,          // 序列参数集
    Pps = 8,          // 图像参数集
    FuA = 28,         // 分片单元
}
```

### 3. H.264 RTP Depacketizer (`rtp/depacketizer.rs`)

将 RTP 包重组为 H.264 Annex B 格式。

#### 特性

- **Single NAL 解包**: 直接输出完整 NAL 单元
- **FU-A 重组**: 跨包重组分片 NAL 单元
- **序列号检查**: 检测丢包和乱序
- **碎片清理**: 自动丢弃过期不完整碎片

#### 使用示例

```rust
use rdcs_codec::rtp::H264Depacketizer;

let mut depacketizer = H264Depacketizer::new();

// 接收 RTP 包
while let Some(rtp_packet) = receive_from_network()? {
    if let Some(annex_b) = depacketizer.depacketize(&rtp_packet)? {
        // 完整帧已接收，送入解码器
        decoder.decode(&annex_b)?;
    }
}

// 查看统计
let stats = depacketizer.stats();
println!("Lost packets: {}", stats.packets_lost);
```

#### 丢包处理

```rust
// 检测序列号间隙
if header.sequence_number != expected {
    let gap = header.sequence_number.wrapping_sub(expected);
    if gap < 32768 {
        // 正向间隙 = 丢包
        stats.packets_lost += gap;
    } else {
        // 反向间隙 = 乱序
        stats.packets_out_of_order += 1;
    }
}
```

### 4. SRTP Encryption (`rtp/srtp.rs`)

基于 `webrtc-srtp` 的加密/解密层。

#### 特性

- **AES-128-GCM**: 现代 AEAD 加密（推荐）
- **AES-128-CM-HMAC-SHA1-80**: 传统模式（兼容性）
- **重放保护**: 自动序列号验证
- **异步 API**: 支持 Tokio 并发

#### 配置

```rust
use rdcs_codec::rtp::{SrtpConfig, SrtpContext, SrtpProfile};

let config = SrtpConfig {
    master_key: vec![0x01; 16],    // 16 字节 AES-128 密钥
    master_salt: vec![0x02; 14],   // 14 字节盐值
    profile: SrtpProfile::Aead_Aes128Gcm,
};

let srtp = SrtpContext::new(config).await?;
```

#### 加密流程

```rust
// 发送端
let rtp_packet = packetizer.packetize(&h264_data, ts)?[0];
let srtp_packet = srtp_context.encrypt(&rtp_packet).await?;
send_to_network(&srtp_packet)?;
```

#### 解密流程

```rust
// 接收端
let srtp_packet = receive_from_network()?;
let rtp_packet = srtp_context.decrypt(&srtp_packet).await?;
let h264 = depacketizer.depacketize(&rtp_packet)?;
```

## 性能考量

### MTU 选择

| MTU  | 说明                     | 适用场景           |
|------|--------------------------|-------------------|
| 1500 | 标准以太网               | 有线网络          |
| 1200 | 安全余量（默认）          | 混合网络          |
| 576  | 最小 IPv4 MTU            | 受限网络          |

### 分片开销

- **单包模式**: 12 字节 RTP 头 + 16 字节 SRTP 认证标签
- **FU-A 模式**: +2 字节分片头（每包）

### 内存使用

```rust
// 发送端（每帧）
let estimated_packets = frame_size / (mtu - 28);
let buffer_size = estimated_packets * mtu;

// 接收端（重组缓冲区）
const MAX_FRAGMENT_AGE: u16 = 1000; // 自动清理
```

## 测试

### 单元测试

```bash
cargo test -p rdcs-codec rtp::packetizer
cargo test -p rdcs-codec rtp::depacketizer
cargo test -p rdcs-codec rtp::srtp
```

### 集成测试

```bash
cargo test -p rdcs-codec --test rtp_integration
```

### 测试覆盖

| 场景                  | 测试                                  |
|-----------------------|---------------------------------------|
| 单 NAL 单元           | `test_end_to_end_single_nal_unit`     |
| FU-A 分片             | `test_end_to_end_fragmented_nal_unit` |
| 多 NAL 单元           | `test_end_to_end_multiple_nal_units`  |
| 丢包检测              | `test_packet_loss_detection`          |
| SRTP 认证失败         | `test_srtp_authentication_failure`    |
| 重放攻击              | `test_srtp_replay_protection`         |

## 错误处理

```rust
pub enum RtpError {
    InvalidPacket(String),      // 格式错误
    NalUnitError(String),       // NAL 解析失败
    PacketTooLarge { .. },      // 超过 MTU
    SrtpError(String),          // 加密/解密失败
    SequenceGap { .. },         // 丢包
    FragmentationError(String), // 分片错误
    WebRtcError(..),            // webrtc-rs 错误
}
```

## 限制和已知问题

### 当前限制

1. **仅支持 H.264**: H.265/VP9 待实现
2. **无 RTCP**: 暂无接收端反馈（NACK/PLI）
3. **无 FEC**: 无前向纠错（未来可考虑 RED/ULPFEC）
4. **固定加密配置**: 密钥需外部管理（通常来自 DTLS）

### 未来改进

- [ ] RTCP 反馈支持（SR/RR/NACK/PLI）
- [ ] 前向纠错（FEC）
- [ ] 带宽自适应（配合 `adaptive` 模块）
- [ ] H.265/VP9 打包支持
- [ ] DTLS 密钥协商集成

## 与 WebRTC 对接

### 发送端集成

```rust
// 在 rdcs-connection 中
use rdcs_codec::rtp::{H264Packetizer, SrtpContext};

// 1. 编码
let h264_data = encoder.encode(&frame)?;

// 2. 打包
let rtp_packets = packetizer.packetize(&h264_data, timestamp)?;

// 3. 加密
for rtp_packet in rtp_packets {
    let srtp_packet = srtp_context.encrypt(&rtp_packet).await?;
    
    // 4. 发送
    data_channel.send(&srtp_packet).await?;
}
```

### 接收端集成

```rust
// 在 rdcs-connection 中
use rdcs_codec::rtp::{H264Depacketizer, SrtpContext};

// 1. 接收
while let Some(srtp_packet) = data_channel.recv().await? {
    // 2. 解密
    let rtp_packet = srtp_context.decrypt(&srtp_packet).await?;
    
    // 3. 解包
    if let Some(h264_data) = depacketizer.depacketize(&rtp_packet)? {
        // 4. 解码
        let frame = decoder.decode(&h264_data)?;
        render(frame);
    }
}
```

## 调试

### 启用日志

```bash
RUST_LOG=rdcs_codec::rtp=trace cargo run
```

### 统计信息

```rust
// Packetizer 统计
let stats = packetizer.stats();
println!("Packets sent: {}", stats.packets_sent);
println!("Fragmented NAL units: {}", stats.fragmented_nal_units);

// Depacketizer 统计
let stats = depacketizer.stats();
println!("Packets lost: {}", stats.packets_lost);
println!("Out of order: {}", stats.packets_out_of_order);

// SRTP 统计
let stats = srtp.stats().await;
println!("Encryption errors: {}", stats.encryption_errors);
println!("Decryption errors: {}", stats.decryption_errors);
```

## 参考文档

- **RFC 3550**: RTP: A Transport Protocol for Real-Time Applications
- **RFC 3711**: The Secure Real-time Transport Protocol (SRTP)
- **RFC 6184**: RTP Payload Format for H.264 Video
- **webrtc-srtp**: <https://docs.rs/webrtc-srtp/0.13>

## 维护者

- 2026 RDCS Contributors

---

**最后更新**: 2026-06-28
