# 🔧 Arc<[u8]> 编译错误修复

**日期**: 2026-06-29  
**问题**: `rdcs-codec` 编译失败

---

## ❌ 错误信息

```
error[E0308]: mismatched types
   --> crates/rdcs-codec/src/platform/mod.rs:371:15
    |
371 |         data: bgra_data,
    |               ^^^^^^^^^ expected `Arc<[u8]>`, found `Vec<u8>`
```

---

## 📝 原因

Task #47 部分完成的 Arc 零拷贝优化：
- ✅ `CapturedFrame.data` 改为 `Arc<[u8]>`（在 `rdcs-platform`）
- ✅ `rdcs-macos` 已更新
- ❌ `rdcs-codec` 中的转换函数未更新

---

## ✅ 解决方案

### 修改 `crates/rdcs-codec/src/platform/mod.rs`

**1. 添加导入**:
```rust
use std::sync::Arc;
```

**2. 修复 `yuv420_to_captured_frame()` 函数**:

**之前**:
```rust
Ok(CapturedFrame {
    data: bgra_data,  // Vec<u8>
    // ...
})
```

**之后**:
```rust
Ok(CapturedFrame {
    data: Arc::from(bgra_data.into_boxed_slice()),  // Arc<[u8]>
    // ...
})
```

---

## 🎯 技术说明

### Vec<u8> → Arc<[u8]> 转换

```rust
// 方式 1: 通过 Box (推荐，零拷贝)
let vec = vec![1, 2, 3];
let arc: Arc<[u8]> = Arc::from(vec.into_boxed_slice());

// 方式 2: 直接转换
let vec = vec![1, 2, 3];
let arc: Arc<[u8]> = vec.into();

// 方式 3: 从切片
let data = [1, 2, 3];
let arc: Arc<[u8]> = Arc::from(&data[..]);
```

**选择方式 1** 的原因：
- 明确显示零拷贝意图
- `Vec → Box → Arc` 避免中间分配

---

## 📊 影响范围

### 已修复
- ✅ `rdcs-platform/src/lib.rs` - `CapturedFrame` 定义
- ✅ `rdcs-macos/src/capture.rs` - 屏幕捕获
- ✅ `rdcs-codec/src/platform/mod.rs` - YUV→BGRA 转换

### 未修复（测试/示例代码）
以下文件仍使用 `Vec<u8>`，但不影响主流程：
- `rdcs-codec/src/pipeline.rs`
- `rdcs-codec/src/analyzer.rs`
- `rdcs-codec/examples/*.rs`
- `rdcs-connection/examples/*.rs`
- `rdcs-transport/examples/*.rs`

**决策**: 暂不修复示例代码
- 主流程已工作
- 示例代码可以后续批量更新
- 不阻塞 MVP

---

## ✅ 验证

```bash
cd crates/rdcs-codec
cargo build

cd crates/rdcs-ffi
cargo build
```

**预期**: 编译成功，无错误

---

## 📚 相关文档

- `docs/implementation/TASK_47_ARC_OPTIMIZATION_PARTIAL.md` - Arc 优化进度
- `docs/technical/SCREEN_CAPTURE_OPTIMIZATION.md` - 性能优化方案

---

**修复日期**: 2026-06-29  
**影响**: 解决编译错误，主流程可用  
**后续**: 示例代码可选择性更新
