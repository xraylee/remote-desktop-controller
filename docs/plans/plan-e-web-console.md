# Plan E: Web 控制台实现计划（Go API + React 前端）

**版本**：v1.0 · **日期**：2026-06-25 · **总工期**：约 25-30 人天
**范围**：PRD §5（Web 控制台功能）+ 架构设计 §5（安全架构 · 数据模型）

## 概述

12 个 Task 分为 Go API（Task 1-8）和 React 前端（Task 9-12）。接口契约在 Task 1 阶段锁定，前后端可并行开发。

**数据表**（6 张）：`teams` · `members` · `devices` · `connection_records` · `audit_logs` · `recordings`
**API 前缀**：`/api/v1` · **认证**：JWT RS256 · **实时推送**：Redis Pub/Sub → WebSocket

## 跨项目依赖

依赖: Plan F (PostgreSQL/Redis/MinIO 基础设施)

## Part 1 — Go API

### Task 1: 项目脚手架 (~2d)

**目标**: 初始化 Go 项目，配置 chi 路由、中间件链、健康检查和 Dockerfile。建立分层目录规范。
**依赖**: 无
**文件**: `server/main.go` · `server/internal/{config,server,middleware}/*.go` · `server/Dockerfile`

**接口契约**:
```go
type Config struct { Port int; DatabaseURL string; RedisURL string; JWTPrivateKey, JWTPublicKey string }
type Server  struct { router chi.Router; cfg *Config; db *sqlx.DB; redis *redis.Client }
func New(cfg *Config, db *sqlx.DB, rdb *redis.Client) *Server
func (s *Server) Start(ctx context.Context) error
func RequestID(next http.Handler) http.Handler
func Logger(next http.Handler) http.Handler
func CORS(origins []string) func(http.Handler) http.Handler
func RateLimit(rps int) func(http.Handler) http.Handler
```

**验收标准**:
- [ ] `GET /api/v1/health` 返回 `{"status":"ok"}` 且 200
- [ ] Dockerfile 多阶段构建，最终镜像 < 30MB
- [ ] RateLimit 超阈值返回 429

---

### Task 2: 数据库层 (~2d)

**目标**: sqlx + golang-migrate 管理 6 张表 Schema，实现 Repository 层 CRUD 和种子数据（1 团队 + 3 成员 + 5 设备）。
**依赖**: Task 1
**文件**: `server/internal/db/migrations/001_init.{up,down}.sql` · `server/internal/repository/{team,member,device,connection,audit,recording}_repo.go`

**接口契约**:
```go
type TeamRepository interface {
    GetByID(ctx, id) (*model.Team, error); Create(ctx, *model.Team) error; Update(ctx, *model.Team) error
}
type MemberRepository interface {
    GetByID(ctx, id) (*model.Member, error); GetByEmail(ctx, email) (*model.Member, error)
    ListByTeam(ctx, teamID) ([]model.Member, error); Create(ctx, *model.Member) error; Update(ctx, *model.Member) error
}
type DeviceRepository interface {
    GetByID(ctx, id) (*model.Device, error); GetByCode(ctx, code) (*model.Device, error)
    ListByTeam(ctx, teamID, DeviceFilter) ([]model.Device, error)
    Create(ctx, *model.Device) error; Update(ctx, *model.Device) error; Delete(ctx, id) error
}
type DeviceFilter struct { Status, Platform, Search string; Limit, Offset int }
type ConnectionRecordRepository interface {
    List(ctx, teamID, ConnFilter) ([]model.ConnectionRecord, int, error)
    Create(ctx, *model.ConnectionRecord) error
    ExportCSV(ctx, teamID, ConnFilter) (io.ReadCloser, error)
}
type AuditLogRepository interface {
    List(ctx, teamID, AuditFilter) ([]model.AuditLog, int, error); Create(ctx, *model.AuditLog) error
}
type RecordingRepository interface {
    List(ctx, teamID, limit, offset) ([]model.Recording, int, error); GetByID(ctx, id) (*model.Recording, error)
}
```

**验收标准**:
- [ ] migration up/down 幂等无报错；所有 Repository 方法有 testcontainers-go 集成测试
- [ ] ExportCSV 输出 UTF-8 BOM 编码 CSV；seed.sql 插入后可查出 3 成员 + 5 设备

---

### Task 3: 认证 — JWT RS256 (~2d)

**目标**: 实现 email+password 登录、RS256 JWT（15min access + 7d refresh）和中间件鉴权。
**依赖**: Task 1, 2
**文件**: `server/internal/auth/{jwt,password}.go` · `server/internal/handler/auth_handler.go`

**接口契约**:
```go
type TokenPair struct { AccessToken, RefreshToken string; ExpiresIn int }
type Claims struct { MemberID, TeamID uuid.UUID; Role string; jwt.RegisteredClaims }
func NewJWTManager(privPEM, pubPEM string) (*JWTManager, error)
func (j *JWTManager) GenerateTokenPair(memberID, teamID uuid.UUID, role string) (*TokenPair, error)
func (j *JWTManager) ValidateToken(token string) (*Claims, error)
func (j *JWTManager) RefreshToken(refresh string) (*TokenPair, error)
func HashPassword(plain string) (string, error)
func ComparePassword(hash, plain string) error
func (h *AuthHandler) Login(w, r)       // POST /api/v1/auth/login
func (h *AuthHandler) RefreshToken(w, r) // POST /api/v1/auth/refresh
func (h *AuthHandler) Logout(w, r)       // POST /api/v1/auth/logout
func (s *Server) AuthMiddleware(next http.Handler) http.Handler
func (s *Server) RequireRole(roles ...string) func(http.Handler) http.Handler
```

**验收标准**:
- [ ] Login 正确凭据返回 TokenPair；错误密码 5 次 → 423 Locked 15min
- [ ] Refresh 返回新 TokenPair；Logout 将 refresh 加入 Redis 黑名单
- [ ] RequireRole("owner") 拒绝 role=member 返回 403

---

### Task 4: TOTP 双因素认证 (~1.5d)

**目标**: 为 owner/manager 实现 TOTP 设置（secret → QR → 验证启用）和登录强制校验。
**依赖**: Task 3
**文件**: `server/internal/auth/totp.go` · `server/internal/handler/totp_handler.go`

**接口契约**:
```go
type TOTPSetup struct { Secret, QRCodeURL, URI string }
func GenerateTOTPSecret(issuer, account string) (*TOTPSetup, error)
func ValidateTOTP(secret, code string) bool
func (h *TOTPHandler) Setup(w, r)   // POST /api/v1/auth/totp/setup → TOTPSetup
func (h *TOTPHandler) Verify(w, r)  // POST /api/v1/auth/totp/verify {code} → 启用
func (h *TOTPHandler) Disable(w, r) // DELETE /api/v1/auth/totp {code} → 关闭
```

**登录两步流程**: Login → `{"require_totp":true,"temp_token":"..."}` → `POST /auth/login/totp {temp_token, code}` → TokenPair

**验收标准**:
- [ ] QR 码可被 Google Authenticator 扫描；启用后登录必须两步，跳过返回 401
- [ ] Disable 需提交当前有效 code

---

### Task 5: 设备管理 (~2d)

**目标**: 设备 CRUD API，在线状态从 Redis 实时叠加，支持筛选/分页/搜索。
**依赖**: Task 2, 3
**文件**: `server/internal/handler/device_handler.go` · `server/internal/service/device_service.go`

**接口契约**:
```go
type DeviceWithStatus struct { model.Device; IsOnline bool; OnlineIP string }
type DeviceService struct { repo DeviceRepository; redis *redis.Client }
func (s *DeviceService) ListDevices(ctx, teamID, DeviceFilter) ([]DeviceWithStatus, int, error)
func (s *DeviceService) RemoveDevice(ctx, teamID, deviceCode) error
func (h *DeviceHandler) ListDevices(w, r)  // GET  /teams/{teamID}/devices?status=&platform=&search=&page=&size=
func (h *DeviceHandler) GetDevice(w, r)    // GET  /teams/{teamID}/devices/{deviceCode}
func (h *DeviceHandler) RemoveDevice(w, r) // DELETE /teams/{teamID}/devices/{deviceCode}
```

**在线叠加**: `GET device:{code}:online` + `SMEMBERS team:{id}:online_devices` 覆盖 DB status 字段

**验收标准**:
- [ ] `is_online` 与 Redis key 一致；`?status=online` 仅返回在线设备；`?search=mac` 模糊匹配 name 和 code
- [ ] RemoveDevice 后列表消失且 Redis key 被删；分页 `page=2&size=10` 正确返回

---

### Task 6: 连接记录与审计 (~2d)

**目标**: 连接记录查询（时间范围筛选）、CSV 导出、审计日志中间件和会话录制管理。所有管理操作自动记录审计。录制列表支持 MinIO 预签名下载 URL。
**依赖**: Task 2, 3
**文件**: `server/internal/handler/{session,audit,recording}_handler.go` · `server/internal/middleware/audit.go` · `server/internal/service/recording_service.go`

**接口契约**:
```go
type ConnectionRecord struct { ID, TeamID uuid.UUID; ControllerCode, ControlledCode, ControllerName, ControlledName, Path string; StartedAt time.Time; DurationSec int; BytesTransferred int64 }
func (h *SessionHandler) ListRecords(w, r)  // GET /teams/{teamID}/sessions?since=&until=&page=&size=
func (h *SessionHandler) ExportCSV(w, r)    // GET /teams/{teamID}/sessions/export → CSV 文件流
func (h *AuditHandler) ListLogs(w, r)       // GET /teams/{teamID}/audit?action=&page=&size=
type AuditMiddleware struct { repo AuditLogRepository }
func (a *AuditMiddleware) Log(action string) func(http.Handler) http.Handler
// 使用: r.With(audit.Log("device_remove")).Delete("/devices/{id}", handler.RemoveDevice)

// 会话录制 — 查询录制列表并返回 MinIO 预签名下载 URL
type RecordingWithStatus struct { model.Recording; DownloadURL string; DownloadExpiresAt time.Time }
type RecordingService struct { repo RecordingRepository; minioClient *minio.Client }
func (s *RecordingService) ListRecordings(ctx, teamID, limit, offset) ([]RecordingWithStatus, int, error)
func (s *RecordingService) GetDownloadURL(ctx, recordingID, teamID) (string, time.Time, error)
func (h *RecordingsHandler) List(w, r)   // GET /teams/{teamID}/recordings?page=&size=
func (h *RecordingsHandler) GetURL(w, r) // GET /teams/{teamID}/recordings/{recordingID}/url → {download_url, expires_at}
```

**验收标准**:
- [ ] ListRecords JOIN devices 返回双方名称；时间范围过滤正确；ExportCSV 返回 attachment + UTF-8 BOM
- [ ] RemoveDevice 后 audit_logs 新增 `action=device_remove` 记录，含 actor_id、ip_address、details JSONB
- [ ] RecordingsHandler.List 返回录制列表含 presigned download_url；GetURL 返回有效 MinIO 预签名 URL（15min 过期）

---

### Task 7: 成员管理 (~2d)

**目标**: 邀请成员（链接 24h 有效）、角色变更（owner/manager/member）、团队设置更新。
**依赖**: Task 2, 3, 6
**文件**: `server/internal/handler/{member,team}_handler.go` · `server/internal/service/invite_service.go`

**接口契约**:
```go
type Member struct { ID, TeamID uuid.UUID; Name, Email, Role string; TOTPEnabled bool; DeviceCount int; LastLogin *time.Time }
type InviteService struct { redis *redis.Client; repo MemberRepository }
func (s *InviteService) CreateInviteLink(ctx, teamID) (string, error) // Redis SET member_invite:{token} EX 86400
func (s *InviteService) AcceptInvite(ctx, token, memberID) error
func (h *MemberHandler) ListMembers(w, r)    // GET    /teams/{teamID}/members
func (h *MemberHandler) InviteMember(w, r)   // POST   /teams/{teamID}/invite → {invite_url, expires_at}
func (h *MemberHandler) UpdateRole(w, r)     // PUT    /teams/{teamID}/members/{memberID} {role}
func (h *MemberHandler) RemoveMember(w, r)   // DELETE /teams/{teamID}/members/{memberID}
func (h *TeamHandler) GetTeam(w, r)          // GET    /teams/{teamID}
func (h *TeamHandler) UpdateTeam(w, r)       // PUT    /teams/{teamID} {name, ...}
```

**验收标准**:
- [ ] InviteMember 返回链接 Redis 24h 过期；AcceptInvite 后成员出现在列表
- [ ] UpdateRole 仅 owner 可调用；所有角色变更自动写入 audit_logs；RemoveMember 后 JWT 立即失效

---

### Task 8: WebSocket 实时推送 (~2d)

**目标**: Redis Pub/Sub → WebSocket 推送设备上下线、连接事件到前端。
**依赖**: Task 1, 3, 5
**文件**: `server/internal/ws/{hub,client,handler}.go`

**接口契约**:
```go
type Hub struct { clients map[uuid.UUID]*Client; redis *redis.Client }
type Event struct { Type string; TeamID uuid.UUID; Payload json.RawMessage; Timestamp time.Time }
func NewHub(rdb *redis.Client) *Hub
func (h *Hub) Run(ctx context.Context) error    // SUBSCRIBE team:*:events → 分发
func (h *Hub) BroadcastToTeam(teamID, Event) error
func (s *Server) WebSocket(w, r)                // GET /api/v1/ws → HTTP Upgrade + JWT 验证
// 事件类型: device_online | device_offline | connection_started | connection_ended
// 前端心跳: {"type":"ping"} → {"type":"pong"}
```

**验收标准**:
- [ ] Redis PUBLISH 后前端 < 100ms 收到事件；JWT 无效时 WS 被拒；重复连接踢出旧连接
- [ ] 服务端关闭时发送 `{"type":"shutdown"}`

## Part 2 — React 前端

### Task 9: 项目脚手架 (~1.5d)

**目标**: Vite + React + TS + Tailwind 项目初始化，API 客户端（axios + 拦截器）、路由和 Zustand 基础结构。
**依赖**: 无（可与 Part 1 并行）
**文件**: `web/{package.json,vite.config.ts,tailwind.config.ts}` · `web/src/{main.tsx,App.tsx}` · `web/src/api/client.ts` · `web/src/stores/authStore.ts` · `web/src/router/index.tsx`

**接口契约**:
```typescript
// api/client.ts — axios 实例 + 拦截器
// 请求: 自动附加 Bearer token  |  响应: 401 → refresh → 重试 1 次 → 失败跳 /login

interface LoginRequest { email: string; password: string; totp_code?: string }
interface LoginResponse { access_token: string; refresh_token: string; expires_in: number }
interface RequireTOTP { require_totp: true; temp_token: string }

// stores/authStore.ts (Zustand)
interface AuthState { user: User | null; accessToken: string | null; isAuthenticated: boolean
  login(req: LoginRequest): Promise<void>; logout(): void; refreshAccessToken(): Promise<string> }
interface User { member_id: string; team_id: string; role: 'owner'|'manager'|'member'; name: string; email: string }

// router/index.tsx
// /login → LoginPage (public)  |  /invite/:token → InviteAcceptPage (public)
// / → DashboardPage  |  /devices → DevicesPage  |  /sessions → SessionsPage
// /recordings → RecordingsPage  |  /members → MembersPage  |  /settings → SettingsPage  —  后 6 个为 ProtectedRoute
```

**验收标准**:
- [ ] `npm run dev` 热更新正常；401 自动 refresh+重试，失败清 token 跳 /login
- [ ] ProtectedRoute 未认证重定向 /login

---

### Task 10: 认证流程 (~2d)

**目标**: 登录页（email+password+TOTP 两步）、TOTP 设置弹窗、JWT 存储/自动刷新。
**依赖**: Task 9（UI）+ Task 4（API）
**文件**: `web/src/pages/LoginPage.tsx` · `web/src/components/auth/{TOTPSetupModal,TOTPVerifyInput}.tsx` · `web/src/hooks/useAuth.ts`

**接口契约**:
```typescript
// hooks/useAuth.ts
function useAuth(): { user: User|null; isLoading: boolean; login(email, password): Promise<void>
  verifyTOTP(code): Promise<void>; logout(): void; setupTOTP(): Promise<TOTPSetupData> }
interface TOTPSetupData { secret: string; qr_code_url: string; uri: string }

// LoginPage — 状态机: idle → submitting → require_totp → submitting_totp → success|error
function LoginPage(): JSX.Element
// TOTPSetupModal — QR 码 + secret 折叠区 + 6 位验证
function TOTPSetupModal({ setup, onConfirm }: Props): JSX.Element
// TOTPVerifyInput — 6 格独立输入，粘贴自动拆分，自动聚焦下一位
function TOTPVerifyInput({ onComplete, error }: Props): JSX.Element
```

**验收标准**:
- [ ] password 正确后若 `require_totp=true` 显示 TOTP 区域；输入支持粘贴自动拆分 6 格
- [ ] token 持久化到 localStorage，刷新无需重新登录；QR 码可被 Authenticator 扫描

---

### Task 11: 仪表盘 + 设备管理页 (~3d)

**目标**: 仪表盘（统计卡片+最近连接）和设备列表（在线状态+筛选+搜索+移除），接入 WS 实时推送。
**依赖**: Task 5, 8, 9
**文件**: `web/src/pages/{DashboardPage,DevicesPage}.tsx` · `web/src/components/{dashboard,devices}/*.tsx` · `web/src/hooks/useWebSocket.ts` · `web/src/stores/wsStore.ts`

**接口契约**:
```typescript
// api/devices.ts
interface Device { id: string; device_code: string; device_name: string; platform: string
  is_online: boolean; ip_address: string; last_seen: string|null }
interface DeviceListResponse { data: Device[]; total: number; page: number; size: number }
function listDevices(teamID, filter: DeviceFilter): Promise<DeviceListResponse>
function removeDevice(teamID, deviceCode): Promise<void>

// api/dashboard.ts
interface DashboardStats { online_devices: number; total_devices: number; today_connections: number; month_bytes_transferred: number }
function getDashboardStats(teamID): Promise<DashboardStats>
function getRecentConnections(teamID, limit?): Promise<RecentConnection[]>

// hooks/useWebSocket.ts
function useWebSocket(teamID): { lastEvent: WSEvent|null; isConnected: boolean }

// 组件签名
function StatCard({ label, value, icon }): JSX.Element
function RecentConnections({ data }): JSX.Element
function DeviceList({ devices, onRemove }): JSX.Element
function DeviceCard({ device, onRemove }): JSX.Element
function DeviceFilters({ filter, onChange }): JSX.Element
function Pagination({ page, total, size, onPageChange }): JSX.Element
```

**验收标准**:
- [ ] 3 张 StatCard 正确展示；流量为人类可读格式；设备码格式化 `123 456 789`
- [ ] 筛选器同步 URL query；搜索防抖 300ms；WS 推送时 DeviceCard 实时更新
- [ ] TanStack Query 缓存列表，切换筛选显示骨架屏

---

### Task 12: 连接记录 + 成员管理 + 设置页 (~3d)

**目标**: 连接记录表格（CSV 导出）、会话录制列表（含下载链接）、成员管理（邀请/角色/移除）和团队设置，完成控制台全部功能。
**依赖**: Task 6, 7, 9
**文件**: `web/src/pages/{SessionsPage,RecordingsPage,MembersPage,SettingsPage}.tsx` · `web/src/components/{sessions,recordings,members,settings,layout}/*.tsx`

**接口契约**:
```typescript
// api/sessions.ts
interface ConnectionRecord { id: string; controller_name: string; controlled_name: string
  path: 'L1'|'L2'|'L3'; started_at: string; duration_sec: number; bytes_transferred: number }
function listSessions(teamID, filter: SessionFilter): Promise<{ data: ConnectionRecord[]; total: number }>
function exportSessionsCSV(teamID, filter): Promise<Blob>

// api/recordings.ts
interface Recording { id: string; connection_id: string; duration_sec: number; file_size: number; created_at: string; download_url?: string; download_expires_at?: string }
function listRecordings(teamID, params: { page: number; size: number }): Promise<{ data: Recording[]; total: number }>
function getRecordingURL(teamID, recordingID): Promise<{ download_url: string; expires_at: string }>

// api/members.ts
interface Member { id: string; name: string; email: string; role: string; totp_enabled: boolean; device_count: number }
function listMembers(teamID): Promise<Member[]>
function inviteMember(teamID): Promise<{ invite_url: string; expires_at: string }>
function updateMemberRole(teamID, memberID, role): Promise<void>
function removeMember(teamID, memberID): Promise<void>

// api/teams.ts
function getTeam(teamID): Promise<TeamSettings>
function updateTeam(teamID, settings: Partial<TeamSettings>): Promise<TeamSettings>

// 组件签名
function SessionTable({ records }): JSX.Element             // 路径 L1/L2/L3 绿/蓝/黄标签
function SessionExportButton({ teamID, filter }): JSX.Element // Blob → 下载
function RecordingList({ recordings, onDownload }): JSX.Element   // 录制列表，含时长/大小/下载链接
function RecordingDownloadButton({ teamID, recordingID }): JSX.Element // 获取 presigned URL 并触发下载
function MemberCard({ member, onRoleChange, onRemove }): JSX.Element
function InviteDialog({ teamID }): JSX.Element              // 生成链接 + 一键复制
function RoleSelector({ current, onChange, disabled }): JSX.Element
function TeamSettingsForm({ team, onSave }): JSX.Element
function Sidebar({ current, collapsed }): JSX.Element       // <768px 折叠为图标
function AppLayout({ children }): JSX.Element               // Sidebar + 主内容 + 顶栏
```

**验收标准**:
- [ ] SessionTable L1/L2/L3 绿/蓝/黄标签；CSV 导出触发浏览器下载
- [ ] RecordingList 显示录制列表（时长、文件大小、创建时间）；RecordingDownloadButton 点击后获取 presigned URL 并触发下载
- [ ] InviteDialog 一键复制到剪贴板 + Toast；RoleSelector 仅 owner 可操作
- [ ] TeamSettingsForm 保存后 Sidebar 同步；< 768px 自动折叠；骨架屏/错误提示

---

## 依赖图与排期

```
Task 1 (Go 脚手架) ──┬── Task 2 (数据库) ──┬── Task 5 (设备) ────────┐
                     │                     ├── Task 6 (记录审计) ────┤
                     │                     │   └── Task 7 (成员) ───┤
                     │                     └── Task 3 (JWT) ──┐     │
                     │                                        └── Task 4 (TOTP)
                     └── Task 8 (WebSocket) ◄─────────────────────┘
                                                                    │
Task 9 (React 脚手架) ──┬── Task 10 (认证) ◄── Task 4              │
                        ├── Task 11 (仪表盘+设备) ◄── Task 5 + 8 ──┘
                        └── Task 12 (记录+成员+设置) ◄── Task 6 + 7
```

**关键路径**: Task 1 → 2 → 3 → 5 → 11（约 11d）
**并行策略**: Task 9 第一天启动，用 Mock API 开发 UI；Task 3 完成后前后端联调

## 技术约定

| 项 | 约定 |
|----|------|
| Go 错误 | `fmt.Errorf("handler: %w", err)` |
| HTTP 响应 | `{"data":..., "meta":{"page","total"}}` / `{"error":{"code","message"}}` |
| 日期 | ISO 8601 `2026-06-25T14:32:00Z` · 分页 `page`（从 1）+ `size`（默认 20，最大 100） |
| React | 函数组件 + Hooks；Zustand 管 auth/WS；TanStack Query 管服务端数据 |
| 样式 | Tailwind utility，禁止内联 style · 测试 Go: testcontainers-go · React: Vitest + Testing Library |
