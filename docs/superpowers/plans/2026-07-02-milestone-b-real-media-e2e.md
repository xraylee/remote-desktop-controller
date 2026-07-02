# 里程碑 B —— 真实媒体端到端 实现计划(Phase 1)

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development / executing-plans。逐任务推进,checkbox 跟踪。

**Goal(Phase 1):** 把 examples 里的 ICE 握手胶水提升为 `rdcs-connection` 的库类型 `MediaSession`,以**可序列化信令消息**(offer/answer/candidate 的 JSON)为接口驱动 `RealIceAgent`,并新增库级集成测试:两个 `MediaSession` 只经序列化消息通信,建 ICE P2P DataChannel 并流真实 H.264 帧,断言 100% 解码。默认软件编解码(OpenH264)。

**上位 spec:** `docs/superpowers/specs/2026-07-02-milestone-b-real-media-e2e-design.md`

**基线(已实测):** `video_e2e_test`(software-encoder)30/30 帧真实 e2e 通过,exit 0。commit 7a12933 已修 CoreFoundation 链接 + example 漂移。

---

## 文件结构(Phase 1)

| 文件 | 职责 | 动作 |
|---|---|---|
| `crates/rdcs-connection/src/media_session.rs` | `MediaSession`:包 RealIceAgent+VideoChannel,信令式建连 + 收发帧 | 新建 |
| `crates/rdcs-connection/src/lib.rs` | 导出 `MediaSession` | 改 |
| `crates/rdcs-connection/tests/media_session_e2e.rs` | 库级集成测试:双 MediaSession 经序列化消息建连流帧 | 新建 |
| `crates/rdcs-connection/Cargo.toml` | dev-dep 加 software-encoder 特性用于测试 | 改(如需) |

---

## Task 1: `MediaSession` 库类型(信令式建连 API)

**Files:** 新建 `crates/rdcs-connection/src/media_session.rs`;改 `src/lib.rs`

设计(接口以序列化消息为界,证明可经真实信令转发):
- `MediaSession::new_offerer(ice_servers) -> Self` / `new_answerer(ice_servers) -> Self`(包 `RealIceAgent::new_with_options(_, is_offerer)`)。
- offerer:`create_local_offer() -> SdpOffer`(内部 gather+create_offer)。
- answerer:`accept_offer(&SdpOffer) -> SdpAnswer`(set_remote_offer + gather + 组 answer)。
- offerer:`accept_answer(SdpAnswer)`(handle_answer + set_remote_candidates)。
- 双方:`add_remote_candidates(Vec<IceCandidate>)`(已含在 offer/answer 内则可选)。
- `wait_connected(timeout) -> Result<()>`(poll connection_state)。
- `data_channel_ready(timeout)`、`video_channel() -> VideoChannel`。
- `send_frame(&[u8])`(经 VideoChannel 分片)。
- `on_frame(cb)`(FrameReassembler 重组后回调完整帧)。

- [ ] **Step 1: 写 `MediaSession`,复刻 example 的 establish_connection 时序但以消息为界**
- [ ] **Step 2: `cargo build -p rdcs-connection` 通过**
- [ ] **Step 3: 导出 + commit**

## Task 2: 库级集成测试(信令式 e2e)

**Files:** 新建 `crates/rdcs-connection/tests/media_session_e2e.rs`

- [ ] **Step 1: 写测试** —— 建 offerer/answerer;`let offer = offerer.create_local_offer()`;**序列化 offer→JSON→反序列化**交给 answerer;`let answer = answerer.accept_offer(&offer)`;序列化往返交回 offerer;`offerer.accept_answer(answer)`;双方 wait_connected + data_channel_ready;offerer 编码 N 帧(用 rdcs-codec software encoder,test 生成帧)send_frame;answerer on_frame 累计,断言收到 N 帧且解码成功。
- [ ] **Step 2: 运行** `cargo test -p rdcs-connection --features software-encoder --test media_session_e2e -- --nocapture`,PASS。
- [ ] **Step 3: commit**

## Task 3: 回归 + memory

- [ ] **Step 1:** `cargo build -p rdcs-connection --examples` 通过(不回归)。
- [ ] **Step 2:** 更新 memory `rdcs-milestone-b-media-baseline` 标注 Phase 1 完成。
- [ ] **Step 3:** commit。
