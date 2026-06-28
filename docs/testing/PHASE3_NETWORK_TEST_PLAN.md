# Phase 3.2 网络环境验证计划

**日期**: 2026-06-28  
**状态**: 进行中  
**前置条件**: Phase 3.1 ICE 连接成功 ✅

---

## 🎯 测试目标

验证 ICE 连接在不同网络环境下的可靠性和性能，为生产部署提供数据支持。

---

## 📋 测试场景

### 场景 1：同一局域网直连 ✅ (已测试)

**网络拓扑**:
```
Peer A (192.168.31.50) ←→ 路由器 ←→ Peer B (192.168.31.50)
```

**预期结果**:
- 使用 Host 候选直连
- 延迟 < 5ms
- 不需要 STUN

**实际结果**:
- ✅ 连接成功
- 候选类型: Host + Srflx (STUN 也成功)
- 延迟: < 1s

---

### 场景 2：不同局域网（STUN 穿透）

**网络拓扑**:
```
Peer A (NAT A) ←→ Internet ←→ (NAT B) Peer B
       ↓                              ↓
   STUN Server                   STUN Server
```

**测试方法**:
1. 在两台不同网络的机器上运行
2. 或使用移动热点模拟不同网络

**预期结果**:
- 使用 Srflx (STUN reflexive) 候选
- 延迟 < 100ms (取决于网络距离)
- NAT 类型: Full Cone / Restricted Cone

**测试步骤**:
```bash
# 机器 A
cargo run -p rdcs-connection --example ice_p2p_test

# 机器 B (需要修改测试为客户端/服务器模式)
# 或使用信令服务器交换 SDP
```

**当前限制**: 需要实现信令服务器来交换 SDP

---

### 场景 3：Symmetric NAT（需要 TURN）

**网络拓扑**:
```
Peer A (Symmetric NAT) ←→ Internet ←→ (Symmetric NAT) Peer B
              ↓                                ↓
          TURN Relay ←─────────────────────→ TURN Relay
```

**NAT 特性**:
- 不同目标 IP:Port 分配不同的公网映射
- STUN 无法穿透
- 必须使用 TURN 中继

**预期结果**:
- STUN 候选无法建立连接
- 回退到 Relay 候选
- 延迟增加 (经过中继)

**前置条件**: 部署 TURN 服务器 (Phase 3.4)

---

### 场景 4：NAT 类型检测

**目标**: 自动检测当前网络的 NAT 类型

**方法**: 使用 `rdcs-nat-test` crate

```bash
cargo run -p rdcs-nat-test
```

**预期输出**:
```
NAT Type: Full Cone NAT
P2P Possible: Yes
Reflexive Address: 116.76.205.196:xxxxx
```

---

## 🧪 测试工具

### 1. 现有工具

**ice_p2p_test**:
- 位置: `crates/rdcs-connection/examples/ice_p2p_test.rs`
- 用途: 单机模拟 P2P 连接
- 限制: 无法真正测试跨网络场景

**nat_detector**:
- 位置: `crates/rdcs-nat-test/src/nat_detector.rs`
- 用途: 检测 NAT 类型
- 功能: STUN Binding 请求测试

### 2. 需要新增的工具

#### A. ICE 客户端/服务器模式

**架构**:
```
ice_server.rs:
  - 启动监听
  - 生成 Offer
  - 打印 Offer JSON
  - 等待接收 Answer JSON
  - 建立连接

ice_client.rs:
  - 接收 Offer JSON (stdin)
  - 生成 Answer
  - 打印 Answer JSON
  - 建立连接
```

**使用流程**:
```bash
# 机器 A
./ice_server > offer.json

# 复制 offer.json 到机器 B

# 机器 B
./ice_client < offer.json > answer.json

# 复制 answer.json 到机器 A

# 机器 A
./ice_server < answer.json

# 等待连接建立
```

#### B. 简易信令服务器

**技术栈**: Actix-web + WebSocket

**功能**:
- 注册 Peer ID
- 转发 SDP Offer/Answer
- 中继 ICE 候选

**API**:
```
POST /register -> { peer_id }
POST /offer -> { to_peer_id, sdp_offer }
POST /answer -> { to_peer_id, sdp_answer }
WebSocket /ws -> 实时候选交换
```

---

## 📊 测试指标

### 连接成功率

| 场景 | 目标 | 当前 |
|------|------|------|
| 同局域网 | 100% | 100% ✅ |
| 不同局域网 (STUN) | > 95% | 待测试 |
| Symmetric NAT (TURN) | > 90% | 待实现 |

### 连接建立时间

| 场景 | 目标 | 当前 |
|------|------|------|
| 同局域网 | < 1s | < 1s ✅ |
| 不同局域网 | < 3s | 待测试 |
| TURN 中继 | < 5s | 待实现 |

### 网络质量

| 指标 | 测量方法 |
|------|----------|
| RTT 延迟 | STUN Binding 响应时间 |
| 丢包率 | 连续 ping 测试 |
| 带宽 | 视频流实际传输速率 |

---

## 🔧 实现计划

### Step 1: 创建 ICE 客户端/服务器工具 (2小时)

**文件**:
```
crates/rdcs-connection/examples/
├── ice_server.rs    # 服务器端
└── ice_client.rs    # 客户端
```

**功能**:
- JSON 格式的 SDP 交换
- 支持手动复制粘贴
- 连接状态显示
- 基本延迟测试

### Step 2: 网络环境测试 (1小时)

**测试矩阵**:
```
[本地 WiFi] → [本地 WiFi]        ✅ 已测试
[本地 WiFi] → [手机热点]         ⏳ 待测试
[公司网络] → [家庭网络]          ⏳ 待测试
[VPN] → [VPN]                    ⏳ 可选
```

### Step 3: 性能基准测试 (1小时)

**测试内容**:
- 连接建立时间统计 (10次重复)
- RTT 延迟测量
- 候选优先级验证
- 网络切换恢复测试

### Step 4: 文档和报告 (30分钟)

- 更新测试报告
- 记录问题和优化点
- 制定改进计划

**总计**: ~4.5 小时

---

## 🚧 当前限制

### 1. 单机测试

**问题**: `ice_p2p_test` 在同一台机器上运行，无法真正测试跨网络

**影响**: 无法验证 STUN 穿透效果

**解决**: 实现客户端/服务器工具

### 2. 缺少信令机制

**问题**: 无自动化方式交换 SDP

**影响**: 需要手动复制粘贴 JSON

**解决方案**:
- 短期: 手动复制粘贴（可接受）
- 长期: 实现 WebSocket 信令服务器

### 3. TURN 未部署

**问题**: 无法测试 Symmetric NAT 场景

**影响**: 连接成功率可能低于 100%

**计划**: Phase 3.4 部署 coturn

---

## ✅ 验收标准

Phase 3.2 完成标准:

- [ ] 创建 ice_server 和 ice_client 工具
- [ ] 在至少 2 个不同网络环境测试成功
- [ ] 记录各场景的连接时间和成功率
- [ ] 生成网络测试报告
- [ ] 识别并记录优化点

---

## 🔄 下一步 (Phase 3.3)

完成 Phase 3.2 后进入:

**Phase 3.3: DTLS 加密传输**
- 生成真实 DTLS 证书
- 提取 fingerprint
- 建立加密 DataChannel
- 端到端加密验证

---

**维护人**: AI Assistant  
**创建日期**: 2026-06-28  
**预计完成**: 2026-06-28 (4.5小时)
