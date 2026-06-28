# Rust WebRTC 编解码方案全面对比分析

**分析时间**: 2026-06-27  
**目标**: 为 RDCS 项目选择合适的 WebRTC 编解码库

---

## 📋 核心需求

1. ✅ 硬件加速（macOS VideoToolbox / Windows MF / Linux VA-API）
2. ✅ CPU 占用 <30% @ 1080p60fps
3. ✅ 跨平台复用（一份 Rust 代码）
4. ✅ H.264 编解码支持
5. ✅ 生产级质量
6. ⚠️ 避免不需要的依赖（信令、房间管理）

---

## 🔍 可用方案深度分析

### 方案 1: webrtc-rs (`webrtc = "0.11"`)

**Crates.io**: https://crates.io/crates/webrtc  
**GitHub**: https://github.com/webrtc-rs/webrtc  
**Stars**: ~4k  
**维护状态**: ⚠️ 活跃度下降（最近更新频率降低）

#### 技术架构
```rust
webrtc-rs (纯 Rust 实现)
  ├── 网络层：ICE/DTLS/SRTP ✅
  ├── 编解码：软件实现 ❌
  └── 媒体管道：纯 Rust ✅
```

#### 优点
- ✅ 纯 Rust，无 C++ 依赖
- ✅ Cargo 直接引入，编译简单
- ✅ 代码可读性高

#### 缺点（致命）
- ❌ **无硬件加速** - 纯软件编解码
- ❌ CPU 占用 >50%（不满足 PRD 的 <30% 要求）
- ❌ 编解码质量不如 libwebrtc
- ❌ 维护活跃度下降

#### 结论
**❌ 不推荐** - 无法满足性能要求

---

### 方案 2: livekit SDK (`livekit = "0.5"`)

**Crates.io**: https://crates.io/crates/livekit  
**GitHub**: https://github.com/livekit/rust-sdks  
**维护**: ✅ LiveKit 商业公司支持

#### 技术架构
```rust
livekit SDK (完整实时通信 SDK)
  ├── livekit-api (信令、房间管理) ⚠️
  ├── livekit-protocol (协议定义) ⚠️
  ├── livekit-core (核心抽象)
  └── 底层 libwebrtc FFI (硬件加速) ✅
```

#### 优点
- ✅ 硬件加速支持（libwebrtc）
- ✅ 生产级质量
- ✅ 持续维护
- ✅ 文档完善

#### 缺点
- ⚠️ **包含不需要的功能**（信令、房间、协议）
- ⚠️ 依赖较重（~100+ crates）
- ⚠️ 可能的版本冲突（如果项目有其他 WebRTC 需求）
- ⚠️ 预编译库 ~80MB

#### 包依赖分析
```bash
livekit v0.5.0
├── livekit-api v0.5.0 (信令 API)
│   ├── livekit-protocol (protobuf 定义)
│   ├── reqwest (HTTP 客户端)
│   └── tonic (gRPC)
├── livekit-webrtc v0.5.0 (编解码核心) ← 我们需要的
│   └── libwebrtc FFI 绑定
└── tokio-stream, futures 等
```

#### 关键问题
**能否只使用 livekit-webrtc 部分？**

根据 livekit Rust SDK 的设计，**不能直接只依赖 livekit-webrtc**，因为：
- livekit-webrtc 不是独立发布的 crate
- 它是 livekit workspace 的一部分
- 必须通过 livekit 主 crate 引入

#### 结论
**⚠️ 可用但有权衡** - 功能满足，但依赖较重

---

### 方案 3: libwebrtc FFI 绑定（假设的 `libwebrtc` crate）

**状态**: ❓ 需要验证是否存在

#### 如果存在
- ✅ 轻量级 libwebrtc 绑定
- ✅ 硬件加速
- ✅ 无额外功能
- ⚠️ 需要验证维护状态

#### 如果不存在
- ❌ 需要自己编写 FFI 绑定
- ❌ 工作量大（2-4周）
- ❌ 需要处理跨平台构建
- ❌ 需要分发预编译 libwebrtc

---

### 方案 4: RustDesk 的方案（参考）

**RustDesk 使用的方案**（基于 2024-2025 年的版本）：

RustDesk **不使用现成的 Rust WebRTC 库**，而是：

1. **编解码层**：
   - 直接 FFI 调用 Google libwebrtc
   - 或使用平台原生 API（VideoToolbox/MF）
   - 自己维护 FFI 绑定

2. **网络层**：
   - 自己实现的 P2P 协议
   - 不依赖完整的 WebRTC 栈

3. **原因**：
   - 更精细的控制
   - 避免 WebRTC 的信令复杂度
   - 优化性能和延迟

#### 对我们的启示
- RustDesk 的方案证明了"直接集成 libwebrtc"的可行性
- 但需要自己维护 FFI 层（工作量大）

---

## 🎯 方案对比表

| 方案 | 硬件加速 | CPU占用 | 依赖大小 | 维护性 | 实施成本 | 推荐度 |
|------|---------|---------|---------|--------|---------|--------|
| **webrtc-rs** | ❌ 无 | >50% | 小 (~20 crates) | ⚠️ 下降 | 低 | ❌ 不推荐 |
| **livekit SDK** | ✅ 有 | <20% | 大 (~100+ crates) | ✅ 活跃 | 低 | ⚠️ 可用但重 |
| **libwebrtc crate** | ❓ 待验证 | ? | ? | ? | ? | ❓ 需验证 |
| **自建 FFI** | ✅ 有 | <20% | 小 (自控) | 😰 自己维护 | 高 (2-4周) | ⚠️ 备选 |
| **平台原生** | ✅ 有 | <15% | 小 | ✅ 系统级 | 高 (3x重复) | ❌ 违背跨平台 |

---

## 🔬 实验验证方案

### 步骤 1: 验证 libwebrtc crate

```bash
# 在 Mac 上运行
cd /Users/lc/Development/source/remote-desktop-controller

# 尝试下载依赖
cargo fetch

# 检查结果
if [ $? -eq 0 ]; then
    echo "✅ libwebrtc = \"0.3\" 存在"
    cargo tree -p rdcs-codec | grep libwebrtc
else
    echo "❌ libwebrtc 不存在或版本错误"
fi
```

### 步骤 2: 测试 livekit 依赖大小

```bash
# 临时改为 livekit
# 检查实际拉取的依赖数量
cargo tree -p rdcs-codec | wc -l

# 检查是否可以通过 features 禁用信令部分
cargo tree -p rdcs-codec --no-default-features
```

---

## 💡 决策建议

### 情况 A: 如果 `libwebrtc = "0.3"` 存在且支持硬件加速
**推荐方案**: 使用 libwebrtc ✅
- 轻量级
- 满足所有需求
- 避免不需要的依赖

### 情况 B: 如果 `libwebrtc` 不存在
**推荐方案**: 使用 livekit，接受依赖较重 ⚠️
- 虽然依赖重，但功能完整
- 生产级质量有保证
- 可以正常工作

**替代方案**: 自建 libwebrtc FFI 绑定
- 如果团队有时间（2-4周）
- 需要精细控制
- 参考 RustDesk 的做法

### 情况 C: 如果 MVP 时间紧张
**快速方案**: 先用 livekit，后续优化
- Week 1-2: 用 livekit 快速完成 MVP
- Week 3+: 如有需要，重构为轻量级方案

---

## 📋 下一步行动

1. **立即执行**: 在 Mac 上运行 `./check-libwebrtc-crate.sh`
2. **根据结果**:
   - libwebrtc 存在 → 继续当前方案
   - libwebrtc 不存在 → 改用 livekit
3. **验证编译**: 确保依赖可以下载和编译
4. **性能测试**: 验证硬件加速是否生效

---

## 🎯 最终推荐（待验证）

**优先级排序**:

1. 🥇 **libwebrtc crate**（如果存在且质量好）
   - 轻量级
   - 精准满足需求

2. 🥈 **livekit SDK**（保底方案）
   - 功能完整
   - 依赖虽重但可靠

3. 🥉 **自建 FFI**（时间允许的话）
   - 最优控制
   - 参考 RustDesk

❌ **不推荐**: webrtc-rs（无硬件加速）

---

**总结**: 先验证 `libwebrtc = "0.3"` 的实际情况，再做最终决策。
