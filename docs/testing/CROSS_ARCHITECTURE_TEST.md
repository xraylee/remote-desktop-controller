# 跨架构 ICE 连接测试指南

**日期**: 2026-06-28  
**目标**: 验证 ARM (Apple Silicon) ↔ Intel Mac 的 P2P 连接

---

## 🎯 测试目标

验证以下功能：
1. ✅ 跨架构兼容性（ARM ↔ Intel）
2. ✅ 真实网络 STUN 穿透
3. ✅ DTLS 加密连接
4. ✅ 连接时间和延迟
5. ✅ ICE 候选收集
6. ✅ 候选对优先级选择

---

## 📋 前置要求

### 两台 Mac
- **Mac A**: Intel Mac（作为 Server/Offerer）
- **Mac B**: Apple Silicon Mac（作为 Client/Answerer）

### 软件要求
- Rust toolchain 已安装
- 网络可以访问 Google STUN 服务器
- 两台机器能互相访问（同一局域网或公网）

---

## 🚀 测试步骤

### 第一步：准备 Intel Mac (Server)

在 Intel Mac 上执行：

```bash
# 1. 进入项目目录
cd /path/to/remote-desktop-controller

# 2. 拉取最新代码（如果是 git 仓库）
git pull

# 3. 运行 ICE Server
cargo run -p rdcs-connection --example ice_server
```

**预期输出**：
```
========================================
ICE Server (Offerer)
========================================

Step 1: Creating ICE agent...
✅ ICE agent created

Step 2: Creating offer and gathering candidates...
✅ Offer created with X candidates

========================================
📋 OFFER (copy this to the client)
========================================
{
  "session_id": "...",
  "ufrag": "...",
  "pwd": "...",
  "fingerprint": "...",
  "candidates": [...]
}
========================================

Step 3: Waiting for ANSWER...
Paste the Answer JSON and press Ctrl+D (Linux/Mac) or Ctrl+Z Enter (Windows):
```

**⚠️ 重要**: 
- 复制完整的 JSON（从 `{` 到 `}`）
- 保存到文本编辑器或直接发送到 Apple Silicon Mac

---

### 第二步：准备 Apple Silicon Mac (Client)

在当前 Mac 上执行：

```bash
# 1. 进入项目目录
cd /Users/lc/Development/source/remote-desktop-controller

# 2. 运行 ICE Client
cargo run -p rdcs-connection --example ice_client
```

**预期输出**：
```
========================================
ICE Client (Answerer)
========================================

Step 1: Waiting for OFFER...
Paste the Offer JSON and press Ctrl+D (Linux/Mac) or Ctrl+Z Enter (Windows):
```

---

### 第三步：交换连接信息

#### 3.1 从 Intel Mac 复制 Offer 到 Apple Silicon Mac

1. 在 Intel Mac 的输出中复制完整的 JSON Offer
2. 粘贴到 Apple Silicon Mac 的 ice_client 窗口
3. 按 `Ctrl+D` 结束输入

**Apple Silicon Mac 预期输出**：
```
✅ Offer received with X candidates

Step 2: Creating ICE agent...
✅ ICE agent created

Step 3: Setting remote offer...
✅ Remote offer set

Step 4: Gathering candidates...
✅ Gathered X candidates

Step 5: Creating answer...
========================================
📋 ANSWER (copy this back to the server)
========================================
{
  "session_id": "...",
  "ufrag": "...",
  "pwd": "...",
  "fingerprint": "...",
  "candidates": [...]
}
========================================

Step 6: Adding remote candidates...
✅ Remote candidates added

Step 7: Waiting for ICE connection...
ICE state: ...
```

#### 3.2 从 Apple Silicon Mac 复制 Answer 到 Intel Mac

1. 在 Apple Silicon Mac 的输出中复制完整的 JSON Answer
2. 粘贴到 Intel Mac 的 ice_server 窗口（仍在等待输入）
3. 按 `Ctrl+D` 结束输入

---

### 第四步：等待连接建立

两边都会显示 ICE 状态变化：

```
ICE state: New
ICE state: Checking
ICE state: Connected  ← 成功！
```

**成功输出**：
```
========================================
✅ ICE CONNECTION ESTABLISHED!
========================================

Connection successful. Press Ctrl+C to exit.
```

---

## 📊 数据收集

### 记录以下指标

#### 1. 候选信息
从 Offer/Answer JSON 中提取：

**Intel Mac 候选**：
```json
{
  "candidates": [
    {
      "typ": "Host",      // 本地候选
      "address": "...",
      "port": ...
    },
    {
      "typ": "Srflx",     // STUN 反射候选
      "address": "...",
      "port": ...
    }
  ]
}
```

**Apple Silicon Mac 候选**：
```json
{
  "candidates": [...]
}
```

记录：
- Host 候选数量
- Srflx 候选数量
- 是否有 Relay 候选

#### 2. 时间指标

**收集间隔时间**：
- Intel Mac: 从启动到 "Offer created" 的时间
- Apple Silicon Mac: 从接收 Offer 到 "Gathered candidates" 的时间

**连接建立时间**：
- 从 "Waiting for ICE connection" 到 "ICE CONNECTION ESTABLISHED" 的时间
- 在两边分别计时

#### 3. 选中的候选对

在日志中查找类似信息（可能需要增加日志级别）：
```
Selected candidate pair: 
  Local: 192.168.1.x:xxxxx (Host)
  Remote: 192.168.1.y:yyyyy (Host)
```

---

## 🔍 验证清单

- [ ] Intel Mac 成功启动 ice_server
- [ ] Intel Mac 输出 Offer JSON
- [ ] Apple Silicon Mac 成功接收并解析 Offer
- [ ] Apple Silicon Mac 输出 Answer JSON
- [ ] Intel Mac 成功接收并解析 Answer
- [ ] 两边都显示 "ICE state: Checking"
- [ ] 两边都显示 "ICE state: Connected"
- [ ] 两边都显示 "ICE CONNECTION ESTABLISHED!"
- [ ] 连接建立时间 < 10 秒

---

## ❌ 常见问题

### 问题 1: cargo: command not found

**解决**：
```bash
# 检查 Rust 是否安装
rustc --version

# 如果未安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 重新加载环境
source $HOME/.cargo/env
```

### 问题 2: 编译失败

**解决**：
```bash
# 清理并重新编译
cargo clean
cargo build -p rdcs-connection --examples
```

### 问题 3: 连接超时

**可能原因**：
1. 防火墙阻止 UDP 流量
2. 无法访问 STUN 服务器
3. NAT 类型不兼容

**调试**：
```bash
# 测试 STUN 服务器连接
nc -u -v stun.l.google.com 19302

# 检查防火墙设置
# macOS: System Preferences -> Security & Privacy -> Firewall
```

### 问题 4: JSON 解析错误

**解决**：
- 确保复制完整的 JSON（从 `{` 到 `}`）
- 不要包含日志行
- 检查 JSON 格式是否有效

---

## 📝 测试报告模板

完成测试后，记录以下信息：

```markdown
# 跨架构 ICE 连接测试报告

**测试日期**: 2026-06-28  
**测试人员**: [Your Name]

## 环境信息

**Intel Mac (Server)**:
- CPU: [Intel Core iX]
- OS: [macOS version]
- IP: [Private IP]

**Apple Silicon Mac (Client)**:
- CPU: [M1/M2/M3]
- OS: [macOS version]
- IP: [Private IP]

**网络环境**:
- [ ] 同一局域网
- [ ] 不同网络
- [ ] VPN 连接

## 测试结果

### ✅ 成功指标
- 候选收集时间: [X] 秒
- 连接建立时间: [Y] 秒
- 选中候选对类型: [Host/Srflx/Relay]
- STUN 服务器: [stun.l.google.com:19302]

### 📊 候选统计
**Intel Mac**:
- Host 候选: [数量]
- Srflx 候选: [数量]
- 总计: [数量]

**Apple Silicon Mac**:
- Host 候选: [数量]
- Srflx 候选: [数量]
- 总计: [数量]

### 🔗 连接详情
- 本地地址: [IP:Port]
- 远程地址: [IP:Port]
- 协议: [UDP]

## 问题记录

[遇到的任何问题和解决方法]

## 结论

- [ ] 跨架构连接成功
- [ ] STUN 穿透工作正常
- [ ] 连接时间在可接受范围内
- [ ] 可以进入下一阶段开发
```

---

## 🎯 下一步

测试成功后，可以继续：

1. **添加数据传输测试**
   - 在连接建立后发送测试数据
   - 验证数据完整性
   - 测量吞吐量和延迟

2. **添加性能测试**
   - 大量数据传输
   - 长时间连接稳定性
   - 网络波动恢复

3. **集成到编解码器**
   - 通过 ICE 连接传输视频流
   - 端到端测试

---

## 📚 相关文档

- [ICE 实现文档](../decisions/WEBRTC_ARCHITECTURE.md)
- [测试指南](TESTING_GUIDELINES.md)
- [rdcs-connection README](../../crates/rdcs-connection/README.md)

---

**维护人**: RDCS Team  
**最后更新**: 2026-06-28
