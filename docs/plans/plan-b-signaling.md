# Plan B: 信令服务 (rdcs-signaling) 精益实现计划

**基于**: `docs/specs/architecture-design.md` Section 3 | **技术栈**: Rust + tokio + axum + redis-rs | **预估**: ~22d
**原则**: 仅定义接口契约，不写实现代码；每个 Task 可独立验证

## 跨项目依赖

依赖: Plan F (Redis/PostgreSQL 基础设施)

---

## 项目结构

```
rdcs-signaling/src/
├── main.rs, config.rs, error.rs       # 入口、配置、错误类型
├── redis/{mod,keys,ttl}.rs            # 连接池、Key 模式、TTL 辅助
├── ws/{mod,message,handler,heartbeat}.rs  # WebSocket 升级、消息协议、分发、心跳
├── device/{mod,registry}.rs           # 设备注册、内存注册表
├── session/{mod,relay,relay_callback}.rs  # 连接协商、中转分配、中转回调
├── invite/{mod,lockout}.rs            # 邀请码、暴力破解防护
├── scale/{mod,router}.rs              # Pub/Sub 桥接、粘性路由
├── offline/mod.rs                     # Keyspace 离线检测
tests/integration/{two_client,heartbeat,invite}.rs
```

---

## Task 1: 项目脚手架 (~1d)

**目标**: 搭建 Cargo workspace，配置 axum HTTP 服务，提供 `/health` 端点。编写 Dockerfile 多阶段构建，`docker compose up` 一行启动。
**依赖**: 无
**文件**: `Cargo.toml`, `src/main.rs`, `src/config.rs`, `src/error.rs`, `Dockerfile`

**接口契约**:
```rust
// 核心依赖: axum 0.8, tokio 1 (full), redis 0.27 (tokio-comp), serde+serde_json, tracing, uuid v4, hmac+sha2
pub struct AppConfig { pub bind_addr: String, pub redis_url: String, pub relay_hmac_secret: String, pub log_level: String }
impl AppConfig { pub fn from_env() -> Result<Self, ConfigError> }
pub enum AppError { Redis(RedisError), Json(JsonError), Ws(AxumError), NotFound(String), RateLimited { retry_after_secs: u64 } }
impl IntoResponse for AppError { fn into_response(self) -> Response }
async fn health_handler() -> impl IntoResponse;  // GET /health -> {"status":"ok"}
async fn main() -> Result<()>;  // Router: /health + /ws, axum::serve
```

**验收标准**:
- [ ] `cargo build` 零 warning | `GET /health` 返回 200 | Docker 镜像 < 30MB

---

## Task 2: Redis 连接池与 Key 模式 (~1d)

**目标**: 封装 Redis ConnectionManager，定义 Key 模式常量（与架构文档 3.3 一致）和 TTL 辅助函数。后续所有 Redis 操作均经此模块。
**依赖**: Task 1
**文件**: `src/redis/{mod,keys,ttl}.rs`

**接口契约**:
```rust
pub type RedisPool = redis::aio::ConnectionManager;
pub async fn create_pool(url: &str) -> Result<RedisPool, RedisError>;

// keys.rs — 生成函数
pub fn device_online_key(code: &str) -> String;    // device:{code}:online
pub fn team_online_key(team_id: &str) -> String;    // team:{team_id}:online_devices
pub fn session_key(id: &str) -> String;              // session:{session_id}
pub fn invite_key(code: &str) -> String;             // device_invite:{code}
pub fn team_events_channel(team_id: &str) -> String; // team:{team_id}:events
pub fn device_signals_channel(code: &str) -> String; // device:{code}:signals
pub fn lockout_key(kind: &str, id: &str) -> String;  // lockout:{kind}:{id}

// ttl.rs — 常量: DEVICE_ONLINE=60s, INVITE=600s, RELAY_TOKEN=30s, LOCKOUT=1800s
pub async fn set_with_ttl(pool: &mut RedisPool, key: &str, value: &str, ttl: u64) -> Result<()>;
pub async fn refresh_ttl(pool: &mut RedisPool, key: &str, ttl: u64) -> Result<()>;
pub async fn del_key(pool: &mut RedisPool, key: &str) -> Result<()>;
```

**验收标准**:
- [ ] `set_with_ttl` 写入后自动过期 | `refresh_ttl` 延长 TTL | key 格式与架构文档一致

---

## Task 3: WebSocket 处理器 (~2d)

**目标**: 实现 WebSocket 升级、消息反序列化分发、心跳定时器。业务逻辑通过 handler 分派，不在此模块实现。
**依赖**: Task 1, 2
**文件**: `src/ws/{mod,message,handler,heartbeat}.rs`

**接口契约**:
```rust
pub enum WsMessage {
    Register { device_code: String, platform: String, version: String, team_id: Option<String> },
    Heartbeat { device_code: String, ts: u64 },
    ConnectRequest { from_code: String, to_code: String, invite_code: Option<String> },
    ConnectResponse { accepted: bool, session_id: String, from_code: String },
    IceOffer { session_id: String, sdp: String, candidates: Vec<String> },
    IceAnswer { session_id: String, sdp: String, candidates: Vec<String> },
    RelayRequest { session_id: String, preferred_region: Option<String> },
    RelayAssigned { session_id: String, relay_addr: String, relay_port: u16, token: String },
    PeerOffline { device_code: String, reason: String },
    NearbyUpdate { devices: Vec<NearbyDevice> },
}
pub struct NearbyDevice { pub code: String, pub name: String, pub platform: String, pub online: bool }
impl WsMessage { pub fn from_json(text: &str) -> Result<Self>; pub fn to_json(&self) -> Result<String> }

pub async fn ws_upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse;
pub async fn dispatch_message(msg: WsMessage, ctx: &mut ClientContext, state: &AppState) -> Result<()>;

pub struct HeartbeatWatchdog { device_code: String, last_heartbeat: Arc<AtomicU64>, timeout_secs: u64 }
impl HeartbeatWatchdog {
    pub fn new(device_code: String, timeout_secs: u64) -> Self;
    pub fn touch(&self);
    pub async fn run(self, cancel: watch::Receiver<bool>);  // 超时→PeerOffline→清理→关闭
}
```

**验收标准**:
- [ ] register 收到确认 | 无效 JSON 返回 error 不断连 | 90s 无心跳自动断开 | 单设备仅 1 个活跃连接

---

## Task 4: 设备注册与在线状态 (~2d)

**目标**: 处理 `register` 消息，写入 Redis 设备在线（TTL 60s），向同团队广播上线通知。心跳刷新 TTL，断开时清理。
**依赖**: Task 2, 3
**文件**: `src/device/{mod,registry}.rs`

**接口契约**:
```rust
pub struct DeviceInfo { pub device_code: String, pub platform: String, pub version: String,
                        pub team_id: Option<String>, pub ip: String, pub connected_at: u64 }

pub async fn handle_register(info: DeviceInfo, pool: &mut RedisPool, ws_tx: &WsSender) -> Result<()>;
// SET device:{code}:online EX 60 → SADD team:{id}:online_devices → PUBLISH NearbyUpdate → 回复 ack
pub async fn handle_heartbeat(device_code: &str, pool: &mut RedisPool) -> Result<()>;
// EXPIRE device:{code}:online 60; Key 不存在则视为重连
pub async fn handle_unregister(device_code: &str, team_id: Option<&str>, pool: &mut RedisPool) -> Result<()>;
// DEL online key → SREM team set → PUBLISH PeerOffline

pub struct DeviceRegistry { connections: DashMap<String, WsSender> }
impl DeviceRegistry {
    pub fn new() -> Self;
    pub fn register(&self, code: String, sender: WsSender);
    pub fn unregister(&self, code: &str);
    pub fn get(&self, code: &str) -> Option<WsSender>;
    pub fn team_devices(&self, team_id: &str, pool: &mut RedisPool) -> Result<Vec<String>>;
}
```

**验收标准**:
- [ ] 注册后 Redis Key TTL ≤ 60s | 心跳刷新 TTL | 同团队收到 NearbyUpdate | 断开清理 | 重复注册踢旧连接

---

## Task 5: 连接协商 (~3d)

**目标**: 转发 connect_request/response，中继 ice_offer/answer 完成 SDP 交换，创建会话记录到 Redis。
**依赖**: Task 3, 4
**文件**: `src/session/mod.rs`

**接口契约**:
```rust
pub struct SessionInfo { pub session_id: String, pub controller_code: String,
                         pub controlled_code: String, pub path: ConnectionPath, pub started_at: u64 }
pub enum ConnectionPath { L1, L2, L3 }

pub async fn handle_connect_request(from: &str, to: &str, invite: Option<&str>,
    registry: &DeviceRegistry, pool: &mut RedisPool) -> Result<()>;
// 检查 to 在线 → 验证 invite → 转发 ConnectRequest; 离线返回 device_offline
pub async fn handle_connect_response(accepted: bool, session_id: &str, from: &str,
    registry: &DeviceRegistry, pool: &mut RedisPool) -> Result<()>;
// accepted → HSET session:{id} + 转发; rejected → 转发 accepted=false
pub async fn handle_ice_offer(session_id: &str, sdp: &str, candidates: &[String],
    registry: &DeviceRegistry, pool: &mut RedisPool) -> Result<()>;
// 查 session 对端 → 转发 IceOffer
pub async fn handle_ice_answer(session_id: &str, sdp: &str, candidates: &[String],
    registry: &DeviceRegistry, pool: &mut RedisPool) -> Result<()>;
// 查 session 发起端 → 转发 IceAnswer → HSET path=L2
```

**验收标准**:
- [ ] 在线转发成功 | 双方收到 session_id | 拒绝返回 false | ICE 正确中继 | 离线返回错误

---

## Task 6: 中转节点分配 (~2d)

**目标**: P2P 穿透失败后，按延迟+负载评分选最优中转节点，生成 HMAC 一次性 Token（30s），返回分配信息给双方。同时提供 `POST /api/relay/session-ended` 回调端点，接收中转节点会话结束上报并更新会话状态。
**依赖**: Task 2, 5
**文件**: `src/session/relay.rs`, `src/session/relay_callback.rs`

**接口契约**:
```rust
pub struct RelayNode { pub addr: String, pub port: u16, pub region: String, pub load: f64, pub last_heartbeat: u64 }
pub struct RelaySelector { nodes: Vec<RelayNode> }
impl RelaySelector {
    pub fn from_config(nodes: Vec<RelayNode>) -> Self;
    pub fn select(&self, ctrl: &str, ctld: &str, region: Option<&str>) -> Option<&RelayNode>;
    // 过滤 heartbeat>30s 和 load>0.8 → score=0.4/lat_A+0.4/lat_B+0.2/(load+0.01) → 最高分
    pub fn refresh_node(&mut self, addr: &str, load: f64, ts: u64);
}
pub fn generate_relay_token(session_id: &str, relay_addr: &str, secret: &[u8]) -> String;
// HMAC-SHA256(secret, session_id||addr||ts) → hex, 30s 有效
pub async fn handle_relay_request(session_id: &str, region: Option<&str>, selector: &RelaySelector,
    secret: &[u8], registry: &DeviceRegistry, pool: &mut RedisPool) -> Result<()>;
// select → token → HSET path=L3 → 双方 RelayAssigned

// 中转节点回调 — 会话结束时由 relay 节点主动上报
pub struct RelaySessionEndedPayload { pub session_id: String, pub relay_addr: String, pub reason: String, pub duration_sec: u64, pub bytes_forwarded: u64 }
pub async fn handle_relay_session_ended(payload: RelaySessionEndedPayload, pool: &mut RedisPool) -> Result<()>;
// POST /api/relay/session-ended → 更新 session:{id} 状态为 ended → 记录 duration/bytes → 清理 session key
```

**验收标准**:
- [ ] 高负载/过期节点被过滤 | 同区域优先 | Token 30s 后失效 | 无节点返回 relay_unavailable
- [ ] `POST /api/relay/session-ended` 接收回调后 session 状态更新为 ended，duration_sec 和 bytes_forwarded 正确记录

---

## Task 7: 离线检测 (~2d)

**目标**: 监听 Redis keyspace 通知（`__keyevent@0__:expired`），设备在线 Key 过期时广播离线事件，通知活跃会话对端。
**依赖**: Task 2, 4
**文件**: `src/offline/mod.rs`

**接口契约**:
```rust
pub struct OfflineDetector { pool: RedisPool, registry: Arc<DeviceRegistry> }
impl OfflineDetector {
    pub fn new(pool: RedisPool, registry: Arc<DeviceRegistry>) -> Self;
    pub async fn start(&self, cancel: watch::Receiver<bool>);
    // SUBSCRIBE __keyevent@0__:expired → 匹配 device:*:online → handle_device_offline
    async fn handle_device_offline(&self, device_code: &str) -> Result<()>;
    // SREM team set → 查活跃 session → 通知对端 PeerOffline → PUBLISH 团队事件 → unregister
}
pub async fn configure_keyspace_notifications(pool: &mut RedisPool) -> Result<()>;
// CONFIG SET notify-keyspace-events Ex
```

**验收标准**:
- [ ] 启动时自动配置 keyspace 通知 | Key 过期后 <1s 收到通知 | 对端收到 PeerOffline | 团队集合移除 | 会话通知

---

## Task 8: 邀请码 (~2d)

**目标**: 生成 4 位邀请码（排除 0/O/1/I/L 等歧义字符，31 字符集 = 923K 组合），600s TTL 一次性消费，5 次错误锁定 30 分钟。
**依赖**: Task 2
**文件**: `src/invite/{mod,lockout}.rs`

**接口契约**:
```rust
const INVITE_CHARSET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";  // 31 chars, 排除歧义
pub struct InviteCode { pub code: String, pub device_code: String, pub team_id: String, pub created_at: u64 }
pub fn generate_invite_code() -> String;  // 随机 4 字符
pub async fn create_invite(device: &str, team: &str, pool: &mut RedisPool) -> Result<String>;
// SET device_invite:{code} EX 600
pub async fn validate_invite(code: &str, pool: &mut RedisPool) -> Result<InviteCode>;
// GET → 不存在返回 InviteExpired
pub async fn consume_invite(code: &str, pool: &mut RedisPool) -> Result<()>;  // DEL

pub struct LockoutManager { pool: RedisPool, max_attempts: u32, lockout_secs: u64 }
impl LockoutManager {
    pub fn new(pool: RedisPool) -> Self;
    pub async fn check_lockout(&self, ip: &str) -> Result<()>;    // >=5 → RateLimited
    pub async fn record_failure(&self, ip: &str) -> Result<()>;   // INCR + EXPIRE 1800
    pub async fn reset(&self, ip: &str) -> Result<()>;            // DEL（成功验证后）
}
```

**验收标准**:
- [ ] 字符集正确、长度 4 | 600s 过期 | 消费后失效 | 5 次错误锁定 | 30 分钟自动解锁 | 成功后重置

---

## Task 9: 水平扩展 (~3d)

**目标**: Redis Pub/Sub 跨实例消息桥接，team_id SipHash 粘性路由确保同团队落同实例，设备专属 channel 跨实例转发。
**依赖**: Task 3, 4
**文件**: `src/scale/{mod,router}.rs`

**接口契约**:
```rust
pub struct PubSubBridge { pool: RedisPool, registry: Arc<DeviceRegistry> }
impl PubSubBridge {
    pub fn new(pool: RedisPool, registry: Arc<DeviceRegistry>) -> Self;
    pub async fn start(&self, cancel: watch::Receiver<bool>);
    // PSUBSCRIBE team:*:events → 本地有成员 → registry 转发
    pub async fn publish_event(&self, channel: &str, msg: &WsMessage) -> Result<()>;
    pub async fn forward_to_device(&self, code: &str, msg: &WsMessage) -> Result<bool>;
    // 本地有 → 直发 true; 无 → PUBLISH device:{code}:signals false
}

pub struct StickyRouter { instance_count: u32 }
impl StickyRouter {
    pub fn new(instance_count: u32) -> Self;
    pub fn route_team(&self, team_id: &str) -> u32;       // siphash(team_id) % count
    pub fn instance_for_team(&self, team_id: &str) -> String;
}

pub struct AppState {
    pub pool: RedisPool, pub registry: Arc<DeviceRegistry>, pub bridge: Arc<PubSubBridge>,
    pub relay_selector: Arc<RwLock<RelaySelector>>, pub lockout: Arc<LockoutManager>, pub config: AppConfig,
}
```

**验收标准**:
- [ ] 同 team_id 路由一致 | 跨实例事件广播 | 设备不在本地走 Pub/Sub | 实例数变更一致性 | 断线自动重连

---

## Task 10: 集成测试 (~2d)

**目标**: 端到端集成测试，使用 testcontainers 启动真实 Redis。验证双客户端流程、心跳超时、邀请码生命周期。
**依赖**: Task 1-9
**文件**: `tests/integration/{common,two_client,heartbeat,invite}.rs`

**接口契约**:
```rust
pub struct TestHarness { pub server_addr: String, pub redis_pool: RedisPool, _container: Container }
impl TestHarness {
    pub async fn start() -> Self;  // Redis testcontainer + axum 随机端口 + /health 就绪
    pub async fn connect_client(&self, code: &str) -> WsTestClient;
    pub async fn stop(self);
}
pub struct WsTestClient { pub tx: mpsc::Sender<WsMessage>, pub rx: mpsc::Receiver<WsMessage> }
impl WsTestClient { pub async fn send(&self, msg: WsMessage); pub async fn recv(&mut self) -> WsMessage;
                     pub async fn recv_timeout(&mut self, d: Duration) -> Option<WsMessage>; pub async fn close(self) }

#[tokio::test] async fn test_full_connection_flow();
// A/B 注册 → connect_request → accept → ice_offer/answer → 双方收到 session_id
#[tokio::test] async fn test_connect_to_offline_device();       // → device_offline
#[tokio::test] async fn test_reject_connection();               // → accepted=false
#[tokio::test] async fn test_heartbeat_timeout_disconnects();   // 加速 5s 超时
#[tokio::test] async fn test_heartbeat_keeps_alive();           // 心跳保持在线
#[tokio::test] async fn test_invite_lifecycle();                // create → validate → consume → 失效
#[tokio::test] async fn test_invite_expiry();                   // 过期返回 InviteExpired
#[tokio::test] async fn test_invite_brute_force_lockout();      // 5 次错误 → RateLimited
```

**验收标准**:
- [ ] 全部通过 | 连接流程 <500ms | 心跳测试加速 <30s | 邀请码覆盖全生命周期 | 测试间隔离

---

## 执行顺序

```
Week 1: Task 1 → 2 → 3 ↘ 4        Week 2: 4 → 5 → 6, Task 8 并行
Week 3: Task 7, Task 9              Week 4: Task 10
```

**关键路径**: 1 → 2 → 3 → 5 → 10 | **可并行**: 6+8, 7+9

| 决策 | 选择 | 理由 |
|------|------|------|
| WebSocket | axum 内置 | 与路由统一 |
| 内存缓存 | DashMap | 无锁并发 |
| 哈希路由 | SipHash | 快速均匀 |
| 测试 Redis | testcontainers | 真实实例 |
| 心跳超时 | 90s (3x) | 容忍 2 次丢失 |
