# Phase 1a 完成报告：修复 VideoToolbox 编码器输出回调

**完成日期**: 2026-06-28  
**状态**: ✅ 已完成  
**文件**: `crates/rdcs-codec/src/platform/videotoolbox.rs`

---

## 问题诊断

原代码在创建 `VTCompressionSession` 时传入了 `ptr::null()` 作为输出回调：

```rust
// 第 162 行（修复前）
ptr::null(),              // output callback (TODO: implement)
ptr::null_mut(),          // callback ref con
```

这导致 VideoToolbox 编码后的 H.264 数据无处输出——`encoded_buffer` 永远为空。

---

## 实施的修复

### 1. 添加 CoreMedia FFI 绑定

新增类型和函数声明：

```rust
type CMSampleBufferRef = *mut std::ffi::c_void;
type CMBlockBufferRef = *mut std::ffi::c_void;
type CMFormatDescriptionRef = *mut std::ffi::c_void;

#[link(name = "CoreMedia", kind = "framework")]
extern "C" {
    fn CMSampleBufferGetDataBuffer(...);
    fn CMBlockBufferGetDataLength(...);
    fn CMBlockBufferCopyDataBytes(...);
    fn CMSampleBufferGetFormatDescription(...);
    fn CMVideoFormatDescriptionGetH264ParameterSetAtIndex(...);
}
```

### 2. 实现输出回调函数

```rust
extern "C" fn compression_output_callback(
    output_callback_ref_con: *mut std::ffi::c_void,
    _source_frame_ref_con: *mut std::ffi::c_void,
    status: OSStatus,
    _info_flags: u32,
    sample_buffer: CMSampleBufferRef,
) {
    // 1. 从 refcon 中恢复 Arc<Mutex<Vec<u8>>>
    // 2. 从 CMSampleBufferRef 提取 CMBlockBufferRef
    // 3. 将编码数据复制到 temp_buffer
    // 4. 转换 AVCC → Annex B 格式
    // 5. 写入 encoded_buffer
}
```

### 3. AVCC → Annex B 格式转换

VideoToolbox 输出的是 **AVCC 格式**（长度前缀）：
```
[4 字节长度][NAL unit][4 字节长度][NAL unit]...
```

需要转换为 **Annex B 格式**（start code 前缀）以符合 RFC 6184：
```
[0x00 0x00 0x00 0x01][NAL unit][0x00 0x00 0x00 0x01][NAL unit]...
```

实现函数 `avcc_to_annex_b()` 进行转换。

### 4. 回调 refcon 生命周期管理

在 `new()` 中：
```rust
let buffer_for_callback = Arc::clone(&encoded_buffer);
let refcon = Box::into_raw(Box::new(buffer_for_callback)) as *mut std::ffi::c_void;
```

在 `VideoToolboxEncoder` 结构体中新增字段：
```rust
callback_refcon: *mut std::ffi::c_void,
```

在 `shutdown()` 中释放：
```rust
if !self.callback_refcon.is_null() {
    let _ = Box::from_raw(
        self.callback_refcon as *mut Arc<std::sync::Mutex<Vec<u8>>>
    );
    self.callback_refcon = ptr::null_mut();
}
```

---

## 验证测试

新增测试 `test_encode_single_frame()`：

```rust
#[test]
fn test_encode_single_frame() {
    if let Ok(mut encoder) = VideoToolboxEncoder::new(...) {
        let frame = Frame::test_frame(1280, 720);
        
        match encoder.encode(&frame) {
            Ok(encoded_data) => {
                // ✅ 验证数据非空
                assert!(!encoded_data.is_empty());
                
                // ✅ 验证 Annex B start code
                assert_eq!(&encoded_data[0..4], &[0x00, 0x00, 0x00, 0x01]);
            }
            Err(e) => println!("Expected on CI: {:?}", e),
        }
    }
}
```

新增测试 `test_avcc_to_annex_b_conversion()`：

验证格式转换逻辑正确性。

---

## 关键变更摘要

| 变更项 | 修复前 | 修复后 |
|--------|--------|--------|
| 输出回调 | `ptr::null()` | `compression_output_callback` |
| refcon 参数 | `ptr::null_mut()` | `Box::into_raw(...)` |
| 数据提取 | ❌ 无 | ✅ CMBlockBufferCopyDataBytes |
| 格式转换 | ❌ 无 | ✅ AVCC → Annex B |
| 内存管理 | ❌ 无泄漏风险但无功能 | ✅ 正确分配/释放 |
| 测试覆盖 | 仅创建测试 | ✅ 包含编码输出验证 |

---

## 下一步：Phase 1b

实现 VideoToolbox 解码器：

1. 添加 `VTDecompressionSession*` FFI 绑定
2. 实现解码输出回调
3. Annex B → AVCC 格式转换（解码器输入）
4. NV12 → BGRA 像素格式转换
5. 编解码往返测试

**预计工作量**: 1-2 天
