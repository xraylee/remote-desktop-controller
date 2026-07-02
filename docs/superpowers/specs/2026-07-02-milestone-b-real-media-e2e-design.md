# 里程碑 B —— 真实媒体端到端 设计规格(Spec)

- 状态:待评审(边推进边细化,按用户指示不中断提问)
- 日期:2026-07-02
- 上位文档:里程碑 A spec `2026-07-02-milestone-a-signaling-handshake-design.md`;架构决策 `docs/decisions/WEBRTC_CODEC_INTEGRATION_DECISION.md`(方案 B:webrtc-rs + 平台原生编解码)
- 范围:让**真实 App 端到端连接并验证核心功能**——被控端屏幕/视频帧经真实 ICE P2P DataChannel 送达主控端并解码。

---

## 0. 基线事实(2026-07-02 已实测,见 memory `rdcs-milestone-b-media-baseline`)

真实管线在 **examples 层已跑通**:`RealIceAgent`(webrtc-rs 0.9)建 ICE P2P + DataChannel,`NativeVideoEncoder/Decoder`(H.264)编解码,`VideoChannel`+`FrameReassembler` 分片重组。
- **软件编解码(OpenH264)路径 100% 通过**:`video_e2e_test` 30/30 帧真实送达并解码,exit 0。→ 里程碑 B 默认依赖此已验证路径。
- VideoToolbox 硬件解码器解第 0 帧 SIGSEGV → 硬件加速属后续优化,非阻塞。
- 已修两个真实缺陷(commit 7a12933):CoreFoundation 链接、example CapturedFrame 漂移。

**核心判断**:里程碑 B 的本质是**整合已有真实能力**(examples 里的胶水提升为库),而非从零实现编解码。真实能力都在,只是没接进 FFI/Dart。

---

## 1. 目标与非目标

**目标(核心功能验证)**:
1. 库层:两个对等体经**真实信令式消息交换**(SdpOffer/SdpAnswer/candidate 的序列化 JSON,而非直接方法调用)建立 ICE P2P DataChannel,单向流真实 H.264 帧并解码。可离线(loopback)集成测试验证。
2. FFI 层:`rdcs_connect` 从 mock 计数器改为真正驱动 ICE 连接;FFI 边界暴露 ICE offer/answer/candidate 的进出(供 Dart 经信令转发)。
3. Dart 层:消费入站 `ice_offer/ice_answer/ice_trickle`(现被丢弃),把信令 String session_id 桥到引擎;`SessionNotifier.connect` 在信令 accept 后驱动媒体(而非停在 connected)。
4. 端到端:两个真实 App 实例(或 `rdcs-desktop` 双进程)经真实信令服务器建媒体连接,被控端画面出现在主控端。

**非目标(本里程碑不做)**:
- VideoToolbox/MediaFoundation/VA-API 硬件编解码修复与优化(软件 OpenH264 先行)。
- SRTP 媒体加密(DataChannel 自带 DTLS,已加密;RTP/SRTP 属方案 B 后续)。
- 输入回控的完整键位映射、文件传输、剪贴板。
- 拥塞控制/码率自适应调优。

---

## 2. 分阶段(每阶段自成可验证单元,按 superpowers 逐任务推进)

**Phase 1 — 库层 MediaSession 抽象 + 信令式集成测试(最高价值、完全可离线验证)**
把 examples 里的握手胶水提升为 `rdcs-connection` 的库类型 `MediaSession`,以**信令消息交换**为接口(offerer 产出 offer JSON,answerer 吃 offer JSON 产出 answer JSON,双方交换 candidate JSON),内部驱动 `RealIceAgent`。新增库级集成测试:两个 `MediaSession` 只经序列化消息通信,建连并流 N 帧,断言 100% 解码。默认软件编解码。

**Phase 2 — FFI 边界接入 ICE**
`rdcs-ffi` 引入 `MediaSession`;`rdcs_connect` 真正建 offerer/answerer;新增 FFI 导出:产出本地 offer/answer/candidate(经事件回调或返回值),吃入远端 offer/answer/candidate;帧到达经既有 `EVENT_FRAME_READY` 派发。int↔String session_id 映射在 FFI 内维护。

**Phase 3 — Dart 消费入站 ICE + 媒体挂载**
`signaling_service.dart` 把 `ice_offer/ice_answer/ice_trickle` 从 `_logUnexpected` 改为广播流 + provider;`SessionNotifier.connect` 在 accept 后:发/收 offer-answer-candidate,喂给引擎,String session_id 桥到 int。统一/协调两个 engine 实例(`engineProvider` vs `engineIsolateProvider`)。

**Phase 4 — 真实双端 e2e 验证**
`rdcs-desktop` 或双 App 实例经活信令服务器建媒体连接;被控端 `CGDisplayCreateImage` 真实捕获 → 编码 → 送达 → 主控端 `VideoRenderer` 出画面。测量帧率/延迟。

**优先级**:Phase 1 立即可做且完全可验证,是里程碑 B 的地基与「真实核心功能」的首个硬证据。后续 Phase 依 App/双机环境,边做边验。

---

## 3. 验收标准(AC)

| # | 场景 | 期望 |
|---|---|---|
| AC1 | 两个 `MediaSession` 仅经序列化 offer/answer/candidate JSON 通信 | ICE 建连 Connected,DataChannel open |
| AC2 | offerer 编码 N 帧真实 H.264 经 DataChannel 送出 | answerer 收到 N 帧、重组、解码成功率 100% |
| AC3 | 软件编解码(OpenH264)特性下 Phase 1 集成测试 | PASS、exit 0、可在 CI/离线跑(loopback ICE) |
| AC4 | 序列化的 SdpOffer/SdpAnswer/IceCandidate 与信令 wire(snake_case)对齐 | 字段名/结构匹配 `rdcs-signaling` 的 `IceCandidate`/SDP 载荷,可直接经信令转发 |
| AC5(Phase 2) | FFI `rdcs_connect` 被调用 | 真正创建 MediaSession(offerer),不再是自增计数器;产出可取的本地 offer |
| AC6(Phase 2) | FFI 吃入远端 answer + candidate | ICE 推进到 Connected;帧到达经 EVENT_FRAME_READY 派发 |
| AC7(Phase 3) | Dart 收到入站 ice_offer/answer/trickle | 不再 `_logUnexpected`;经新流交给引擎;String↔int session_id 正确桥接 |
| AC8(Phase 4) | 双端经活信令服务器 | 被控端真实屏幕帧出现在主控端 VideoRenderer |
| AC9 | 不回归里程碑 A | 信令握手测试全绿;既有 Rust lib + Dart 里程碑测试不破 |

---

## 4. 关键设计约束

- **默认软件编解码**:Phase 1 集成测试用 `software-encoder` 特性(OpenH264),避免 VideoToolbox 崩溃阻塞;硬件路径后续单独修。
- **信令式接口而非直接调用**:`MediaSession` 的建连必须经可序列化消息(证明能经真实信令转发),不能靠对等体互相持有引用直接调方法——否则测试是假的 e2e。
- **对齐 snake_case wire**:序列化结构须与 `rdcs-signaling` 的 `WsMessage::IceOffer/IceAnswer/IceTrickle` + `IceCandidate` 字段一致(见 memory `rdcs-signaling-protocol-snakecase`)。
- **loopback 可跑**:Phase 1 测试不依赖公网 STUN(host candidate 在 loopback/LAN 即可建连,实测软件路径已通)。
- **每单元可提交**:遵循里程碑 A 的粒度,一个改动单元 = 一个 commit。

---

## 5. 上位真实能力清单(复用,不重造)

| 能力 | 位置 | 状态 |
|---|---|---|
| ICE P2P + DataChannel | `rdcs-connection/src/real_ice_agent.rs` `RealIceAgent` | 真实(webrtc-rs 0.9) |
| SDP/candidate 结构 | `rdcs-connection/src/ice.rs` `SdpOffer/SdpAnswer/IceCandidate` | 真实、serde |
| 帧分片/重组 | `rdcs-connection/src/{video_channel,frame_reassembler}.rs` | 真实 |
| H.264 编解码 | `rdcs-codec` `NativeVideoEncoder/Decoder`(OpenH264 软件 / VideoToolbox 硬件) | 软件真实可用;硬件解码崩溃 |
| 屏幕捕获 | `rdcs-macos/src/capture.rs` `MacOsScreenCapture`(CGDisplayCreateImage) | 真实 |
| 信令 ICE 转发 | `rdcs-signaling/src/handlers/connect.rs` `handle_ice_offer/answer/trickle` | 真实、有测试 |
| Dart 出站 ICE | `signaling_service.dart` `sendIceOffer/Answer/Candidate` | 已实现 |
| Dart 帧渲染 | `features/session/video_renderer.dart` `VideoRenderer` | 真实 BGRA 渲染器,缺帧源 |

---

**下一步**:写 plan(逐任务),从 Phase 1(库层 MediaSession + 信令式集成测试)开始执行。
