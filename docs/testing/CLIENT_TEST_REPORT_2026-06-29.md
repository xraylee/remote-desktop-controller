# RDCS 客户端完整测试报告

**日期**: 2026-06-29  
**版本**: Phase 2（视频传输 95%）  
**测试范围**: Flutter APP 客户端 + Web 管理控制台  
**测试规范**: Superpowers Skills 标准

---

## 执行摘要

| 客户端 | 测试文件数 | 测试用例数 | 预计通过 | 已知失败 | 缺口 |
|--------|-----------|-----------|---------|---------|------|
| Flutter APP | 7 文件 | ~75 用例 | ~68 | 2 文件 | FFI 层无单元测试 |
| Web Admin | 5 文件 | ~20 用例 | ~20 | 0 | 3 个页面无测试 |

---

## 一、Flutter APP 客户端测试

### 1.1 测试基础设施 (`test/helpers.dart`)

**评级**: ✅ 优秀

测试基础设施完善，架构清晰：

- `FakeEngineIsolate` — 实现完整的 `EngineIsolate` 接口，带调用追踪字段（`lastConnectCode`、`lastDisconnectSessionId` 等），支持注入事件流
- `FakeConfigRepository` — 内存实现，无磁盘 I/O，支持 `ensureDeviceCode`
- `TestSessionNotifier` — 暴露 `setSessionState()` 方便测试直接控制状态
- `pumpTestApp()` — 统一创建带完整路由的测试应用，注入所有 provider 覆盖

**策略**: 全面采用 Mock，无真实 FFI 调用，符合规范中"Mock 优先"原则。

---

### 1.2 各测试文件分析

#### `widget_test.dart` ❌ 已损坏
```
testWidgets('Counter increments smoke test', ...)
  → import 'package:rdcs_client/main.dart'
  → 使用 MyApp() — 不存在于当前代码库
```
**问题**: 这是 Flutter 脚手架自动生成的模板测试，从未更新。`main.dart` 中没有 `MyApp` 类（已改为 `RdcsApp`），运行时会编译失败。

**处置**: 应删除或替换为真实的 smoke test。

---

#### `home_page_test.dart` ✅ 覆盖完善
**测试数量**: 14 个

| 测试场景 | 状态 |
|---------|------|
| 格式化设备代码（XXX-XXX-XXX） | ✅ |
| 空设备代码显示占位符 | ✅ |
| 应用标题显示 | ✅ |
| "设备已就绪" 状态 | ✅ |
| 点击代码复制到剪贴板 + snackbar | ✅ |
| 空代码时不触发复制 | ✅ |
| "点击代码可复制" 提示的显示/隐藏 | ✅ |
| 导航到 /connect | ✅ |
| 复制图标的显示/隐藏 | ✅ |
| 生成邀请码按钮 | ✅ |
| 连接中/已连接/已断开/错误 4种会话状态 | ✅ |

---

#### `connect_page_test.dart` ✅ 覆盖完善
**测试数量**: 12 个

| 测试场景 | 状态 |
|---------|------|
| 页面标题和说明文字 | ✅ |
| 表单验证（空/少于9位/多于9位） | ✅ |
| 带空格的代码自动去除 | ✅ |
| 有效代码触发 engine.connect() | ✅ |
| 连接成功后导航到 /session | ✅ |
| 连接失败显示错误 snackbar | ✅ |
| 返回导航 | ✅ |
| 按钮/输入框状态 | ✅ |
| hint 文本格式 | ✅ |

---

#### `session_screen_test.dart` ✅ 覆盖完善
**测试数量**: 16 个

| 测试场景 | 状态 |
|---------|------|
| null session 显示加载动画 | ✅ |
| 连接中状态（spinner + 格式化设备代码） | ✅ |
| 连接中显示返回按钮 | ✅ |
| 已连接：设备名称/代码显示 | ✅ |
| 已连接：延迟（ms）指示器 | ✅ |
| 已连接：FPS 指示器 | ✅ |
| 3种画质模式（自动/清晰/流畅） | ✅ |
| 底部工具栏按钮（键盘/文件传输/消息/隐藏面板） | ✅ |
| 断开连接按钮（call_end 图标） | ✅ |
| 点击断开调用 engine.disconnect(sessionId) | ✅ |
| 全屏按钮 | ✅ |
| 视频占位区域 | ✅ |
| 断开/错误状态显示 + 自动导航回首页 | ✅ |
| 背景深色验证（#111111） | ✅ |

---

#### `settings_screen_test.dart` ✅ 覆盖完善
**测试数量**: 22 个

| 标签页 | 场景 | 状态 |
|--------|------|------|
| 安全 | 密码表单（空/太短/不一致/成功） | ✅ |
| 安全 | TOTP 两步验证区域 | ✅ |
| 安全 | "允许远程连接" 开关（默认开/切换） | ✅ |
| 网络 | 服务器配置（信令/中继/API URL） | ✅ |
| 网络 | TLS 加密开关 | ✅ |
| 网络 | 编解码器下拉（4个选项/默认H.264/切换VP9→configProvider更新） | ✅ |
| 网络 | 硬件加速、带宽设置 | ✅ |
| 通用 | 语言、启动行为 | ✅ |
| 通用 | 剪贴板/音频/关于区域 | ✅ |
| 通用 | 开关切换同步更新 configProvider | ✅ |

---

#### `ui_integration_test.dart` ❌ 包名错误
```dart
import 'package:rdcs_flutter/features/home/home_page.dart';
// 应该是: package:rdcs_client/...
```
**问题**: 整个文件引用了错误的包名 `rdcs_flutter`（旧名称），导致全部测试无法编译。

包含有价值的测试场景：
- 首页→连接页面完整流程
- 渲染性能测试（< 500ms）
- 连接确认对话框（倒计时、接受/拒绝、排队计数）
- 无障碍测试（语义标签、最小点击目标 44×44）

---

### 1.3 Flutter 测试覆盖缺口

| 模块 | 状态 | 说明 |
|------|------|------|
| `engine_isolate.dart` | ❌ 无测试 | **最关键缺口**：FFI 层完全没有单元测试，本次"engine not create"问题就在这里 |
| `bindings.dart` | ❌ 无测试 | 库加载逻辑未测试 |
| `config_repository.dart` | ❌ 无测试 | 文件读写逻辑 |
| `tray_service.dart` | ❌ 无测试 | 系统托盘 |
| `auto_start_service.dart` | ❌ 无测试 | 开机自启 |
| `video_renderer.dart` | ❌ 无测试 | 视频渲染 |
| FFI 库名称一致性 | ❌ 无测试 | CI 中没有验证 `librdcs_core.dylib` 存在 |

---

## 二、Web 管理控制台测试

### 2.1 测试框架
- **测试框架**: Vitest v2 + jsdom
- **UI 测试**: @testing-library/react v16
- **交互模拟**: @testing-library/user-event v14
- **运行命令**: `npm test` / `npm run test:coverage`

### 2.2 各测试文件分析

#### `authStore.test.ts` ✅ 覆盖完善
**测试数量**: 5 个

| 场景 | 状态 |
|------|------|
| 登录成功：存储 token，设置 isAuthenticated | ✅ |
| 登录成功：持久化到 localStorage | ✅ |
| 登录失败：保持未认证状态，抛出错误 | ✅ |
| 登出：清除所有状态和 localStorage | ✅ |
| 恢复会话：有/无 refreshToken 两种情况 | ✅ |

**注意**: 测试依赖 localStorage（jsdom 提供），在 Claude Desktop 的 artifact 环境中不可用，但在 Node 测试环境中没问题。

---

#### `ProtectedRoute.test.tsx` ✅ 核心功能覆盖
**测试数量**: 2 个

| 场景 | 状态 |
|------|------|
| 未认证 → 重定向到 /login | ✅ |
| 已认证 → 渲染子组件 | ✅ |

---

#### `client.test.ts` ⚠️ 部分覆盖
**测试数量**: 5 个

| 场景 | 状态 |
|------|------|
| 有 token 时附加 Authorization header | ⚠️ 间接验证（spy mock，未验证真实 header 注入） |
| 无 token 时不附加 header | ✅ |
| setAccessToken/getAccessToken 存储和清除 | ✅ |
| 401 响应 → 重定向 /login | ⚠️ 无法验证真实 interceptor 行为 |

**已知限制**: 无法在单元测试中轻易触发真实 axios interceptor。注释中已记录此限制，是合理的技术决策。

---

#### `LoginPage.test.tsx` ✅ 覆盖完善
**测试数量**: 4 个

| 场景 | 状态 |
|------|------|
| 表单字段渲染（邮箱/密码/登录按钮） | ✅ |
| 提交时调用 loginRequest（含凭据） | ✅ |
| 提交中显示加载状态（按钮禁用） | ✅ |
| 失败时显示错误消息，按钮重新启用 | ✅ |
| TOTP 双因素验证码字段切换 | ✅ |

---

#### `DevicesPage.test.tsx` ✅ 基本覆盖
**测试数量**: 4 个

| 场景 | 状态 |
|------|------|
| 渲染页面标题 | ✅ |
| 数据加载后显示设备列表 | ✅ |
| 搜索过滤功能 | ✅ |
| 挂载时调用 GET /api/devices | ✅ |

---

### 2.3 Web 测试覆盖缺口

| 页面/模块 | 状态 | 说明 |
|-----------|------|------|
| `SessionsPage` | ❌ 无测试 | 会话监控页面 |
| `ConnectionRecordsPage` | ❌ 无测试 | 连接记录页面 |
| `MembersPage` | ❌ 无测试 | 成员管理页面 |
| `SettingsPage` | ❌ 无测试 | 系统设置页面 |
| `DashboardPage` | ❌ 无测试 | 仪表盘 |
| `Layout` 组件 | ❌ 无测试 | 导航栏 |
| WebSocket 实时更新 | ❌ 无测试 | 信令连接状态 |
| API 错误处理 | ❌ 无测试 | 超时、网络错误 |

---

## 三、Rust FFI 层测试（与客户端直接相关）

### 3.1 `crates/rdcs-ffi/src/lib.rs` 测试分析

**测试数量**: 7 个

| 场景 | 状态 |
|------|------|
| engine lifecycle (create/destroy) | ✅ |
| null handle 返回正确错误码 | ✅ |
| register_and_dispatch_callback | ✅ |
| generate_invite 返回4位字符串 | ✅ |
| free null string 安全 | ✅ |
| platform_bundle_initialized（capture/input/notify/clipboard）| ✅ |
| start_and_stop_capture | ✅ |
| crypto_factory 产生不同 session ID | ✅ |

### 3.2 关键发现

**为什么 FFI 测试没有捕获库名不匹配问题？**

Rust 单元测试在**编译时**直接链接 `rdcs-ffi` crate，不经过动态库加载，所以：
- Rust 测试无法验证 `dlopen("librdcs_core.dylib")` 是否成功
- 只有在 Flutter 运行时才会触发动态加载失败

这说明需要一个**跨层集成测试**来验证库文件名一致性（建议作为 CI 脚本）。

---

## 四、运行时状态检查

### 4.1 服务状态
| 服务 | 状态 | 说明 |
|------|------|------|
| 信令服务器 (`:8443`) | ⚠️ 进程运行中，HTTP 无响应 | pgrep 找到进程，但 curl 失败 |
| Web 控制台 (`:5173`) | ❌ 未运行 | 需手动 `npm run dev` |
| Redis | 未检测 | 信令服务器依赖 |
| Flutter APP | ❌ 上次运行崩溃 | flutter_output.log 记录了 FFI 加载失败 |

### 4.2 FFI 库状态
| 文件 | 状态 | 大小 |
|------|------|------|
| `target/debug/librdcs_ffi.dylib` | ✅ 存在（旧名） | 5.3 MB |
| `target/debug/librdcs_core.dylib` | ❌ 不存在（需重编译） | — |

**Cargo.toml 修改状态**: ✅ 已修改（`name = "rdcs_core"` 已写入），**待重新编译**。

---

## 五、问题清单

### P0 — 阻塞运行

| ID | 问题 | 文件 | 影响 |
|----|------|------|------|
| P0-01 | `librdcs_core.dylib` 不存在（未重编译） | `crates/rdcs-ffi/Cargo.toml` | Flutter APP 无法启动 |
| P0-02 | `widget_test.dart` 引用不存在的 `MyApp` | `test/widget_test.dart` | flutter test 编译失败 |
| P0-03 | `ui_integration_test.dart` 包名为 `rdcs_flutter` | `test/ui_integration_test.dart` | 全部集成测试无法编译 |

### P1 — 影响测试完整性

| ID | 问题 | 说明 |
|----|------|------|
| P1-01 | `EngineIsolate` 无单元测试 | FFI 最关键路径无覆盖 |
| P1-02 | CI 无库文件名验证 | 本次 bug 的根本防御缺失 |
| P1-03 | Web 5个页面无测试 | Dashboard/Sessions/Records/Members/Settings |

### P2 — 改进建议

| ID | 建议 | 优先级 |
|----|------|--------|
| P2-01 | 添加 FFI 库加载集成测试 | 高 |
| P2-02 | 添加信令服务器 WebSocket 集成测试 | 中 |
| P2-03 | 覆盖 Web 剩余页面 | 中 |
| P2-04 | 添加视频帧渲染测试 | 低 |

---

## 六、立即修复操作

### 修复 P0-01：重新编译 FFI 库

```bash
# 在项目根目录
cargo build -p rdcs-ffi
# 验证
ls -lh target/debug/librdcs_core.dylib
```

### 修复 P0-02：删除过时模板测试

```bash
# 删除或替换 widget_test.dart
cat > client/flutter/test/widget_test.dart << 'EOF'
// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/app.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter/material.dart';

void main() {
  testWidgets('RdcsApp smoke test — renders without crash', (tester) async {
    await tester.pumpWidget(
      const ProviderScope(child: RdcsApp()),
    );
    // 仅验证启动不崩溃，不依赖 FFI
    expect(find.byType(MaterialApp), findsOneWidget);
  });
}
EOF
```

### 修复 P0-03：修正包名

```bash
# 批量替换 rdcs_flutter → rdcs_client
sed -i '' 's/package:rdcs_flutter\//package:rdcs_client\//g' \
  client/flutter/test/ui_integration_test.dart
```

### 运行 Web 测试（当前可执行）

```bash
cd web/admin
npm test
# 预期: ~20 个测试全部通过
```

### 运行 Flutter 测试（修复 P0-02/03 后）

```bash
cd client/flutter
flutter test test/home_page_test.dart
flutter test test/connect_page_test.dart
flutter test test/session_screen_test.dart
flutter test test/settings_screen_test.dart
# 预期: ~64 个测试通过，widget_test 和 ui_integration 修复后也通过
```

---

## 七、推荐补充测试

### 7.1 FFI 库名称验证（CI 脚本）

**文件**: `scripts/verify-ffi-name.sh`

```bash
#!/bin/bash
set -e
cargo build -p rdcs-ffi
if [ ! -f "target/debug/librdcs_core.dylib" ]; then
  echo "❌ librdcs_core.dylib not found — check [lib] name in crates/rdcs-ffi/Cargo.toml"
  exit 1
fi
echo "✅ librdcs_core.dylib exists"
```

### 7.2 Flutter FFI 加载测试（最高优先级新增测试）

**文件**: `client/flutter/test/ffi_bindings_test.dart`

```dart
// 测试核心：验证 FakeEngineIsolate 接口完整性
// 无法在 Flutter 测试中测试真实 dlopen，
// 但可以验证 EngineIsolate 接口契约
void main() {
  group('EngineIsolate interface contract', () {
    test('FakeEngineIsolate implements full interface', () {
      // 如果 FakeEngineIsolate 编译通过，接口契约就被验证了
      final fake = FakeEngineIsolate();
      expect(fake, isA<EngineIsolate>());
    });
    
    test('connect() records target code', () async {
      final fake = FakeEngineIsolate(connectResult: 1);
      await fake.connect('123456789');
      expect(fake.lastConnectCode, '123456789');
    });
  });
}
```

### 7.3 信令服务器 WebSocket 测试（Phase 2 剩余 5%）

```bash
# 需要服务器运行
wscat -c ws://localhost:8443/ws -x '{"type":"register","device_code":"123456789"}'
# 预期: {"type":"registered","device_code":"123456789",...}
```

---

## 八、总结与结论

### 当前测试质量评估

| 维度 | 评级 | 说明 |
|------|------|------|
| Flutter UI 测试广度 | ✅ 良好 | 4 个主页面覆盖充分，Mock 策略正确 |
| Flutter UI 测试深度 | ✅ 良好 | 覆盖所有状态、验证和交互流程 |
| Flutter FFI 测试 | ❌ 缺失 | EngineIsolate 完全没有测试 |
| Web 测试广度 | ⚠️ 部分 | 5 个页面有测试，5 个页面无测试 |
| Web 测试深度 | ✅ 良好 | 有测试的页面覆盖主流程 |
| 端到端测试 | ❌ 缺失 | 无跨客户端-服务器的集成测试 |
| CI 集成验证 | ❌ 缺失 | 无库文件名一致性检查 |

### 最重要结论

本次"engine not create"问题的根本原因（FFI 库名不匹配）**没有任何测试能够提前发现**，因为：
1. Rust 单元测试在编译时直接链接，不走 `dlopen`
2. Flutter widget 测试使用 `FakeEngineIsolate`，完全绕过真实 FFI
3. CI 中没有验证 `librdcs_core.dylib` 是否存在

**建议优先级**: 在下次 Flutter APP 客户端测试迭代中，首要任务是添加 FFI 库名称验证到 CI，这能在 1 行脚本内消除此类问题。

---

**测试人**: Claude (Superpowers Agent)  
**参考文档**: `docs/testing/TESTING_GUIDELINES.md`  
**生成时间**: 2026-06-29
