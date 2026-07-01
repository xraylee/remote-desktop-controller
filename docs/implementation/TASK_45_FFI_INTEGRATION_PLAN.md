# Task #45 技术分析：FFI 视频集成方案

**日期**: 2026-06-29  
**子任务**: Subtask 45.1 - 分析 FFI 视频集成需求  
**状态**: ✅ 分析完成

---

## 🔍 当前状态分析

### 1. FFI 层现状

**文件**: `crates/rdcs-ffi/src/lib.rs`

**当前实现**：
```rust
pub struct EngineHandle {
    _runtime: Runtime,              // ✅ Tokio runtime 已有
    callbacks: Arc<Mutex<Vec<...>>>,// ✅ 事件回调机制已有
    platform: Arc<PlatformBundle>,  // ✅ 平台实现已有
    // ❌ 缺少：WebRTC PeerConnection
    // ❌ 缺少：VideoChannel
    // ❌ 缺少：VideoFrameHandler
}
```

**已有的 FFI 函数**：
- `rdcs_start_capture()` - ✅ 启动捕获（Mock 实现）
- `rdcs_connect()` - ✅ 连接远程（Mock 实现）
- `rdcs_send_input()` - ✅ 发送输入（真实实现）
- `rdcs_register_callback()` - ✅ 注册事件回调

**缺失的功能**：
- ❌ 真实的 WebRTC 连接
- ❌ VideoChannel 集成
- ❌ 视频帧接收和解码

### 2. Flutter 端现状

**文件**: `client/flutter/lib/core/ffi/engine_isolate.dart`

**当前实现**：
```dart
class EngineIsolate {
  Isolate? _isolate;                               // ✅ 后台 isolate
  SendPort? _commandPort;                          // ✅ 命令通道
  final _eventController = StreamController<...>(); // ✅ 事件流
  
  /// Public event stream ✅
  Stream<EngineEvent> get eventStream => _eventController.stream;
}
```

**已有的功能**：
- ✅ 事件流基础设施（`_eventController`）
- ✅ 后台 isolate 执行 FFI 调用
- ✅ 原生回调 → Dart 事件转换（`_nativeEventCallback`）

**缺失的连接**：
- ❌ `VideoRenderer` 未订阅 `eventStream`
- ❌ `EVENT_FRAME_READY` 事件未处理

### 3. 视频管道现状

**已实现的组件**：

```
✅ rdcs-macos::MacOsScreenCapture    (真实捕获)
✅ rdcs-codec::VideoToolboxEncoder   (硬件编码)
✅ rdcs-connection::VideoChannel     (DataChannel 封装)
✅ rdcs-connection::FrameReassembler (分片重组)
✅ rdcs-codec::OpenH264Decoder       (软件解码)
✅ rdcs-ffi::video_handler           (YUV→BGRA + 事件分发)
✅ Flutter VideoRenderer             (UI 渲染)
```

**缺失的连接**：

```
❌ FFI 层集成 WebRTC
  └─ 需要在 EngineHandle 中添加 PeerConnection
  └─ 需要设置 DataChannel 回调
  └─ 需要调用 video_handler

❌ Flutter 事件流连接
  └─ VideoRenderer 需要订阅 eventStream
  └─ 解析 FramePayload
  └─ 渲染到 UI
```

---

## 💡 集成方案

### 方案 A：最小可行集成（推荐 MVP）

**目标**: 快速实现本地回环测试

**架构**：
```
┌──────────────────────────────────────┐
│ FFI Layer (rdcs-ffi)                 │
│                                      │
│  EngineHandle {                      │
│    ✅ platform: MacOsScreenCapture   │
│    ✅ callbacks: Event dispatch      │
│    ❌ → 添加: encoder               │
│    ❌ → 添加: video_handler         │
│  }                                   │
│                                      │
│  rdcs_start_capture() {              │
│    1. 启动捕获线程                    │
│    2. 启动编码线程（新增）            │
│    3. 模拟接收并解码（新增）          │
│    4. 分发 EVENT_FRAME_READY         │
│  }                                   │
└──────────────────────────────────────┘
         ↓ EVENT_FRAME_READY
┌──────────────────────────────────────┐
│ Flutter (EngineIsolate)              │
│                                      │
│  eventStream ✅                       │
│    ↓                                 │
│  VideoRenderer ✅                     │
│    ↓                                 │
│  RawImage 渲染 ✅                     │
└──────────────────────────────────────┘
```

**实现步骤**：

1. **在 EngineHandle 中添加编码器和解码器**
   ```rust
   pub struct EngineHandle {
       // 现有字段...
       encoder: Arc<Mutex<Option<NativeVideoEncoder>>>,
       video_handler: Arc<VideoFrameHandler>,
   }
   ```

2. **更新 rdcs_start_capture()**
   ```rust
   pub extern "C" fn rdcs_start_capture(...) {
       // 1. 启动捕获（已有）
       let frame_rx = platform.capture.start(config)?;
       
       // 2. 启动编码+解码循环（新增）
       let engine_ptr = handle as *const EngineHandle;
       tokio::spawn(async move {
           while let Ok(frame) = frame_rx.recv() {
               // 编码
               let encoded = encoder.encode(&frame)?;
               
               // 解码并分发（模拟接收）
               video_handler.handle_encoded_frame(
                   unsafe { &*engine_ptr },
                   &encoded,
                   session_id,
               )?;
           }
       });
   }
   ```

3. **连接 Flutter 事件流**（见方案细节）

**优点**：
- ✅ 快速实现（1-2 天）
- ✅ 可以立即测试视频渲染
- ✅ 无需 WebRTC 复杂性
- ✅ 本地回环，易于调试

**缺点**：
- ⚠️ 不是真实的远程连接
- ⚠️ 需要后续迁移到真实 WebRTC

---

### 方案 B：完整 WebRTC 集成

**目标**: 实现真实的远程桌面连接

**架构**：
```
┌──────────────────────────────────────┐
│ FFI Layer (rdcs-ffi)                 │
│                                      │
│  EngineHandle {                      │
│    ice_agent: RealIceAgent,          │
│    peer_connections: HashMap<...>,   │
│    video_channels: HashMap<...>,     │
│  }                                   │
│                                      │
│  rdcs_connect() {                    │
│    1. 建立 ICE 连接                  │
│    2. 创建 DataChannel               │
│    3. 设置接收回调                   │
│    4. 返回 session_id                │
│  }                                   │
│                                      │
│  DataChannel.on_message {            │
│    video_handler.handle_encoded_frame() │
│  }                                   │
└──────────────────────────────────────┘
```

**实现步骤**：

1. **添加 WebRTC 依赖**
   ```rust
   // EngineHandle 新增字段
   ice_agent: Arc<Mutex<Option<RealIceAgent>>>,
   sessions: Arc<Mutex<HashMap<u64, SessionState>>>,
   
   struct SessionState {
       peer_connection: Arc<...>,
       video_channel: Arc<VideoChannel>,
       reassembler: Arc<Mutex<FrameReassembler>>,
   }
   ```

2. **实现真实的 rdcs_connect()**
   ```rust
   pub extern "C" fn rdcs_connect(...) -> c_int {
       // 1. 创建 ICE agent
       let agent = RealIceAgent::new(ice_servers).await?;
       
       // 2. 建立连接（信令交换）
       // ... 

       // 3. 获取 DataChannel
       let dc = agent.get_data_channel()?;
       let video_channel = VideoChannel::new(dc);
       
       // 4. 设置接收回调
       video_channel.on_message({
           let engine_ptr = handle as *const EngineHandle;
           let video_handler = &engine.video_handler;
           move |chunk| {
               // 重组帧
               if let Some(complete) = reassembler.add_chunk(...) {
                   video_handler.handle_encoded_frame(
                       unsafe { &*engine_ptr },
                       &complete,
                       session_id,
                   ).ok();
               }
           }
       });
       
       // 5. 返回 session_id
       session_id as c_int
   }
   ```

**优点**：
- ✅ 完整的远程桌面功能
- ✅ 真实的 P2P 连接
- ✅ 符合最终架构

**缺点**：
- ⚠️ 实现复杂（3-5 天）
- ⚠️ 需要信令服务器
- ⚠️ 调试困难

---

## 🎯 推荐方案：混合策略

**阶段 1：本地回环（方案 A）** - 1-2 天
- 实现最小可行的视频渲染
- 验证 Flutter 渲染管道
- 验证性能

**阶段 2：WebRTC 集成（方案 B）** - 2-3 天
- 添加真实的 P2P 连接
- 迁移到 DataChannel 接收
- 完整的远程桌面

---

## 📋 Subtask 45.2 实现计划：Flutter 事件流连接

### 问题分析

**当前代码** (`video_renderer.dart`):
```dart
void _subscribeToFrameEvents() {
  // TODO: Get event stream from engine provider
  // _eventSubscription = ref.read(engineProvider).eventStream.listen(...);
}
```

**需要**：
1. 暴露 `EngineIsolate` 到 Riverpod provider
2. `VideoRenderer` 订阅 `eventStream`
3. 解析 `FramePayload`
4. 渲染到 UI

### 实现步骤

#### Step 1: 创建 Engine Provider

**文件**: `client/flutter/lib/core/ffi/engine_providers.dart`（新建）

```dart
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'engine_isolate.dart';
import 'engine_events.dart';

/// Singleton engine isolate provider.
final engineIsolateProvider = Provider<EngineIsolate>((ref) {
  final engine = EngineIsolate();
  ref.onDispose(() => engine.dispose());
  return engine;
});

/// Stream provider for engine events.
final engineEventsProvider = StreamProvider<EngineEvent>((ref) {
  final engine = ref.watch(engineIsolateProvider);
  return engine.eventStream;
});
```

#### Step 2: 更新 VideoRenderer

**文件**: `client/flutter/lib/features/session/video_renderer.dart`

```dart
@override
void initState() {
  super.initState();
  _subscribeToFrameEvents();
}

void _subscribeToFrameEvents() {
  // 订阅引擎事件流
  _eventSubscription = ref
      .read(engineEventsProvider.stream)
      .listen((event) {
    if (event.type == EngineEventId.frameReady) {
      try {
        final payload = FramePayload.fromJson(event.payload);
        _onFrameReceived(payload);
      } catch (e) {
        debugPrint('❌ Failed to parse frame payload: $e');
      }
    }
  });
}
```

#### Step 3: 初始化引擎

**文件**: `client/flutter/lib/app.dart` 或 `main.dart`

```dart
// 在 app 启动时初始化引擎
Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  
  final container = ProviderContainer();
  final engine = container.read(engineIsolateProvider);
  await engine.init();  // 启动 isolate
  
  runApp(
    UncontrolledProviderScope(
      container: container,
      child: MyApp(),
    ),
  );
}
```

---

## ✅ 下一步行动

### 立即执行（Subtask 45.2）

1. **创建 `engine_providers.dart`** - 15 分钟
2. **更新 `video_renderer.dart`** - 15 分钟
3. **测试事件流连接** - 30 分钟

**总计**: 1 小时

### 后续执行（Subtask 45.3）

1. **实现方案 A（本地回环）** - 4-6 小时
2. **端到端测试** - 2 小时
3. **性能测量** - 1 小时

**总计**: 1 天

---

## 📊 预期结果

完成后：
- ✅ Flutter 可以接收视频帧事件
- ✅ VideoRenderer 可以渲染视频
- ✅ FPS/延迟监控正常工作
- ✅ Task #45 达到 90%

**MVP 进度**: 90% → **95%** 🎉

---

**维护人**: AI Assistant  
**创建日期**: 2026-06-29  
**建议**: 先完成 Flutter 事件流连接（快速见效）
