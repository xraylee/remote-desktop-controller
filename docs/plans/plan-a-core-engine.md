# Plan A: Core Engine (librdcs_core) 实现计划

**基线**: `docs/specs/architecture-design.md` Sec 1-2 | **周期**: 8 周 | **产出**: 9 crate + FFI

## 跨项目依赖

本项目无外部项目依赖（其他项目依赖本项目）

### Task 1: Project Setup — Cargo Workspace & CI (~2d)

**目标**: 搭建 Cargo workspace 骨架，9 个子 crate，统一 CI。确保 build/test/clippy 全绿。
**依赖**: 无
**文件**: `Cargo.toml`, `crates/rdcs-{core,platform,crypto,transport,codec,connection,transfer,ffi,macos}/Cargo.toml`, `.github/workflows/ci.yml`
**接口契约**:
- `[workspace] members = ["crates/*"]` + `resolver = "2"` + 统一 `edition = "2021"`, `license = "Apache-2.0"`
- `[workspace.dependencies]`: `tokio`, `serde`, `tracing`, `thiserror`, `anyhow`, `zeroize`
- CI stages: `fmt -> clippy -> build -> test`

**验收标准**:
- [ ] workspace_build: `cargo build --workspace` 通过
- [ ] clippy_clean: `cargo clippy -- -D warnings` 零警告
- [ ] ci_green: GitHub Actions 全 pass
### Task 2: Platform Abstraction Traits (~3d)

**目标**: 定义 5 个平台抽象 trait（ScreenCapture / InputInjector / AudioCapture / SystemNotify / ClipboardProvider），Core Engine 与 OS 之间的隔离层，上层逻辑 100% 跨平台。
**依赖**: Task 1
**文件**: `crates/rdcs-platform/src/{lib,capture,input,audio,notify,clipboard}.rs`
**接口契约**:
```rust
pub trait ScreenCapture: Send + Sync {
    fn start(&self, config: CaptureConfig) -> Result<mpsc::Receiver<CapturedFrame>>;
    fn stop(&self) -> Result<()>;
    fn is_capturing(&self) -> bool;
    fn displays(&self) -> Result<Vec<DisplayInfo>>;
}
pub trait InputInjector: Send + Sync {
    fn inject_mouse(&self, event: MouseEvent) -> Result<()>;
    fn inject_key(&self, event: KeyEvent) -> Result<()>;
    fn inject_scroll(&self, event: ScrollEvent) -> Result<()>;
}
pub trait AudioCapture: Send + Sync {
    fn start(&self, config: AudioConfig) -> Result<mpsc::Receiver<AudioChunk>>;
    fn stop(&self) -> Result<()>;
    fn devices(&self) -> Result<Vec<AudioDeviceInfo>>;
}
pub trait SystemNotify: Send + Sync {
    fn show_notification(&self, title: &str, body: &str) -> Result<()>;
    fn set_tray_status(&self, status: TrayStatus) -> Result<()>;
    fn play_sound(&self, sound: SystemSound) -> Result<()>;
}
pub trait ClipboardProvider: Send + Sync {
    fn get_text(&self) -> Result<String>;
    fn set_text(&self, text: &str) -> Result<()>;
    fn watch(&self) -> Result<mpsc::Receiver<ClipboardEvent>>;
}
// 辅助类型 (纯数据结构): CapturedFrame, CaptureConfig, PixelFormat, DisplayInfo,
// MouseEvent, KeyEvent, ScrollEvent, AudioChunk, TrayStatus, ClipboardEvent, ClipboardContent
```
**验收标准**:
- [ ] trait_object_safe: 所有 trait 可作 `dyn ScreenCapture` 使用
- [ ] mock_impl: 提供 MockCapture/MockInput 等测试桩
- [ ] cross_compile: `cargo check --target x86_64-pc-windows-msvc` 通过
### Task 3: Crypto Layer (~4d)

**目标**: X25519 密钥交换 + XSalsa20-Poly1305 AEAD + 会话密钥派生。中继零知识——无法解密内容。密钥销毁时 zeroize。
**依赖**: Task 1
**文件**: `crates/rdcs-crypto/src/{lib,key_exchange,aead,session,error}.rs`
**接口契约**:
```rust
pub fn generate_keypair() -> KeyPair;
pub fn derive_shared_secret(our_secret: &SecretKey, their_public: &PublicKey) -> SharedSecret;
pub fn encrypt(key: &SessionKey, nonce: &[u8; 24], plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
pub fn decrypt(key: &SessionKey, nonce: &[u8; 24], ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>>;
pub fn derive_session_keys(shared: &SharedSecret, salt: &[u8]) -> (SessionKey, SessionKey);
pub fn rotate_nonce(counter: &mut u64) -> [u8; 24];

pub struct CryptoSession { session_id: u64, local_keypair: KeyPair, shared_key: Option<SessionKey> }
impl CryptoSession {
    pub fn new(session_id: u64) -> Self;
    pub fn complete_handshake(&mut self, remote_public: &PublicKey, salt: &[u8]) -> Result<()>;
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedPayload>;
    pub fn decrypt(&self, payload: &EncryptedPayload) -> Result<Vec<u8>>;
    pub fn destroy(self); // zeroize all key material
}
```
**验收标准**:
- [ ] roundtrip: 1000 次随机数据加密→解密，明文一致
- [ ] wrong_key_reject: 错误密钥返回 `DecryptionFailed`
- [ ] forward_secrecy: `destroy()` 后内存清零（`zeroize` 验证）
- [ ] nonce_unique: `rotate_nonce` 2^64 次不重复
### Task 4: Transport Layer (~5d)

**目标**: UDP 之上构建可靠传输——包格式、序号管理、GCC 拥塞控制、NACK 重传、FEC 前向纠错。
**依赖**: Task 1
**文件**: `crates/rdcs-transport/src/{lib,packet,sequencer,congestion,nack,fec,channel}.rs`
**接口契约**:
```rust
pub struct PacketHeader { version: u8, packet_type: PacketType, session_id: u64, sequence: u32, timestamp: u32, payload_len: u16 }
pub enum PacketType { Data, Ack, Nack, Fec, Heartbeat, Control }
pub fn encode_packet(header: &PacketHeader, payload: &[u8]) -> Vec<u8>;
pub fn decode_packet(raw: &[u8]) -> Result<(PacketHeader, &[u8])>;
pub trait Sequencer {
    fn next_sequence(&mut self) -> u32;
    fn is_duplicate(&self, seq: u32) -> bool;
    fn gap_report(&self) -> Vec<Range<u32>>;
}
pub trait CongestionController: Send {
    fn on_packet_sent(&mut self, bytes: usize, now_ms: u64);
    fn on_ack_received(&mut self, ack: &AckInfo, now_ms: u64);
    fn on_loss_detected(&mut self, seq: u32);
    fn target_bitrate(&self) -> u64;
}
pub trait NackManager: Send {
    fn on_packet_received(&mut self, seq: u32);
    fn pending_retransmits(&self) -> Vec<u32>;
    fn on_nack_response(&mut self, seq: u32);
}
pub trait FecEncoder: Send {
    fn encode_group(&mut self, data_packets: &[&[u8]]) -> Vec<Vec<u8>>;
    fn adjust_redundancy(&mut self, loss_rate: f32);
}
pub trait FecDecoder: Send {
    fn decode_group(&mut self, packets: &[FecPacket]) -> Result<Vec<Vec<u8>>>;
}
pub struct TransportChannel { /* sequencer + congestion + nack + fec */ }
impl TransportChannel {
    pub async fn send(&self, data: &[u8]) -> Result<()>;
    pub async fn recv(&self) -> Result<Vec<u8>>;
    pub fn stats(&self) -> TransportStats;
}
```
**验收标准**:
- [ ] packet_roundtrip: encode + decode 数据无损
- [ ] sequencer_gap: 乱序 [1,2,4,5,7] → gap_report 返回 [3..4, 6..7]
- [ ] gcc_adapt: 10% 丢包 → target_bitrate 5 轮内降 ≥30%
- [ ] fec_recovery: 8+2 丢 2 个数据包 → decode_group 完整恢复
- [ ] nack_retry: 单包最多重传 3 次后放弃
### Task 5: Codec Pipeline (~5d)

**目标**: 视频编解码管线：内容分析（文字/视频场景检测）→ 自适应编码参数切换。文字场景 5fps 高清，视频 30-120fps 流畅。
**依赖**: Task 2, Task 3
**文件**: `crates/rdcs-codec/src/{lib,analyzer,encoder,decoder,adaptive,pipeline}.rs`
**接口契约**:
```rust
pub trait ContentAnalyzer: Send { fn analyze(&mut self, frame: &CapturedFrame) -> SceneInfo; }
// SceneInfo { scene_type, motion_level, text_region_ratio, suggested_fps, suggested_quality }
// SceneType: StaticText | MixedContent | Video | FullMotion
pub trait VideoEncoder: Send {
    fn configure(&mut self, config: EncoderConfig) -> Result<()>;
    fn encode(&mut self, frame: &CapturedFrame) -> Result<EncodedFrame>;
    fn flush(&mut self) -> Result<Vec<EncodedFrame>>;
}
// EncoderConfig { codec: CodecType(H264|H265|Vp9|Av1), width, height, target_fps, target_bitrate_bps, keyframe_interval, hardware_accel }
pub trait VideoDecoder: Send {
    fn configure(&mut self, config: DecoderConfig) -> Result<()>;
    fn decode(&mut self, frame: &EncodedFrame) -> Result<DecodedFrame>;
    fn reset(&mut self) -> Result<()>;
}
pub trait AdaptiveController: Send {
    fn on_scene_change(&mut self, info: &SceneInfo);
    fn on_bandwidth_update(&mut self, bitrate_bps: u64);
    fn on_latency_update(&mut self, rtt_ms: u32);
    fn current_config(&self) -> EncoderConfig;
}
pub struct EncodePipeline { analyzer: Box<dyn ContentAnalyzer>, encoder: Box<dyn VideoEncoder>, adaptive: Box<dyn AdaptiveController> }
pub struct DecodePipeline { decoder: Box<dyn VideoDecoder> }
impl EncodePipeline {
    pub async fn process_frame(&mut self, frame: CapturedFrame) -> Result<EncodedFrame>;
    pub fn update_bitrate(&mut self, bitrate_bps: u64);
}
impl DecodePipeline { pub async fn process_frame(&mut self, frame: &EncodedFrame) -> Result<DecodedFrame>; }
```
**验收标准**:
- [ ] scene_text: 纯文字截图 → `StaticText`, `suggested_fps <= 10`
- [ ] scene_video: 高运动画面 → `Video`, `suggested_fps >= 30`
- [ ] adaptive_downgrade: 带宽降至 500kbps → 自动降 720P
- [ ] adaptive_recover: 带宽恢复 → <=3 步回到 1080P
- [ ] pipeline_e2e: encode → decode 后 PSNR > 40dB
### Task 6: Connection Manager (~5d)

**目标**: 三层连接（L1 mDNS 局域网 → L2 ICE 打洞 → L3 中继），路径自动选择、心跳、断线重连。用户仅感知"已连接"。
**依赖**: Task 1, Task 4
**文件**: `crates/rdcs-connection/src/{lib,mdns,ice,path,heartbeat,reconnect}.rs`
**接口契约**:
```rust
pub trait MdnsDiscovery: Send + Sync {
    fn register(&self, service: MdnsService) -> Result<()>;
    fn browse(&self) -> Result<mpsc::Receiver<MdnsEvent>>;
}
pub enum MdnsEvent { Found(MdnsService), Lost(String) }
pub trait IceAgent: Send {
    fn gather_candidates(&mut self) -> Result<Vec<IceCandidate>>;
    fn set_remote_candidates(&mut self, candidates: Vec<IceCandidate>) -> Result<()>;
    fn create_offer(&self) -> Result<SdpOffer>;
    fn handle_answer(&mut self, answer: SdpAnswer) -> Result<()>;
    fn connection_state(&self) -> IceState;
}
pub enum CandidateType { Host, Srflx, Prflx, Relay }
pub enum IceState { New, Checking, Connected, Failed, Closed }
pub trait PathSelector: Send {
    fn select_path(&mut self, candidates: &[PathCandidate]) -> Result<ConnectionPath>;
    fn on_path_failed(&mut self, path: ConnectionPath) -> Option<ConnectionPath>;
}
pub enum ConnectionPath { L1Direct(SocketAddr), L2Punch(SocketAddr), L3Relay(SocketAddr) }
pub enum ConnectionMode { Auto, LanOnly, ForceRelay }
pub trait HeartbeatManager: Send { fn start(&self, peer: SocketAddr) -> Result<()>; fn is_alive(&self) -> bool; fn last_rtt(&self) -> Option<u32>; }
pub trait ReconnectStrategy: Send { fn next_delay(&mut self) -> Duration; fn reset(&mut self); fn attempts_remaining(&self) -> u32; }
pub struct ConnectionManager { /* mdns + ice + path + heartbeat + reconnect */ }
impl ConnectionManager {
    pub async fn connect(&mut self, target_code: &str) -> Result<SessionHandle>;
    pub async fn accept(&mut self, request: ConnectionRequest) -> Result<SessionHandle>;
    pub async fn disconnect(&mut self, session_id: u64) -> Result<()>;
    pub async fn events(&self) -> mpsc::Receiver<ConnectionEvent>;
}
pub enum ConnectionEvent { NearbyDeviceFound{..}, NearbyDeviceLost{..}, ConnectionEstablished{..}, ConnectionLost{..}, ConnectionRestored{..} }
```
**验收标准**:
- [ ] path_priority: L1 可用选 L1 → L1 不可用降 L2 → L2 失败降 L3
- [ ] heartbeat_detect: 30s 无心跳 → `is_alive() == false`
- [ ] reconnect_backoff: 序列 [1s, 2s, 4s, 8s, 16s, 30s, 30s, ...]
- [ ] transparent_failover: L2→L3 切换上层无感知
- [ ] mdns_discover: 同网两设备 2s 内互相发现
### Task 7: File Transfer & Clipboard Sync (~4d)

**目标**: 64KB 分块文件传输 + 断点续传 + SHA256 校验 + 双向剪贴板同步。中断记录断点，重连后续传。
**依赖**: Task 2, Task 4
**文件**: `crates/rdcs-transfer/src/{lib,file_sender,file_receiver,clipboard_sync,checksum}.rs`
**接口契约**:
```rust
pub trait FileSender: Send {
    fn start_transfer(&mut self, request: TransferRequest) -> Result<TransferHandle>;
    fn pause(&mut self, handle: &TransferHandle) -> Result<()>;
    fn resume(&mut self, handle: &TransferHandle) -> Result<()>;
    fn cancel(&mut self, handle: &TransferHandle) -> Result<()>;
    fn progress(&self, handle: &TransferHandle) -> TransferProgress;
}
pub trait FileReceiver: Send {
    fn accept(&mut self, offer: FileOffer, dest_dir: &Path) -> Result<TransferHandle>;
    fn reject(&mut self, offer: &FileOffer) -> Result<()>;
    fn verify(&self, handle: &TransferHandle) -> Result<bool>;
}
// TransferState: Pending | InProgress | Paused{offset} | Completed | Failed
// FileOffer { session_id, file_name, file_size, checksum: [u8;32], modified }
pub fn compute_sha256(path: &Path) -> Result<[u8; 32]>;
pub fn compute_chunk_hash(data: &[u8]) -> [u8; 32];
pub trait ClipboardSync: Send {
    fn start(&self, provider: Box<dyn ClipboardProvider>) -> Result<()>;
    fn stop(&self) -> Result<()>;
    fn local_change(&self) -> mpsc::Receiver<ClipboardEvent>;
    fn apply_remote(&self, event: ClipboardEvent) -> Result<()>;
    fn set_filter(&self, filter: ClipboardFilter);
}
pub enum ClipboardFilter { None, TextOnly, MaxSize(usize) }
pub struct TransferManager { /* sender + receiver + clipboard */ }
impl TransferManager {
    pub async fn send_file(&mut self, request: TransferRequest) -> Result<TransferHandle>;
    pub async fn on_file_offer(&mut self, offer: FileOffer, auto_accept: bool) -> Result<()>;
    pub async fn events(&self) -> mpsc::Receiver<TransferEvent>;
}
```
**验收标准**:
- [ ] chunk_integrity: 1GB 文件分块传输后 SHA256 一致
- [ ] resume_disconnect: 50% 断连 → 重连后续传（非重头）
- [ ] cancel_cleanup: 取消后临时文件清理
- [ ] clipboard_sync: 一端复制 → 另一端 200ms 内收到
- [ ] clipboard_filter: TextOnly 模式下图片不触发同步
### Task 8: FFI Bridge (~3d)

**目标**: 12 个 C ABI 导出函数，Flutter Dart 与 Rust Core 的唯一桥接。同步返回 + 回调异步通知。EngineHandle 生命周期管理。
**依赖**: Task 1 (接口定义)
**文件**: `crates/rdcs-ffi/src/{lib,handle,callback,convert}.rs`
**接口契约**:
```rust
pub struct EngineHandle { runtime: Runtime, engine: Arc<CoreEngine>, callbacks: CallbackRegistry }
pub type EventCallback = extern "C" fn(event_id: u32, payload: *const c_char, payload_len: usize);

// 12 个 #[no_mangle] extern "C" 函数 (架构文档 2.2 节对齐):
pub extern "C" fn rdcs_engine_create(config_json: *const c_char) -> *mut EngineHandle;
pub extern "C" fn rdcs_engine_destroy(handle: *mut EngineHandle);
pub extern "C" fn rdcs_start_capture(handle: *mut EngineHandle, config_json: *const c_char) -> i32;
pub extern "C" fn rdcs_stop_capture(handle: *mut EngineHandle) -> i32;
pub extern "C" fn rdcs_connect(handle: *mut EngineHandle, target_code: *const c_char) -> i32;
pub extern "C" fn rdcs_disconnect(handle: *mut EngineHandle, session_id: u64) -> i32;
pub extern "C" fn rdcs_send_input(handle: *mut EngineHandle, session_id: u64, event_json: *const c_char) -> i32;
pub extern "C" fn rdcs_send_file(handle: *mut EngineHandle, session_id: u64, path: *const c_char, dest: *const c_char) -> i32;
pub extern "C" fn rdcs_send_message(handle: *mut EngineHandle, session_id: u64, text: *const c_char) -> i32;
pub extern "C" fn rdcs_set_quality(handle: *mut EngineHandle, session_id: u64, mode: i32) -> i32;
pub extern "C" fn rdcs_generate_invite(handle: *mut EngineHandle) -> *const c_char;
pub extern "C" fn rdcs_register_callback(handle: *mut EngineHandle, event_id: u32, cb: EventCallback) -> i32;
pub fn cstr_to_string(ptr: *const c_char) -> Result<String>;
pub fn free_cstr(ptr: *mut c_char);
```
**验收标准**:
- [ ] null_safety: null 指针返回 -1，不 panic
- [ ] no_leak: create → destroy 无内存泄漏（Valgrind）
- [ ] callback_dispatch: 注册后触发，回调被正确调用
- [ ] thread_safety: 多线程并发调用无 race
- [ ] c_header: `cbindgen` 生成 `rdcs_core.h`
### Task 9: macOS Platform (rdcs-macos) (~7d)

**目标**: macOS 5 个 trait 实现：ScreenCaptureKit 捕获、CGEvent 输入、CoreAudio 音频、NSStatusBar 托盘、NSPasteboard 剪贴板。含权限检测。MVP 唯一平台。
**依赖**: Task 2
**文件**: `crates/rdcs-macos/src/{lib,capture,input,audio,notify,clipboard,permissions}.rs`
**接口契约**:
```rust
pub struct MacScreenCapture { stream: Option<SCStream>, .. }
impl ScreenCapture for MacScreenCapture { /* start/stop/is_capturing/displays */ }
pub struct MacInputInjector;
impl InputInjector for MacInputInjector { /* inject_mouse/inject_key/inject_scroll */ }
pub struct MacAudioCapture { audio_unit: Option<AudioUnit>, .. }
impl AudioCapture for MacAudioCapture { /* start/stop/devices */ }
pub struct MacSystemNotify { status_item: Option<NSStatusItem> }
impl SystemNotify for MacSystemNotify { /* show_notification/set_tray_status/play_sound */ }
pub struct MacClipboard { change_count: i64 }
impl ClipboardProvider for MacClipboard { /* get_text/set_text/watch */ }
pub fn check_screen_recording_permission() -> bool;
pub fn request_screen_recording_permission() -> Result<bool>;
pub fn check_accessibility_permission() -> bool;
pub struct MacPlatform;
impl MacPlatform {
    pub fn create_capture() -> Box<dyn ScreenCapture>;
    pub fn create_input() -> Box<dyn InputInjector>;
    pub fn create_audio() -> Box<dyn AudioCapture>;
    pub fn create_notify() -> Box<dyn SystemNotify>;
    pub fn create_clipboard() -> Box<dyn ClipboardProvider>;
}
```
**验收标准**:
- [ ] capture_permission: 无权限 → 明确错误 + 引导授权
- [ ] capture_1080p: 1080P ≥60fps, CPU <15%
- [ ] input_accuracy: 100 次点击坐标偏差 <1px
- [ ] clipboard_watch: 外部复制 100ms 内检测
- [ ] tray_update: 状态更新后菜单栏图标变化
### Task 10: Integration Tests (~4d)

**目标**: 端到端集成测试：完整管线、加密往返、FEC 恢复、路径降级、文件传输、FFI 生命周期。各模块协作验证。
**依赖**: Task 3, 4, 5, 6, 7, 8
**文件**: `tests/integration/{pipeline,crypto,transport,connection,transfer,ffi}.rs`
**接口契约**:
```rust
pub struct PipelineTestFixture { encoder: EncodePipeline, decoder: DecodePipeline, crypto: CryptoSession }
impl PipelineTestFixture {
    pub async fn roundtrip_frame(&mut self, frame: CapturedFrame) -> Result<DecodedFrame>;
    pub fn pipeline_latency_ms(&self) -> u32;
}
pub fn test_handshake_and_encrypt() -> Result<()>;
pub fn test_wrong_key_rejected() -> Result<()>;
pub fn test_concurrent_sessions() -> Result<()>;
pub fn test_fec_recovery_under_loss(loss_rate: f32, fec_ratio: f32) -> Result<f32>;
pub fn test_gcc_bandwidth_convergence() -> Result<()>;
pub fn test_packet_reorder_recovery() -> Result<()>;
pub fn test_l1_preferred_over_l2() -> Result<()>;
pub fn test_l2_fallback_to_l3() -> Result<()>;
pub fn test_reconnect_after_disconnect() -> Result<()>;
pub fn test_large_file_transfer(size_mb: u64) -> Result<()>;
pub fn test_resume_after_interruption(interrupt_at_pct: f32) -> Result<()>;
pub fn test_engine_lifecycle() -> Result<()>;
pub fn test_null_safety() -> Result<()>;
```
**验收标准**:
- [ ] e2e_pipeline: encode→encrypt→transport→decrypt→decode 延迟 <50ms
- [ ] crypto_roundtrip: 1000 组随机数据零失败
- [ ] fec_recovery_rate: 20% 丢包 → 恢复率 >95%
- [ ] path_order: L1 可用 100% 选 L1, L1 故障 <500ms 切 L2
- [ ] file_1gb: 1GB 传输 SHA256 一致
- [ ] ffi_no_leak: 100 次 create/destroy 无泄漏

---

## 依赖关系

```
T1 ─┬─ T2 ─┬─ T5(◄T3) ─┐
    │      ├─ T7 ───────┤
    │      └─ T9 ───────┤
    ├─ T3 ──────────────┤
    ├─ T4 ─┬─ T5 ───────┤
    │      └─ T6 ───────┤
    ├─ T8 ──────────────┤
    └─ T10 ◄── 全部 ────┘
```

## 时间线 (8 周)

| 周 | 任务 | 并行 |
|----|------|------|
| W1 | T1 + T2 + T3 | T1 后 T2/T3 并行 |
| W2 | T4 + T7 | T7 依赖 T2 |
| W3 | T5 + T6 | T5 需 T2+T3, T6 需 T4 |
| W4 | T8 + T9(前半) | 捕获+输入 |
| W5 | T9(后半) | 音频+通知+剪贴板 |
| W6-7 | T10 | 集成测试 |
| W8 | 修复+调优 | 根据结果迭代 |

## 风险与缓解

- **ScreenCaptureKit API 变更**: 锁定 macOS 14.0+, CI 覆盖 14/15
- **GCC 参数难调**: 先用 AIMD, 后替换完整 GCC
- **FEC 不自适应**: 初始固定 20%, 后接 AdaptiveController
- **FFI 泄漏**: Miri + Valgrind CI 步骤
