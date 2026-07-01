# Task #45 实现计划：Flutter UI 视频显示

**日期**: 2026-06-29  
**状态**: 🔄 规划中  
**优先级**: ⭐⭐⭐

---

## 问题分析

### 当前状态

1. **✅ 已有组件**：
   - VideoToolbox 硬件编码器（macOS）
   - OpenH264 软件解码器
   - WebRTC DataChannel 视频传输
   - Flutter 视频占位符 UI
   - FFI 事件回调机制

2. **❌ 缺失环节**：
   - **Rust FFI → Flutter 的视频帧传递**
   - **Flutter 端的视频解码和渲染**
   - **帧率/延迟指标显示**

### 技术挑战

1. **FFI 数据传递**：
   - H.264 编码数据需要通过 FFI 传递到 Flutter
   - 选项 A：通过事件回调传递 Base64 编码的数据（简单但低效）
   - 选项 B：使用共享内存 + Texture（高效但复杂）
   - 选项 C：直接传递解码后的像素数据（中等复杂度）

2. **Flutter 视频渲染**：
   - 选项 A：`video_player` 包（需要文件或网络流）
   - 选项 B：`Texture` widget + 原生纹理 ID（需要平台通道）
   - 选项 C：`RawImage` + `Image.memory`（简单，适合低帧率）

3. **解码位置**：
   - 选项 A：Rust 端解码 → 传递 BGRA 像素数据（推荐）
   - 选项 B：Flutter 端解码（需要集成 H.264 解码库）

---

## 推荐方案

### 方案：Rust 解码 + Flutter RawImage 渲染

**理由**：
- ✅ 实现简单，快速见效
- ✅ 复用现有 OpenH264 解码器
- ✅ 适合 MVP 验证
- ⚠️ 性能一般（~30fps，足够初期使用）
- 📝 后续可升级到 Texture 方案

**数据流**：
```
Capture → Encode (H.264) → DataChannel
    ↓
Receive → Decode (BGRA) → FFI Event
    ↓
Flutter: EVENT_FRAME_READY → Image.memory() → UI
```

---

## 实现步骤

### Step 1: 扩展 FFI 层以传递解码后的视频帧

**文件**: `crates/rdcs-ffi/src/lib.rs`

**变更**：
1. 添加接收端解码器状态
2. 接收 H.264 数据并解码为 BGRA
3. 通过 `EVENT_FRAME_READY` 回调传递帧数据
4. 使用 Base64 编码传递像素数据（临时方案）

**伪代码**：
```rust
// 在 EngineHandle 中添加
decoder: Arc<Mutex<Option<Box<dyn PlatformDecoder>>>>,

// 接收编码帧时
fn on_encoded_frame_received(engine: &EngineHandle, h264_data: &[u8]) {
    let mut decoder = engine.decoder.lock().unwrap();
    if decoder.is_none() {
        *decoder = Some(Box::new(OpenH264Decoder::new(VideoCodec::H264)?));
    }
    
    let frame = decoder.as_mut().unwrap().decode(h264_data)?;
    
    // 转换为 BGRA（如果需要）
    let bgra_data = yuv_to_bgra(&frame);
    
    // Base64 编码
    let base64_data = base64::encode(&bgra_data);
    
    // 发送事件
    let payload = json!({
        "width": frame.width,
        "height": frame.height,
        "format": "bgra",
        "data": base64_data,
        "timestamp": frame.timestamp_us,
    });
    
    dispatch_event(engine, EVENT_FRAME_READY, &payload.to_string());
}
```

### Step 2: Flutter 端接收和渲染视频帧

**文件**: `client/flutter/lib/features/session/session_screen.dart`

**变更**：
1. 监听 `EVENT_FRAME_READY` 事件
2. 解码 Base64 数据
3. 使用 `Image.memory()` 显示
4. 添加帧率计数器

**实现**：
```dart
class _VideoRenderer extends ConsumerStatefulWidget {
  const _VideoRenderer();
  
  @override
  ConsumerState<_VideoRenderer> createState() => _VideoRendererState();
}

class _VideoRendererState extends ConsumerState<_VideoRenderer> {
  Uint8List? _currentFrame;
  int _frameWidth = 0;
  int _frameHeight = 0;
  int _frameCount = 0;
  DateTime? _lastFpsUpdate;
  double _currentFps = 0;
  
  @override
  void initState() {
    super.initState();
    _subscribeToFrames();
  }
  
  void _subscribeToFrames() {
    // 监听引擎事件
    ref.read(engineProvider).eventStream.listen((event) {
      if (event.type == EngineEventId.frameReady) {
        final payload = FramePayload.fromJson(event.payload);
        _onFrameReceived(payload);
      }
    });
  }
  
  void _onFrameReceived(FramePayload payload) {
    setState(() {
      _currentFrame = base64Decode(payload.dataBase64);
      _frameWidth = payload.width;
      _frameHeight = payload.height;
      
      // 更新帧率
      _frameCount++;
      final now = DateTime.now();
      if (_lastFpsUpdate != null) {
        final elapsed = now.difference(_lastFpsUpdate!).inMilliseconds;
        if (elapsed >= 1000) {
          _currentFps = _frameCount * 1000 / elapsed;
          _frameCount = 0;
          _lastFpsUpdate = now;
        }
      } else {
        _lastFpsUpdate = now;
      }
    });
  }
  
  @override
  Widget build(BuildContext context) {
    if (_currentFrame == null) {
      return const _VideoPlaceholder();
    }
    
    return Stack(
      children: [
        // 视频画面
        RawImage(
          image: _createImage(),
          fit: BoxFit.contain,
        ),
        
        // 帧率指示器
        Positioned(
          top: 8,
          right: 8,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: Colors.black.withOpacity(0.5),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              '${_currentFps.toStringAsFixed(1)} FPS',
              style: const TextStyle(
                color: Colors.white,
                fontSize: 12,
                fontFamily: 'monospace',
              ),
            ),
          ),
        ),
      ],
    );
  }
  
  ui.Image? _createImage() {
    if (_currentFrame == null) return null;
    
    // 将 BGRA 数据转换为 Image
    // 注意：这需要使用 decodeImageFromPixels
    final completer = Completer<ui.Image>();
    ui.decodeImageFromPixels(
      _currentFrame!,
      _frameWidth,
      _frameHeight,
      ui.PixelFormat.bgra8888,
      (image) => completer.complete(image),
    );
    return completer.future; // 需要改为 FutureBuilder
  }
}
```

### Step 3: 添加解码器依赖

**文件**: `crates/rdcs-ffi/Cargo.toml`

```toml
[dependencies]
base64 = "0.22"
rdcs-codec = { path = "../rdcs-codec", features = ["software-encoder"] }
```

### Step 4: 性能优化（可选）

**优化点**：
1. **帧缓冲复用**：避免每次分配新内存
2. **解码线程池**：异步解码，不阻塞主线程
3. **跳帧策略**：如果渲染速度跟不上，丢弃旧帧

---

## 测试计划

### 单元测试

1. **FFI 解码测试**：
   ```bash
   cargo test -p rdcs-ffi test_decode_and_dispatch
   ```

2. **Flutter 事件处理测试**：
   ```bash
   cd client/flutter
   flutter test test/session_screen_video_test.dart
   ```

### 集成测试

1. **端到端视频流测试**：
   - 启动服务端（屏幕捕获 + 编码）
   - 启动客户端（接收 + 解码 + 渲染）
   - 验证视频显示
   - 测量帧率和延迟

### 手动测试

1. 运行 Flutter 应用
2. 连接到本地或远程桌面
3. 验证视频画面显示
4. 检查帧率指示器
5. 测试鼠标/键盘控制是否正常

---

## 性能目标

| 指标 | 目标 | 备注 |
|------|------|------|
| 帧率 | ≥ 24 fps | 可接受的流畅度 |
| 延迟 | < 150ms | 端到端（捕获→显示）|
| CPU 占用 | < 30% | 单核心 |
| 内存占用 | < 100MB | Flutter app |

---

## 风险和限制

### 已知风险

1. **Base64 编码开销**：
   - 1280x720 BGRA = 3.6MB/帧
   - Base64 后 = 4.8MB/帧
   - 30fps = 144MB/s
   - **缓解**：初期可接受，后续升级到共享内存

2. **Flutter 解码性能**：
   - `decodeImageFromPixels` 是异步的
   - 可能造成帧延迟
   - **缓解**：使用 FutureBuilder 或预解码

3. **内存压力**：
   - 每帧 3.6MB，60fps = 216MB/s
   - **缓解**：帧缓冲复用、跳帧

### 技术债务

1. ⚠️ Base64 传输临时方案（待升级为共享内存）
2. ⚠️ 同步解码（待改为异步线程池）
3. ⚠️ 无帧缓冲复用（待优化内存分配）

---

## 后续优化方向

### Phase 2: 共享内存 + Texture

**目标**: 60fps @ 1080p

**实现**：
1. 使用 `dart:ffi` 的 `Pointer` 共享内存
2. Flutter Texture widget
3. 平台通道注册纹理 ID
4. GPU 直接渲染

**参考**：
- [Flutter Texture Class](https://api.flutter.dev/flutter/widgets/Texture-class.html)
- [Platform Channels](https://docs.flutter.dev/platform-integration/platform-channels)

### Phase 3: 硬件解码

**目标**: < 10ms 解码延迟

**实现**：
1. VideoToolbox 解码器（macOS）
2. Media Foundation 解码器（Windows）
3. VA-API 解码器（Linux）

---

## 参考资料

### 代码示例

- `crates/rdcs-codec/src/platform/openh264_decoder.rs` - 解码器实现
- `client/flutter/lib/core/ffi/engine_events.dart` - 事件系统
- `client/flutter/lib/features/session/session_screen.dart` - UI 层

### Flutter 文档

- [RawImage widget](https://api.flutter.dev/flutter/widgets/RawImage-class.html)
- [decodeImageFromPixels](https://api.flutter.dev/flutter/dart-ui/decodeImageFromPixels.html)
- [Image.memory](https://api.flutter.dev/flutter/widgets/Image/Image.memory.html)

### 相关文档

- [HARDWARE_ENCODER_PERFORMANCE.md](../testing/HARDWARE_ENCODER_PERFORMANCE.md)
- [NEXT_STEPS.md](../NEXT_STEPS.md)

---

**维护人**: AI Assistant  
**创建日期**: 2026-06-29  
**预计完成时间**: 2-3 天
