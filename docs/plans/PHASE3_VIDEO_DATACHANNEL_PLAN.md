# Phase 3.4 视频流 DataChannel 传输实现计划

**日期**: 2026-06-28  
**状态**: 进行中  
**目标**: 通过加密 DataChannel 传输视频帧，建立完整的 P2P 视频流水线

---

## 🎯 目标

将现有的 TCP 视频传输迁移到 WebRTC DataChannel：

**当前架构** (Phase 2):
```
编码器 → TCP Socket → 网络 → TCP Socket → 解码器
```

**目标架构** (Phase 3.4):
```
编码器 → DataChannel (DTLS加密) → P2P/TURN → DataChannel → 解码器
```

**优势**:
- ✅ 端到端加密 (DTLS)
- ✅ NAT 穿透 (ICE)
- ✅ 自动重连
- ✅ 拥塞控制
- ✅ 更低延迟（UDP）

---

## 📋 当前状态

### 已完成
- ✅ ICE 连接建立
- ✅ DTLS 加密通道
- ✅ PeerConnection 状态管理
- ✅ OpenH264 编解码器
- ✅ TCP 视频传输层

### 待实现
- ⏳ DataChannel 创建和配置
- ⏳ 视频帧序列化
- ⏳ DataChannel 发送/接收
- ⏳ 帧重组和去重
- ⏳ 端到端测试

---

## 🔧 实施方案

### 方案 1: 直接使用 DataChannel ⭐ 推荐

**优点**:
- 利用现有 PeerConnection
- 自动分片和重组
- 可靠有序传输
- 简单直接

**缺点**:
- 消息大小限制 (16KB - 256KB)
- 需要手动分片大帧

**实现**:
```rust
// 1. 在 RealIceAgent 中暴露 DataChannel
pub struct RealIceAgent {
    peer_connection: Arc<RTCPeerConnection>,
    data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,
    // ...
}

// 2. 创建 DataChannel
let dc = peer_connection.create_data_channel(
    "video",
    Some(RTCDataChannelInit {
        ordered: Some(true),      // 保证顺序
        max_retransmits: None,    // 可靠传输
        ..Default::default()
    })
).await?;

// 3. 发送视频帧
dc.send(&frame_bytes).await?;

// 4. 接收视频帧
dc.on_message(Box::new(move |msg| {
    let frame = parse_frame(&msg.data);
    // 解码并显示
}));
```

---

### 方案 2: 使用 RTP over DataChannel

**优点**:
- 标准 RTP 协议
- 更好的媒体控制
- 时间戳同步

**缺点**:
- 更复杂
- 需要实现 RTP 封装
- 不是本阶段重点

**暂不采用**，留待后续优化。

---

## 📝 实施步骤

### Step 1: 扩展 RealIceAgent 暴露 DataChannel (30分钟)

**文件**: `crates/rdcs-connection/src/real_ice_agent.rs`

```rust
impl RealIceAgent {
    /// Get the DataChannel for video transmission.
    pub fn get_data_channel(&self) -> Result<Arc<RTCDataChannel>, ConnectionError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if let Some(dc) = self.data_channel.lock().await.as_ref() {
                    Ok(dc.clone())
                } else {
                    Err(ConnectionError::IceError("DataChannel not ready".to_string()))
                }
            })
        })
    }
}
```

---

### Step 2: 创建视频帧传输层 (1小时)

**新文件**: `crates/rdcs-connection/src/video_channel.rs`

```rust
use webrtc::data_channel::RTCDataChannel;

/// Video frame transmission over DataChannel.
pub struct VideoChannel {
    data_channel: Arc<RTCDataChannel>,
    max_chunk_size: usize, // DataChannel 消息大小限制
}

impl VideoChannel {
    /// Send a video frame.
    pub async fn send_frame(&self, frame: &[u8]) -> Result<(), ConnectionError> {
        if frame.len() <= self.max_chunk_size {
            // 小帧直接发送
            self.data_channel.send(&Bytes::from(frame.to_vec())).await?;
        } else {
            // 大帧分片发送
            for chunk in frame.chunks(self.max_chunk_size) {
                self.data_channel.send(&Bytes::from(chunk.to_vec())).await?;
            }
        }
        Ok(())
    }
    
    /// Set up frame receiver.
    pub fn on_frame<F>(&self, callback: F)
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        let callback = Arc::new(callback);
        self.data_channel.on_message(Box::new(move |msg| {
            let data = msg.data.to_vec();
            callback(data);
            Box::pin(async {})
        }));
    }
}
```

---

### Step 3: 实现帧序列化 (30分钟)

**协议设计**:

```
Frame Header (8 bytes):
  - frame_id: u32 (4 bytes)
  - flags: u8 (1 byte)
    - bit 0: is_keyframe
    - bit 1-3: reserved
  - chunk_index: u8 (1 byte)
  - total_chunks: u8 (1 byte)
  - reserved: u8 (1 byte)

Frame Data:
  - payload: [u8]
```

**实现**:
```rust
#[derive(Debug)]
struct FrameHeader {
    frame_id: u32,
    is_keyframe: bool,
    chunk_index: u8,
    total_chunks: u8,
}

impl FrameHeader {
    fn serialize(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.frame_id.to_be_bytes());
        buf[4] = if self.is_keyframe { 1 } else { 0 };
        buf[5] = self.chunk_index;
        buf[6] = self.total_chunks;
        buf
    }
    
    fn deserialize(buf: &[u8]) -> Result<Self, Error> {
        if buf.len() < 8 {
            return Err(Error::InvalidHeader);
        }
        Ok(Self {
            frame_id: u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            is_keyframe: buf[4] == 1,
            chunk_index: buf[5],
            total_chunks: buf[6],
        })
    }
}
```

---

### Step 4: 实现帧重组 (45分钟)

**新文件**: `crates/rdcs-connection/src/frame_reassembler.rs`

```rust
use std::collections::HashMap;

/// Reassembles fragmented video frames.
pub struct FrameReassembler {
    pending_frames: HashMap<u32, PartialFrame>,
    max_pending: usize,
}

struct PartialFrame {
    chunks: Vec<Option<Vec<u8>>>,
    total_chunks: u8,
    received_chunks: u8,
    is_keyframe: bool,
}

impl FrameReassembler {
    pub fn new(max_pending: usize) -> Self {
        Self {
            pending_frames: HashMap::new(),
            max_pending,
        }
    }
    
    /// Add a chunk. Returns complete frame if all chunks received.
    pub fn add_chunk(
        &mut self,
        header: FrameHeader,
        data: Vec<u8>,
    ) -> Option<(u32, Vec<u8>, bool)> {
        let frame = self.pending_frames
            .entry(header.frame_id)
            .or_insert_with(|| PartialFrame {
                chunks: vec![None; header.total_chunks as usize],
                total_chunks: header.total_chunks,
                received_chunks: 0,
                is_keyframe: header.is_keyframe,
            });
        
        if frame.chunks[header.chunk_index as usize].is_none() {
            frame.chunks[header.chunk_index as usize] = Some(data);
            frame.received_chunks += 1;
        }
        
        // Check if complete
        if frame.received_chunks == frame.total_chunks {
            let complete_frame: Vec<u8> = frame.chunks
                .iter()
                .filter_map(|c| c.as_ref())
                .flat_map(|c| c.iter().copied())
                .collect();
            
            let is_keyframe = frame.is_keyframe;
            self.pending_frames.remove(&header.frame_id);
            
            return Some((header.frame_id, complete_frame, is_keyframe));
        }
        
        // Clean up old frames
        if self.pending_frames.len() > self.max_pending {
            if let Some(oldest_id) = self.pending_frames.keys().min().copied() {
                self.pending_frames.remove(&oldest_id);
            }
        }
        
        None
    }
}
```

---

### Step 5: 集成到现有架构 (1小时)

**修改**: `crates/rdcs-connection/src/lib.rs`

```rust
pub mod video_channel;
pub mod frame_reassembler;

pub use video_channel::VideoChannel;
pub use frame_reassembler::FrameReassembler;
```

**创建端到端示例**: `crates/rdcs-connection/examples/video_over_datachannel.rs`

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. 建立 ICE 连接
    let mut peer_a = RealIceAgent::new(ice_servers).await?;
    let mut peer_b = RealIceAgent::new(ice_servers).await?;
    
    // 2. 交换 SDP
    let offer = peer_a.create_offer()?;
    peer_b.set_remote_offer(&offer)?;
    let answer = peer_b.create_answer()?;
    peer_a.handle_answer(answer)?;
    
    // 3. 等待连接
    wait_for_connection(&peer_a, &peer_b).await?;
    
    // 4. 获取 DataChannel
    let dc_a = peer_a.get_data_channel()?;
    let dc_b = peer_b.get_data_channel()?;
    
    // 5. 创建视频通道
    let video_tx = VideoChannel::new(dc_a);
    let video_rx = VideoChannel::new(dc_b);
    
    // 6. 发送端：编码并发送
    tokio::spawn(async move {
        let mut encoder = OpenH264Encoder::new(1920, 1080, 30)?;
        loop {
            let frame = capture_frame()?;
            let encoded = encoder.encode(&frame)?;
            video_tx.send_frame(&encoded).await?;
            tokio::time::sleep(Duration::from_millis(33)).await; // 30fps
        }
    });
    
    // 7. 接收端：接收并解码
    let mut decoder = OpenH264Decoder::new()?;
    let mut reassembler = FrameReassembler::new(10);
    
    video_rx.on_frame(move |chunk| {
        let header = FrameHeader::deserialize(&chunk[..8])?;
        let data = chunk[8..].to_vec();
        
        if let Some((frame_id, complete_frame, is_key)) = 
            reassembler.add_chunk(header, data) {
            let decoded = decoder.decode(&complete_frame)?;
            display_frame(decoded);
        }
    });
    
    Ok(())
}
```

---

### Step 6: 测试和验证 (30分钟)

```bash
# 1. 编译
cargo build -p rdcs-connection --example video_over_datachannel

# 2. 运行测试
RUST_LOG=info cargo run -p rdcs-connection --example video_over_datachannel

# 3. 验证指标
# - 视频流畅播放
# - 延迟 < 100ms
# - 无明显丢帧
# - CPU 使用率合理
```

---

## 🎯 验收标准

- [ ] RealIceAgent 暴露 DataChannel API
- [ ] VideoChannel 实现发送/接收
- [ ] 帧序列化协议定义
- [ ] FrameReassembler 实现分片重组
- [ ] 端到端示例程序
- [ ] 视频流成功传输
- [ ] 延迟测试 < 100ms
- [ ] 无内存泄漏

---

## 📊 性能目标

| 指标 | 目标 | 说明 |
|------|------|------|
| 端到端延迟 | < 100ms | 编码 + 传输 + 解码 |
| 帧率 | 30 fps | 稳定 |
| CPU 使用率 | < 30% | 单核 |
| 内存占用 | < 100MB | 不含系统缓冲 |
| 连接建立时间 | < 2s | ICE + DTLS |

---

## ⚠️ 技术挑战

### 1. DataChannel 消息大小限制

**问题**: DataChannel 单条消息有大小限制（通常 16KB - 256KB）

**解决**: 
- 实现帧分片机制
- 大帧拆分成多个 chunk
- 接收端重组

### 2. 无序到达

**问题**: 虽然配置了 `ordered: true`，网络抖动可能导致延迟

**解决**:
- 使用 frame_id 和 chunk_index
- 接收端缓冲和排序
- 超时丢弃过旧帧

### 3. 背压控制

**问题**: 发送速度过快可能导致 DataChannel 缓冲区溢出

**解决**:
- 监控 `buffered_amount()`
- 动态调整发送速率
- 实现简单的流控

---

## 🚀 后续优化

完成基础版本后，可以考虑：

### Phase 3.5: 优化传输性能
- 实现不可靠模式（更低延迟）
- 自适应码率控制
- 前向纠错 (FEC)

### Phase 3.6: 网络质量监控
- RTT 测量
- 丢包率统计
- 带宽估算
- 拥塞检测

### Phase 3.7: 多流支持
- 音频流
- 屏幕共享流
- 文件传输

---

## 📝 实施时间估算

| 步骤 | 预计时间 | 说明 |
|------|---------|------|
| Step 1: 暴露 DataChannel | 30分钟 | 简单封装 |
| Step 2: VideoChannel | 1小时 | 核心传输逻辑 |
| Step 3: 帧序列化 | 30分钟 | 协议设计 |
| Step 4: 帧重组 | 45分钟 | 状态管理 |
| Step 5: 端到端集成 | 1小时 | 示例程序 |
| Step 6: 测试验证 | 30分钟 | 性能测试 |
| **总计** | **~4.5小时** | |

---

**维护人**: AI Assistant  
**创建时间**: 2026-06-28  
**预计完成**: 2026-06-28 (4.5小时)  
**下一里程碑**: Phase 3.5 性能优化 + Phase 3.6 网络监控
