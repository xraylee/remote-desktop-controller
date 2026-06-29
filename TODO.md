# 待办任务清单

**更新日期**: 2026-06-29  
**当前阶段**: Phase 4.1 完成，硬件编码器集成成功

---

## 🔥 立即待办

### 1. Phase 4.2: 屏幕捕获性能分析完成 ⭐⭐⭐ ✅ 已完成
**完成内容**: 
- ✅ 真实屏幕捕获集成完成
- ✅ 性能瓶颈分析完成
- ✅ 技术方案调研完成
- ✅ 代码注释优化

**测试结果**:
```
实际帧率: 6.6 fps (目标 30 fps)
捕获+编码: 150.28ms (目标 < 33ms)
编码器单独: 22.11ms ✅ (硬件加速正常)
瓶颈: CGDisplayCreateImage 同步捕获 ~128ms
```

**技术分析**:
- CGDisplayStream 实现复杂（需要 Objective-C Block + RunLoop）
- 不适合纯 Rust FFI
- 已记录 4 种优化方案（见文档）

**已创建文档**:
- `docs/testing/REAL_SCREEN_CAPTURE_PERFORMANCE.md` - 性能测试报告
- `docs/technical/SCREEN_CAPTURE_OPTIMIZATION.md` - 技术方案分析

**推荐方案**:
- 短期：优化现有实现（Arc 零拷贝、分辨率缩放、双缓冲）
- 中期：Swift Helper + 共享内存（使用 ScreenCaptureKit）
- 长期：纯 Rust CGDisplayStream（如果可行）

**下一步**: 根据项目优先级决定是否进入性能优化实施阶段

### 2. ~~代码提交~~ ⭐⭐⭐ ✅ 已完成
```bash
# 添加新文件
git add crates/rdcs-connection/src/video_channel.rs
git add crates/rdcs-connection/src/frame_reassembler.rs
git add crates/rdcs-connection/examples/video_e2e_test.rs
git add crates/rdcs-connection/src/lib.rs
git add crates/rdcs-connection/Cargo.toml

# 添加文档
git add docs/testing/E2E_VIDEO_STREAMING_SUCCESS.md
git add docs/CURRENT_PHASE.md
git add docs/MVP.md
git add docs/E2E_TEST_PLAN.md
git add docs/EXECUTION_CHECKLIST.md
git add docs/STANDARD_STRUCTURE.md
git add docs/archived/

# 添加工具
git add TEST_COMMANDS.sh

# 提交
git commit -m "feat: Phase 3.4+ - End-to-end video streaming over DataChannel

Major achievements:
- Implement VideoChannel wrapper for DataChannel
- Implement FrameReassembler for chunk reassembly
- Fix DataChannel offerer/answerer role asymmetry
- Integrate OpenH264 encoder/decoder
- Complete end-to-end video streaming test
- 100% success rate (30/30 frames)
- Average latency < 100ms

New files:
- crates/rdcs-connection/src/video_channel.rs
- crates/rdcs-connection/src/frame_reassembler.rs
- crates/rdcs-connection/examples/video_e2e_test.rs
- docs/testing/E2E_VIDEO_STREAMING_SUCCESS.md
- TEST_COMMANDS.sh

Modified files:
- crates/rdcs-connection/src/real_ice_agent.rs
- crates/rdcs-connection/Cargo.toml
- crates/rdcs-connection/src/lib.rs

Documentation:
- Archive 50+ deprecated docs to docs/archived/
- Create standardized documentation structure
- Add comprehensive test report"

# 推送
git push origin main
```

### 2. 清理编译警告 ⭐⭐
```bash
# 运行自动修复
cargo fix --lib -p rdcs-codec
cargo fix --example "video_e2e_test" -p rdcs-connection

# 检查剩余警告
cargo clippy --all-targets --all-features
```

### 3. 更新 lib.rs 导出 ⭐
确认 `crates/rdcs-connection/src/lib.rs` 正确导出了新模块：
```rust
pub mod video_channel;
pub mod frame_reassembler;

pub use video_channel::VideoChannel;
pub use frame_reassembler::{FrameReassembler, FrameHeader, FrameError};
```

---

## 📋 近期计划（1-2周）

### Phase 4: 真实环境集成

#### 4.1 硬件编码器集成 ⭐⭐⭐ ✅ 已完成
- ✅ macOS: VideoToolbox 硬件编码器集成
- ✅ 修复 CMTime FFI 崩溃问题
- ✅ 修复 CompleteFrames 崩溃问题
- ✅ 修复 UV plane 边界溢出
- ✅ 创建性能对比测试脚本
- ✅ 运行性能基准测试
- ✅ 验证性能提升：1.97x 倍加速
- ✅ 验证编码延迟：43.70ms → 22.11ms
- ✅ 验证端到端延迟：77.70ms → 56.11ms
- ✅ 完整性能报告文档

#### 4.2 真实屏幕捕获 ⭐⭐⭐
- [ ] 替换测试帧生成为真实捕获
- [ ] macOS: 使用 `rdcs-macos` CGDisplayStream
- [ ] 测试不同分辨率
- [ ] 验证端到端性能

#### 4.3 Flutter UI 显示 ⭐⭐
- [ ] 集成视频渲染
- [ ] 显示实时视频流
- [ ] 添加连接状态显示

---

## 🎯 中期目标（2-4周）

### Phase 5: 控制与交互

#### 5.1 鼠标控制 ⭐⭐⭐
- [ ] 捕获鼠标事件
- [ ] 通过 DataChannel 传输
- [ ] 远程端模拟鼠标操作

#### 5.2 键盘控制 ⭐⭐⭐
- [ ] 捕获键盘事件
- [ ] 通过 DataChannel 传输
- [ ] 远程端模拟键盘输入

#### 5.3 性能优化 ⭐⭐
- [ ] 实现无序不可靠模式（降低延迟 10-20ms）
- [ ] 自适应码率控制
- [ ] 带宽估算

---

## 🚀 长期规划（1-3个月）

### Phase 6: 生产就绪

#### 6.1 网络监控 ⭐
- [ ] RTT 测量
- [ ] 丢包率统计
- [ ] 拥塞检测
- [ ] QoS 仪表盘

#### 6.2 TURN 中继服务器 ⭐
- [ ] 部署 coturn 服务器
- [ ] Symmetric NAT 支持
- [ ] 回退策略

#### 6.3 多平台支持 ⭐⭐
- [ ] Windows 客户端
- [ ] Linux 客户端
- [ ] 跨平台测试

---

## 📊 当前状态

### 已完成
- ✅ Phase 1: 本地回环测试
- ✅ Phase 2: TCP 网络传输
- ✅ Phase 3.1: ICE 连接
- ✅ Phase 3.2: 跨网络测试
- ✅ Phase 3.3: DTLS 加密
- ✅ Phase 3.4: DataChannel 传输
- ✅ Phase 3.4+: 端到端编解码器集成
- ✅ Phase 4.1: VideoToolbox 硬件编码器集成 ✨

### 进行中
- 🔄 Phase 4.2: 真实屏幕捕获集成

### 最近完成
- ✅ 编译警告清理（2026-06-28）
  - 修复 `SdpAnswer` 缺少 `fingerprint` 字段（2处）
  - 移除不必要的 `mut` 修饰符
  - 添加下划线前缀到未使用变量

### 待开始
- ⏳ Phase 4: 真实环境集成
- ⏳ Phase 5: 控制与交互
- ⏳ Phase 6: 生产就绪

---

## 🎓 技术债务

### 低优先级但需要处理

1. **单元测试覆盖** 📝
   - 为 VideoChannel 添加单元测试
   - 为 FrameReassembler 添加更多边界测试
   - 提高代码覆盖率

2. **错误处理优化** 📝
   - 更详细的错误信息
   - 错误恢复机制
   - 日志级别优化

3. **性能基准测试** 📝
   - 建立性能基准
   - 持续性能监控
   - 回归测试

4. **API 文档** 📝
   - 添加 API 文档注释
   - 生成 rustdoc
   - 使用示例

---

## 🔍 已知问题

### 需要关注但不紧急

1. **IPv6 警告**
   - WebRTC 尝试监听 IPv6 地址失败
   - 不影响 IPv4 连接
   - 可以忽略或配置禁用

2. **STUN 超时**
   - 偶尔出现 STUN 服务器超时
   - 有多个备用服务器
   - 不影响本地连接

3. **死代码警告**
   - `real_ice_agent.rs` 中的 `ufrag` 和 `pwd` 字段
   - VideoToolbox 中的一些未使用函数
   - 可以通过 `cargo fix` 清理

---

## 💡 建议的工作流程

### 本周
1. ✅ 完成代码提交
2. ✅ 清理编译警告
3. 🔄 开始硬件编码器集成

### 下周
1. 🔄 完成硬件编码器
2. 🔄 集成真实屏幕捕获
3. 🔄 Flutter UI 基础显示

### 两周后
1. 🔄 鼠标键盘控制
2. 🔄 性能优化
3. 🔄 用户体验改进

---

## 📞 需要决策的问题

### 技术选型

1. **视频渲染方案**
   - Option A: Flutter Texture
   - Option B: Platform View
   - 建议: 先尝试 Flutter Texture

2. **控制协议**
   - Option A: 复用视频 DataChannel
   - Option B: 单独的控制 DataChannel
   - 建议: 单独 DataChannel（更清晰）

3. **音频支持**
   - 是否需要音频传输？
   - 如果需要，何时开始？
   - 建议: MVP 暂不包含，后续添加

---

**维护人**: AI Assistant  
**更新**: 2026-06-28  
**下次审查**: 完成代码提交后
