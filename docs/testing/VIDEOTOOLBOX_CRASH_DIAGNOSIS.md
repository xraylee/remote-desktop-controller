# VideoToolbox 崩溃诊断记录

**日期**: 2026-06-28  
**问题**: VideoToolbox 编码器在 encode 步骤发生 SIGSEGV  
**状态**: 🔴 待修复

---

## 📋 问题描述

使用 `hardware-accel` feature 运行 `local_roundtrip` 示例时，在编码阶段崩溃：

```bash
4️⃣  编码帧...
Segmentation fault: 11
```

**复现步骤**：
```bash
cd /Users/lc/Development/source/remote-desktop-controller
./run-local-roundtrip.sh
```

---

## 🔍 已尝试的修复

### 修复 1: 添加边界检查和错误处理

**文件**: `crates/rdcs-codec/src/platform/videotoolbox.rs`  
**修改**: `create_pixel_buffer` 方法

**改进**：
1. ✅ 添加输入数据大小验证
2. ✅ 检查所有 FFI 调用返回值
3. ✅ 添加空指针检查
4. ✅ 添加数组边界检查

**结果**: ❌ 仍然崩溃

---

## 🧪 诊断方案

### 方案 A: 使用 LLDB 调试（推荐）

**脚本**: `diagnose-videotoolbox-crash.sh`

```bash
./diagnose-videotoolbox-crash.sh
```

**输出**: 
- 崩溃位置
- 堆栈跟踪
- 寄存器状态

### 方案 B: 添加详细日志

在 `VideoToolboxEncoder::encode` 和 `create_pixel_buffer` 中添加 `println!` 调试输出，定位崩溃点。

### 方案 C: 简化测试用例

创建最小化测试：
1. 只创建 VTCompressionSession
2. 只创建 CVPixelBuffer
3. 只调用 VTCompressionSessionEncodeFrame

逐步定位问题。

---

## 🚧 可能的根本原因

### 1. YUV420 → NV12 转换问题

**症状**: 内存访问越界  
**位置**: `create_pixel_buffer` 的 UV 平面拷贝

**分析**：
- 输入: YUV420 planar (Y + U + V 独立平面)
- 输出: NV12 (Y + 交错 UV)
- 风险: 索引计算错误导致越界

### 2. CVPixelBuffer 生命周期

**症状**: 野指针访问  
**位置**: `encode` 方法中的 `CVPixelBufferRelease`

**分析**：
- `CVPixelBufferCreate` 创建的对象需要手动管理
- 可能在 callback 中仍在使用时被释放

### 3. 回调函数 refcon 指针

**症状**: 回调中访问无效内存  
**位置**: `compression_output_callback`

**分析**：
- `refcon` 是 `Arc<Mutex<Vec<u8>>>` 的裸指针
- 可能在 encoder drop 时被提前释放

### 4. 线程安全问题

**症状**: 数据竞争  
**位置**: 回调函数与主线程共享 `encoded_buffer`

**分析**：
- VideoToolbox 回调在后台线程执行
- 可能存在并发访问冲突

---

## 📝 下一步行动

### 立即执行

**选项 1: 使用 Apple 示例代码参考**（推荐）
- 查找 Apple 官方 VideoToolbox 示例
- 对比 API 调用方式
- 确认正确的使用模式

**选项 2: 使用成熟的 Rust 绑定**
- 调研现有的 VideoToolbox Rust crate
- 参考其实现方式
- 避免重复造轮子

**选项 3: 先用软件编码器**
- 暂时放弃硬件加速
- 使用 `openh264` 或 `x264` 软件编码器
- 确保 MVP 流程可用

### 中期计划

1. **隔离问题**
   - 创建单元测试只测试 `create_pixel_buffer`
   - 创建单元测试只测试 `VTCompressionSessionCreate`
   - 逐个验证 FFI 调用

2. **添加日志**
   - 在每个 unsafe 块前后添加日志
   - 记录所有指针地址
   - 记录所有返回状态码

3. **使用调试器**
   - 运行 `diagnose-videotoolbox-crash.sh`
   - 分析崩溃时的内存状态
   - 确定确切的崩溃指令

---

## 🎯 建议

根据 Superpowers 规则和 MVP 优先级：

**推荐方案**: 暂停 VideoToolbox 修复，先用软件编码器

**理由**：
1. ✅ Mock 版本已验证流程正确
2. ✅ 硬件加速不是 MVP 的阻塞项
3. ✅ 可以先完成 Phase 2（网络传输）
4. ✅ 后续有时间再优化性能

**时间对比**：
- 修复 VideoToolbox: 1-3 天（不确定）
- 集成软件编码器: 0.5-1 天（确定）
- Phase 2 网络传输: 2-3 天

**MVP 路径**：
```
Phase 1 ✅ → 软件编码器 → Phase 2 → Phase 3 → 优化(VideoToolbox)
```

---

## 📚 参考资料

- [VideoToolbox Programming Guide](https://developer.apple.com/documentation/videotoolbox)
- [AVFoundation Programming Guide](https://developer.apple.com/library/archive/documentation/AudioVideo/Conceptual/AVFoundationPG/)
- [Core Video Programming Guide](https://developer.apple.com/library/archive/documentation/GraphicsImaging/Conceptual/CoreVideo/)

---

**最后更新**: 2026-06-28  
**负责人**: AI Assistant  
**优先级**: P1（非阻塞）
