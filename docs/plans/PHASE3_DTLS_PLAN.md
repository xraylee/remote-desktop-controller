# Phase 3.3 DTLS 加密传输实现计划

**日期**: 2026-06-28  
**状态**: 进行中  
**目标**: 修复 DTLS fingerprint 错误，建立加密连接

---

## 🎯 目标

修复当前的 DTLS 握手失败问题：
```
Failed to start manager dtls: remote certificate does not match any fingerprint
```

建立真正的端到端加密 DataChannel，为视频流传输做准备。

---

## 🔍 问题分析

### 当前状态

**成功部分**:
- ✅ ICE 连接建立成功
- ✅ STUN 候选收集正常
- ✅ ICE 凭据验证通过

**失败部分**:
- ❌ DTLS 握手失败
- ❌ DataChannel 无法建立
- ❌ 视频流无法传输

### 根本原因

在 `format_sdp_offer` 和 `format_sdp_answer` 中使用了假的 fingerprint：

```rust
sdp.push_str("a=fingerprint:sha-256 00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00\r\n");
```

webrtc-rs 会生成真实的 DTLS 证书和 fingerprint，我们需要从生成的 SDP 中提取真实值。

---

## 🔧 解决方案

### 方案 1: 从 SDP 提取 fingerprint（推荐）✅

**原理**: webrtc-rs 在 `create_offer()` 和 `create_answer()` 时已生成真实证书和 fingerprint

**实现**:
```rust
// 在 create_offer 中
let offer = peer_connection.create_offer(None).await?;
peer_connection.set_local_description(offer.clone()).await?;

// 从生成的 SDP 中提取 fingerprint
let fingerprint = extract_fingerprint(&offer.sdp)?;

// 存储到 SdpOffer 结构
Ok(SdpOffer {
    session_id,
    ufrag,
    pwd,
    fingerprint,  // 新增字段
    candidates,
})
```

**优点**:
- 简单直接
- 使用 webrtc-rs 内置证书生成
- 无需额外配置

**缺点**:
- 依赖 SDP 解析
- 证书管理由 webrtc-rs 控制

---

### 方案 2: 自己生成证书（复杂）

**原理**: 使用 `rcgen` 或 `openssl` 生成自签名证书

**复杂度**: 高，需要处理证书生成、存储、交换

**不推荐**: webrtc-rs 已内置证书管理

---

## 📋 实施步骤

### Step 1: 扩展 SDP 结构 (15分钟)

**修改文件**: `crates/rdcs-connection/src/ice.rs`

```rust
/// An SDP offer containing local ICE candidates and credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdpOffer {
    pub session_id: String,
    pub ufrag: String,
    pub pwd: String,
    pub fingerprint: String,  // 新增
    pub candidates: Vec<IceCandidate>,
}

/// An SDP answer containing remote ICE candidates and credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdpAnswer {
    pub session_id: String,
    pub ufrag: String,
    pub pwd: String,
    pub fingerprint: String,  // 新增
    pub candidates: Vec<IceCandidate>,
}
```

---

### Step 2: 实现 fingerprint 提取 (20分钟)

**修改文件**: `crates/rdcs-connection/src/real_ice_agent.rs`

```rust
/// Extract DTLS fingerprint from SDP string.
fn extract_fingerprint(sdp: &str) -> Result<String, ConnectionError> {
    for line in sdp.lines() {
        if line.starts_with("a=fingerprint:") {
            // 格式: a=fingerprint:sha-256 XX:XX:XX:...
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() == 2 {
                return Ok(parts[1].trim().to_string());
            }
        }
    }
    Err(ConnectionError::IceError("Failed to extract fingerprint from SDP".to_string()))
}
```

---

### Step 3: 更新 create_offer (10分钟)

```rust
fn create_offer(&self) -> Result<SdpOffer, ConnectionError> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let offer = peer_connection.create_offer(None).await?;
            peer_connection.set_local_description(offer.clone()).await?;

            // 等待候选收集...

            // 提取凭据和 fingerprint
            let (ufrag, pwd) = extract_ice_credentials(&offer.sdp)?;
            let fingerprint = extract_fingerprint(&offer.sdp)?;

            Ok(SdpOffer {
                session_id: self.session_id.clone(),
                ufrag,
                pwd,
                fingerprint,
                candidates,
            })
        })
    })
}
```

---

### Step 4: 更新 SDP 格式化函数 (15分钟)

```rust
fn format_sdp_offer(offer: &SdpOffer) -> String {
    let mut sdp = String::new();
    let session_id_numeric = offer.session_id.bytes().map(|b| b as u64).sum::<u64>();

    sdp.push_str("v=0\r\n");
    sdp.push_str(&format!("o=- {} 2 IN IP4 0.0.0.0\r\n", session_id_numeric));
    sdp.push_str("s=-\r\n");
    sdp.push_str("t=0 0\r\n");
    sdp.push_str(&format!("a=ice-ufrag:{}\r\n", offer.ufrag));
    sdp.push_str(&format!("a=ice-pwd:{}\r\n", offer.pwd));
    
    // 使用真实的 fingerprint
    sdp.push_str(&format!("a=fingerprint:sha-256 {}\r\n", offer.fingerprint));
    
    // ... rest of SDP
}

fn format_sdp_answer(answer: &SdpAnswer) -> String {
    // 同样使用 answer.fingerprint
    // ...
}
```

---

### Step 5: 更新测试示例 (10分钟)

**ice_p2p_test.rs**:
```rust
// 获取 Peer B 的 fingerprint
let local_desc = peer_b.peer_connection.local_description().await.unwrap();
let fingerprint_b = extract_fingerprint(&local_desc.sdp)?;

let answer = SdpAnswer {
    session_id: offer.session_id.clone(),
    ufrag: ufrag_b,
    pwd: pwd_b,
    fingerprint: fingerprint_b,  // 使用真实值
    candidates: candidates_b,
};
```

**ice_server.rs / ice_client.rs**:
- 更新 JSON 结构添加 `fingerprint` 字段

---

### Step 6: 测试验证 (20分钟)

```bash
# 1. 编译
cargo build -p rdcs-connection

# 2. 运行 ice_p2p_test
RUST_LOG=info cargo run -p rdcs-connection --example ice_p2p_test

# 3. 验证 DTLS 成功
# 应该看到:
#   - ICE connection state changed: connected
#   - peer connection state changed: connected  (不再是 failed)
#   - 无 "remote certificate does not match" 错误
```

---

## ✅ 成功标准

- [ ] 扩展 SdpOffer/SdpAnswer 结构
- [ ] 实现 fingerprint 提取函数
- [ ] 更新 create_offer 方法
- [ ] 更新 SDP 格式化函数
- [ ] 更新测试示例
- [ ] ice_p2p_test 通过且无 DTLS 错误
- [ ] PeerConnection 状态变为 Connected
- [ ] DataChannel 成功建立

---

## 📊 预期结果

### 修复前
```
ICE connection state changed: connected
peer connection state changed: failed
Failed to start manager dtls: remote certificate does not match any fingerprint
```

### 修复后
```
ICE connection state changed: connected
peer connection state changed: connected
✅ DTLS handshake successful
✅ DataChannel opened
```

---

## 🧪 测试场景

### 1. 本地 P2P (ice_p2p_test)
- 验证 DTLS 握手
- 验证 DataChannel 建立

### 2. 跨网络 (ice_server + ice_client)
- 在 Intel Mac 上运行 ice_server
- 在当前 Mac 上运行 ice_client
- 验证跨架构 DTLS 连接

### 3. DataChannel 消息传输
- 发送测试消息
- 验证加密传输

---

## 🚀 后续工作

完成 DTLS 后:

### Phase 3.4: 视频流集成
- 通过 DataChannel 传输视频帧
- 替换 TCP 直连为 P2P 加密通道
- 端到端延迟测试

### Phase 3.5: TURN 中继
- 部署 coturn 服务器
- 测试 Symmetric NAT 场景

### Phase 3.6: 生产优化
- 自适应码率
- 网络质量监控
- 连接失败重试

---

## 🎓 技术要点

### DTLS vs TLS

**DTLS (Datagram TLS)**:
- 基于 UDP
- 适合实时媒体传输
- WebRTC 标准协议

**TLS (Transport Layer Security)**:
- 基于 TCP
- 不适合低延迟场景

### Fingerprint 格式

```
a=fingerprint:sha-256 XX:XX:XX:XX:...:XX
```

- 算法: sha-256
- 格式: 32 字节的十六进制，用冒号分隔
- 示例: `A1:B2:C3:D4:E5:...`

### 证书验证流程

1. Offerer 生成证书，计算 fingerprint
2. Offerer 在 SDP Offer 中包含 fingerprint
3. Answerer 收到 Offer，验证 fingerprint
4. Answerer 生成自己的证书和 fingerprint
5. Answerer 在 SDP Answer 中包含 fingerprint
6. Offerer 收到 Answer，验证 fingerprint
7. 双方建立 DTLS 连接

---

**维护人**: AI Assistant  
**创建时间**: 2026-06-28  
**预计完成**: 1.5 小时  
**下一里程碑**: Intel Mac 跨架构集成测试
