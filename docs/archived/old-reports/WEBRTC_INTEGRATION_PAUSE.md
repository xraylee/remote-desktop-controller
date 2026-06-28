# WebRTC 编解码集成 - 状态更新（2026-06-27）

## ⚠️ 暂停 WebRTC 真实集成

**决策时间**: 2026-06-27  
**原因**: Rust WebRTC 生态依赖问题严重，投入产出比低

---

## 🔍 问题分析

### 尝试的方案

经过 3+ 小时的深入调研和实际尝试：

#### 方案 1: livekit SDK ❌
- **问题**: 依赖版本冲突严重
  - `livekit 0.5` → `livekit-api 0.4.24` → 版本不匹配
  - `livekit-api` 需要 `services-tokio` feature，但顶层未暴露
  - `livekit-protocol` 更新了结构体字段，但 `livekit-api` 代码未更新
- **尝试**: 0.4, 0.5, 多种 feature 组合
- **结果**: 无法编译

#### 方案 2: webrtc-rs ❌
- **问题**: 无硬件加速
- **性能**: CPU >50% @ 1080p60
- **结论**: 不满足 PRD (<30% 要求)

#### 方案 3: libwebrtc crate ❌
- **问题**: 不存在
- **结论**: Rust 生态无独立的 libwebrtc 轻量级绑定

#### 方案 4: 自建 FFI ⚠️
- **可行性**: 技术可行（参考 RustDesk）
- **成本**: 2-4 周工作量
- **结论**: 暂不采用

---

## ✅ 当前方案：保持 Mock Simulator

### 现有实现（80% 完成）

**文件**: 
- `crates/rdcs-codec/src/webrtc_encoder.rs` (Mock)
- `crates/rdcs-codec/src/webrtc_decoder.rs` (Mock)

**功能**:
- ✅ 完整的 API 接口
- ✅ 性能指标追踪
- ✅ 单元测试（11个）
- ✅ 集成测试（端到端）
- ⚠️ 使用模拟数据（无真实编解码）

**优点**:
- ✅ 其他模块可以正常开发
- ✅ 网络层、传输层、UI 都能工作
- ✅ 端到端流程测试可以运行
- ✅ 不阻塞项目整体进度

**限制**:
- ⚠️ 无真实视频画面
- ⚠️ 无法测试真实性能
- ⚠️ CPU/延迟指标是模拟值

---

## 🎯 后续计划：平台原生 API（推荐）

### 方案概述

**等其他模块完成后**（预计 Q3-Q4 2026），采用平台原生实现：

```rust
// 抽象层（trait）
pub trait VideoEncoder {
    fn encode(&mut self, frame: &FrameBuffer) -> Result<EncodedData>;
}

// 平台实现
#[cfg(target_os = "macos")]
pub struct VideoToolboxEncoder { ... }

#[cfg(target_os = "windows")]
pub struct MediaFoundationEncoder { ... }

#[cfg(target_os = "linux")]
pub struct VAAPIEncoder { ... }
```

### 技术方案

| 平台 | API | 硬件加速 | 预计工作量 |
|------|-----|---------|-----------|
| macOS | VideoToolbox | ✅ | 3-5 天 |
| Windows | Media Foundation | ✅ | 3-5 天 |
| Linux | VA-API | ✅ | 4-6 天 |

**总计**: 10-15 天

### 优点

- ✅ **最佳性能** - 系统级硬件加速，CPU <15%
- ✅ **最稳定** - 操作系统原生 API
- ✅ **无依赖问题** - 不依赖第三方 Rust crate
- ✅ **已验证** - RustDesk 使用相同方案

### 实施时间点

**建议在以下模块完成后再实施**:
1. ✅ 网络层（STUN/TURN/ICE）
2. ✅ 传输层（SRTP/DTLS）
3. ✅ Flutter 客户端 UI
4. ✅ Go API 服务
5. ✅ Web 管理后台

---

## 📊 项目整体影响

### 不影响的模块 ✅

- **网络层**: NAT 穿透、P2P 连接 → 可正常开发
- **传输层**: SRTP 加密传输 → 可正常开发
- **信令层**: WebSocket 信令 → 可正常开发
- **UI 层**: Flutter 客户端 → 可正常开发
- **API 层**: Go 服务 → 可正常开发
- **管理后台**: Web 界面 → 可正常开发

### 受限的功能 ⚠️

- **真实视频传输**: 需要等待真实编解码实现
- **性能测试**: CPU/延迟测试需要真实实现
- **用户演示**: 无法展示真实画面

### 时间线调整

**原计划**:
- Week 1: WebRTC 集成 ← **暂停**
- Week 2-3: 网络层 + UI

**新计划**:
- Week 1-3: 网络层 + UI + API（并行）
- Week 4+: 平台原生编解码（当其他模块完成后）

**MVP 时间**: 不变（Q3 2026）

---

## 📝 记录的经验

### Rust WebRTC 生态现状（2026 年中）

1. **livekit SDK**:
   - 依赖管理混乱
   - 版本兼容性差
   - 不适合生产使用

2. **webrtc-rs**:
   - 纯 Rust 实现
   - 无硬件加速
   - 性能不足

3. **最佳实践**:
   - 桌面应用应使用平台原生 API
   - 不要依赖第三方 Rust WebRTC 库
   - 参考 RustDesk 的实现方式

### 建议给未来的开发者

- ✅ 优先使用平台原生 API
- ✅ 不要浪费时间在 livekit 依赖问题上
- ✅ Mock 实现是快速开发的好策略
- ✅ 先完成其他模块，再回来解决编解码

---

## 🔗 相关文档

- `WEBRTC_SOLUTION_COMPARISON.md` - 完整方案对比分析
- `WEBRTC_ARCHITECTURE.md` - 原架构决策（已过时）
- `codec-integration-status.md` - Mock 实现状态
- `real-environment-integration-plan.md` - 原集成计划（已调整）

---

**总结**: 暂停 WebRTC 真实集成，保持 Mock Simulator，优先完成其他模块，后续采用平台原生 API 实现。
