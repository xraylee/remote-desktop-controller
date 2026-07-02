# 里程碑 A —— 信令握手闭合 设计规格(Spec)

- 状态:待评审
- 日期:2026-07-02
- 上位文档:[`2026-07-02-p2p-connection-architecture.md`](./2026-07-02-p2p-connection-architecture.md)
- 范围:本 spec 仅覆盖**里程碑 A**(信令握手闭合)。里程碑 B(WebRTC 媒体)另出 spec。

---

## 1. 目标

A 发起连接 → B 端**无论在哪个页面**都弹出接受/拒绝对话框 → B 决策 → A 收到 `connect_response`,信令握手闭合。发起方从"发完 request 乐观直连"改为"**等 response 再继续**"。

**本里程碑不建立媒体**,画面仍不出——这是预期,媒体属里程碑 B。

**已确认决策(来自上位文档评审 + 本轮澄清):**
- session_id 缝:采纳 **5-1**(服务端生成 UUID,经 `ConnectRequest` 带给 B,B 原样回传)。
- 并发策略:**单活跃 + 其余即拒**(dialog 开着时再来的 `connect_request` 自动回 `accepted:false`)。
- 发起方 gating:**等 `connect_response` 再继续**(不再乐观直连 `_engine.connect`)。

---

## 2. 验收标准(AC)

| # | 场景 | 期望 |
|---|---|---|
| AC1 | A 发起,B 在首页 | B 弹 `ConnectionConfirmDialog`,显示 A 的设备码 |
| AC2 | A 发起,B 在非首页(如 /settings) | B 仍弹框(常驻宿主监听坐实) |
| AC3 | B 点"允许" | 服务端收到 `connect_response{accepted:true, session_id, from_code}`;A 收到该 response |
| AC4 | B 点"拒绝" | A 收到 `accepted:false`,A 回 idle 并提示"对方已拒绝" |
| AC5 | B 30s 未响应(dialog 超时) | dialog 自动回 `accepted:false`(视为拒绝),A 收到 rejected |
| AC6 | A 发起,B 设备离线 | A 收到 `error{code:device_offline}`,提示后回 idle(现状已实现,验证不回归) |
| AC7 | B dialog 开着时,C 也发起连接 | C 立即收到 `accepted:false`;B 的当前 dialog 不受干扰 |
| AC8 | A 发起后,收到 accepted 前 | A 处于 `connecting` 态,**不**调用 `_engine.connect`、**不**进 `/session` |
| AC9 | A 等 response 网络超时(35s 无 response 且无 error) | A 报错回 idle |
| AC10 | session_id 全程一致 | B 收到的 ConnectRequest.session_id(非空)== 服务端生成的 UUID;B 原样回传 == controller 收到的 session_id |

---

## 3. 组件设计

分四个改动单元,各有单一职责、可独立测试。

### 3.1 服务端协议补丁:`ConnectRequest` 携带 `session_id`(Rust)

**文件**:`crates/rdcs-signaling/src/ws/message.rs`、`src/handlers/connect.rs`

- `WsMessage::ConnectRequest` 新增字段 `session_id: Option<String>`(`#[serde(skip_serializing_if = "Option::is_none")]`)。**为何 Option 而非 required**:`ConnectRequest` 是**同一 union variant 双向复用**——controller→server 方向(`requestConnection` 发出时)还没有 session_id,server→B 方向才带。required 会破坏 controller→server 的序列化。
- `handle_connect_request`:UUID 已在 `let session_id = Uuid::new_v4()` 处生成,把它 `Some(session_id)` 填入转发给 B 的 `ConnectRequest`。inbound 路由 `handler.rs:225` 的解构忽略入站 session_id(controller 发的是 None)。
- 现有测试更新:`connect_request_forwards_to_target` 断言转发的 `ConnectRequest` 的 `session_id` 为 `Some(非空)`;`connect_request_round_trip`(`message.rs`)补 session_id 字段。

**接口契约**(wire,snake_case,见 memory `rdcs-signaling-protocol-snakecase`)。server→B 方向带 session_id;controller→server 方向省略(None 不序列化):
```json
{ "type": "connect_request",
  "from_code": "761335217", "to_code": "123456789",
  "session_id": "<uuid>", "invite_code": null }
```

### 3.2 Dart 侧 `ConnectRequestMessage` 加 `session_id`(freezed)

**文件**:`client/flutter/lib/core/signaling/models/signaling_message.dart`(+ 重新 codegen `.freezed/.g`)

- `ConnectRequestMessage` factory 加 `@JsonKey(name: 'session_id') String? sessionId`(**nullable**,与 Rust `Option` 对齐;controller 侧发出时为 null)。
- `SignalingService._handleMessage` 的 `connectRequest:` 分支签名随之 `(fromCode, toCode, sessionId, inviteCode)`,把 sessionId 一并放进 `_invitationsController.add(...)`。收到的入站请求 sessionId 应非空(服务端填了);宿主消费时用它回 response。
- `requestConnection`(发起方)不传 session_id(仍由服务端生成);只有 server→client 的 ConnectRequest 带它。

> 风险控制:3.1 与 3.2 必须**同一 PR 同步改**并跑双端序列化测试,否则 client freezed 与 server serde 不匹配 → 反序列化炸(memory 已坐实的坑)。

### 3.3 `connectResponse` 流打通(Dart SignalingService + Provider)

**文件**:`signaling_service.dart`、`signaling_provider.dart`

- `SignalingService` 新增 `_connectResponseController`(broadcast)+ `Stream<ConnectResponseMessage> get connectResponses`;`dispose()` 关闭它。
- `_handleMessage` 的 `connectResponse:` 分支:从当前 `_logUnexpected('connect_response')` 改为 `_connectResponseController.add(ConnectResponseMessage(...))`。
- `signaling_provider.dart` 新增 `connectResponsesProvider = StreamProvider<ConnectResponseMessage>`。

### 3.4 全局宿主监听器 + navigatorKey(Dart app 层)

**文件**:`app.dart`(+ 可能新增 `features/session/invitation_host.dart`)

- 新增全局 `final navigatorKey = GlobalKey<NavigatorState>();`,注入 `GoRouter(navigatorKey: navigatorKey, ...)`。
- 新增常驻监听 widget(包在 `MaterialApp.router` 的 `builder:` 里,或作为 router 之上的 `Consumer`),职责:
  - `ref.listen(invitationsProvider, (prev, next) { next.whenData((req) => _showInvite(req)); })`
  - `_showInvite`:若已有 dialog 活跃 → 立即 `respondToConnection(accepted:false, sessionId:req.sessionId, fromCode:req.fromCode)`(单活跃+即拒);否则经 `navigatorKey.currentContext` 弹 `ConnectionConfirmDialog.show(...)`,按返回的 `ConnectionConfirmResult` 调 `respondToConnection(accepted: result==accepted, ...)`。
  - 用一个 `bool _dialogActive` / provider 记活跃态。
- **为何常驻宿主而非子页面**:用户不在特定页就收不到(memory `rdcs-invitation-ui-missing` 坐实)。

### 3.5 发起方 gating 改造(Dart SessionNotifier)

**文件**:`features/session/session_providers.dart`

- `SessionNotifier.connect` 现有流程:发 `requestConnection` → 立即 `await _engine.connect(code)`。
- 改为:发 `requestConnection` 后,`await` 一个"等 `connectResponses` 流里 fromCode 匹配的 response 或超时(35s,略大于 B dialog 30s 倒计时)"的 Future:
  - `accepted:true` → **A 里程碑到此为止**,状态标记为已握手(媒体由 B 里程碑接管;A 阶段可临时进 `/session` 占位或停在 connecting——见 §5 开放问题)。
  - `accepted:false` → `state = error` / 回 idle,提示"对方已拒绝"。
  - 超时/`device_offline` error → `state = error`,提示。
- `SessionSignaling` 抽象接口(`session_signaling.dart`)需扩展,暴露 `Stream<ConnectResponseMessage> get connectResponses`,以便 notifier 消费(保持可测)。

---

## 4. 数据流(A 完整时序)

```
A(controller)                    server                    B(target)
  connect(code)
  requestConnection(code) ─────────►
                          gen session_id=UUID
                          add_pending(to=B, from=A)
                          ConnectRequest{session_id} ─────►
                                                    invitationsProvider 触发
                                                    宿主监听 → dialog(30s)
                                                    ┌ 允许 → respondToConnection
                                                    │        {accepted:true, session_id, from_code=B}
                                                    └ 拒绝/超时 → {accepted:false,...}
                                          ◄──────── connect_response
                          take_pending(B)→A
                          insert_session_participants(session_id, A, B)
                          (Redis session:{id})
  ◄──── ConnectResponse{accepted, session_id, from_code=B}
  connectResponses 流触发 → gating 解除
  ├ accepted → 握手闭合(媒体属 B)
  └ rejected → 回 idle 提示
```

---

## 5. 错误处理

| 情况 | 处理 |
|---|---|
| B 离线 | 服务端 `error{device_offline}`(已实现)→ A `signalingErrorsProvider` 消费并提示。**注**:该错误流当前无 UI 消费方,A 里程碑需接线到发起流程的错误提示。 |
| B 拒绝 / dialog 超时 | `accepted:false` → A 回 idle,提示"对方已拒绝或超时" |
| A 等 response 超时(35s) | A `state=error` 回 idle |
| B dialog 开着又来请求 | 新请求即拒(`accepted:false`),不打断当前 dialog |
| 信令 WS 断线(A 等待中) | 复用现有重连(memory `rdcs-reconnect-reregister`);gating Future 超时兜底 |

---

## 6. 测试策略

**Rust(3.1)**
- `connect.rs` 现有单元/集成测试更新:`ConnectRequest` 断言含非空 `session_id`。
- 全流程测试 `full_connection_negotiation_flow`:断言转发的 session_id 与最终 response 透传的一致(AC10)。

**Dart(3.2–3.5)**
- 序列化:`ConnectRequestMessage` 含 session_id 的 fromJson/toJson 往返(防 3.1/3.2 不同步)。
- widget:宿主监听收到 invitation → dialog 出现(AC1);非首页路由下仍出现(AC2)。
- 交互:点允许/拒绝 → `respondToConnection` 被调且参数(accepted、session_id、fromCode)正确(AC3/AC4)。
- 并发:dialog 活跃时二次 invitation → 直接 `respondToConnection(accepted:false)`,不新开 dialog(AC7)。
- gating:`SessionNotifier.connect` 在收到 accepted 前 `state==connecting` 且未调 `_engine.connect`(AC8);收到 rejected → error(AC4);超时 → error(AC9)。用 fake `SessionSignaling` 注入可控 `connectResponses` 流。

**端到端**
- 扩展 `test/integration/live_two_client_invite_test.dart`:双客户端跑真实握手至 `connect_response`,断言 session_id 一致(AC3/AC10)。

---

## 7. 已定决策(2026-07-02 评审采纳)

1. **A 收到 accepted 后停在哪?** **取 (a)**:停在 `connecting` 态显示"已连接,等待画面"占位,**不**进 `/session` 空壳页。媒体由里程碑 B 接管后再进 session。
2. **等 response 超时时长**:**取 35s**(略大于 B 侧 dialog 的 30s 倒计时),避免"A 先超时回 idle 而 B 仍在弹框"的竞态。
3. **navigatorKey 注入方式**:GoRouter 用 `navigatorKey` 参数注入全局 key;宿主监听 widget 放 `MaterialApp.router` 的 `builder:`。实现细节在 plan 落地。

---

## 8. 影响面清单

| 文件 | 改动 |
|---|---|
| `crates/rdcs-signaling/src/ws/message.rs` | `ConnectRequest` 加 `session_id` |
| `crates/rdcs-signaling/src/handlers/connect.rs` | 填入 session_id;测试更新 |
| `client/flutter/lib/core/signaling/models/signaling_message.dart` | `ConnectRequestMessage` 加 sessionId(+codegen) |
| `client/flutter/lib/core/signaling/signaling_service.dart` | connectRequest 分支带 sessionId;新增 connectResponses 流 |
| `client/flutter/lib/core/signaling/signaling_provider.dart` | 新增 `connectResponsesProvider` |
| `client/flutter/lib/core/signaling/session_signaling.dart` | 接口加 `connectResponses` |
| `client/flutter/lib/app.dart` | 全局 navigatorKey + 常驻宿主监听 |
| `client/flutter/lib/features/session/session_providers.dart` | `connect` 改为等 response gating |
| `client/flutter/lib/features/session/invitation_host.dart`(可选新增) | 宿主监听 widget |
| 测试 | Rust connect.rs、Dart widget/交互/gating、集成 live_two_client |
