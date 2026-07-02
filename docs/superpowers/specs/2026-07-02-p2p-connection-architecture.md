# 两客户端 P2P 连接 —— 总体架构与里程碑

- 状态:架构草案(待评审)
- 日期:2026-07-02
- 范围:发起方(controller)与被控方(target)之间,从「发起连接」到「看到远程桌面画面」的完整链路补全。
- 本文层级:**架构 / 里程碑**。不含逐行实现细节;每个里程碑落地时各自出独立 spec + plan。

---

## 1. 背景与问题

现象:A 连 B,B 端不弹接受框;即便强行接受,也看不到画面。双端复现(两端跑同一套代码)。

2026-07-02 双向(运行时日志 + 代码)排查结论:**链路三层全断**,不是单一"缺弹窗胶水"。

| 层 | 现状 | 关键证据 |
|---|---|---|
| ① 邀请弹窗胶水 | ❌ 未写 | `invitationsProvider` 零订阅;`ConnectionConfirmDialog.show()` 零调用零 import(`features/session/connection_confirm_dialog.dart`) |
| ② Dart 信令消费 | ❌ 未写 | `signaling_provider.dart` 未暴露 `connect_response`/ICE 流;`SessionNotifier.connect`(`session_providers.dart:98`)发完 `requestConnection` **不等 response**,直接 `await _engine.connect(code)`;offer/answer/trickle 在 Dart 侧从未收发 |
| ③ FFI / 引擎传输 | ❌ mock | `rdcs_connect`(`crates/rdcs-ffi/src/lib.rs:535`)= `next_session_id += 1` 后立刻 `dispatch_event(CONNECTION_ESTABLISHED)`,注释 `// TODO: Wire to ConnectionManager` |

**已存在但未接线的资产**(降低 ③ 的风险):
- 真实 WebRTC 实现:`crates/rdcs-connection/src/real_ice_agent.rs`(webrtc-rs,offer/answer/candidate/DataChannel 齐全,offerer/answerer 双角色)。
- 采集/编码是**部分真实**的:`rdcs_start_capture`(`crates/rdcs-ffi/src/lib.rs:250+`)真的从 `engine.platform.capture.start()` 收帧并编码。
- 引擎已持有 tokio `Runtime`、`video_handler`、`crypto_factory`(`lib.rs:95-109`)。
- 服务端信令路由**完整且已测**:`handle_connect_request/response/ice_*`(`crates/rdcs-signaling/src/handlers/connect.rs`),含单元 + 全流程集成测试。

**一个协议缝(贯穿 A)**:服务端 `handle_connect_request` 生成 `session_id = Uuid`,并 `add_pending_connection(to_code, from_code)`,但**转发给 B 的 `ConnectRequest` 不带 session_id**(`ws/message.rs:40-45` 只有 from/to/invite)。因此 B 回 `connect_response` 时那个**必填**的 `session_id` 当前无来源。详见 §5。

---

## 2. 目标与非目标

**目标**
- G1:B 端收到连接请求时弹出接受/拒绝对话框,且无论 B 当前在哪个页面都能弹(常驻宿主监听)。
- G2:B 接受/拒绝 → A 收到 `connect_response`,信令握手闭合;A 的连接流程**等待** response 而非乐观直连。
- G3:握手闭合后,双端经 WebRTC 建立 DataChannel,采集→编码→传输→解码→渲染,双端看到远程桌面画面。

**非目标(本轮不做)**
- 中继(relay/TURN)路径:`relay_request`/`relay_assigned` 已有协议位,但 P2P 直连打通前不接。
- 邀请码(invite_code)鉴权语义强化:沿用现状透传,不在本轮改语义。
- 多并发会话的完整调度:先支持"单活跃会话 + 排队提示"(`ConnectionConfirmDialog` 已有 `queuedCount` 徽标位)。
- 团队(team_id)路由。

---

## 3. 里程碑拆分

「端到端出画面」跨 Dart + Rust,单 spec 过大且难分阶段验收。按**信令缝**切成两个可独立验收的里程碑:

### 里程碑 A —— 信令握手闭合(纯 Dart + 一处服务端协议补丁)

范围:
- 全局宿主(`app.dart` 顶层 / 全局 `navigatorKey`)监听 `invitationsProvider` → `ConnectionConfirmDialog.show` → `respondToConnection(accepted)`。
- 补 session_id 协议缝(§5 二选一)。
- 发起方 `SessionNotifier.connect` 改为**等 `connect_response`** 再进入下一步(新增 `connectResponseProvider` 流消费)。
- 拒绝 / 超时(dialog 已有 30s 倒计时)/ 设备离线(`error: device_offline`)的 UI 反馈。

**独立验收**:A 发起 → B 弹框 → B 接受或拒绝 → A 收到对应 response。纯信令层,可写端到端集成测试(已有 `test/integration/live_two_client_invite_test.dart` 作骨架)。**此里程碑不碰媒体**,画面仍不出——这是预期。

### 里程碑 B —— WebRTC 媒体通路(FFI 接 real_ice_agent + Dart 中继 ICE)

范围:
- FFI `rdcs_connect` 接 `real_ice_agent`:controller 侧建 offerer(带 DataChannel),生成 SDP offer + candidates,经 FFI 回调交给 Dart。
- 被控方接受后,引擎建 answerer,消费 offer、产出 answer。
- Dart 新增 `iceOffer/iceAnswer/iceTrickle` 双向消费,和引擎互喂(见 §4 决策)。
- DataChannel 建通后:capture→encode→channel→decode→render 贯通。
- ICE 失败的降级/报错(暂不接 relay,仅上报 `IceState::Failed`)。

**独立验收**:双端真正看到远程桌面画面。跨 Dart+Rust,工作量最大;A 是其前置(session_id 缝、握手 gating 都在 A 补齐)。

**推荐顺序:A → B。** A 最小、可端到端测试、且解决 B 依赖的 session_id 前置。

---

## 4. 关键架构决策:ICE 信令由谁经服务器收发 → **Dart 中继**

**决策:WebRTC 的 offer/answer/candidate 由 Rust 引擎产生/消费,但经 FFI 回调交给 Dart,用现有的 signaling WebSocket 收发。Rust 不自持第二条 WS。**

理由:
1. Dart 已有一条注册好的 signaling WS,且经上一轮重连修复(边沿驱动重注册 + 心跳检测半开 TCP —— 见 memory `rdcs-reconnect-reregister`)。服务端的 session 参与者(`insert_session_participants`)正是按**这条连接的 device_code** 建立路由。
2. 若 Rust 另起一条 WS 用同一 device_code 注册,会与 Dart 那条**双注册**;服务端 `send_to` / 路由全按设备码,两条连接互抢注册、ICE 转发目标错乱。
3. 代价:FFI 面变宽(需在 FFI 边界传 SDP 字符串 + candidate JSON,双向)。可控,且换来单一信令连接、复用现成重连,远比在 Rust 重建 WS+注册+重连+心跳便宜。

**被否方案(Rust 自持信令连接)**:让引擎自己连 signaling 服务器收发 ICE。否决——见上第 2、3 点,双连接注册冲突是硬伤,除非重构服务端按「连接实例」而非「设备码」路由,超出本轮范围。

**跨 FFI 的 ICE 数据流(里程碑 B):**
```
Controller(发起)                     Target(被控)
  Dart                Rust             Rust                Dart
  connect()  ─FFI→  offerer
                    gen offer ─cb→ Dart
  send ice_offer ──────── signaling server ──────→ 收 ice_offer
                                    Dart ─FFI→ answerer
                                              consume offer
                                              gen answer ─cb→ Dart
  收 ice_answer ←──────── signaling server ──────── send ice_answer
  Dart ─FFI→ offerer
           consume answer
  (trickle candidates 双向,同上经 signaling 中继)
  → DataChannel open → 媒体贯通
```
新增 FFI 面(B 里程碑细化):`rdcs_connect` 返回后异步产出 offer(经回调);新增 `rdcs_feed_remote_sdp(session, sdp, role)`、`rdcs_feed_remote_candidate(session, cand)`;新增 outbound 事件 `EVENT_LOCAL_SDP` / `EVENT_LOCAL_CANDIDATE`。具体签名在 B 的 spec 定。

---

## 5. 待决:session_id 协议缝补法(A 里程碑,请评审拍板)

B 端回 `connect_response` 需要一个 `session_id`(wire 上 required)。当前服务端生成的 UUID 没传给 B。两个补法:

### 方案 5-1:ConnectRequest 携带 session_id(**推荐**)

服务端把已生成的 UUID 放进转发给 B 的 `ConnectRequest`;B 原样带回 `connect_response`。

- 改动:Rust `WsMessage::ConnectRequest` 加 `session_id: String` 字段 + `handle_connect_request` 填入;Dart `ConnectRequestMessage`(freezed)加对应字段;`connect.rs` 相关测试更新。
- 优点:session_id **全程唯一来源 = 服务端**,与 `add_pending_connection` / Redis `session:{id}` 记录天然一致;B/controller 都用同一个 id;语义最干净。
- 缺点:动 wire 协议 = 双端 freezed/serde 必须同步(见 memory `rdcs-signaling-protocol-snakecase`:client freezed 必须匹配 server serde,否则反序列化炸)。属于协调成本,非技术风险。

### 方案 5-2:B 端生成 session_id 回传

wire 不变,B 端自己 `Uuid` 生成回传。

- 优点:不动 `ConnectRequest` wire。
- 缺点:服务端 `handle_connect_request` **已经**用自己的 UUID 建了 `add_pending_connection` 和(可选)Redis 记录。B 端另生成一个 → 两个 id 不一致。`handle_connect_response` 目前用 `take_pending_connection(from_device)` 按**设备码**取回 controller(不依赖 id 匹配),所以**能跑**,但服务端日志/Redis 里的 id 与实际会话 id 不同,后续接 relay/审计/断线重连按 id 查会对不上。埋雷。

**倾向 5-1**:一次性动 wire,换全程 id 一致。5-2 省一次协议改动,但留一个"服务端记的 id 和真实会话 id 不同"的隐患,B 里程碑接 relay 时会被绊到。

> **决策(2026-07-02 已定):采纳 5-1。** ConnectRequest 携带服务端生成的 session_id,B 原样带回,全程 id 唯一来源=服务端。里程碑 A 的 spec 以此为前提。

---

## 6. 里程碑 A 组件与数据流(架构级)

**新增 / 改动单元:**

1. **全局宿主监听器**(新)——位置:`app.dart` 的 `MaterialApp.router` 之上或 `builder` 注入的常驻 widget。
   - 职责:`ref.listen(invitationsProvider, ...)`,收到 `ConnectRequestMessage` → 经全局 `navigatorKey` 弹 `ConnectionConfirmDialog`。
   - 依赖:全局 `navigatorKey`(GoRouter 支持注入);`signalingServiceProvider`。
   - 为何在此:放任何子页面 → 用户不在该页就收不到(memory 已坐实的坑)。宿主随 app 常驻。
   - 并发:同一时刻至多一个 dialog;后到请求走 `queuedCount` 徽标或暂拒(A 里程碑定最简策略:单活跃 + 其余即时拒绝并可选提示)。

2. **`connectResponseProvider`**(新,`signaling_provider.dart`)——暴露 `service.connectResponse` 流(service 侧需把 `connectResponse` 从 `_logUnexpected` 改为 add 进新 controller)。
   - 消费方:`SessionNotifier.connect`,用于把"发 request 后乐观直连"改为"等 response(accepted=true)再 `_engine.connect` / 进 `/session`"。

3. **`SignalingService` 改动**——`_handleMessage` 的 `connectResponse:` 分支改为 add 进 `_connectResponseController`;`connectRequest:` 分支带上 session_id(采 5-1)。

4. **服务端协议补丁**(5-1)——`ConnectRequest` 加 `session_id`,`handle_connect_request` 填入。

**A 的时序:**
```
A: connect(code)
   └ requestConnection(code)  ──ws──> server ──ws──> B: invitationsProvider 触发
B: ConnectionConfirmDialog(接受/拒绝/30s超时)
   ├ 接受 → respondToConnection(accepted:true, sessionId, fromCode) ──ws──> server
   └ 拒绝/超时 → respondToConnection(accepted:false, ...)
server ──ws──> A: connectResponseProvider 触发
   ├ accepted → 进入媒体阶段(A 里程碑到此为止,画面属 B)
   └ rejected → A 显示"对方拒绝",回 idle
```

**A 的错误分支:** 设备离线(`error: device_offline`,已实现)→ A 提示;B 超时未响应 → 视为拒绝;A 等 response 超时(网络)→ 报错回 idle。

---

## 7. 里程碑 B 组件与数据流(架构级,概要)

细节留 B 的 spec;此处仅定架构骨架:

- **FFI offerer/answerer 接线**:`rdcs_connect` 用 `real_ice_agent` 建 offerer(`create_data_channel=true`);被控端在接受后经新 FFI 入口建 answerer(`create_data_channel=false`)。
- **ICE 经 Dart 中继**(§4):新增 outbound 事件(local SDP / local candidate)与 inbound FFI(feed remote SDP / candidate)。
- **session_id 映射**:Dart 侧字符串 session_id(信令)↔ 引擎侧会话句柄。当前 `_engine.connect` 返回 int、ICE 用 String —— B 里程碑需在引擎内建立 String session_id → 内部会话的映射(controller 用服务端 UUID 作 key)。
- **媒体贯通**:DataChannel `on_open` → 启动 capture→encode 管线,帧经 channel 送对端 → decode → 渲染到 `SessionScreen`。
- **降级**:`IceState::Failed` 上报 → A 显示连接失败(本轮不自动切 relay)。

---

## 8. 测试策略

- **里程碑 A**:
  - Rust:`connect.rs` 现有单元/集成测试随 5-1 更新(ConnectRequest 带 session_id 的断言)。
  - Dart:widget 测试宿主监听 → 收到 invitation → dialog 出现;接受/拒绝 → `respondToConnection` 被调且参数正确;`SessionNotifier.connect` 在收到 accepted 前不进 `/session`。
  - 端到端:扩展 `test/integration/live_two_client_invite_test.dart`,双客户端跑真实握手至 `connect_response`。
- **里程碑 B**:
  - Rust:`real_ice_agent` 已有测试;新增 FFI offer/answer 往返、DataChannel open 的集成测试(loopback)。
  - 端到端:双端真机/双进程,断言画面帧到达(帧计数 > 0)。

---

## 9. 风险与依赖

| 风险 | 缓解 |
|---|---|
| wire 协议改动双端不同步(5-1)→ 反序列化炸 | 同一 PR 内改 Rust serde + Dart freezed 并跑双端序列化测试;参考 memory `rdcs-signaling-protocol-snakecase` |
| 陈旧信令连接(服务端重启后旧客户端)干扰握手 | memory `rdcs-stale-connection-peer-offline`:重启客户端恢复;测试前确保双端新鲜连接 |
| B 里程碑 FFI 面变宽,SDP/candidate 跨边界字符串管理(内存/生命周期) | 沿用现有 `rdcs_free_string` 模式;B spec 明确 owner |
| 引擎 int sessionId 与信令 String session_id 语义错位 | B 里程碑在引擎内建映射,controller 以服务端 UUID 为准 |

---

## 10. 一句话总结

「双端连不上」是**三层全断**(弹窗胶水 / Dart 信令消费 / FFI 真实传输),不是一层。按信令缝拆成 **A(信令握手,纯 Dart + 一处服务端补丁)** 与 **B(WebRTC 媒体,FFI 接 real_ice_agent + Dart 中继 ICE)**,先 A 后 B。ICE 中继定为 Dart 侧(复用现有 WS + 重连);session_id 缝推荐 5-1(ConnectRequest 携带,全程 id 唯一来源=服务端),待你拍板。
