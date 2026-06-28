# Plan C: 中转节点 (rdcs-relay) 实现计划

**组件**: rdcs-relay
**语言**: Rust (tokio 异步运行时)
**定位**: 零知识 UDP 中继，纯转发加密数据包，不持有解密密钥
**预估工期**: ~20 天
**架构参考**: `docs/specs/architecture-design.md` Section 4

## 跨项目依赖

依赖: Plan B (信令回调接口), Plan F (网络环境)

---

## 依赖关系总览

```
Task 1 (脚手架)
  └─► Task 2 (UDP Listener)
        └─► Task 3 (Token Auth)
              └─► Task 4 (Slot Allocation)
                    ├─► Task 5 (零拷贝转发)
                    ├─► Task 6 (心跳超时)
                    └─► Task 7 (会话释放)
                          └─► Task 8 (Metrics)
                                └─► Task 9 (Health)
                                      └─► Task 10 (压测)
```

---

### Task 1: 项目脚手架与 CLI (~1d)

**目标**: 创建 rdcs-relay Cargo workspace，集成 clap CLI 参数解析和多阶段 Dockerfile。
提供 `--listen`、`--port-range`、`--hmac-secret`、`--metrics-port` 等运行时配置入口。

**依赖**: 无
**文件**: `relay/Cargo.toml`, `relay/src/main.rs`, `relay/Dockerfile`

**接口契约**:
- `pub struct RelayConfig { listen_addr, port_range, hmac_secret, metrics_port, health_port }`
- `fn parse_cli() -> RelayConfig`
- Dockerfile: multi-stage (rust:1.80-slim build → debian:bookworm-slim run)

**验收标准**:
- [ ] `cargo build --release` 编译通过，二进制 < 5MB
- [ ] `rdcs-relay --help` 输出所有 CLI 参数及说明
- [ ] `docker build` 生成可运行镜像，`docker run` 启动后监听指定端口

---

### Task 2: UDP 监听器与协议解析 (~2d)

**目标**: 绑定 UDP 3478 端口，接收数据包并按消息类型分发。
支持三种控制消息（ALLOCATE / RELEASE / KEEPALIVE）和纯数据转发包。

**依赖**: Task 1
**文件**: `relay/src/listener.rs`, `relay/src/protocol.rs`

**接口契约**:
- `pub enum RelayMessage { Allocate { session_id, token }, Release { session_id }, Keepalive { session_id }, Data }`
- `pub struct UdpListener { socket: Arc<UdpSocket> }`
- `impl UdpListener { pub async fn bind(addr: SocketAddr) -> Result<Self>; pub async fn recv_loop(&self, dispatcher: Dispatcher) -> Result<()>; }`
- `fn parse_message(buf: &[u8]) -> Result<RelayMessage>`

**验收标准**:
- [ ] `test_parse_allocate`: 正确解析 ALLOCATE 消息头及 session_id/token 字段
- [ ] `test_parse_keepalive`: 正确解析 KEEPALIVE 消息，忽略无效长度包
- [ ] `test_bind_and_recv`: 绑定端口后能接收 UDP 包并分发到对应处理逻辑

---

### Task 3: Token 认证 (~2d)

**目标**: 实现基于 HMAC-SHA256 的一次性 Token 验证机制。
Token 由信令服务签发，30 秒有效，携带 nonce 防重放攻击。
HMAC 计算覆盖 session_id + relay_addr + nonce + expires_at，确保 Token 与指定 relay 地址绑定。

**依赖**: Task 2
**文件**: `relay/src/auth.rs`

**接口契约**:
- `pub struct TokenPayload { session_id: Uuid, relay_addr: String, nonce: u64, expires_at: u64 }`
- `pub struct TokenVerifier { secret: [u8; 32], used_nonces: Arc<DashSet<u64>> }`
- `impl TokenVerifier { pub fn new(secret: [u8; 32]) -> Self; pub fn verify(&self, token: &[u8], payload: &TokenPayload) -> Result<()>; fn cleanup_expired_nonces(&self); }`
- `pub enum AuthError { InvalidSignature, TokenExpired, NonceReused }`

**验收标准**:
- [ ] `test_valid_token`: 合法 HMAC + 未过期 + 新 nonce → 验证通过
- [ ] `test_expired_token`: 合法 HMAC 但超过 30s → 返回 TokenExpired
- [ ] `test_replay_nonce`: 相同 nonce 二次提交 → 返回 NonceReused
- [ ] `test_invalid_signature`: 篡改 token 载荷 → 返回 InvalidSignature

---

### Task 4: Slot 分配与会话跟踪 (~2d)

**目标**: 为已认证的 ALLOCATE 请求分配 UDP 端口对（控制端/被控端各一个），
从 49152-65535 范围选取可用端口，维护会话状态映射表。

**依赖**: Task 3
**文件**: `relay/src/slot.rs`, `relay/src/session.rs`

**接口契约**:
- `pub struct SlotPair { controller_port: u16, controlled_port: u16 }`
- `pub struct RelaySlot { session_id: Uuid, slots: SlotPair, controller_addr: Option<SocketAddr>, controlled_addr: Option<SocketAddr>, created_at: Instant, last_keepalive: Instant }`
- `pub struct SlotManager { port_pool: PortPool, sessions: DashMap<Uuid, RelaySlot> }`
- `impl SlotManager { pub fn allocate(session_id, controller_addr, controlled_addr) -> Result<SlotPair>; pub fn release(session_id) -> Result<()>; pub fn get_session(session_id) -> Option<RelaySlot>; pub fn available_ports() -> usize; }`
- `pub struct PortPool { range: RangeInclusive<u16>, used: DashSet<u16> }`

**验收标准**:
- [ ] `test_allocate_slot`: 分配成功，返回 49152-65535 范围内的端口对
- [ ] `test_port_exhaustion`: 端口耗尽后返回 PoolExhausted 错误
- [ ] `test_release_reclaim`: 释放后端口回到可用池，可被重新分配
- [ ] `test_concurrent_allocate`: 100 并发分配不出现端口冲突

---

### Task 5: 零拷贝数据转发 (~3d)

**目标**: 实现基于 recvmsg/sendmsg 的零拷贝 UDP 转发核心。
中继仅替换包头源地址，不解析、不解密载荷内容，实现零知识转发。

**依赖**: Task 4
**文件**: `relay/src/forwarder.rs`

**接口契约**:
- `pub struct Forwarder { socket: Arc<UdpSocket>, sessions: Arc<DashMap<Uuid, RelaySlot>> }`
- `impl Forwarder { pub async fn run(&self) -> Result<()>; async fn forward_packet(&self, src_addr: SocketAddr, buf: &[u8], session: &RelaySlot) -> Result<usize>; }`
- `fn resolve_peer(session: &RelaySlot, src_addr: SocketAddr) -> Option<SocketAddr>`

**验收标准**:
- [ ] `test_forward_bidirectional`: A→Relay→B 和 B→Relay→A 双向转发正确
- [ ] `test_unknown_session`: 未知 session_id 的数据包被静默丢弃
- [ ] `test_zero_knowledge`: 转发后数据内容与原始载荷完全一致（不解密不修改）
- [ ] `bench_forward_throughput`: 单 session 转发吞吐 >= 50Mbps（本地回环测试）

---

### Task 6: 心跳保活与超时回收 (~2d)

**目标**: 客户端每 15 秒发送 KEEPALIVE，中继刷新 last_keepalive 时间戳。
30 秒内未收到任一方的 KEEPALIVE 则判定超时，触发 Slot 回收。

**依赖**: Task 4
**文件**: `relay/src/keepalive.rs`

**接口契约**:
- `pub struct KeepaliveMonitor { sessions: Arc<DashMap<Uuid, RelaySlot>>, interval: Duration, timeout: Duration }`
- `impl KeepaliveMonitor { pub fn new(sessions, interval: Duration(15s), timeout: Duration(30s)) -> Self; pub async fn run(&self, on_timeout: impl Fn(Uuid) + Send + Sync) -> !; fn refresh(session_id, peer_addr) -> Result<()>; fn scan_expired(&self) -> Vec<Uuid>; }`

**验收标准**:
- [ ] `test_keepalive_refresh`: 收到 KEEPALIVE 后 last_keepalive 被更新
- [ ] `test_timeout_detection`: 30s 无心跳的 session 被 scan_expired 返回
- [ ] `test_partial_keepalive`: 仅一方发送心跳，session 仍保持活跃
- [ ] `test_monitor_loop`: 超时 session 自动触发 on_timeout 回调

---

### Task 7: 会话释放与通知 (~2d)

**目标**: 处理显式 RELEASE 消息和隐式超时两种释放路径。
回收 Slot 端口对，通过 HTTP 回调通知信令服务记录会话结束。

**依赖**: Task 4, Task 6
**文件**: `relay/src/release.rs`

**接口契约**:
- `pub struct SessionReleaser { slot_manager: Arc<SlotManager>, signaling_callback_url: Option<Url> }`
- `impl SessionReleaser { pub async fn release_explicit(&self, session_id: Uuid, reason: ReleaseReason) -> Result<()>; pub async fn release_timeout(&self, session_id: Uuid) -> Result<()>; async fn notify_signaling(&self, session_id: Uuid, reason: ReleaseReason) -> Result<()>; }`
- `pub enum ReleaseReason { ClientRelease, Timeout, ServerShutdown }`
- `pub struct ReleaseEvent { session_id: Uuid, reason: ReleaseReason, duration_sec: u64, bytes_forwarded: u64 }`

**验收标准**:
- [ ] `test_explicit_release`: RELEASE 消息触发端口回收，session 从映射表移除
- [ ] `test_timeout_release`: 超时触发自动释放，端口回到可用池
- [ ] `test_double_release`: 重复释放同一 session 不报错（幂等）
- [ ] `test_signaling_callback`: 释放后向信令服务发送 HTTP POST，携带 ReleaseEvent

---

### Task 8: Prometheus 指标采集 (~2d)

**目标**: 暴露 Prometheus 格式的运营指标，覆盖活跃会话数、转发字节数、
端口池使用率等核心数据，支持 Grafana 看板接入。

**依赖**: Task 5, Task 7
**文件**: `relay/src/metrics.rs`

**接口契约**:
- `pub struct RelayMetrics { active_sessions: IntGauge, total_sessions: IntCounter, bytes_forwarded: IntCounter, packets_forwarded: IntCounter, slots_available: IntGauge, slots_total: IntGauge, auth_failures: IntCounter, token_expired: IntCounter }`
- `impl RelayMetrics { pub fn register(registry: &Registry) -> Result<Self>; pub fn on_session_created(&self); pub fn on_session_released(&self); pub fn on_packet_forwarded(&self, bytes: u64); pub fn on_auth_failure(&self); pub fn snapshot(&self) -> MetricsSnapshot; }`
- `pub struct MetricsSnapshot { active, total, bytes, packets, slots_used, slots_free }`

**验收标准**:
- [ ] `test_metrics_register`: 所有指标成功注册到 Prometheus Registry
- [ ] `test_session_counters`: 创建/释放会话后 active_sessions 和 total_sessions 值正确
- [ ] `test_bytes_counter`: 转发 N 字节后 bytes_forwarded 累加准确
- [ ] `test_prometheus_scrape`: HTTP GET /metrics 返回合法 Prometheus 文本格式

---

### Task 9: Health 健康检查端点 (~1d)

**目标**: 在独立 HTTP 端口提供健康检查接口，供负载均衡和容器编排探活。
返回 JSON 格式的运行状态，包括版本、运行时长、负载概况。

**依赖**: Task 8
**文件**: `relay/src/health.rs`

**接口契约**:
- `pub struct HealthServer { port: u16, metrics: Arc<RelayMetrics>, started_at: Instant }`
- `impl HealthServer { pub async fn run(&self) -> Result<()>; }`
- `pub struct HealthResponse { status: HealthStatus, version: &'static str, uptime_sec: u64, active_sessions: usize, slots_available: usize, slots_total: usize }`
- `pub enum HealthStatus { Healthy, Degraded, Unhealthy }`

**验收标准**:
- [ ] `test_health_healthy`: 正常状态下 GET /health 返回 200 + status=healthy
- [ ] `test_health_degraded`: 端口池使用率 > 90% 时返回 status=degraded
- [ ] `test_health_includes_uptime`: 响应中 uptime_sec 随时间递增
- [ ] `test_health_separate_port`: Health 端口与 UDP 转发端口互不干扰

---

### Task 10: 并发压测与性能验证 (~3d)

**目标**: 编写集成压测工具，模拟 200 并发会话的完整生命周期
（ALLOCATE → KEEPALIVE → Data 转发 → RELEASE），验证吞吐量、延迟和资源回收。

**依赖**: Task 5, Task 6, Task 7, Task 9
**文件**: `relay/tests/load_test.rs`, `relay/benches/forward_bench.rs`

**接口契约**:
- `pub struct LoadTestConfig { concurrent_sessions: u32, duration_sec: u64, packet_size: usize, packet_interval_ms: u64 }`
- `pub struct LoadTestResult { sessions_established: u32, sessions_completed: u32, total_bytes: u64, avg_latency_us: u64, p99_latency_us: u64, errors: Vec<TestError> }`
- `fn run_load_test(config: LoadTestConfig, relay_addr: SocketAddr) -> LoadTestResult`

**验收标准**:
- [ ] `test_200_concurrent_sessions`: 200 session 全部成功建立并完成数据交换
- [ ] `test_slot_reclaim_under_load`: 压测结束后所有端口被正确回收，无泄漏
- [ ] `test_keepalive_during_load`: 高负载下 KEEPALIVE 响应延迟 < 100ms
- [ ] `bench_forward_latency`: 单包转发延迟 P99 < 1ms（本地回环）
- [ ] `test_memory_bounded`: 200 session 下内存占用 < 200MB

---

## 技术约束速查

| 项目 | 约束 |
|------|------|
| 端口范围 | 控制端口 3478 (UDP)，Slot 端口 49152-65535 |
| Token 算法 | HMAC-SHA256，30s 过期，nonce 防重放 |
| 心跳间隔 | 客户端 15s，超时阈值 30s |
| 转发模式 | 零拷贝 recvmsg/sendmsg，仅替换源地址 |
| 加密 | 零知识 — 中继不持有密钥，不解析载荷 |
| 最大并发 | 单节点 ~200 session（4C8G / 100Mbps） |
| 指标格式 | Prometheus text exposition |
| 部署 | Docker multi-stage，本地 `docker-compose.dev.yml` |
