# Phase 3.4 视频 DataChannel 传输 - 实现报告

**日期**: 2026-06-28  
**状态**: ✅ 实现完成，待测试  
**里程碑**: 视频帧通过加密 DataChannel 传输

---

## 🎯 实现目标

将视频帧传输从 TCP 迁移到 WebRTC DataChannel：

**架构变化**:
```
旧: 编码器 → TCP Socket → 网络 → TCP Socket → 解码器
新: 编码器 → DataChannel (DTLS) → P2P/NAT → DataChannel → 解码器
```

**优势**:
- ✅ 端到端加密 (DTLS)
- ✅ NAT 穿透 (ICE)
- ✅ 自动重连
- ✅ 拥塞控制
- ✅ 更低延迟（UDP）

---

## 📦 实现内容

### 1. 扩展 RealIceAgent 暴露 DataChannel

**文件**: `crates/rdcs-connection/src/real_ice_agent.rs`

**修改**:
```rust
pub struct RealIceAgent {
    peer_connection: Arc<RTCPeerConnection>,
    data_channel: Arc<Mutex<Option<Arc<RTCDataChannel>>>>,  // 新增
    // ...
}

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

**说明**:
- 之前创建的 `rdcs-control` DataChannel 被丢弃了
- 现在保存到 `data_channel` 字段中
- 通过 `get_data_channel()` 方法暴露给外部使用

---

### 2. 创建 VideoChannel 封装

**新文件**: `crates/rdcs-connection/src/video_channel.rs`

**核心功能**:
```rust
pub struct VideoChannel {
    data_channel: Arc<RTCDataChannel>,
    max_chunk_size: usize,  // 默认 16KB
}

impl VideoChannel {
    /// 发送视频帧（自动分片）
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

    /// 设置接收回调
    pub fn on_message<F>(&self, callback: F)
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static
    {
        let callback = Arc::new(callback);
        self.data_channel.on_message(Box::new(move |msg| {
            let data = msg.data.to_vec();
            callback(data);
            Box::pin(async {})
        }));
    }

    /// 检查缓冲区（用于背压控制）
    pub fn buffered_amount(&self) -> usize {
        self.data_channel.buffered_amount()
    }
}
```

**特点**:
- 自动处理大帧分片（>16KB）
- 简洁的发送/接收 API
- 支持背压监控

---

### 3. 实现帧序列化协议

**新文件**: `crates/rdcs-connection/src/frame_reassembler.rs`

**协议设计**:

```
Frame Message (8 + N bytes):
┌─────────────────────────────────────┐
│ Header (8 bytes)                    │
├─────────────────────────────────────┤
│ frame_id: u32     (4 bytes)         │
│ flags: u8         (1 byte)          │
│   bit 0: is_keyframe                │
│   bit 1-7: reserved                 │
│ chunk_index: u8   (1 byte)          │
│ total_chunks: u8  (1 byte)          │
│ reserved: u8      (1 byte)          │
├─────────────────────────────────────┤
│ Payload (N bytes)                   │
└─────────────────────────────────────┘
```

**FrameHeader 结构**:
```rust
pub struct FrameHeader {
    pub frame_id: u32,
    pub is_keyframe: bool,
    pub chunk_index: u8,
    pub total_chunks: u8,
}

impl FrameHeader {
    pub const SIZE: usize = 8;

    pub fn serialize(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0..4].copy_from_slice(&self.frame_id.to_be_bytes());
        buf[4] = if self.is_keyframe { 1 } else { 0 };
        buf[5] = self.chunk_index;
        buf[6] = self.total_chunks;
        buf[7] = 0; // reserved
        buf
    }

    pub fn deserialize(buf: &[u8]) -> Result<Self, FrameError> {
        if buf.len() < 8 {
            return Err(FrameError::InvalidHeader);
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

### 4. 实现帧重组器

**FrameReassembler 功能**:
```rust
pub struct FrameReassembler {
    pending_frames: HashMap<u32, PartialFrame>,
    max_pending: usize,
}

impl FrameReassembler {
    /// 添加接收到的 chunk
    pub fn add_chunk(
        &mut self,
        header: FrameHeader,
        data: Vec<u8>,
    ) -> Option<(u32, Vec<u8>, bool)> {
        // 1. 验证 chunk_index
        // 2. 存储 chunk
        // 3. 检查是否完整
        // 4. 重组并返回完整帧
    }
}
```

**特性**:
- ✅ 支持乱序接收
- ✅ 自动去重
- ✅ 超时清理（max_pending 限制）
- ✅ 返回完整帧：`(frame_id, data, is_keyframe)`

---

### 5. 集成测试示例

**新文件**: `crates/rdcs-connection/examples/video_datachannel_test.rs`

**测试流程**:
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建两个 ICE agents
    let mut peer_a = RealIceAgent::new(ice_servers).await?;
    let mut peer_b = RealIceAgent::new(ice_servers).await?;

    // 2. 交换 SDP offer/answer
    let offer = peer_a.create_offer()?;
    peer_b.set_remote_offer(&offer)?;
    let answer = peer_b.create_answer()?;
    peer_a.handle_answer(answer)?;

    // 3. 等待 ICE 连接
    wait_for_connection(&peer_a, &peer_b).await?;

    // 4. 获取 DataChannel
    let dc_a = peer_a.get_data_channel()?;
    let dc_b = peer_b.get_data_channel()?;

    // 5. 创建 VideoChannel
    let video_tx = VideoChannel::new(dc_a);
    let video_rx = VideoChannel::new(dc_b);

    // 6. 设置接收器
    let mut reassembler = FrameReassembler::new(10);
    video_rx.on_message(move |chunk| {
        let header = FrameHeader::deserialize(&chunk[..8])?;
        let data = chunk[8..].to_vec();

        if let Some((frame_id, complete_frame, is_key)) = 
            reassembler.add_chunk(header, data) {
            println!("Received frame {}: {} bytes", frame_id, complete_frame.len());
        }
    });

    // 7. 发送测试帧
    send_frame_with_header(&video_tx, 1, &small_frame, true).await?;
    send_frame_with_header(&video_tx, 2, &medium_frame, false).await?;
    send_frame_with_header(&video_tx, 3, &large_frame, false).await?;

    Ok(())
}
```

**测试用例**:
- ✅ 小帧 (1KB) - 单个 chunk
- ✅ 中帧 (32KB) - 多个 chunks
- ✅ 大帧 (100KB) - 多个 chunks

---

## 📝 文件清单

### 新增文件
1. `crates/rdcs-connection/src/video_channel.rs` - VideoChannel 封装
2. `crates/rdcs-connection/src/frame_reassembler.rs` - 帧重组器
3. `crates/rdcs-connection/examples/video_datachannel_test.rs` - 测试示例

### 修改文件
1. `crates/rdcs-connection/src/real_ice_agent.rs`
   - 添加 `data_channel` 字段
   - 添加 `get_data_channel()` 方法

2. `crates/rdcs-connection/src/lib.rs`
   - 导出 `VideoChannel`
   - 导出 `FrameReassembler`, `FrameHeader`, `FrameError`

3. `crates/rdcs-connection/Cargo.toml`
   - 添加 `bytes = "1.5"` 依赖

---

## 🧪 单元测试

### FrameHeader 序列化测试
```rust
#[test]
fn test_frame_header_serialization() {
    let header = FrameHeader {
        frame_id: 42,
        is_keyframe: true,
        chunk_index: 2,
        total_chunks: 5,
    };

    let bytes = header.serialize();
    let decoded = FrameHeader::deserialize(&bytes).unwrap();

    assert_eq!(decoded.frame_id, 42);
    assert_eq!(decoded.is_keyframe, true);
    assert_eq!(decoded.chunk_index, 2);
    assert_eq!(decoded.total_chunks, 5);
}
```

### FrameReassembler 测试
```rust
#[test]
fn test_multi_chunk_frame() {
    let mut reassembler = FrameReassembler::new(10);

    // 乱序添加 chunks
    add_chunk(header2, chunk2); // chunk 1
    add_chunk(header3, chunk3); // chunk 2
    let result = add_chunk(header1, chunk1); // chunk 0 - 完成

    assert!(result.is_some());
    let (frame_id, data, _) = result.unwrap();
    assert_eq!(data, vec![1,2,3,4,5,6,7,8,9]);
}
```

**测试覆盖**:
- ✅ 单 chunk 帧
- ✅ 多 chunk 帧
- ✅ 乱序接收
- ✅ 重复 chunk 去重
- ✅ 超过 max_pending 清理

---

## 🔧 技术细节

### DataChannel 消息大小限制

**问题**: DataChannel 有消息大小限制（通常 16KB - 256KB）

**解决方案**:
- 使用保守的 16KB 块大小
- 自动分片大于 16KB 的帧
- 协议头标记 chunk_index 和 total_chunks

### 可靠传输

**配置**:
```rust
let dc = peer_connection.create_data_channel(
    "rdcs-control",
    None  // 默认: ordered=true, reliable=true
).await?;
```

**特性**:
- 有序传输（ordered: true）
- 可靠传输（类似 TCP）
- 自动重传丢失的消息

### 背压控制

**API**:
```rust
let buffered = video_channel.buffered_amount();
if buffered > THRESHOLD {
    // 暂停编码或降低码率
}
```

**未来优化**:
- 监控 `buffered_amount()`
- 动态调整编码码率
- 实现流控机制

---

## ⚠️ 当前限制

### 1. 同步 API 限制

`get_data_channel()` 使用 `block_in_place` 在同步上下文中调用：
```rust
pub fn get_data_channel(&self) -> Result<Arc<RTCDataChannel>, ConnectionError> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // ...
        })
    })
}
```

**原因**: `IceAgent` trait 是同步的，但 `Mutex::lock()` 是异步的

**影响**: 可能阻塞当前线程

**未来**: 考虑异步化 `IceAgent` trait

### 2. 单一 DataChannel

当前只使用一个 DataChannel (`rdcs-control`)。

**未来扩展**:
- 视频流专用 channel
- 音频流 channel
- 控制信令 channel
- 文件传输 channel

### 3. 无序传输未实现

当前使用可靠有序传输（类似 TCP）。

**未来优化**:
```rust
let dc = peer_connection.create_data_channel(
    "video",
    Some(RTCDataChannelInit {
        ordered: Some(false),      // 无序
        max_retransmits: Some(0),  // 不重传
        ..Default::default()
    })
).await?;
```

**收益**: 更低延迟（~10-20ms）

---

## ✅ 验收标准

- [x] RealIceAgent 暴露 `get_data_channel()` API
- [x] VideoChannel 实现发送/接收
- [x] FrameHeader 协议定义和序列化
- [x] FrameReassembler 实现分片重组
- [x] 端到端测试示例
- [ ] 实际编译通过（待本地测试）
- [ ] 集成测试通过
- [ ] 延迟测试 < 100ms

---

## 🚀 下一步工作

### 立即: 编译和测试

```bash
# 1. 编译
cargo build -p rdcs-connection

# 2. 运行单元测试
cargo test -p rdcs-connection

# 3. 运行集成测试
RUST_LOG=info cargo run -p rdcs-connection --example video_datachannel_test
```

### 后续: 集成真实编解码器

**目标**: 将 OpenH264 编码器输出连接到 VideoChannel

**步骤**:
1. 修改 `rdcs-capture` 发送端使用 VideoChannel
2. 修改 `rdcs-capture` 接收端使用 FrameReassembler
3. 端到端延迟测试
4. 性能优化

### Phase 3.5: 传输优化

- 实现不可靠无序模式（更低延迟）
- 自适应码率控制
- 前向纠错 (FEC)
- 带宽估算

### Phase 3.6: 网络监控

- RTT 测量
- 丢包率统计
- 拥塞检测
- QoS 报告

---

## 📊 预期性能

| 指标 | 目标 | 说明 |
|------|------|------|
| 端到端延迟 | < 100ms | 编码 + 传输 + 解码 |
| 帧率 | 30 fps | 稳定 |
| CPU 使用率 | < 30% | 单核 |
| 内存占用 | < 100MB | 不含系统缓冲 |
| 最大帧大小 | 1MB | 分片处理 |

---

## 🔗 相关文档

- [Phase 3.4 实施计划](../plans/PHASE3_VIDEO_DATACHANNEL_PLAN.md)
- [Phase 3.3 DTLS 成功报告](PHASE3_DTLS_SUCCESS_REPORT.md)
- [Phase 3 ICE 连接报告](PHASE3_ICE_SUCCESS_REPORT.md)

---

**维护人**: AI Assistant  
**完成日期**: 2026-06-28  
**状态**: 实现完成，待编译测试  
**下一里程碑**: 本地编译测试 → 集成 OpenH264 编解码器
