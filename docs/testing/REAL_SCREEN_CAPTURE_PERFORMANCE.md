# 真实屏幕捕获性能分析

**日期**: 2026-06-29  
**状态**: ⚠️ 性能不达标 - 需要优化  
**测试**: `real_screen_capture_test`

---

## 📊 测试结果

### 测试配置

- **捕获源**: 真实 macOS 屏幕（主显示器）
- **分辨率**: 实际屏幕分辨率（动态检测）
- **目标帧率**: 30 fps
- **目标码率**: 2 Mbps
- **编码器**: VideoToolbox 硬件加速
- **测试时长**: 3 秒

### 实际性能

| 指标 | 目标值 | 实际值 | 状态 |
|------|--------|--------|------|
| **帧率** | 30 fps | **6.6 fps** | ❌ 降低 78% |
| **平均延迟** | < 33ms (30fps) | **150.28ms** | ❌ 慢 4.5x |
| **捕获帧数** | ~90 帧 (3秒) | **20 帧** | ❌ |
| **总数据量** | ~750 KB | 1299 KB | ⚠️ 高 73% |
| **平均码率** | 2 Mbps | 3.50 Mbps | ⚠️ 高 75% |

### 完整测试日志

```
INFO real_screen_capture_test: ========================================
INFO real_screen_capture_test: Real Screen Capture + Hardware Encoder Test
INFO real_screen_capture_test: ========================================
INFO real_screen_capture_test: FPS: 30
INFO real_screen_capture_test: Bitrate: 2 Mbps
INFO real_screen_capture_test: Duration: 3 seconds
INFO real_screen_capture_test: Encoder: VideoToolbox (Hardware)

INFO real_screen_capture_test: Results
INFO real_screen_capture_test: ========================================
INFO real_screen_capture_test: Frames captured: 20
INFO real_screen_capture_test: Total bytes: 1299 KB
INFO real_screen_capture_test: Average encode time: 150.28ms
INFO real_screen_capture_test: Actual FPS: 6.6
INFO real_screen_capture_test: Average bitrate: 3.50 Mbps

⚠️  Encode time exceeds 30fps requirement
⚠️  FPS below target
```

---

## 🔍 问题诊断

### 性能对比

| 测试场景 | 编码延迟 | 帧率 | 说明 |
|----------|----------|------|------|
| **合成测试帧** | 22.11ms | 30 fps | ✅ 基准（仅编码器） |
| **真实屏幕捕获** | **150.28ms** | **6.6 fps** | ❌ 包含捕获 + 编码 |
| **性能差异** | **+128ms** | **-23.4 fps** | 瓶颈在捕获 |

### 延迟分解

```
总延迟: 150.28ms
├─ 屏幕捕获: ~128ms  ⚠️ 瓶颈
└─ 硬件编码: ~22ms   ✅ 正常
```

### 根本原因

**CGDisplayCreateImage API 的限制**：

1. **同步阻塞调用**
   - 每次调用都等待完整的屏幕捕获完成
   - 无法流水线处理

2. **GPU → CPU 数据传输**
   - 需要等待 GPU 渲染完成
   - 内存拷贝开销大

3. **无缓冲机制**
   - 无法利用帧间缓存
   - 每帧都重新完整捕获

4. **性能特征**
   ```rust
   // crates/rdcs-macos/src/capture.rs:133
   let image = CGDisplayCreateImage(display_id); // ⚠️ 阻塞 ~128ms
   ```

---

## ✅ 解决方案

### CGDisplayStream API

macOS 提供了高性能的异步流式捕获 API：

| API | 类型 | 延迟 | 帧率 | 说明 |
|-----|------|------|------|------|
| **CGDisplayCreateImage** | 同步 | ~150ms | ~6 fps | ❌ 当前实现 |
| **CGDisplayStream** | 异步 | **< 10ms** | **60+ fps** | ✅ 推荐方案 |

### CGDisplayStream 优势

1. **异步回调机制**
   - 屏幕更新时自动触发回调
   - 无阻塞等待

2. **零拷贝 IOSurface**
   - 直接访问 GPU 缓冲区
   - 无需 CPU 拷贝

3. **帧间优化**
   - 只传输变化区域
   - 降低数据量

4. **性能预期**
   ```
   捕获延迟: < 10ms
   编码延迟: 22ms
   总延迟: < 32ms  ✅ 达到 30fps 要求
   ```

### 实现计划

```rust
// 新实现结构
CGDisplayStreamCreate(
    display_id,
    width,
    height,
    pixel_format,
    properties,
    |status, display_time, frame_surface, update_ref| {
        // 异步回调处理帧
        // 使用 IOSurface 零拷贝访问像素数据
    }
)
```

**关键改进点**：
- 使用 `IOSurface` 替代 `CFData` 数据拷贝
- 异步回调处理帧，不阻塞主线程
- 支持增量更新（仅传输变化区域）

---

## 📈 性能提升预期

### 优化前 (当前)

```
屏幕捕获: 128ms
编码: 22ms
────────────────
总计: 150ms
实际帧率: 6.6 fps ❌
```

### 优化后 (CGDisplayStream)

```
屏幕捕获: < 10ms  ✅
编码: 22ms
────────────────
总计: < 32ms
预期帧率: 30+ fps ✅
```

### 性能对比表

| 指标 | 当前 | 优化后 | 提升 |
|------|------|--------|------|
| **捕获延迟** | 128ms | < 10ms | **12.8x** |
| **总延迟** | 150ms | < 32ms | **4.7x** |
| **帧率** | 6.6 fps | 30+ fps | **4.5x** |
| **CPU 使用** | 高 | 低 | 降低 60% |

---

## 🎯 下一步计划

### Phase 4.2: CGDisplayStream 集成

1. **实现 CGDisplayStream FFI 绑定**
   - [ ] 声明 C API 函数
   - [ ] 实现回调处理
   - [ ] IOSurface 数据访问

2. **替换现有 capture_display()**
   - [ ] 移除 `CGDisplayCreateImage` 调用
   - [ ] 使用 `CGDisplayStreamCreate`
   - [ ] 保持接口兼容

3. **性能验证**
   - [ ] 运行 `real_screen_capture_test`
   - [ ] 确认延迟 < 33ms
   - [ ] 确认帧率 ≥ 30 fps

4. **文档更新**
   - [ ] 更新性能报告
   - [ ] 添加实现说明
   - [ ] 对比基准数据

---

## 📚 技术参考

### CGDisplayStream API

**Apple 官方文档**:
- [CGDisplayStream](https://developer.apple.com/documentation/coregraphics/cgdisplaystream)
- [IOSurface](https://developer.apple.com/documentation/iosurface)
- [ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit) (macOS 12.3+)

**性能最佳实践**:
- 使用 `kCGDisplayStreamShowCursor` 控制光标渲染
- 设置 `kCGDisplayStreamMinimumFrameTime` 限制帧率
- 使用 `IOSurfaceGetBaseAddress` 零拷贝访问像素

### ScreenCaptureKit (替代方案)

macOS 12.3+ 引入的新 API，性能更好：
- 更低延迟 (< 5ms)
- 更好的能效
- 支持窗口级捕获
- 自动处理权限

**长期方案**: 
- 优先使用 `ScreenCaptureKit` (macOS 12.3+)
- 回退到 `CGDisplayStream` (macOS 10.8+)
- 最后回退到 `CGDisplayCreateImage` (所有版本)

---

## ✅ 验收标准

- [ ] 捕获延迟 < 10ms
- [ ] 编码延迟 < 25ms
- [ ] 总延迟 < 33ms
- [ ] 稳定 30 fps
- [ ] CPU 使用率 < 30%
- [ ] 内存占用合理

---

**维护人**: AI Assistant  
**测试日期**: 2026-06-29  
**状态**: ⚠️ 待优化  
**下一里程碑**: CGDisplayStream 实现
