# macOS 屏幕捕获性能优化技术分析

**日期**: 2026-06-29  
**状态**: 技术调研  
**目标**: 将屏幕捕获性能从 6.6 fps 提升到 30 fps

---

## 📊 当前性能问题

### 测试结果

```
捕获 API: CGDisplayCreateImage (同步)
捕获延迟: ~128ms
编码延迟: 22ms (VideoToolbox 硬件加速 ✅)
总延迟: 150.28ms
实际帧率: 6.6 fps ❌
```

### 性能瓶颈

**CGDisplayCreateImage 的限制**：

1. **同步阻塞调用** - 每帧等待完整捕获
2. **GPU → CPU 数据传输** - 大量内存拷贝
3. **无缓冲优化** - 每帧重新完整捕获
4. **无法流水线** - 捕获和编码串行执行

---

## 🎯 优化方案对比

### 方案 1: CGDisplayStream + IOSurface ⭐⭐⭐⭐⭐

**性能预期**: < 10ms 捕获延迟，30+ fps

**优势**：
- 异步回调机制
- IOSurface 零拷贝访问
- 增量更新支持
- GPU 加速

**实现难度**: ⚠️ **非常高**

**技术障碍**：

1. **需要 Objective-C Block**
   ```c
   // CGDisplayStream 的回调是 Objective-C Block
   typedef void (^CGDisplayStreamFrameAvailableHandler)(
       CGDisplayStreamFrameStatus status,
       uint64_t displayTime,
       IOSurfaceRef frameSurface,
       CGDisplayStreamUpdateRef updateRef
   );
   ```
   - Rust FFI 不直接支持 Block
   - 需要 `block` crate 或手写 Block 结构

2. **需要 Core Foundation RunLoop**
   ```c
   // CGDisplayStream 需要在 RunLoop 中运行
   CFRunLoopRun();
   ```
   - 必须有一个 CF RunLoop
   - 复杂的线程管理

3. **IOSurface 像素格式转换复杂**
   - IOSurface 可能是各种格式
   - 需要处理不同的颜色空间

**实现成本**: 2-3 周开发 + 测试

**代码示例**（理论）：
```rust
// 需要 block crate 和复杂的 FFI
extern "C" {
    fn CGDisplayStreamCreate(
        display: CGDirectDisplayID,
        output_width: usize,
        output_height: usize,
        pixel_format: i32,
        properties: CFDictionaryRef,
        handler: *const Block, // ⚠️ Objective-C Block
    ) -> CGDisplayStreamRef;
}

// 需要在 RunLoop 中运行
// 需要管理 Block 的生命周期
// 需要处理 IOSurface 的锁定和释放
```

**结论**: 性能最好，但实现太复杂，不适合纯 Rust 项目

---

### 方案 2: ScreenCaptureKit (macOS 12.3+) ⭐⭐⭐⭐⭐

**性能预期**: < 5ms 捕获延迟，60+ fps

**优势**：
- 最新最快的 API
- Swift/Objective-C 友好的接口
- 自动权限管理
- 支持窗口级捕获

**实现难度**: ⚠️ **极高**

**技术障碍**：

1. **纯 Swift/Objective-C API**
   - 没有 C 接口
   - 需要创建 Objective-C 包装器

2. **需要 Objective-C 运行时**
   - `objc` crate 集成
   - 复杂的消息传递

3. **macOS 版本要求**
   - 仅支持 macOS 12.3+
   - 需要回退方案

**实现成本**: 3-4 周开发 + 测试

**结论**: 性能最好且最现代，但需要 Objective-C 互操作，复杂度极高

---

### 方案 3: 优化现有 CGDisplayCreateImage ⭐⭐⭐

**性能预期**: ~50-80ms 捕获延迟，15-20 fps

**可行的优化点**：

#### 3.1 减少内存拷贝

**当前实现**：
```rust
// 数据被拷贝两次
let ptr = CFDataGetBytePtr(cf_data);
let bytes = std::slice::from_raw_parts(ptr, len).to_vec(); // ❌ 拷贝 1
CFRelease(cf_data);

// CapturedFrame 持有数据
data: bytes, // ❌ 拷贝 2（移动到 channel）
```

**优化后**：
```rust
// 使用 Arc<[u8]> 共享数据
let bytes = Arc::from(std::slice::from_raw_parts(ptr, len));
// 只拷贝一次，后续通过引用计数共享
```

**预期提升**: 10-20ms

#### 3.2 降低捕获分辨率

```rust
// 捕获时缩小分辨率
let scale_factor = 0.5; // 1920x1080 → 960x540
let scaled_width = (width as f64 * scale_factor) as u32;
```

**预期提升**: 30-40ms（4K → 1080p）

#### 3.3 并行捕获和编码

```rust
// 使用双缓冲：一个线程捕获，一个线程编码
let (capture_tx, capture_rx) = mpsc::channel();
let (encode_tx, encode_rx) = mpsc::channel();

thread::spawn(|| {
    // 捕获线程
});

thread::spawn(|| {
    // 编码线程
});
```

**预期提升**: 20-30ms（流水线并行）

#### 3.4 跳帧策略

```rust
// 当编码跟不上时，丢弃旧帧
if capture_rx.len() > 2 {
    let _ = capture_rx.try_recv(); // 丢弃一帧
}
```

**预期提升**: 保持低延迟

**总体预期**：
- 捕获延迟：128ms → 60-80ms
- 总延迟：150ms → 82-102ms
- 帧率：6.6 fps → 10-15 fps

**实现难度**: ⭐ 低

**实现成本**: 2-3 天

**结论**: 性能提升有限，但实现简单，可快速见效

---

### 方案 4: 外部工具 + 共享内存 ⭐⭐⭐⭐

**性能预期**: < 10ms 捕获延迟，30+ fps

**架构**：

```
┌─────────────────────────────────────┐
│  Swift Helper Process               │
│  - ScreenCaptureKit                 │
│  - 写入共享内存                      │
└───────────────┬─────────────────────┘
                │ Shared Memory
┌───────────────▼─────────────────────┐
│  Rust Main Process                  │
│  - 从共享内存读取                    │
│  - VideoToolbox 编码                │
│  - WebRTC 传输                      │
└─────────────────────────────────────┘
```

**实现**：

1. **Swift Helper (`rdcs-capture-helper`)**
   ```swift
   import ScreenCaptureKit
   
   class CaptureHelper {
       func start(shmName: String) {
           // 使用 ScreenCaptureKit
           // 写入共享内存
       }
   }
   ```

2. **Rust Main Process**
   ```rust
   // 使用 shared_memory crate
   let shm = SharedMemory::open("rdcs-capture")?;
   let frame_data = shm.as_slice();
   ```

**优势**：
- 使用最快的捕获 API
- Rust 和 Swift 各自做擅长的事
- 进程隔离，稳定性好

**实现难度**: ⭐⭐⭐ 中等

**实现成本**: 1 周开发 + 测试

**结论**: 性能好，架构清晰，是一个实用的折中方案

---

## 💡 推荐方案

### 短期（1周内）：方案 3 - 优化现有实现

**理由**：
- 快速见效
- 风险低
- 不改变架构

**实施步骤**：
1. 使用 `Arc<[u8]>` 减少拷贝
2. 添加分辨率缩放选项
3. 实现双缓冲并行处理
4. 添加跳帧策略

**预期结果**：
- 帧率：6.6 fps → 10-15 fps
- 延迟：150ms → 90-110ms

### 中期（1个月内）：方案 4 - Swift Helper

**理由**：
- 性能提升显著
- 架构合理
- 可维护性好

**实施步骤**：
1. 创建 Swift helper 项目
2. 实现 ScreenCaptureKit 捕获
3. 实现共享内存通信
4. 集成到主项目

**预期结果**：
- 帧率：→ 30+ fps
- 延迟：→ < 50ms

### 长期（3个月内）：方案 1 或 2 - 纯 Rust 实现

**理由**：
- 最佳性能
- 无外部依赖
- 完全控制

**前置条件**：
- 评估 `block` crate
- 研究 `objc2` crate
- 验证可行性

---

## 📚 技术参考

### CGDisplayStream

- [Apple 文档](https://developer.apple.com/documentation/coregraphics/cgdisplaystream)
- [示例代码](https://github.com/serg-vinnie/CGDisplayStream-example)
- 需要 Objective-C Block

### ScreenCaptureKit

- [Apple 文档](https://developer.apple.com/documentation/screencapturekit)
- [WWDC 2022 视频](https://developer.apple.com/videos/play/wwdc2022/10156/)
- macOS 12.3+ 专用

### 共享内存方案

- [shared_memory crate](https://crates.io/crates/shared_memory)
- [interprocess crate](https://crates.io/crates/interprocess)

### Rust-Objective-C 互操作

- [objc2 crate](https://crates.io/crates/objc2)
- [block crate](https://crates.io/crates/block)
- [fruity crate](https://crates.io/crates/fruity)

---

## ✅ 行动计划

### 第一步：快速优化（本周）

- [ ] 实现 `Arc<[u8]>` 零拷贝
- [ ] 添加分辨率缩放
- [ ] 实现双缓冲
- [ ] 测试性能提升

### 第二步：Swift Helper（下月）

- [ ] 创建 Swift helper 项目
- [ ] ScreenCaptureKit 集成
- [ ] 共享内存通信
- [ ] 性能基准测试

### 第三步：长期优化（3个月）

- [ ] 评估纯 Rust 可行性
- [ ] CGDisplayStream 研究
- [ ] 原型验证

---

**维护人**: AI Assistant  
**更新日期**: 2026-06-29  
**下次审查**: 优化方案 3 完成后
