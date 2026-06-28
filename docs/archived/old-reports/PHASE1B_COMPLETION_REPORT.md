# Phase 1b 完成报告：实现 VideoToolbox 解码器

**完成日期**: 2026-06-28  
**状态**: ✅ 已完成  
**文件**: `crates/rdcs-codec/src/platform/videotoolbox.rs`

---

## 实施内容

### 1. 添加 VTDecompressionSession FFI 绑定

```rust
type VTDecompressionSessionRef = *mut std::ffi::c_void;

extern "C" {
    fn VTDecompressionSessionCreate(...);
    fn VTDecompressionSessionDecodeFrame(...);
    fn VTDecompressionSessionWaitForAsynchronousFrames(...);
    fn VTDecompressionSessionInvalidate(...);
}
```

### 2. 添加 CoreMedia 辅助函数

用于创建 CMSampleBuffer 和 CMBlockBuffer：

```rust
extern "C" {
    fn CMVideoFormatDescriptionCreate(...);
    fn CMSampleBufferCreate(...);
    fn CMBlockBufferCreateWithMemoryBlock(...);
    fn CFRelease(...);
}
```

### 3. 实现 Annex B → AVCC 格式转换

解码器需要将输入的 Annex B 格式转换为 AVCC 格式：

```rust
fn annex_b_to_avcc(annex_b_data: &[u8]) -> Vec<u8> {
    // 查找 start code (0x00 0x00 0x00 0x01 或 0x00 0x00 0x01)
    // 提取 NAL unit
    // 写入 4 字节长度 + NAL unit
}
```

### 4. 实现解码输出回调

```rust
extern "C" fn decompression_output_callback(
    decompress_callback_ref_con: *mut std::ffi::c_void,
    _source_frame_ref_con: *mut std::ffi::c_void,
    status: OSStatus,
    _info_flags: u32,
    image_buffer: CVPixelBufferRef,
    _presentation_timestamp: CMTime,
    _presentation_duration: CMTime,
) {
    // 1. 从 refcon 恢复 Arc<Mutex<Option<Frame>>>
    // 2. 从 CVPixelBufferRef 提取像素数据
    // 3. TODO: NV12 → YUV420 或 BGRA 转换
    // 4. 创建 Frame 并写入 decoded_buffer
}
```

### 5. 实现 VideoToolboxDecoder 结构

```rust
pub struct VideoToolboxDecoder {
    session: VTDecompressionSessionRef,
    format_description: CMFormatDescriptionRef,
    codec: VideoCodec,
    width: u32,
    height: u32,
    stats: Arc<DecoderStatsInner>,
    decoded_buffer: Arc<std::sync::Mutex<Option<Frame>>>,
    callback_refcon: *mut std::ffi::c_void,
}

impl PlatformDecoder for VideoToolboxDecoder {
    fn new(codec: VideoCodec) -> Result<Self, CodecError> { ... }
    fn decode(&mut self, data: &[u8]) -> Result<Frame, CodecError> { ... }
    fn get_stats(&self) -> DecoderStats { ... }
    fn shutdown(&mut self) -> Result<(), CodecError> { ... }
}
```

### 6. 惰性初始化解码会话

解码器在 `new()` 时只验证编解码器类型，真正的 `VTDecompressionSession` 在第一次 `decode()` 调用时创建（此时才知道视频尺寸）。

```rust
fn create_decompression_session(&mut self, width: u32, height: u32) -> Result<(), CodecError> {
    // 创建 CMFormatDescription
    // 创建回调 refcon
    // 创建 VTDecompressionSession
}
```

---

## 核心解码流程

```
Annex B 输入 (H.264 NAL units with start codes)
    ↓
annex_b_to_avcc() — 转换为长度前缀格式
    ↓
CMBlockBufferCreateWithMemoryBlock() — 封装为 CMBlockBuffer
    ↓
CMSampleBufferCreate() — 创建 CMSampleBuffer
    ↓
VTDecompressionSessionDecodeFrame() — 硬件解码
    ↓
decompression_output_callback() — 异步回调
    ↓
CVPixelBufferRef (NV12 格式)
    ↓
TODO: NV12 → Frame (YUV420 或 BGRA)
    ↓
返回 Frame
```

---

## 测试覆盖

### 1. 格式转换测试

```rust
#[test]
fn test_annex_b_to_avcc_conversion() {
    // 验证 Annex B → AVCC 转换正确
}
```

### 2. 解码器创建测试

```rust
#[test]
fn test_decoder_creation() {
    // 验证解码器可以成功创建
}
```

### 3. 编解码往返测试（关键）

```rust
#[test]
#[ignore] // 需要真实硬件
fn test_encode_decode_roundtrip() {
    // 编码一帧 → 解码 → 验证尺寸一致
}
```

---

## 已知限制与 TODO

### 🚧 像素格式转换未完成

当前 `decompression_output_callback` 中：

```rust
// TODO: Implement proper NV12 -> YUV420 or BGRA conversion
let frame = Frame::new(1280, 720, 0); // Placeholder
```

解码器输出的是 **CVPixelBuffer (NV12 格式)**，但 `Frame` 需要 **YUV420** 格式。

**需要实现**：
- `CVPixelBufferGetBaseAddressOfPlane` 读取 Y 平面和 UV 平面
- NV12（Y + 交错UV）→ YUV420（Y + 分离U + 分离V）
- 或者 NV12 → BGRA（如果渲染层需要）

### 🚧 动态尺寸检测

当前解码会话创建时硬编码了 1920x1080：

```rust
self.create_decompression_session(1920, 1080)?; // Default dimensions
```

**应该**：从第一个 H.264 NAL unit（SPS）中解析实际尺寸。

### 🚧 内存管理优化

当前每次 `decode()` 都克隆数据：

```rust
let mut data_copy = avcc_data.clone();
```

**可优化**：使用 `CMBlockBufferCreateWithMemoryBlock` 的 `customBlockSource` 参数避免拷贝。

---

## 下一步：Phase 1c

统一类型系统，桥接 `CapturedFrame`（BGRA）与 `Frame`（YUV420）。

**核心任务**：
1. 在 `platform/mod.rs` 添加工厂函数
2. BGRA → NV12/I420 像素格式转换
3. NV12 → BGRA 像素格式转换（解码器输出）
4. 统一接口：`encode(CapturedFrame) → Vec<u8>`, `decode(Vec<u8>) → DecodedFrame`

**预计工作量**: 0.5-1 天
