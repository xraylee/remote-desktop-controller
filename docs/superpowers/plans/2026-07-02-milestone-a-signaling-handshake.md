# 里程碑 A —— 信令握手闭合 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A 发起连接后,B 端无论在哪个页面都弹出接受/拒绝框;B 决策经 `connect_response` 回到 A,发起方等 response 而非乐观直连。信令握手端到端闭合(不含媒体)。

**Architecture:** 服务端把已生成的 `session_id`(UUID)经 `ConnectRequest` 带给 B(字段为 `Option<String>`,双向复用同一 variant);Dart `ConnectRequestMessage` 同步加 nullable `sessionId`。`SignalingService` 打通 `connectResponses` 广播流。`app.dart` 加全局 `navigatorKey` + 常驻宿主监听 `invitationsProvider`,弹 `ConnectionConfirmDialog` 并回 `respondToConnection`(单活跃+其余即拒)。`SessionNotifier.connect` 改为等匹配的 `connect_response`(超时 35s)再决定 connected/error。

**Tech Stack:** Rust(rdcs-signaling,serde/tokio),Dart/Flutter(Riverpod,freezed,GoRouter),cargo test + flutter test。

**上位 spec:** `docs/superpowers/specs/2026-07-02-milestone-a-signaling-handshake-design.md`

---

## 文件结构

| 文件 | 职责 | 动作 |
|---|---|---|
| `crates/rdcs-signaling/src/ws/message.rs` | wire 定义 `ConnectRequest.session_id` | 改 + 测试 |
| `crates/rdcs-signaling/src/handlers/connect.rs` | 转发时填入 session_id | 改 + 测试 |
| `crates/rdcs-signaling/src/ws/handler.rs` | inbound 解构忽略入站 session_id | 改(编译对齐) |
| `client/flutter/lib/core/signaling/models/signaling_message.dart` | `ConnectRequestMessage.sessionId` | 改 + codegen |
| `client/flutter/lib/core/signaling/signaling_service.dart` | connectRequest 带 sessionId;新增 connectResponses 流 | 改 |
| `client/flutter/lib/core/signaling/session_signaling.dart` | 接口暴露 connectResponses | 改 |
| `client/flutter/lib/core/signaling/signaling_provider.dart` | `connectResponsesProvider` | 改 |
| `client/flutter/lib/features/session/invitation_host.dart` | 常驻宿主监听 + 弹框 + 回 response | 新建 |
| `client/flutter/lib/app.dart` | 全局 navigatorKey + 挂宿主 | 改 |
| `client/flutter/lib/features/session/session_providers.dart` | connect 等 response gating | 改 |
| `client/flutter/lib/features/connect/connect_page.dart` | 占位态导航调整(决策 a) | 改 |
| `client/flutter/test/helpers.dart` | FakeSessionSignaling 加 connectResponses | 改 |

**任务顺序**:先 Rust(1–2),再 Dart 模型/服务(3–5),再 UI 宿主(6–7),再发起方 gating(8),最后集成(9)。每个改动单元自成可提交单元。

---

## Task 1: 服务端 `ConnectRequest` 加 `session_id` 字段(wire)

**Files:**
- Modify: `crates/rdcs-signaling/src/ws/message.rs:40-45`(ConnectRequest variant)
- Modify: `crates/rdcs-signaling/src/ws/message.rs`(`connect_request_round_trip` 测试)

- [ ] **Step 1: 更新 round-trip 测试为携带 session_id(先失败)**

把 `message.rs` 里的 `connect_request_round_trip` 测试替换为:

```rust
    #[test]
    fn connect_request_round_trip() {
        // server→B 方向:带 session_id
        let msg = WsMessage::ConnectRequest {
            from_code: "CTRL".into(),
            to_code: "TARGET".into(),
            session_id: Some("sess-abc".into()),
            invite_code: Some("INV001".into()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""type":"connect_request""#));
        assert!(json.contains(r#""session_id":"sess-abc""#));
        let parsed: WsMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, msg);

        // controller→server 方向:无 session_id,应被省略
        let msg_none = WsMessage::ConnectRequest {
            from_code: "CTRL".into(),
            to_code: "TARGET".into(),
            session_id: None,
            invite_code: None,
        };
        let json_none = serde_json::to_string(&msg_none).unwrap();
        assert!(!json_none.contains("session_id"));
        let parsed_none: WsMessage = serde_json::from_str(&json_none).unwrap();
        assert_eq!(parsed_none, msg_none);
    }
```

- [ ] **Step 2: 运行测试确认失败(字段不存在,编译错)**

Run: `cargo test -p rdcs-signaling connect_request_round_trip`
Expected: 编译失败,`struct variant WsMessage::ConnectRequest has no field named session_id`

- [ ] **Step 3: 给 ConnectRequest 加字段**

`message.rs` 中 ConnectRequest variant 改为:

```rust
    /// Controller requests a connection to a target device.
    ///
    /// `session_id` is present only on the server→target forward (the server
    /// mints it); the controller→server request omits it.
    #[serde(rename = "connect_request")]
    ConnectRequest {
        from_code: String,
        to_code: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        session_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        invite_code: Option<String>,
    },
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p rdcs-signaling connect_request_round_trip`
Expected: PASS(此单测通过;其它文件因缺字段仍编译错,Task 2 修)

- [ ] **Step 5: 提交**

```bash
git add crates/rdcs-signaling/src/ws/message.rs
git commit -m "feat(signaling): add optional session_id to ConnectRequest wire message"
```

---

## Task 2: 服务端转发填入 session_id + inbound 解构对齐

**Files:**
- Modify: `crates/rdcs-signaling/src/handlers/connect.rs:97-106`(转发构造)
- Modify: `crates/rdcs-signaling/src/handlers/connect.rs`(`connect_request_forwards_to_target` 等测试)
- Modify: `crates/rdcs-signaling/src/ws/handler.rs:225-229`(inbound 解构)

- [ ] **Step 1: 更新转发测试断言 session_id 非空(先失败)**

`connect.rs` 的 `connect_request_forwards_to_target` 测试,把 match 分支改为:

```rust
        match msg {
            WsMessage::ConnectRequest {
                from_code,
                to_code,
                session_id,
                invite_code,
            } => {
                assert_eq!(from_code, "CTRL");
                assert_eq!(to_code, "TARGET");
                assert!(
                    session_id.as_deref().is_some_and(|s| !s.is_empty()),
                    "forwarded ConnectRequest must carry a non-empty session_id"
                );
                assert!(invite_code.is_none());
            }
            other => panic!("expected ConnectRequest, got: {other:?}"),
        }
```

并把同文件 `connect_request_with_invite_code` 的 match 臂补上 `session_id,`(destructure 需完整):

```rust
            WsMessage::ConnectRequest { invite_code, session_id, .. } => {
                assert_eq!(invite_code.as_deref(), Some("INV-99"));
                assert!(session_id.is_some());
            }
```

- [ ] **Step 2: 运行确认失败**

Run: `cargo test -p rdcs-signaling -- connect_request`
Expected: 编译失败(转发处构造缺 session_id 字段)+ 断言失败

- [ ] **Step 3: 转发时填入 session_id**

`connect.rs` `handle_connect_request` 里转发 ConnectRequest 的构造(约 97-106 行)改为:

```rust
    // 3. Forward the ConnectRequest to the target, carrying the session_id.
    let _ = session_mgr
        .send_to(
            to_code,
            WsMessage::ConnectRequest {
                from_code: from_code.to_string(),
                to_code: to_code.to_string(),
                session_id: Some(session_id.clone()),
                invite_code: invite_code.map(String::from),
            },
        )
        .await;
```

- [ ] **Step 4: 对齐 inbound 解构(controller→server 忽略入站 session_id)**

`handler.rs:225-229` 的解构改为(加 `session_id: _`):

```rust
        WsMessage::ConnectRequest {
            from_code,
            to_code,
            session_id: _,
            invite_code,
        } => {
```

- [ ] **Step 5: 运行整个 crate 测试确认通过**

Run: `cargo test -p rdcs-signaling`
Expected: PASS(含 `connect_request_forwards_to_target`、`full_connection_negotiation_flow` 等)

- [ ] **Step 6: 提交**

```bash
git add crates/rdcs-signaling/src/handlers/connect.rs crates/rdcs-signaling/src/ws/handler.rs
git commit -m "feat(signaling): forward server-minted session_id in ConnectRequest to target"
```

---

## Task 3: Dart `ConnectRequestMessage` 加 nullable `sessionId` + codegen

**Files:**
- Modify: `client/flutter/lib/core/signaling/models/signaling_message.dart:57-62`
- Regenerate: `signaling_message.freezed.dart`、`signaling_message.g.dart`

- [ ] **Step 1: 加字段**

`signaling_message.dart` 的 connectRequest factory 改为:

```dart
  /// Request a connection to another device.
  ///
  /// [sessionId] is populated only on the server→target forward (the server
  /// mints it); the controller→server request leaves it null.
  const factory SignalingMessage.connectRequest({
    @JsonKey(name: 'from_code') required String fromCode,
    @JsonKey(name: 'to_code') required String toCode,
    @JsonKey(name: 'session_id') String? sessionId,
    @JsonKey(name: 'invite_code') String? inviteCode,
  }) = ConnectRequestMessage;
```

- [ ] **Step 2: 重新生成 freezed/json 代码**

Run: `cd client/flutter && dart run build_runner build --delete-conflicting-outputs`
Expected: 成功,`signaling_message.freezed.dart` / `.g.dart` 更新;`connectRequest` 的 `when` 回调签名变为 4 参 `(fromCode, toCode, sessionId, inviteCode)`

- [ ] **Step 3: 加序列化往返测试(先可能失败于编译)**

在 `test/core/signaling/signaling_message_test.dart` 增加:

```dart
  test('connect_request with session_id round-trips', () {
    const msg = SignalingMessage.connectRequest(
      fromCode: '761335217',
      toCode: '123456789',
      sessionId: 'sess-xyz',
      inviteCode: null,
    );
    final json = msg.toJson();
    expect(json['session_id'], 'sess-xyz');
    final parsed = SignalingMessage.fromJson(json);
    expect(parsed, msg);
  });

  test('connect_request without session_id omits the key when serialized', () {
    const msg = SignalingMessage.connectRequest(
      fromCode: '761335217',
      toCode: '123456789',
    );
    final json = msg.toJson();
    // freezed/json_serializable emits null; server side tolerates absent.
    expect(json['session_id'], isNull);
  });
```

- [ ] **Step 4: 运行确认通过**

Run: `cd client/flutter && flutter test test/core/signaling/signaling_message_test.dart`
Expected: PASS

- [ ] **Step 5: 提交**

```bash
git add client/flutter/lib/core/signaling/models/signaling_message.dart \
        client/flutter/lib/core/signaling/models/signaling_message.freezed.dart \
        client/flutter/lib/core/signaling/models/signaling_message.g.dart \
        client/flutter/test/core/signaling/signaling_message_test.dart
git commit -m "feat(client): add nullable sessionId to ConnectRequestMessage"
```

---

## Task 4: `SignalingService` 打通 connectResponses 流 + connectRequest 带 sessionId

**Files:**
- Modify: `client/flutter/lib/core/signaling/signaling_service.dart`(controller/流/dispose/handleMessage)

- [ ] **Step 1: 新增 connectResponses controller + getter**

在 `signaling_service.dart` 的 `_errorsController` 附近(约 64 行后)加:

```dart
  /// Connection responses (connect_response from the target device).
  final _connectResponsesController =
      StreamController<ConnectResponseMessage>.broadcast();
  Stream<ConnectResponseMessage> get connectResponses =>
      _connectResponsesController.stream;
```

- [ ] **Step 2: dispose 关闭它**

`dispose()`(约 229-238 行)在 `_errorsController.close();` 后加:

```dart
    _connectResponsesController.close();
```

- [ ] **Step 3: connectRequest 分支带 sessionId;connectResponse 分支入流**

`_handleMessage`(约 300-311 行)两处改为:

```dart
      connectRequest: (fromCode, toCode, sessionId, inviteCode) {
        // This is an incoming connection request.
        _invitationsController.add(ConnectRequestMessage(
          fromCode: fromCode,
          toCode: toCode,
          sessionId: sessionId,
          inviteCode: inviteCode,
        ));
        print('📞 Connection request from $fromCode (session $sessionId)');
      },
      connectResponse: (accepted, sessionId, fromCode) {
        _connectResponsesController.add(ConnectResponseMessage(
          accepted: accepted,
          sessionId: sessionId,
          fromCode: fromCode,
        ));
        print(
            '${accepted ? "✅" : "❌"} connect_response from $fromCode (session $sessionId)');
      },
```

- [ ] **Step 4: 加 service 层测试**

在 `test/core/signaling/signaling_service_test.dart` 增加一条:注入一个 `connect_response` 入站消息,断言 `service.connectResponses` 收到对应对象。参照该文件既有 `invitations` 测试的 harness(约 250 行处 `final invitations = <ConnectRequestMessage>[];`),仿写:

```dart
  test('connect_response is routed to connectResponses stream', () async {
    final responses = <ConnectResponseMessage>[];
    final sub = service.connectResponses.listen(responses.add);

    // Inject an inbound connect_response through the fake WS message stream
    // (use the same injection mechanism the invitations test uses).
    injectInbound(const SignalingMessage.connectResponse(
      accepted: true,
      sessionId: 'sess-1',
      fromCode: 'TARGET',
    ));
    await Future<void>.delayed(Duration.zero);

    expect(responses, hasLength(1));
    expect(responses.first.accepted, isTrue);
    expect(responses.first.sessionId, 'sess-1');
    await sub.cancel();
  });
```

> 注:`injectInbound` 是占位——用该测试文件既有的入站注入方式(fake WebSocketClient 的 messages sink);读 250 行附近 invitations 测试照抄其注入调用。

- [ ] **Step 5: 运行确认通过**

Run: `cd client/flutter && flutter test test/core/signaling/signaling_service_test.dart`
Expected: PASS

- [ ] **Step 6: 提交**

```bash
git add client/flutter/lib/core/signaling/signaling_service.dart \
        client/flutter/test/core/signaling/signaling_service_test.dart
git commit -m "feat(client): expose connectResponses stream and carry sessionId on invitations"
```

---

## Task 5: 接口 + Provider 暴露 connectResponses

**Files:**
- Modify: `client/flutter/lib/core/signaling/session_signaling.dart`
- Modify: `client/flutter/lib/core/signaling/signaling_provider.dart`
- Modify: `client/flutter/test/helpers.dart`(FakeSessionSignaling)

- [ ] **Step 1: 接口加 connectResponses**

`session_signaling.dart` 改为:

```dart
// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'models/signaling_message.dart';
import 'websocket_client.dart';

/// Minimal signaling surface needed by session initiation.
abstract interface class SessionSignaling {
  String get deviceCode;
  WsConnectionState get currentConnectionState;

  /// Responses to our outgoing connect_request (accept/reject from target).
  Stream<ConnectResponseMessage> get connectResponses;

  Future<void> connect();

  void requestConnection(String targetCode, {String? inviteCode});
}
```

- [ ] **Step 2: Provider 暴露 connectResponsesProvider**

`signaling_provider.dart` 在 `invitationsProvider` 后加:

```dart
/// Provider for responses to our outgoing connection requests.
final connectResponsesProvider =
    StreamProvider<ConnectResponseMessage>((ref) {
  final service = ref.watch(signalingServiceProvider);
  return service.connectResponses;
});
```

- [ ] **Step 3: FakeSessionSignaling 实现 connectResponses**

`test/helpers.dart` 的 `FakeSessionSignaling`(150 行)加一个可控 controller:

```dart
  final _connectResponsesController =
      StreamController<ConnectResponseMessage>.broadcast();

  @override
  Stream<ConnectResponseMessage> get connectResponses =>
      _connectResponsesController.stream;

  /// Test hook: push a fake connect_response to the notifier under test.
  void emitConnectResponse(ConnectResponseMessage msg) =>
      _connectResponsesController.add(msg);

  void disposeFake() => _connectResponsesController.close();
```

(确保 `test/helpers.dart` 顶部 `import 'dart:async';` 与 signaling_message import 存在;`SignalingService` 已 implements SessionSignaling,新增 getter 已在 Task 4 提供,故其无需再改。)

- [ ] **Step 4: 编译确认(现有测试不回归)**

Run: `cd client/flutter && flutter test test/integration/session_flow_test.dart`
Expected: PASS(FakeSessionSignaling 现满足接口)

- [ ] **Step 5: 提交**

```bash
git add client/flutter/lib/core/signaling/session_signaling.dart \
        client/flutter/lib/core/signaling/signaling_provider.dart \
        client/flutter/test/helpers.dart
git commit -m "feat(client): expose connectResponses on SessionSignaling + provider + fake"
```

---

## Task 6: 常驻宿主监听 widget(invitation_host.dart)

**Files:**
- Create: `client/flutter/lib/features/session/invitation_host.dart`

- [ ] **Step 1: 写宿主 widget**

新建 `invitation_host.dart`:

```dart
// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/signaling/signaling_provider.dart';
import '../../core/signaling/models/signaling_message.dart';
import 'connection_confirm_dialog.dart';

/// App-wide host that listens for incoming connection invitations and shows
/// the accept/reject dialog regardless of the current route.
///
/// Policy: single active dialog. If a new request arrives while one is
/// showing, it is auto-rejected immediately (accepted: false).
class InvitationHost extends ConsumerStatefulWidget {
  const InvitationHost({super.key, required this.navigatorKey, required this.child});

  final GlobalKey<NavigatorState> navigatorKey;
  final Widget child;

  @override
  ConsumerState<InvitationHost> createState() => _InvitationHostState();
}

class _InvitationHostState extends ConsumerState<InvitationHost> {
  bool _dialogActive = false;

  Future<void> _handleInvite(ConnectRequestMessage req) async {
    final service = ref.read(signalingServiceProvider);
    final sessionId = req.sessionId ?? '';

    // Single-active policy: reject extras immediately.
    if (_dialogActive || sessionId.isEmpty) {
      service.respondToConnection(
        sessionId: sessionId,
        fromCode: req.fromCode,
        accepted: false,
      );
      return;
    }

    final context = widget.navigatorKey.currentContext;
    if (context == null) return;

    _dialogActive = true;
    try {
      final result = await ConnectionConfirmDialog.show(
        context,
        requesterName: req.fromCode,
        requesterCode: req.fromCode,
      );
      service.respondToConnection(
        sessionId: sessionId,
        fromCode: req.fromCode,
        accepted: result == ConnectionConfirmResult.accepted,
      );
    } finally {
      _dialogActive = false;
    }
  }

  @override
  Widget build(BuildContext context) {
    ref.listen<AsyncValue<ConnectRequestMessage>>(invitationsProvider,
        (prev, next) {
      next.whenData(_handleInvite);
    });
    return widget.child;
  }
}
```

- [ ] **Step 2: 静态分析确认无误**

Run: `cd client/flutter && flutter analyze lib/features/session/invitation_host.dart`
Expected: No issues(或仅既有风格提示)

- [ ] **Step 3: 提交**

```bash
git add client/flutter/lib/features/session/invitation_host.dart
git commit -m "feat(client): add InvitationHost widget (single-active accept dialog)"
```

---

## Task 7: 全局 navigatorKey + 在 app.dart 挂宿主

**Files:**
- Modify: `client/flutter/lib/app.dart`(goRouterProvider、build)

- [ ] **Step 1: 全局 navigatorKey 并注入 GoRouter**

`app.dart` 顶部(`darkModeProvider` 附近)加:

```dart
/// Root navigator key — lets app-wide hosts (e.g. invitation dialogs) show
/// overlays regardless of the current route.
final rootNavigatorKey = GlobalKey<NavigatorState>();
```

`goRouterProvider` 的 `GoRouter(` 构造首行加:

```dart
  return GoRouter(
    navigatorKey: rootNavigatorKey,
    initialLocation: '/',
```

- [ ] **Step 2: 在 MaterialApp.router 外包 InvitationHost**

`app.dart` 顶部加 import:

```dart
import 'features/session/invitation_host.dart';
```

`build`(约 168-179 行)的 `MaterialApp.router(...)` 用 `builder` 包宿主:

```dart
    return MaterialApp.router(
      title: 'RDCS 远程桌面',
      debugShowCheckedModeBanner: false,
      theme: isDark ? _buildDarkTheme() : RdcsTheme.light,
      darkTheme: _buildDarkTheme(),
      themeMode: isDark ? ThemeMode.dark : ThemeMode.light,
      routerConfig: router,
      builder: (context, child) => InvitationHost(
        navigatorKey: rootNavigatorKey,
        child: child ?? const SizedBox.shrink(),
      ),
    );
```

- [ ] **Step 3: 静态分析**

Run: `cd client/flutter && flutter analyze lib/app.dart`
Expected: No issues

- [ ] **Step 4: widget 测试 — 非首页也弹框(AC1/AC2)**

新建 `test/features/session/invitation_host_test.dart`:

```dart
// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';
import 'package:rdcs_client/core/signaling/signaling_provider.dart';
import 'package:rdcs_client/features/session/invitation_host.dart';

import '../../helpers.dart';

void main() {
  testWidgets('shows dialog when an invitation arrives', (tester) async {
    final navKey = GlobalKey<NavigatorState>();
    final controller = StreamController<ConnectRequestMessage>.broadcast();
    addTearDown(controller.close);

    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          // Override the service so respondToConnection is a no-op fake,
          // and invitationsProvider draws from our controller.
          invitationsProvider.overrideWith((ref) => controller.stream),
          signalingServiceProvider.overrideWithValue(
            FakeSignalingServiceForHost(),
          ),
        ],
        child: MaterialApp(
          navigatorKey: navKey,
          home: InvitationHost(
            navigatorKey: navKey,
            child: const Scaffold(body: Text('any-non-home-page')),
          ),
        ),
      ),
    );

    controller.add(const ConnectRequestMessage(
      fromCode: '761335217',
      toCode: '123456789',
      sessionId: 'sess-1',
    ));
    await tester.pump(); // deliver stream event
    await tester.pump(const Duration(milliseconds: 50)); // show dialog

    expect(find.text('远程连接请求'), findsOneWidget);
    expect(find.text('761335217'), findsWidgets);
  });
}
```

在 `test/helpers.dart` 追加一个记录 respond 调用的 fake(供本测试与 Task 后续用):

```dart
class FakeSignalingServiceForHost implements SignalingService {
  String? lastRespondSessionId;
  bool? lastRespondAccepted;

  @override
  void respondToConnection({
    required String sessionId,
    required String fromCode,
    required bool accepted,
  }) {
    lastRespondSessionId = sessionId;
    lastRespondAccepted = accepted;
  }

  @override
  noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}
```

> 注:`FakeSignalingServiceForHost` 用 `noSuchMethod` 兜底未用到的成员;`SignalingService` 是具体类,fake 需 `implements` 它——若 analyzer 抱怨缺成员,`noSuchMethod` 已兜底(需 `@override noSuchMethod`)。若项目禁用 noSuchMethod,改为在测试里只 override `signalingServiceProvider` 为真实 service 并断言 dialog 出现(不断言 respond)。

- [ ] **Step 5: 运行确认通过**

Run: `cd client/flutter && flutter test test/features/session/invitation_host_test.dart`
Expected: PASS(dialog 出现)

- [ ] **Step 6: 提交**

```bash
git add client/flutter/lib/app.dart \
        client/flutter/test/features/session/invitation_host_test.dart \
        client/flutter/test/helpers.dart
git commit -m "feat(client): mount InvitationHost app-wide with root navigator key"
```

---

## Task 8: 发起方 gating —— 等 connect_response(超时 35s)

**Files:**
- Modify: `client/flutter/lib/features/session/session_providers.dart:75-125`(connect)
- Modify: `client/flutter/lib/features/connect/connect_page.dart:60-66`(占位态导航)

- [ ] **Step 1: gating 测试(先失败)**

新建 `test/features/session/session_gating_test.dart`:

```dart
// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/core/signaling/models/signaling_message.dart';
import 'package:rdcs_client/features/session/session_providers.dart';

import '../../helpers.dart';

void main() {
  test('connect stays connecting until accepted response arrives (AC8)',
      () async {
    final engine = FakeEngineIsolate();
    final signaling = FakeSessionSignaling();
    final notifier = SessionNotifier(engine, signaling);

    final future = notifier.connect('123456789');
    // Before any response: connecting, engine.connect NOT called.
    expect(notifier.state?.state, SessionState.connecting);
    expect(engine.connectCalled, isFalse);

    signaling.emitConnectResponse(const ConnectResponseMessage(
      accepted: true, sessionId: 'sess-1', fromCode: '123456789'));
    await future;

    expect(notifier.state?.state, SessionState.connected);
  });

  test('rejected response -> error state (AC4)', () async {
    final engine = FakeEngineIsolate();
    final signaling = FakeSessionSignaling();
    final notifier = SessionNotifier(engine, signaling);

    final future = notifier.connect('123456789');
    signaling.emitConnectResponse(const ConnectResponseMessage(
      accepted: false, sessionId: 'sess-1', fromCode: '123456789'));
    await future;

    expect(notifier.state?.state, SessionState.error);
    expect(engine.connectCalled, isFalse);
  });
}
```

> 依赖:`FakeEngineIsolate` 需暴露 `connectCalled`(bool,默认 false,`connect` 调用时置 true)。若其尚无该字段,在 `test/helpers.dart` 的 `FakeEngineIsolate`(53 行)加 `bool connectCalled = false;` 并在其 `connect` override 里置 true。

- [ ] **Step 2: 运行确认失败**

Run: `cd client/flutter && flutter test test/features/session/session_gating_test.dart`
Expected: FAIL(当前 connect 乐观直连,不等 response;或 connectCalled 为 true)

- [ ] **Step 3: 改写 SessionNotifier.connect 为等 response**

`session_providers.dart` 的 `connect`(75-125 行)整体替换为:

```dart
  Future<void> connect(String targetCode, {String? inviteCode}) async {
    final code = targetCode.replaceAll(RegExp(r'[\s-]'), '');

    state = SessionInfo(
      sessionId: 0,
      remoteDeviceCode: code,
      remoteDeviceName: code,
      state: SessionState.connecting,
    );

    _subscribeToEvents();

    try {
      // Send connect_request with bounded retries.
      var requestError;
      for (var attempt = 1;
          attempt <= _maxSignalingRequestAttempts;
          attempt++) {
        try {
          if (_signaling.currentConnectionState !=
              WsConnectionState.connected) {
            await _signaling.connect();
          }
          _signaling.requestConnection(code, inviteCode: inviteCode);
          requestError = null;
          break;
        } catch (e) {
          requestError = e;
          if (attempt < _maxSignalingRequestAttempts) {
            await Future.delayed(Duration(milliseconds: 150 * attempt));
          }
        }
      }
      if (requestError != null) throw requestError;

      // Wait for the target's connect_response (accept/reject), matched by
      // fromCode == the code we dialed. 35s > the 30s dialog countdown on
      // the target side, avoiding a race where we time out while they decide.
      final response = await _signaling.connectResponses
          .firstWhere((r) => r.fromCode == code)
          .timeout(const Duration(seconds: 35));

      if (!response.accepted) {
        state = state?.copyWith(state: SessionState.error);
        return;
      }

      // Accepted. Milestone A stops here — media is Milestone B. Remain in
      // `connected` (signaling-established) without driving the mock engine.
      state = state?.copyWith(state: SessionState.connected);
    } catch (e) {
      state = state?.copyWith(state: SessionState.error);
    }
  }
```

> 说明:决策 (a) —— 收到 accepted 后停在 `connected`(信令握手态),**不**调 `_engine.connect`(那是 mock,B 里程碑接真实传输后再调)。`FakeEngineIsolate.connectCalled` 因此保持 false,AC8 成立。

- [ ] **Step 4: 运行确认通过**

Run: `cd client/flutter && flutter test test/features/session/session_gating_test.dart`
Expected: PASS

- [ ] **Step 5: 调整 connect_page 导航为决策 (a)(不进 /session,留在本页提示)**

决策 (a):A 里程碑不进 `/session`。`connect_page.dart` 的 `_onConnect`(约 47-66 行)结果分支整体替换为——connected 显示握手成功提示、error 显示失败提示、**都不导航**:

```dart
      final session = ref.read(sessionProvider);
      if (!mounted) return;

      if (session != null && session.state == SessionState.connected) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('对方已接受,信令握手成功(画面通道将在后续里程碑接入)'),
            backgroundColor: Color(0xFF10B981),
          ),
        );
      } else if (session != null && session.state == SessionState.error) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('连接失败:对方已拒绝、超时或设备离线'),
            backgroundColor: Color(0xFFEF4444),
          ),
        );
      }
      // connect() 只在终态(connected/error)返回,故无需旧的
      // "still connecting → context.go('/session')" 乐观分支;一并删除。
```

- [ ] **Step 6: 静态分析 + 运行相关测试**

Run: `cd client/flutter && flutter analyze lib/features/session/session_providers.dart lib/features/connect/connect_page.dart && flutter test test/features/session/session_gating_test.dart test/connect_page_test.dart`
Expected: analyze 无错;gating 测试 PASS。`connect_page_test.dart` 若断言旧的 `/session` 导航,按新行为更新其期望为 SnackBar 文案(connected→绿色成功、error→红色失败)。

- [ ] **Step 7: 提交**

```bash
git add client/flutter/lib/features/session/session_providers.dart \
        client/flutter/lib/features/connect/connect_page.dart \
        client/flutter/test/features/session/session_gating_test.dart \
        client/flutter/test/helpers.dart \
        client/flutter/test/connect_page_test.dart
git commit -m "feat(client): gate connect on connect_response with 35s timeout (milestone A)"
```

---

## Task 9: 端到端集成 —— 双客户端握手闭合(AC3/AC10)

**Files:**
- Modify: `client/flutter/test/integration/live_two_client_invite_test.dart`

- [ ] **Step 1: 读现有 live_two_client 测试结构**

Run: `cd client/flutter && sed -n '1,120p' test/integration/live_two_client_invite_test.dart`
Expected: 了解它如何起双客户端、拿 `invitationFuture`(85 行)。判断它是否需真实 server(可能标 `@Tags(['live'])` 或需本地 8443)。

- [ ] **Step 2: 扩展为完整握手断言**

在该测试里,B 收到 invite 后回 accept,断言 A 收到 `connect_response` 且 session_id 一致。追加:

```dart
    // B accepts using the server-minted session_id carried on the invite.
    expect(invite.sessionId, isNotNull);
    clientB.respondToConnection(
      sessionId: invite.sessionId!,
      fromCode: clientB.deviceCode,
      accepted: true,
    );

    // A must receive a matching connect_response with the SAME session_id.
    final response = await clientA.connectResponses
        .firstWhere((r) => r.fromCode == clientB.deviceCode)
        .timeout(const Duration(seconds: 10));
    expect(response.accepted, isTrue);
    expect(response.sessionId, invite.sessionId);
```

> 若该测试是 live-tagged(需真实 server),按其既有运行方式跑;否则用 in-memory/loopback harness。具体注入方式沿用该文件既有 setup(读 Step 1 输出后照抄其 client 构造)。

- [ ] **Step 3: 运行(按其 tag/前置)**

Run: `cd client/flutter && flutter test test/integration/live_two_client_invite_test.dart`
（若需真实 server:先 `cargo run -p rdcs-signaling` 起 8443,见 memory `rdcs-build-and-log-workflow`。)
Expected: PASS —— A 收到 accepted 且 session_id == invite.sessionId

- [ ] **Step 4: 提交**

```bash
git add client/flutter/test/integration/live_two_client_invite_test.dart
git commit -m "test(client): assert two-client handshake closes with consistent session_id"
```

---

## Task 10: 全量回归 + memory 更新

- [ ] **Step 1: Rust 全测**

Run: `cargo test -p rdcs-signaling`
Expected: PASS

- [ ] **Step 2: Dart 全测 + 分析**

Run: `cd client/flutter && flutter analyze && flutter test`
Expected: analyze 无错;测试全绿(若个别既有测试因行为变更需更新期望,更新之)

- [ ] **Step 3: 更新 memory `rdcs-invitation-ui-missing`**

把该 memory 标注为"里程碑 A 已实现:弹窗胶水 + connect_response gating + session_id 缝(5-1)已补;里程碑 B(WebRTC 媒体/FFI)仍未接",避免后续会话再当未修 bug 排查。

- [ ] **Step 4: 提交**

```bash
git add -A && git commit -m "chore: milestone A regression green; update project memory"
```

---

## 自查(spec 覆盖)

- AC1/AC2(常驻宿主、非首页弹框)→ Task 6/7 + invitation_host_test
- AC3/AC10(握手闭合、session_id 一致)→ Task 1/2/9
- AC4(拒绝→error)→ Task 8 gating_test
- AC5(dialog 30s 超时→reject)→ dialog 既有能力 + Task 6 回 accepted:false
- AC6(离线 error)→ 现状,Task 8 error 分支提示不回归
- AC7(单活跃即拒)→ Task 6 `_dialogActive`/空 sessionId 即拒
- AC8(收 accepted 前不进 session/不调 engine)→ Task 8 gating_test(connectCalled false)
- AC9(35s 超时→error)→ Task 8 `.timeout(35s)`
- 影响面 §8 全部文件在 Task 1–8 覆盖
