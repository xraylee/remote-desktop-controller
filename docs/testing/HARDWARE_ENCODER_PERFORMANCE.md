# VideoToolbox 硬件编码器性能报告

**日期**: 2026-06-29  
**状态**: ✅ 集成成功  
**里程碑**: Phase 4.1 完成

---

## 🎉 性能提升总结

| 指标 | OpenH264 软件 | VideoToolbox 硬件 | 提升 |
|------|--------------|------------------|------|
| **平均编码延迟** | 43.70ms | 22.11ms | **1.97x 倍** |
| **端到端延迟** | 77.70ms | 56.11ms | **降低 27.8%** |
| **延迟改进** | - | - | **-21.59ms** |

### 测试配置

- **分辨率**: 1280x720
- **帧率**: 30 fps
- **码率**: 2 Mbps
- **测试帧数**: 60 帧 (2秒)
- **平台**: macOS (Apple Silicon / Intel)

---

## 📊 详细性能数据

### OpenH264 软件编码器（基线）

```
平均编码时间: 43.70ms
端到端延迟: 77.70ms (编码 43.70ms + 解码 32ms + 传输 2ms)
```

### VideoToolbox 硬件编码器

```
平均编码时间: 22.11ms
端到端延迟: 56.11ms (编码 22.11ms + 解码 32ms + 传输 2ms)
```

### 性能提升

```
编码速度提升: 1.97x 倍
延迟降低: 21.59ms
端到端改进: 27.8%
目标达成: ✅ 端到端延迟 < 100ms
```

---

## 🔧 关键技术突破

### 问题 1: 段错误 (Segmentation Fault)

**症状**: `VTCompressionSessionEncodeFrame` 调用时崩溃

**根本原因**: 
- `presentationTimeStamp` 和 `duration` 参数传递了 `ptr::null()`
- VideoToolbox 需要有效的 `CMTime` 结构指针，不接受 NULL

**解决方案**:
```rust
// 修复前
VTCompressionSessionEncodeFrame(
    self.session,
    pixel_buffer,
    ptr::null(), // ❌ 导致崩溃
    ptr::null(), // ❌ 导致崩溃
    ptr::null(),
    ptr::null_mut(),
    &mut info_flags,
);

// 修复后
let presentation_time: CMTime = [0, 1, 1]; // time 0, timescale 1, valid flag
let duration: CMTime = [1, 30, 1]; // 1/30 second for 30fps

VTCompressionSessionEncodeFrame(
    self.session,
    pixel_buffer,
    &presentation_time as *const CMTime as *const std::ffi::c_void, // ✅
    &duration as *const CMTime as *const std::ffi::c_void, // ✅
    ptr::null(),
    ptr::null_mut(),
    &mut info_flags,
);
```

### 问题 2: CompleteFrames 崩溃

**症状**: `VTCompressionSessionCompleteFrames` 调用时崩溃

**根本原因**:
- `completeUntilPresentationTimeStamp` 参数传递了 `ptr::null()`
- 需要传递 `kCMTimeInvalid` 指针表示"完成所有帧"

**解决方案**:
```rust
// 修复前
VTCompressionSessionCompleteFrames(self.session, ptr::null()); // ❌

// 修复后
let k_cm_time_invalid: CMTime = [0, 0, 0]; // kCMTimeInvalid
VTCompressionSessionCompleteFrames(
    self.session,
    &k_cm_time_invalid as *const CMTime as *const std::ffi::c_void, // ✅
);
```

### 问题 3: UV Plane 边界溢出

**症状**: YUV420 → NV12 转换时潜在的内存越界

**根本原因**:
- CVPixelBuffer 的 UV plane stride 可能大于计算值
- 未对目标地址进行边界检查

**解决方案**:
```rust
// 修复前
let dst_idx = row * uv_stride + col * 2;
*uv_plane.add(dst_idx) = frame.data[u_src_idx]; // ❌ 可能越界

// 修复后
let dst_row_ptr = uv_plane.add(row * uv_stride);
let dst_offset = col * 2;
if dst_offset + 1 < uv_stride { // ✅ 边界检查
    *dst_row_ptr.add(dst_offset) = frame.data[u_src_idx];
    *dst_row_ptr.add(dst_offset + 1) = frame.data[v_src_idx];
}
```

---

## 🎓 技术要点

### CMTime 结构

VideoToolbox 的 `CMTime` 是一个 `[i64; 3]` 数组：

```rust
type CMTime = [i64; 3]; // [value, timescale, flags]

// 示例
let presentation_time: CMTime = [0, 1, 1];     // 时间点 0，时间刻度 1，有效标志
let duration: CMTime = [1, 30, 1];             // 1/30秒（30fps）
let k_cm_time_invalid: CMTime = [0, 0, 0];    // 无效时间（特殊标记）
```

### YUV420 → NV12 转换

**YUV420 Planar** (输入):
```
Y plane: width × height
U plane: (width/2) × (height/2)
V plane: (width/2) × (height/2)
```

**NV12 BiPlanar** (VideoToolbox):
```
Y plane: width × height
UV plane (interleaved): width × (height/2)
  - U at even offsets: [0], [2], [4], ...
  - V at odd offsets:  [1], [3], [5], ...
```

### Feature Flag 配置

确保 `hardware-accel` feature 正确传递：

```toml
# crates/rdcs-codec/Cargo.toml
[features]
hardware-accel = []

# crates/rdcs-connection/Cargo.toml
[features]
hardware-accel = ["rdcs-codec/hardware-accel"]

[dev-dependencies]
rdcs-codec = { path = "../rdcs-codec", features = ["hardware-accel"] }
```

---

## 📈 端到端延迟分析

### 软件编码器流水线

```
捕获 (0ms) → OpenH264编码 (43.70ms) → ICE传输 (2ms) → OpenH264解码 (32ms)
────────────────────────────────────────────────────────────────────────
总计: 77.70ms
```

### 硬件编码器流水线

```
捕获 (0ms) → VideoToolbox编码 (22.11ms) → ICE传输 (2ms) → OpenH264解码 (32ms)
────────────────────────────────────────────────────────────────────────
总计: 56.11ms ✅ (目标 < 100ms)
```

### 优化潜力

| 组件 | 当前 | 优化方案 | 预估 |
|------|------|----------|------|
| 编码 | 22.11ms | 已优化（硬件） | - |
| 解码 | 32ms | 硬件解码器 | ~15ms |
| 传输 | 2ms | 无序不可靠模式 | ~1ms |
| **总计** | **56.11ms** | **完全硬件加速** | **~38ms** |

---

## ✅ 验收标准

- [x] VideoToolbox 编码器正常初始化
- [x] 60 帧全部成功编码（100% 成功率）
- [x] 无段错误或崩溃
- [x] 编码延迟 < 50ms
- [x] 端到端延迟 < 100ms
- [x] 性能提升 > 1.5x 倍
- [x] Feature flag 正确配置
- [x] 编译警告清理

---

## 🚀 下一步计划

### Phase 4.2: 真实屏幕捕获

- [ ] 集成 `rdcs-macos` CGDisplayStream
- [ ] 替换测试帧生成
- [ ] 测试不同分辨率性能

### Phase 4.3: 硬件解码器

- [ ] 集成 VideoToolbox 解码器
- [ ] 进一步降低端到端延迟至 ~38ms

### Phase 4.4: Flutter UI 集成

- [ ] 显示实时视频流
- [ ] 鼠标键盘控制
- [ ] 连接状态显示

---

## 📝 相关文档

- [Phase 3.4+ 端到端视频流成功报告](E2E_VIDEO_STREAMING_SUCCESS.md)
- [Phase 3.4 DataChannel 传输报告](PHASE3_VIDEO_DATACHANNEL_SUCCESS.md)
- [VideoToolbox API 文档](https://developer.apple.com/documentation/videotoolbox)
- [CMTime 参考](https://developer.apple.com/documentation/coremedia/cmtime)

---

**维护人**: AI Assistant  
**完成日期**: 2026-06-29  
**状态**: ✅ Phase 4.1 完成  
**下一里程碑**: Phase 4.2 真实屏幕捕获
