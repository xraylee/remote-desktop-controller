# Plan D: Flutter 客户端实现计划

**版本**: v1.0 | **日期**: 2026-06-30 | **状态**: 草稿
**目标平台**: macOS (MVP), Windows (V1.1)
**依赖文档**: architecture-design.md (Section 2), prd-v1.md (Section 4/6/7)

## 概览

覆盖 RDCS Flutter 客户端 **UI 层**（Flutter Widgets + Riverpod 状态管理）。Bridge 层仅含 Dart 侧封装，Rust 侧实现在 Plan C。

**技术栈**: Flutter 3.x, flutter_riverpod, go_router, dart:ffi, path_provider, window_manager, tray_manager, freezed

## 跨项目依赖

依赖: Plan A (librdcs_core FFI), Plan F (开发环境), Plan B (信令服务联调)

### Task 1: Project Setup (~1d)

**目标**: 初始化 Flutter 项目骨架，配置核心依赖和基础主题。确保 `flutter run -d macos` 可启动空白窗口。

**依赖**: 无
**文件**: `pubspec.yaml`, `lib/main.dart`, `lib/app/theme.dart`, `lib/app/router.dart`

**接口契约**:

```dart
// theme.dart
class RdcsTheme {
  static ThemeData get light;
  static ThemeData get dark;
  static const Color primary = Color(0xFF2563EB);
  static const Color success = Color(0xFF22C55E);
  static const Color warning = Color(0xFFEAB308);
  static const Color danger = Color(0xFFEF4444);
}
// router.dart — Routes: / (main), /session/:id, /settings
final routerProvider = Provider<GoRouter>((ref) { ... });

// pubspec.yaml deps:
// flutter_riverpod ^2.5, go_router ^14.0, path_provider ^2.1
// window_manager ^0.4, tray_manager ^0.3, freezed_annotation ^2.4, ffi ^2.1
```

**验收标准**:
- [ ] `flutter run -d macos` 启动无错误
- [ ] GoRouter 路由 `/` → `/settings` → `/` 切换正常
- [ ] 主题色 `primary` 应用到 MaterialApp

### Task 2: Config Management (~1d)

**目标**: 实现 `~/.rdcs/config.json` 的加载/保存，类型安全配置模型。覆盖服务器、画质、通用三组配置（architecture-design.md 2.7）。
**依赖**: Task 1 | **文件**: `lib/config/rdcs_config.dart`, `lib/config/config_repository.dart`

**接口契约**:

```dart
@freezed class ServerConfig with _$ServerConfig {
  factory ServerConfig({required String signalingUrl, required String relayAddress,
    required String apiUrl, required ConnectionMode connectionMode}) = _ServerConfig;
}
@freezed class QualityConfig with _$QualityConfig {
  factory QualityConfig({required QualityMode mode, required String maxResolution,
    required int maxFps, required bool hardwareAccel}) = _QualityConfig;
}
@freezed class GeneralConfig with _$GeneralConfig {
  factory GeneralConfig({required bool autostart, required bool minimizeToTray,
    required ConfirmMode confirmMode, required bool soundEnabled,
    required bool telemetry, required String language}) = _GeneralConfig;
}
@freezed class RdcsConfig with _$RdcsConfig {
  factory RdcsConfig({required ServerConfig server, required QualityConfig quality,
    required GeneralConfig general}) = _RdcsConfig;
}
enum ConnectionMode { auto, lanOnly, forceRelay }
enum QualityMode { auto, sharpness, smoothness }
enum ConfirmMode { ask, autoSameTeam, autoAll }

abstract class ConfigRepository {
  Future<RdcsConfig> load();
  Future<void> save(RdcsConfig config);
  Future<String> configPath(); // ~/.rdcs/config.json
}
class FileConfigRepository implements ConfigRepository { ... }
```

**验收标准**:
- [ ] `config_roundtrip_test`: save → load 数据一致
- [ ] `config_defaults_test`: 首次启动生成默认 config.json
- [ ] `config_path_test`: 路径为 `~/.rdcs/config.json`

### Task 3: FFI Bridge — Dart Side (~2d)

**目标**: 封装 `dart:ffi` DynamicLibrary 加载和 12 个 FFI 函数绑定（architecture-design.md 2.2）。通过 ReceivePort 将 Rust 异步回调转为 Dart `Stream<EngineEvent>`。
**依赖**: Task 1 | **文件**: `lib/ffi/bridge.dart`, `lib/ffi/bindings.dart`, `lib/ffi/events.dart`

**接口契约**:

```dart
// events.dart — sealed class, 12 种事件 (对应 architecture-design.md 2.3)
sealed class EngineEvent { const EngineEvent(); }
class FrameReady extends EngineEvent { String sessionId; int textureId, width, height; }
class ConnectionRequest extends EngineEvent { String fromDeviceCode, fromName; }
class ConnectionEstablished extends EngineEvent { String sessionId, path; int latencyMs; }
class ConnectionLost extends EngineEvent { String sessionId, reason; }
class ConnectionRestored extends EngineEvent { String sessionId, newPath; int latencyMs; }
class NearbyDeviceFound extends EngineEvent { String deviceCode, name, platform; }
class NearbyDeviceLost extends EngineEvent { String deviceCode; }
class FileTransferProgress extends EngineEvent { String transferId; double progressPct, speed; }
class FileTransferComplete extends EngineEvent { String transferId, destPath; bool success; }
class ChatMessageEvent extends EngineEvent { String sessionId, text; DateTime timestamp; }
class QualityChanged extends EngineEvent { String sessionId, newMode, reason; }
class InputReceived extends EngineEvent { String sessionId, eventType, data; }

// bindings.dart — 12 个 C ABI 函数 (DynamicLibrary lookup)
class RdcsBindings {
  late final DynamicLibrary _lib;
  // rdcsEngineCreate(configJson)→handle, rdcsEngineDestroy(handle),
  // rdcsStartCapture(handle,cfg)→i32, rdcsStopCapture(handle)→i32,
  // rdcsConnect(handle,code)→i32, rdcsDisconnect(handle,sid)→i32,
  // rdcsSendInput(handle,sid,json)→i32, rdcsSendFile(handle,sid,path,dest)→i32,
  // rdcsSendMessage(handle,sid,text)→i32, rdcsSetQuality(handle,sid,mode)→i32,
  // rdcsGenerateInvite(handle)→*char, rdcsRegisterCallback(handle,evtId,fn)→i32
  RdcsBindings() { _lib = _loadLibrary(); }
}

// bridge.dart — 高层封装
class RdcsBridge {
  Stream<EngineEvent> get events;
  Future<void> init(RdcsConfig config);
  Future<void> dispose();
  Future<int> connect(String targetCode);
  Future<void> disconnect(String sessionId);
  Future<int> startCapture(QualityConfig config);
  Future<void> stopCapture();
  Future<void> sendInput(String sessionId, Map<String, dynamic> event);
  Future<int> sendFile(String sessionId, String path, String dest);
  Future<void> sendMessage(String sessionId, String text);
  Future<void> setQuality(String sessionId, int mode);
  Future<String> generateInvite();
}
```

**验收标准**:
- [ ] `bindings_load_test`: DynamicLibrary 加载 mock .dylib 无异常
- [ ] `events_parse_test`: JSON → EngineEvent 反序列化覆盖全部 12 种事件
- [ ] `bridge_stream_test`: ReceivePort 消息正确转为 `Stream<EngineEvent>`

### Task 4: State Management (~1.5d)

**目标**: Riverpod Provider 覆盖连接、会话、设备列表、聊天、文件传输五个领域。消费 `RdcsBridge` 事件流更新 UI 状态。
**依赖**: Task 2, Task 3 | **文件**: `lib/state/{connection,session,device_list,chat,file_transfer}_notifier.dart`

**接口契约**:

```dart
// connection_notifier.dart
enum ConnectionStatus { idle, connecting, connected, reconnecting, failed }
@freezed class ConnectionState with _$ConnectionState {
  factory ConnectionState({required ConnectionStatus status, String? sessionId,
    String? path, int? latencyMs, String? errorMessage, int? reconnectCountdown}) = _ConnectionState;
}
class ConnectionNotifier extends StateNotifier<ConnectionState> {
  void connect(String deviceCode);
  void disconnect();
}
final connectionProvider = StateNotifierProvider<ConnectionNotifier, ConnectionState>();

// session_notifier.dart
@freezed class SessionState with _$SessionState {
  factory SessionState({required String sessionId, required String remoteName,
    required int latencyMs, required int fps, required String resolution,
    required String path, required bool isRecording}) = _SessionState;
}
final sessionProvider = StateNotifierProvider<SessionNotifier, SessionState?>();

// device_list_notifier.dart
@freezed class NearbyDevice with _$NearbyDevice {
  factory NearbyDevice({required String deviceCode, required String name,
    required String platform, required bool online}) = _NearbyDevice;
}
class DeviceListNotifier extends StateNotifier<List<NearbyDevice>> { }
final deviceListProvider = StateNotifierProvider<DeviceListNotifier, List<NearbyDevice>>();

// chat_notifier.dart
@freezed class ChatMessage with _$ChatMessage {
  factory ChatMessage({required String text, required DateTime timestamp, required bool isLocal}) = _ChatMessage;
}
class ChatNotifier extends StateNotifier<List<ChatMessage>> {
  Future<void> send(String sessionId, String text);
}
final chatProvider = StateNotifierProvider<ChatNotifier, List<ChatMessage>>();

// file_transfer_notifier.dart
@freezed class FileTransfer with _$FileTransfer {
  factory FileTransfer({required String transferId, required String fileName,
    required double progressPct, required double speed, required bool complete, required bool success}) = _FileTransfer;
}
class FileTransferNotifier extends StateNotifier<List<FileTransfer>> {
  Future<void> startTransfer(String sessionId, String path, String dest);
}
final fileTransferProvider = StateNotifierProvider<FileTransferNotifier, List<FileTransfer>>();
```

**验收标准**:
- [ ] `connection_state_test`: idle → connecting → connected 流转正确
- [ ] `device_list_test`: Found/Lost 事件正确增删列表
- [ ] `chat_test`: send + onReceived 消息按时间排序

### Task 5: Main Interface (~2d)

**目标**: 主界面双卡片布局（PRD 6.2.1）——上方"让别人连接我"（设备码 + 邀请码），下方"连接远程电脑"（输入 + 附近设备）。底部状态栏含路径标签和管理控制台入口。
**依赖**: Task 4 | **文件**: `lib/screens/main_screen.dart`, `lib/screens/widgets/{device_code_card,connect_card,nearby_device_tile,invite_code_panel,connection_status_bar}.dart`

**接口契约**:

```dart
class MainScreen extends ConsumerWidget { }          // 双卡片垂直布局
class DeviceCodeCard extends ConsumerWidget { }       // 9 位设备码 (123 456 789) + 复制按钮
class InviteCodePanel extends ConsumerStatefulWidget { } // 4 位邀请码 + 10 分钟倒计时 + 刷新
class ConnectCard extends ConsumerStatefulWidget { }  // 设备码输入 + 连接按钮 + 附近设备列表
class NearbyDeviceTile extends StatelessWidget { } // 在线绿点 + 设备名 + [连接]
  // final NearbyDevice device; final VoidCallback onConnect;
class ConnectionStatusBar extends ConsumerWidget { }  // L1/L2/L3 路径标签 + 管理控制台链接
```

**验收标准**:
- [ ] `main_layout_test`: 双卡片垂直排列，窗口 480x640 最小尺寸无溢出
- [ ] `device_code_display_test`: 格式 `XXX XXX XXX` 9 位数字
- [ ] `connect_input_test`: 非数字自动过滤，不足 9 位时连接按钮禁用
- [ ] `nearby_sort_test`: 在线优先、同级按最近连接时间倒序

### Task 6: Remote Session View (~2d)

**目标**: 远程会话界面（PRD 6.2.2）——Texture 渲染远程画面、浮动工具栏（<600px 溢出）、底部性能栏（颜色阈值）、网络异常横幅。
**依赖**: Task 4 | **文件**: `lib/screens/session_screen.dart`, `lib/screens/widgets/{session_toolbar,performance_bar,network_banner,chat_panel}.dart`

**接口契约**:

```dart
class SessionScreen extends ConsumerWidget { } // Texture + Stack(Toolbar/PerfBar/Banner)
class SessionToolbar extends ConsumerWidget { } // 全屏/文件/剪贴板/录制/聊天/画质▾/断开; <600px入OverflowMenu
class PerformanceBar extends ConsumerWidget { } // 延迟<50ms绿/50-150ms黄/>150ms红; 帧率>30绿/15-30黄/<15红
class NetworkBanner extends ConsumerStatefulWidget { } // >150ms黄/>300ms红; 恢复后3s淡出
class ChatPanel extends ConsumerWidget { } // 侧边抽屉消息列表 + 输入框
```

**验收标准**:
- [ ] `session_layout_test`: Texture + 工具栏 + 性能栏 Stack 正确
- [ ] `performance_color_test`: 阈值对应颜色切换
- [ ] `toolbar_overflow_test`: <600px 低频操作移入溢出菜单
- [ ] `banner_fade_test`: 异常出现、恢复后 3 秒淡出

### Task 7: Controlled-End Experience (~1.5d)

**目标**: 被控端完整体验（PRD 4.2.2 + arch 2.6）——连接确认弹窗（30s 倒计时）、浮动条（3s 淡出）、并发请求对话框、会话结束 Toast。所有被控端 UI 不参与屏幕捕获。
**依赖**: Task 4 | **文件**: `lib/screens/widgets/{connection_confirm_dialog,concurrent_request_dialog,controlled_floating_bar,session_end_toast}.dart`

**接口契约**:

```dart
class ConnectionConfirmDialog extends ConsumerStatefulWidget {
  final String fromName, fromDeviceCode;
} // 居中弹窗 + [拒绝]次要 + [允许连接]主色 + 30s倒计时自动拒绝
class ConcurrentRequestDialog extends StatelessWidget {
  final String fromName;
} // "已有远程连接，是否接受新连接？" + [拒绝] + [接受并断开当前]
class ControlledFloatingBar extends StatefulWidget {
  final String controllerName;
  final VoidCallback onDisconnect;
} // "● 张工 正在查看你的屏幕 [断开连接]" — 3s后AnimatedOpacity淡出
class SessionEndToast extends StatefulWidget {
  final String controllerName;
  final Duration sessionDuration;
} // "张工 已断开连接，时长 XX 分钟" — 3s后自动消失
```

**验收标准**:
- [ ] `confirm_timeout_test`: 30s 无操作自动拒绝
- [ ] `floating_bar_fade_test`: 3 秒淡出 opacity 1.0→0.0
- [ ] `toast_duration_test`: 时长格式正确（分钟），3 秒后移除
- [ ] `concurrent_dialog_test`: 已有会话时弹出确认对话框

### Task 8: Settings Panel (~1.5d)

**目标**: 设置面板（arch 2.7）——服务器配置、画质设置、通用设置三分区。修改实时持久化到 config.json。
**依赖**: Task 2, Task 4 | **文件**: `lib/screens/settings_screen.dart`, `lib/screens/widgets/{server,quality,general}_config_section.dart`

**接口契约**:

```dart
class SettingsScreen extends ConsumerWidget { } // 三分区垂直 + debounce 500ms save
class ServerConfigSection extends ConsumerStatefulWidget { } // 信令/中转/API地址(Text) + 连接模式(Segmented)
class QualityConfigSection extends ConsumerWidget { } // 画质(Segmented) + 分辨率/帧率(Dropdown) + 硬件加速(Switch)
class GeneralConfigSection extends ConsumerWidget { } // 自启动/最小化/提示音/遥测(Switch) + 确认模式/语言(Dropdown)

final configProvider = StateNotifierProvider<ConfigNotifier, RdcsConfig>();
class ConfigNotifier extends StateNotifier<RdcsConfig> {
  Future<void> updateServer(ServerConfig c);
  Future<void> updateQuality(QualityConfig c);
  Future<void> updateGeneral(GeneralConfig c);
}
```

**验收标准**:
- [ ] `settings_persist_test`: 修改信令地址 → 重启后值保留
- [ ] `settings_debounce_test`: 快速连续修改仅触发一次 save
- [ ] `settings_defaults_test`: 首次打开显示所有默认值

### Task 9: Menu Bar / Tray (~1.5d)

**目标**: macOS NSStatusBar + Windows 系统托盘（PRD 6.2.4/7.2.2）。关闭窗口≠退出，保持在线。会话中图标蓝色脉冲，点击展开快捷菜单。
**依赖**: Task 4, Task 5 | **文件**: `lib/platform/tray_service.dart`, `lib/platform/{macos,windows}_tray.dart`

**接口契约**:

```dart
abstract class TrayService {
  Future<void> init();
  Future<void> dispose();
  Future<void> setOnlineStatus(bool online);
  Future<void> setSessionActive(bool active, {String? controllerName, Duration? duration});
  Future<void> updateNearbyDevices(List<NearbyDevice> devices);
  Stream<TrayAction> get actions;
}
enum TrayAction { openMainWindow, generateInvite, connectDevice, openWebConsole,
  openSettings, toggleAutostart, disconnectSession, quit }

class MacOSTrayService implements TrayService { }  // NSStatusBar + NSMenu
class WindowsTrayService implements TrayService { } // 系统托盘 + 右键菜单

class WindowCloseHandler {
  // minimizeToTray==true: 隐藏窗口保持进程; false: 正常退出
}
```

**验收标准**:
- [ ] `tray_init_test`: 启动后菜单栏图标可见
- [ ] `tray_close_test`: 关闭主窗口后进程仍运行
- [ ] `tray_session_test`: 会话中图标变蓝脉冲
- [ ] `tray_actions_test`: 各菜单项触发对应 TrayAction

### Task 10: Widget Tests (~2d)

**目标**: Widget 测试覆盖布局、交互流、配置持久化、FFI mock 四维度。Mock FFI 层隔离 Rust 依赖。
**依赖**: Task 1-9 | **文件**: `test/` 目录下对应测试文件

**接口契约**:

```dart
// test/helpers/mock_bridge.dart
class MockRdcsBridge implements RdcsBridge {
  final _ctrl = StreamController<EngineEvent>.broadcast();
  Stream<EngineEvent> get events => _ctrl.stream;
  void emitEvent(EngineEvent event); // 注入模拟事件
}
// test/helpers/mock_config_repository.dart
class MockConfigRepository implements ConfigRepository {
  RdcsConfig? _stored;
  Future<RdcsConfig> load() async => _stored ?? RdcsConfig.defaults();
  Future<void> save(RdcsConfig config) async => _stored = config;
}
```

**测试矩阵** (15 个测试文件):
- **Config**: `config_repository_test` — load/save/defaults/路径
- **State**: `connection_notifier_test` (状态流转), `device_list_notifier_test` (增删), `chat_notifier_test` (排序), `file_transfer_notifier_test` (进度)
- **Screen**: `main_screen_test` (双卡片/设备码/输入), `session_screen_test` (Texture/工具栏/性能栏), `settings_screen_test` (分区/持久化/debounce)
- **Widget**: `connection_confirm_dialog_test` (30s倒计时), `controlled_floating_bar_test` (3s淡出), `performance_bar_test` (颜色阈值), `network_banner_test` (异常/恢复)
- **FFI**: `bridge_test` (DynamicLibrary mock), `events_test` (12种事件反序列化)
- **Platform**: `tray_service_test` (初始化/菜单/关闭拦截)

**验收标准**:
- [ ] `flutter test` 全部通过，零跳过
- [ ] Mock FFI 覆盖全部 12 种 EngineEvent
- [ ] 所有 P0 组件有 Widget 测试

## 依赖关系与工时

```
Task 1 → Task 2 → Task 4 → Task 5/6/7/8/9 (并行)
Task 1 → Task 3 → Task 4      Task 10 穿插进行
```

| Task | 名称 | 预估 | | Task | 名称 | 预估 |
|------|------|------|-|------|------|------|
| 1 | Project Setup | 1d | | 6 | Remote Session View | 2d |
| 2 | Config Management | 1d | | 7 | Controlled-End Experience | 1.5d |
| 3 | FFI Bridge (Dart) | 2d | | 8 | Settings Panel | 1.5d |
| 4 | State Management | 1.5d | | 9 | Menu Bar / Tray | 1.5d |
| 5 | Main Interface | 2d | | 10 | Widget Tests | 2d |

**总计: 16d (~3 周)** | 并行压缩后: **2~2.5 周**
