# RDCS 项目状态评估（基于 Superpowers 规则）

**评估时间**: 2026-06-27  
**目标**: 基于 Superpowers 最佳实践，确定 MVP 最快路径

---

## 📊 当前状态概览

### Rust 核心模块 ✅

| 模块 | 完成度 | 状态 | 备注 |
|------|--------|------|------|
| rdcs-codec | 60% | ⚠️ Mock | 编解码器 trait + Mock Simulator |
| rdcs-platform | 80% | ✅ | 屏幕捕获已实现 |
| rdcs-connection | 40% | ⏳ | 网络连接框架 |
| rdcs-signaling | 30% | ⏳ | WebSocket 信令 |
| rdcs-transport | 40% | ⏳ | 传输层抽象 |
| rdcs-crypto | 80% | ✅ | 加密/认证已实现 |

**问题**：
- ❌ 编解码器使用 Mock（无真实视频）
- ⚠️ 网络层未完成（NAT 穿透、P2P）
- ⚠️ 信令层未完成（SDP 交换）

### Go API 服务 ⏳

```
api/
├── cmd/server/       # 主服务入口
├── internal/
│   ├── auth/        # 认证服务 (JWT)
│   ├── device/      # 设备管理
│   ├── session/     # 会话管理
│   └── user/        # 用户管理
└── pkg/
    └── proto/       # gRPC/REST API 定义
```

**状态**: 
- ✅ 基础框架搭建完成
- ⚠️ API 实现不完整
- ❌ 数据库集成缺失

### Flutter 客户端 ⏳

```
client/
├── lib/
│   ├── screens/     # UI 界面
│   ├── widgets/     # 通用组件
│   ├── services/    # API 调用
│   └── models/      # 数据模型
└── test/
```

**状态**:
- ✅ 基础框架搭建完成
- ⚠️ UI 实现不完整
- ❌ 与 Rust 核心集成缺失

### Web 管理后台 ⏳

```
web/admin/
├── src/
│   ├── pages/       # 管理页面
│   ├── components/  # UI 组件
│   └── services/    # API 调用
└── package.json
```

**状态**:
- ✅ 基础框架搭建完成
- ⚠️ 管理功能不完整

---

## 🎯 Superpowers 规则分析

### 规则 1: 最小可行产品（MVP）优先

**当前问题**：项目范围过大，多个模块并行开发

**Superpowers 建议**：
- ✅ 聚焦核心功能：远程桌面连接 + 基础控制
- ❌ 避免过早优化：暂停高级特性（自适应码率、QoS）
- ✅ 快速验证：先实现端到端流程，再优化细节

### 规则 2: 垂直切片（Vertical Slice）

**什么是垂直切片**：
- 从用户界面 → API → 核心逻辑 → 数据库，完整实现一个功能
- 而不是水平分层（先做完所有 UI，再做所有 API）

**当前问题**：
- 各层独立开发，未端到端打通
- Rust 核心、Go API、Flutter UI 都是半成品

**Superpowers 建议**：
- 选择一个最小场景（如：本地网络远程桌面）
- 端到端实现：Flutter UI → Go API → Rust 核心 → 屏幕捕获 → 返回显示

### 规则 3: 技术债务管理

**当前技术债**：
- 🔴 **高优先级**：编解码器使用 Mock（阻塞真实功能）
- 🟡 **中优先级**：网络层未完成（无法 P2P 连接）
- 🟢 **低优先级**：另一个会话的 session/rtp 模块（已禁用）

**Superpowers 建议**：
- 先解决阻塞 MVP 的债务（编解码器）
- 延后不影响 MVP 的债务（高级 QoS）

### 规则 4: 依赖倒置（从结果倒推）

**MVP 目标**：用户能通过 Flutter 客户端远程控制另一台设备

**倒推依赖**：
```
用户看到远程屏幕
  ↑ 需要 Flutter UI 显示视频流
    ↑ 需要 Go API 提供视频流
      ↑ 需要 Rust 核心捕获+编码
        ↑ 需要真实编解码器（非 Mock）
          ↑ 需要平台原生 API（VideoToolbox/MF/VA-API）
```

**关键路径**：编解码器 → 网络传输 → API 对接 → UI 显示

---

## 🚀 MVP 最快路径（Superpowers 推荐）

### Phase 1: 本地回环测试（1 周）

**目标**：在同一台机器上验证捕获→编码→解码→显示

**任务**：
1. ✅ 实现平台原生编解码器（macOS VideoToolbox）
   - 使用另一个会话已有的 `platform/videotoolbox.rs`
   - 修复 FFI 调用问题
   - 单元测试验证

2. ✅ 屏幕捕获集成
   - `rdcs-platform` 已实现
   - 测试捕获 → 编码流程

3. ✅ 本地解码显示
   - 编码数据 → 解码 → BGRA 帧
   - 保存为图片验证

**验收标准**：
- 能捕获屏幕并编码为 H.264
- 能解码 H.264 并还原为图片
- CPU < 30%，延迟 < 50ms

### Phase 2: 本地网络传输（1 周）

**目标**：两台设备在同一局域网通过 TCP 传输视频

**任务**：
1. ✅ 简化网络层（暂不做 NAT 穿透）
   - 使用简单的 TCP Socket
   - 发送端：捕获 → 编码 → TCP 发送
   - 接收端：TCP 接收 → 解码 → 显示

2. ✅ Go API 基础服务
   - 设备注册 API
   - 会话创建 API
   - 简单的设备发现（局域网）

3. ✅ Flutter UI 基础界面
   - 设备列表
   - 连接按钮
   - 视频显示区域

**验收标准**：
- 两台 Mac 能在局域网互相发现
- 能建立连接并传输视频
- 用户能看到远程屏幕

### Phase 3: 基础控制（1 周）

**目标**：用户能远程控制鼠标和键盘

**任务**：
1. ✅ 输入事件捕获（Flutter）
   - 鼠标点击、移动
   - 键盘输入

2. ✅ 输入事件传输（Go API + Rust）
   - 事件序列化
   - 网络传输
   - 远程执行

3. ✅ 权限管理
   - macOS 辅助功能权限
   - 输入注入

**验收标准**：
- 能远程移动鼠标
- 能远程点击和输入

---

## 📋 优先级排序（基于 Superpowers）

### P0 - 立即执行（阻塞 MVP）

1. **恢复平台原生编解码器** ⭐⭐⭐
   - 任务：修复 `platform/videotoolbox.rs` 的测试崩溃
   - 原因：Mock 无法提供真实视频流
   - 时间：1-2 天

2. **本地回环测试** ⭐⭐⭐
   - 任务：捕获 → 编码 → 解码 → 保存图片
   - 原因：验证编解码器可用性
   - 时间：1 天

3. **简化网络层** ⭐⭐
   - 任务：TCP Socket 传输（暂不做 P2P）
   - 原因：快速验证端到端流程
   - 时间：2-3 天

### P1 - 重要但不紧急

1. **Go API 基础服务**
   - 设备注册、会话管理
   - 时间：3-5 天

2. **Flutter UI 基础界面**
   - 设备列表、连接、显示
   - 时间：3-5 天

### P2 - 可以延后

1. **NAT 穿透**（STUN/TURN）
   - 等 MVP 完成后再实现
   - 时间：5-7 天

2. **高级特性**
   - 自适应码率
   - QoS 优化
   - 多显示器支持

---

## 🎯 下一步行动（Superpowers 推荐）

### 立即执行（今天）

**任务 1: 诊断并修复 VideoToolbox 测试崩溃**

```rust
// 问题：测试时调用 VideoToolbox FFI 导致 SIGSEGV
// 解决方案：
// 1. 添加 feature gate
// 2. 测试时使用 Mock
// 3. 集成测试时使用真实 VideoToolbox
```

**任务 2: 创建本地回环测试**

```rust
#[test]
fn test_local_roundtrip() {
    // 1. 捕获屏幕
    let frame = capture_screen();
    
    // 2. 编码
    let encoded = encoder.encode(&frame);
    
    // 3. 解码
    let decoded = decoder.decode(&encoded);
    
    // 4. 保存为图片
    save_as_png(&decoded, "output.png");
    
    // 5. 验证
    assert!(decoded.width == frame.width);
}
```

### 本周目标

- ✅ VideoToolbox 编解码器可用
- ✅ 本地回环测试通过
- ✅ 能保存捕获的屏幕为图片

### 下周目标

- ✅ 简单的 TCP 网络传输
- ✅ 两台设备能互相发现
- ✅ 能传输并显示视频流

---

## 📝 Superpowers 最佳实践应用

### ✅ 做的好的地方

1. **暂停 livekit 集成**
   - 避免在依赖问题上浪费时间
   - 符合 "快速失败" 原则

2. **禁用未完成的模块**
   - session/rtp/platform 暂时禁用
   - 让项目保持可编译状态

3. **保留决策文档**
   - WEBRTC_SOLUTION_COMPARISON.md
   - WEBRTC_INTEGRATION_PAUSE.md

### ⚠️ 需要改进的地方

1. **过度设计**
   - session 模块功能过于复杂
   - 应该先实现最简单的传输

2. **并行开发过多**
   - Rust/Go/Flutter 同时推进
   - 应该聚焦一个垂直切片

3. **缺少端到端测试**
   - 各模块独立测试
   - 需要集成测试验证整体流程

---

## 🎯 最终建议

**立即行动**：
1. 修复 VideoToolbox 编解码器
2. 实现本地回环测试
3. 验证捕获 → 编码 → 解码流程

**短期目标**（1-2 周）：
- 完成 Phase 1 + Phase 2
- 实现本地网络视频传输

**中期目标**（1 个月）：
- 完成 Phase 3
- 实现基础远程控制
- MVP 可演示

**遵循 Superpowers 原则**：
- ✅ 垂直切片优先
- ✅ 最小可行产品
- ✅ 快速验证假设
- ✅ 延后非关键特性

---

**总结**：当前最关键的是修复 VideoToolbox 编解码器并完成本地回环测试，验证核心技术可行性。
