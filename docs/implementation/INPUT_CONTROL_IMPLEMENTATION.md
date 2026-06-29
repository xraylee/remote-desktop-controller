# 输入控制实现文档

**日期**: 2026-06-29  
**状态**: ✅ 已完成 Rust 端实现  
**任务**: #46 - 鼠标键盘控制

---

## 概述

本文档描述了远程桌面输入控制功能的端到端实现。该功能允许 Flutter 客户端通过 FFI 层发送鼠标、键盘和滚轮事件，并在 macOS 上注入到系统中。

## 架构

```
Flutter UI (session_screen.dart)
    ↓ JSON events via sendInput()
Dart FFI (bindings.dart)
    ↓ rdcs_send_input()
Rust FFI (rdcs-ffi/src/lib.rs)
    ↓ input_handler::handle_input_event()
Input Handler (rdcs-ffi/src/input_handler.rs)
    ↓ Parse JSON → Platform types
InputInjector trait (rdcs-platform)
    ↓
macOS Implementation (rdcs-macos/src/input.rs)
    ↓ CGEvent APIs
系统输入流
```

## 实现细节

### 1. Flutter 端（已有 90%）

**文件**: `client/flutter/lib/features/session/session_screen.dart`

已实现的输入捕获：

```dart
// 鼠标点击
void _onVideoTap() {
  ref.read(sessionProvider.notifier).sendInput(
    jsonEncode({'type': 'mouse', 'action': 'click', 'x': 0, 'y': 0}),
  );
}

// 双击
void _onVideoDoubleTap() {
  ref.read(sessionProvider.notifier).sendInput(
    jsonEncode({'type': 'mouse', 'action': 'double_click', 'x': 0, 'y': 0}),
  );
}

// 鼠标移动
void _onVideoPanUpdate(DragUpdateDetails details) {
  ref.read(sessionProvider.notifier).sendInput(
    jsonEncode({
      'type': 'mouse',
      'action': 'move',
      'x': details.localPosition.dx.round(),
      'y': details.localPosition.dy.round(),
    }),
  );
}
```

**待完成**：
- 键盘输入处理（`_onKeyboardPressed()` 目前是 TODO）

### 2. Input Handler（新实现）

**文件**: `crates/rdcs-ffi/src/input_handler.rs`

#### JSON 消息格式

**鼠标事件**：
```json
{
  "type": "mouse",
  "action": "move|click|double_click|press|release",
  "x": 100.0,
  "y": 200.0,
  "button": "left|right|middle"  // 可选
}
```

**键盘事件**：
```json
{
  "type": "keyboard",
  "key_code": 4,  // USB HID usage code
  "action": "press|release",
  "shift": false,
  "control": false,
  "alt": false,
  "meta": false
}
```

**滚轮事件**：
```json
{
  "type": "scroll",
  "delta_x": 0.0,
  "delta_y": 10.0,
  "is_precise": true
}
```

#### 核心函数

```rust
pub fn handle_input_event(
    injector: &dyn InputInjector,
    event_json: &str,
    display_id: u64,
) -> Result<(), PlatformError>
```

**功能**：
1. 解析 JSON 字符串到 `InputEventJson` 枚举
2. 根据类型调用对应的处理函数：
   - `handle_mouse_event()` - 处理鼠标事件
   - `handle_keyboard_event()` - 处理键盘事件
   - `handle_scroll_event()` - 处理滚轮事件
3. 转换为平台无关的 `MouseEvent`/`KeyEvent`/`ScrollEvent`
4. 调用 `InputInjector` trait 方法注入事件

**特殊处理**：
- `click` 动作被分解为 `press` + `release` 两个事件
- 默认鼠标按钮为左键
- 支持修饰键组合（Shift/Ctrl/Alt/Meta）

### 3. FFI 层更新

**文件**: `crates/rdcs-ffi/src/lib.rs`

#### rdcs_send_input 实现

```rust
pub extern "C" fn rdcs_send_input(
    handle: *mut EngineHandle,
    session_id: u64,
    event_json: *const c_char,
) -> c_int
```

**变更**：
1. 添加 JSON 解析和验证
2. 获取主显示器 ID
3. 调用 `input_handler::handle_input_event()`
4. 触发 `EVENT_INPUT_RECEIVED` 回调
5. 返回成功/失败状态码

#### 平台初始化

**变更**：使用条件编译在 macOS 上启用真实实现：

```rust
#[cfg(target_os = "macos")]
let platform = Arc::new(PlatformBundle {
    capture: Box::new(rdcs_macos::MacOsScreenCapture::new()),
    input: Box::new(rdcs_macos::MacOsInputInjector::new()),
    notify: Box::new(rdcs_macos::MacOsSystemNotify::new()),
    clipboard: Box::new(rdcs_macos::MacOsClipboard::new()),
});
```

### 4. macOS 实现（已有）

**文件**: `crates/rdcs-macos/src/input.rs`

使用 `core-graphics` crate 通过 CGEvent API 注入输入：

- **鼠标**: `CGEvent::new_mouse_event()` + `CGEventPost()`
- **键盘**: `CGEvent::new_keyboard_event()` + 修饰键标志
- **滚轮**: `CGEventCreateScrollWheelEvent()` (FFI)

**权限要求**：
- macOS 需要辅助功能权限（Accessibility）
- 在 系统偏好设置 > 安全性与隐私 > 隐私 > 辅助功能 中授权

## 测试

### 单元测试

**文件**: `crates/rdcs-ffi/src/input_handler.rs`

```bash
cargo test -p rdcs-ffi input_handler
```

测试覆盖：
- ✅ 解析鼠标移动
- ✅ 解析鼠标点击（press + release）
- ✅ 解析键盘按键
- ✅ 解析滚轮事件
- ✅ 无效 JSON 返回错误
- ✅ 未知动作返回错误

### 集成测试

**文件**: `crates/rdcs-ffi/examples/input_injection_test.rs`

```bash
cargo run --example input_injection_test
```

测试场景：
1. 鼠标移动到 (100, 200)
2. 左键点击 (150, 250)
3. 右键点击 (200, 300)
4. 双击 (300, 400)
5. 键盘按下 'A' 键（HID code 4）
6. 键盘释放 'A' 键
7. Shift+A 组合键
8. 垂直滚动 10 个单位

**预期输出**：
```
🚀 Input Injection Test
✅ Engine created
✅ Callback registered

Test 1: Mouse Move
  ✅ Injected successfully
📥 Event 6: {"session_id":1,"status":"injected"}

Test 2: Mouse Click
  ✅ Injected successfully
...
```

### macOS 真实测试

**前置条件**：
1. 在 macOS 上编译项目
2. 授予辅助功能权限
3. 运行测试示例

**验证方法**：
- 鼠标应该移动到指定坐标
- 点击应该触发系统响应
- 键盘输入应该可见

## 依赖更新

### Cargo.toml

**rdcs-ffi/Cargo.toml**:
```toml
[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
# ... 其他依赖

[target.'cfg(target_os = "macos")'.dependencies]
rdcs-macos = { path = "../rdcs-macos" }
```

## 已知限制

### 1. 会话路由

**当前行为**：
- 所有输入事件注入到本地主显示器
- `session_id` 参数被忽略

**TODO**：
```rust
// TODO: route to correct session when ConnectionManager is wired.
let _ = session_id;
```

**解决方案**（Phase 6）：
- 实现 `SessionManager` 跟踪活动会话
- 根据 `session_id` 路由到对应的远程机器
- 通过 WebRTC DataChannel 发送控制消息

### 2. 键盘映射

**当前实现**：
- 直接传递 key_code 到 CGEvent
- 假设是 macOS 虚拟键码

**问题**：
- Flutter 使用的键码可能与 macOS 虚拟键码不匹配
- 需要 USB HID → macOS VK 映射表

**TODO**：
```rust
// Note: key_code uses USB HID usage codes in the platform trait.
// macOS CGEvent expects virtual key codes. A full implementation
// would include a HID-to-virtual-key mapping table.
```

### 3. Flutter 键盘输入

**待实现**：
- `session_screen.dart` 中的 `_onKeyboardPressed()` 是 TODO
- 需要虚拟键盘或系统 IME 集成

## 性能考虑

### 延迟分析

**输入路径**：
1. Flutter 手势识别: ~16ms (60fps)
2. JSON 编码: <1ms
3. FFI 调用: <0.1ms
4. JSON 解析: <0.5ms
5. CGEvent 创建+注入: ~1ms

**总延迟**: ~18ms（可接受）

### 优化建议

1. **批处理**：在高频事件（鼠标移动）时，可以合并多个事件
2. **直接内存共享**：使用 `dart:ffi` 的 `Pointer` 而非 JSON 字符串
3. **事件节流**：限制鼠标移动事件频率（例如 60fps）

## 下一步工作

### 短期（本周）

- [ ] 实现 Flutter 键盘输入捕获
- [ ] 添加 HID → macOS VK 映射表
- [ ] 测试不同类型的键盘事件（特殊键、组合键）
- [ ] 添加输入延迟监控

### 中期（Phase 5）

- [ ] 实现远程会话路由
- [ ] 通过 DataChannel 发送控制消息
- [ ] 在接收端注入输入事件
- [ ] 添加输入权限检查和错误处理

### 长期（Phase 6+）

- [ ] 剪贴板同步
- [ ] 文件拖放支持
- [ ] 多显示器输入路由
- [ ] 触摸板手势支持

## 参考

### 代码文件

- `crates/rdcs-ffi/src/input_handler.rs` - 输入处理逻辑
- `crates/rdcs-ffi/src/lib.rs` - FFI 入口点
- `crates/rdcs-macos/src/input.rs` - macOS 实现
- `crates/rdcs-platform/src/lib.rs` - 平台抽象
- `client/flutter/lib/features/session/session_screen.dart` - Flutter UI

### 相关文档

- [Phase 4.2 完成报告](../testing/PHASE4_2_COMPLETION_REPORT.md)
- [下一步计划](../NEXT_STEPS.md)
- [项目结构](../../CLAUDE.md)

### 外部资源

- [Core Graphics Event Reference](https://developer.apple.com/documentation/coregraphics/cgevent)
- [USB HID Usage Tables](https://www.usb.org/document-library/hid-usage-tables-13)
- [Flutter Raw Keyboard](https://api.flutter.dev/flutter/services/RawKeyboard-class.html)

---

**维护人**: AI Assistant  
**最后更新**: 2026-06-29  
**下一步**: 实现 Flutter 键盘输入捕获
