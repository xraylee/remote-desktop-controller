# RDCS API 测试用例规范

**项目**: RDCS Remote Desktop Control System  
**版本**: v0.1.0  
**日期**: 2026-06-29  
**状态**: Draft

---

## 测试环境配置

### 前置条件

```bash
# 1. 启动数据库服务
cd deploy/docker
docker compose -f docker-compose.minimal.yml up -d

# 2. 生成 JWT 密钥对
cd services/api
go run ../../gen_jwt_keys.go > .env

# 3. 更新测试密码哈希
docker exec -it rdcs-postgres psql -U rdcs -d rdcs -c "UPDATE members SET password_hash = '\$2a\$10\$mmaNLZS/sqGHkQ5OKeRCG.o5EnYWmVDsZKop78CbD/M.8c7vZmQQC';"

# 4. 启动 API 服务
go run cmd/api/main.go
```

### 测试数据

| 角色 | 邮箱 | 密码 | Team ID |
|------|------|------|---------|
| Owner | admin@rdcs-test.local | test123 | a0000000-0000-0000-0000-000000000001 |
| Manager | manager@rdcs-test.local | test123 | a0000000-0000-0000-0000-000000000001 |
| Member | member@rdcs-test.local | test123 | a0000000-0000-0000-0000-000000000001 |

---

## 1. 认证模块 (Authentication)

### 1.1 用户登录

#### TC-AUTH-001: 成功登录（Owner 角色）

**优先级**: P0 (Critical)

**前置条件**:
- API 服务运行在 `localhost:8080`
- 数据库包含测试用户数据
- JWT 密钥已配置

**测试步骤**:
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@rdcs-test.local",
    "password": "test123"
  }'
```

**预期结果**:
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "member": {
    "id": "b0000000-0000-0000-0000-000000000001",
    "team_id": "a0000000-0000-0000-0000-000000000001",
    "name": "张管理员",
    "email": "admin@rdcs-test.local",
    "role": "owner",
    "created_at": "2026-01-01T00:00:00Z"
  }
}
```

**验证点**:
- ✅ HTTP 状态码 = 200
- ✅ 返回 `access_token` 字段（JWT 格式）
- ✅ 返回 `refresh_token` 字段
- ✅ `expires_in` = 3600（1小时）
- ✅ `member.role` = "owner"
- ✅ `member.email` = "admin@rdcs-test.local"

---

#### TC-AUTH-002: 登录失败 - 错误密码

**优先级**: P0

**测试步骤**:
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@rdcs-test.local",
    "password": "wrongpassword"
  }'
```

**预期结果**:
```json
{
  "error": "invalid_credentials"
}
```

**验证点**:
- ✅ HTTP 状态码 = 401
- ✅ 返回错误信息 "invalid_credentials"
- ✅ 不返回敏感信息（用户是否存在）

---

#### TC-AUTH-003: 登录失败 - 不存在的用户

**优先级**: P0

**测试步骤**:
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "notexist@example.com",
    "password": "test123"
  }'
```

**预期结果**:
```json
{
  "error": "invalid_credentials"
}
```

**验证点**:
- ✅ HTTP 状态码 = 401
- ✅ 返回通用错误信息（防止用户枚举）

---

#### TC-AUTH-004: 登录失败 - 缺少必填字段

**优先级**: P1

**测试步骤**:
```bash
# 缺少 password
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@rdcs-test.local"
  }'
```

**预期结果**:
```json
{
  "error": "email and password are required"
}
```

**验证点**:
- ✅ HTTP 状态码 = 400
- ✅ 返回明确的验证错误

---

#### TC-AUTH-005: 登录失败 - 无效 JSON 格式

**优先级**: P2

**测试步骤**:
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d 'invalid json'
```

**预期结果**:
```json
{
  "error": "invalid_request_body"
}
```

**验证点**:
- ✅ HTTP 状态码 = 400

---

### 1.2 TOTP 两步验证

#### TC-AUTH-101: TOTP 设置流程

**优先级**: P1

**前置条件**:
- 用户已登录，获得 `access_token`

**测试步骤**:
```bash
# 1. 设置 TOTP
TOKEN="<access_token>"
curl -X POST http://localhost:8080/api/v1/auth/totp/setup \
  -H "Authorization: Bearer $TOKEN"

# 预期返回:
# {
#   "secret": "JBSWY3DPEHPK3PXP",
#   "uri": "otpauth://totp/RDCS:admin@rdcs-test.local?secret=JBSWY3DPEHPK3PXP&issuer=RDCS"
# }

# 2. 使用身份验证器生成 6 位数字码
# 3. 验证 TOTP 码
curl -X POST http://localhost:8080/api/v1/auth/totp/verify \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"code": "123456"}'

# 预期返回:
# {"enabled": true}

# 4. 使用 TOTP 登录
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@rdcs-test.local",
    "password": "test123",
    "totp_code": "123456"
  }'
```

**验证点**:
- ✅ TOTP secret 生成成功
- ✅ URI 格式正确（otpauth://）
- ✅ 验证码验证成功后 `enabled: true`
- ✅ 登录时需要提供 `totp_code`

---

## 2. 设备管理模块 (Device Management)

### 2.1 设备列表查询

#### TC-DEVICE-001: 查询团队设备列表

**优先级**: P0

**前置条件**:
- 用户已登录（Owner 或 Manager 角色）

**测试步骤**:
```bash
TOKEN="<access_token>"
TEAM_ID="a0000000-0000-0000-0000-000000000001"

curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/devices" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
```json
{
  "devices": [
    {
      "id": "c0000000-0000-0000-0000-000000000001",
      "team_id": "a0000000-0000-0000-0000-000000000001",
      "device_code": "100200301",
      "device_name": "MacBook-Pro-张三",
      "platform": "macos",
      "os_version": "15.0",
      "client_version": "0.1.0",
      "status": "online",
      "ip_address": "192.168.1.101",
      "last_seen_at": "2026-06-29T12:00:00Z",
      "created_at": "2026-01-01T00:00:00Z"
    }
  ],
  "total": 5,
  "page": 1,
  "page_size": 20
}
```

**验证点**:
- ✅ HTTP 状态码 = 200
- ✅ 返回设备数组
- ✅ 每个设备包含完整字段
- ✅ `device_code` 格式为 9 位数字
- ✅ `status` 为 online/offline
- ✅ 分页信息正确

---

#### TC-DEVICE-002: 分页查询设备

**优先级**: P1

**测试步骤**:
```bash
# 查询第2页，每页2条
curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/devices?page=2&page_size=2" \
  -H "Authorization: Bearer $TOKEN"
```

**验证点**:
- ✅ 返回正确的分页数据
- ✅ `page` = 2
- ✅ `page_size` = 2
- ✅ `total` 为总设备数

---

#### TC-DEVICE-003: 按状态筛选设备

**优先级**: P1

**测试步骤**:
```bash
# 仅查询在线设备
curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/devices?status=online" \
  -H "Authorization: Bearer $TOKEN"
```

**验证点**:
- ✅ 所有返回设备的 `status` = "online"

---

### 2.2 设备详情查询

#### TC-DEVICE-101: 查询单个设备信息

**优先级**: P0

**测试步骤**:
```bash
DEVICE_CODE="100200301"

curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/devices/$DEVICE_CODE" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
```json
{
  "id": "c0000000-0000-0000-0000-000000000001",
  "device_code": "100200301",
  "device_name": "MacBook-Pro-张三",
  "status": "online",
  "last_seen_at": "2026-06-29T12:00:00Z"
}
```

**验证点**:
- ✅ HTTP 状态码 = 200
- ✅ 返回设备详细信息

---

#### TC-DEVICE-102: 查询不存在的设备

**优先级**: P1

**测试步骤**:
```bash
curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/devices/999999999" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
```json
{
  "error": "device_not_found"
}
```

**验证点**:
- ✅ HTTP 状态码 = 404

---

### 2.3 设备注册

#### TC-DEVICE-201: 注册新设备

**优先级**: P0

**测试步骤**:
```bash
curl -X POST "http://localhost:8080/api/v1/teams/$TEAM_ID/devices" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "device_name": "Test-Device-01",
    "platform": "macos",
    "os_version": "15.0",
    "client_version": "0.1.0"
  }'
```

**预期结果**:
```json
{
  "id": "<uuid>",
  "device_code": "123456789",
  "device_name": "Test-Device-01",
  "status": "offline"
}
```

**验证点**:
- ✅ HTTP 状态码 = 201
- ✅ 自动生成唯一的 9 位 `device_code`
- ✅ 初始状态为 "offline"

---

### 2.4 设备删除

#### TC-DEVICE-301: 删除设备

**优先级**: P1

**前置条件**:
- 用户为 Owner 或 Manager 角色

**测试步骤**:
```bash
curl -X DELETE "http://localhost:8080/api/v1/teams/$TEAM_ID/devices/$DEVICE_CODE" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
- HTTP 状态码 = 204（无内容）

**验证点**:
- ✅ 设备成功删除
- ✅ 再次查询返回 404

---

## 3. 成员管理模块 (Member Management)

### 3.1 成员列表查询

#### TC-MEMBER-001: 查询团队成员列表

**优先级**: P0

**测试步骤**:
```bash
curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/members" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
```json
{
  "members": [
    {
      "id": "b0000000-0000-0000-0000-000000000001",
      "name": "张管理员",
      "email": "admin@rdcs-test.local",
      "role": "owner",
      "totp_enabled": false,
      "created_at": "2026-01-01T00:00:00Z"
    }
  ],
  "total": 3
}
```

**验证点**:
- ✅ HTTP 状态码 = 200
- ✅ 不返回 `password_hash`
- ✅ 不返回 `totp_secret`

---

### 3.2 邀请成员

#### TC-MEMBER-101: 邀请新成员

**优先级**: P0

**前置条件**:
- 用户为 Owner 或 Manager 角色

**测试步骤**:
```bash
curl -X POST "http://localhost:8080/api/v1/teams/$TEAM_ID/invite" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "newuser@example.com",
    "name": "新用户",
    "role": "member"
  }'
```

**预期结果**:
```json
{
  "id": "<uuid>",
  "email": "newuser@example.com",
  "name": "新用户",
  "role": "member",
  "invite_token": "<random_token>"
}
```

**验证点**:
- ✅ HTTP 状态码 = 201
- ✅ 返回邀请 token
- ✅ 成员状态为待激活

---

### 3.3 更新成员角色

#### TC-MEMBER-201: 更新成员角色

**优先级**: P1

**前置条件**:
- 用户为 Owner 角色

**测试步骤**:
```bash
MEMBER_ID="b0000000-0000-0000-0000-000000000003"

curl -X PUT "http://localhost:8080/api/v1/teams/$TEAM_ID/members/$MEMBER_ID" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "role": "manager"
  }'
```

**预期结果**:
```json
{
  "id": "b0000000-0000-0000-0000-000000000003",
  "role": "manager"
}
```

**验证点**:
- ✅ HTTP 状态码 = 200
- ✅ 角色更新成功

---

### 3.4 移除成员

#### TC-MEMBER-301: 移除团队成员

**优先级**: P1

**测试步骤**:
```bash
curl -X DELETE "http://localhost:8080/api/v1/teams/$TEAM_ID/members/$MEMBER_ID" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
- HTTP 状态码 = 204

**验证点**:
- ✅ 成员成功移除
- ✅ 该成员无法再登录

---

## 4. 连接记录模块 (Connection Records)

### 4.1 会话列表查询

#### TC-SESSION-001: 查询活跃会话

**优先级**: P0

**测试步骤**:
```bash
curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/sessions" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
```json
{
  "sessions": [
    {
      "id": "<uuid>",
      "controller_code": "100200301",
      "controlled_code": "100200304",
      "status": "active",
      "started_at": "2026-06-29T12:00:00Z",
      "duration": 3600
    }
  ],
  "total": 1
}
```

**验证点**:
- ✅ 返回当前活跃的会话
- ✅ `status` = "active"

---

### 4.2 连接历史查询

#### TC-RECORD-001: 查询连接历史记录

**优先级**: P1

**测试步骤**:
```bash
curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/records?start_date=2026-06-01&end_date=2026-06-30" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
```json
{
  "records": [
    {
      "id": "d0000000-0000-0000-0000-000000000001",
      "controller_code": "100200301",
      "controlled_code": "100200304",
      "path": "L1",
      "started_at": "2026-06-29T10:00:00Z",
      "ended_at": "2026-06-29T10:30:00Z",
      "duration_sec": 1800,
      "bytes_transferred": 524288000
    }
  ],
  "total": 3
}
```

**验证点**:
- ✅ 返回指定时间范围内的记录
- ✅ 包含数据传输量

---

### 4.3 导出连接记录

#### TC-RECORD-101: 导出 CSV 格式报表

**优先级**: P2

**测试步骤**:
```bash
curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/records/export?format=csv" \
  -H "Authorization: Bearer $TOKEN" \
  -o connections.csv
```

**验证点**:
- ✅ HTTP 状态码 = 200
- ✅ Content-Type = text/csv
- ✅ 文件包含所有字段

---

## 5. 仪表板统计模块 (Dashboard)

### 5.1 统计数据

#### TC-DASH-001: 获取仪表板统计

**优先级**: P1

**测试步骤**:
```bash
curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/dashboard/stats" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
```json
{
  "total_devices": 5,
  "online_devices": 3,
  "total_members": 3,
  "active_sessions": 1,
  "total_connections_today": 12,
  "data_transferred_today": 10737418240
}
```

**验证点**:
- ✅ 返回实时统计数据
- ✅ 数值准确

---

### 5.2 连接趋势

#### TC-DASH-101: 查询连接趋势

**优先级**: P2

**测试步骤**:
```bash
curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/dashboard/trends?days=7" \
  -H "Authorization: Bearer $TOKEN"
```

**预期结果**:
```json
{
  "trends": [
    {"date": "2026-06-23", "connections": 5},
    {"date": "2026-06-24", "connections": 8},
    {"date": "2026-06-25", "connections": 12}
  ]
}
```

**验证点**:
- ✅ 返回指定天数的趋势数据
- ✅ 按日期排序

---

## 6. WebSocket 实时通信

### 6.1 WebSocket 连接

#### TC-WS-001: 建立 WebSocket 连接

**优先级**: P0

**测试步骤**:
```bash
# 使用 websocat 工具测试
websocat "ws://localhost:8080/api/v1/ws?token=$TOKEN"
```

**预期行为**:
- ✅ 连接成功建立
- ✅ 接收心跳消息
- ✅ 接收设备状态变更事件

---

## 7. 权限控制测试

### 7.1 角色权限验证

#### TC-AUTH-ROLE-001: Member 角色访问限制

**优先级**: P0

**测试步骤**:
```bash
# 使用 Member 角色登录
TOKEN_MEMBER="<member_token>"

# 尝试邀请新成员（应失败）
curl -X POST "http://localhost:8080/api/v1/teams/$TEAM_ID/invite" \
  -H "Authorization: Bearer $TOKEN_MEMBER" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "test@example.com",
    "name": "Test",
    "role": "member"
  }'
```

**预期结果**:
```json
{
  "error": "insufficient_permissions"
}
```

**验证点**:
- ✅ HTTP 状态码 = 403
- ✅ Member 无法邀请新成员

---

## 8. 性能测试

### 8.1 并发登录测试

#### TC-PERF-001: 100 并发登录

**优先级**: P2

**测试工具**: Apache Bench 或 wrk

**测试步骤**:
```bash
ab -n 1000 -c 100 -p login.json -T application/json \
  http://localhost:8080/api/v1/auth/login
```

**验证点**:
- ✅ 成功率 > 99%
- ✅ 平均响应时间 < 500ms
- ✅ P99 < 2s

---

## 9. 安全测试

### 9.1 SQL 注入防护

#### TC-SEC-001: SQL 注入尝试

**优先级**: P0

**测试步骤**:
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "admin@rdcs-test.local'\'' OR 1=1--",
    "password": "anything"
  }'
```

**预期结果**:
```json
{
  "error": "invalid_credentials"
}
```

**验证点**:
- ✅ 返回通用错误，不暴露系统信息
- ✅ 不发生 SQL 注入

---

### 9.2 JWT 令牌安全

#### TC-SEC-101: 伪造 JWT 令牌

**优先级**: P0

**测试步骤**:
```bash
# 使用伪造的 token
FAKE_TOKEN="eyJhbGciOiJub25lIn0.eyJzdWIiOiIxMjM0NTY3ODkwIn0."

curl -X GET "http://localhost:8080/api/v1/teams/$TEAM_ID/devices" \
  -H "Authorization: Bearer $FAKE_TOKEN"
```

**预期结果**:
```json
{
  "error": "invalid_token"
}
```

**验证点**:
- ✅ HTTP 状态码 = 401
- ✅ 拒绝伪造的令牌

---

## 10. 测试执行清单

### 自动化测试

```bash
# 运行所有单元测试
cd services/api
go test ./... -v -cover

# 运行集成测试
go test ./internal/server/... -tags=integration -v

# 生成覆盖率报告
go test ./... -coverprofile=coverage.out
go tool cover -html=coverage.out -o coverage.html
```

### 手动测试清单

- [ ] TC-AUTH-001 ~ TC-AUTH-005 (登录功能)
- [ ] TC-AUTH-101 (TOTP 两步验证)
- [ ] TC-DEVICE-001 ~ TC-DEVICE-003 (设备列表)
- [ ] TC-DEVICE-101 ~ TC-DEVICE-102 (设备详情)
- [ ] TC-DEVICE-201 (设备注册)
- [ ] TC-DEVICE-301 (设备删除)
- [ ] TC-MEMBER-001 (成员列表)
- [ ] TC-MEMBER-101 (邀请成员)
- [ ] TC-MEMBER-201 (更新角色)
- [ ] TC-MEMBER-301 (移除成员)
- [ ] TC-SESSION-001 (活跃会话)
- [ ] TC-RECORD-001 (连接历史)
- [ ] TC-RECORD-101 (导出 CSV)
- [ ] TC-DASH-001 (统计数据)
- [ ] TC-DASH-101 (连接趋势)
- [ ] TC-WS-001 (WebSocket 连接)
- [ ] TC-AUTH-ROLE-001 (权限控制)
- [ ] TC-PERF-001 (并发测试)
- [ ] TC-SEC-001 (SQL 注入防护)
- [ ] TC-SEC-101 (JWT 安全)

---

## 附录

### A. 测试环境重置

```bash
# 重置数据库
docker exec -it rdcs-postgres psql -U rdcs -d rdcs -f /docker-entrypoint-initdb.d/001_init_schema.sql
docker exec -it rdcs-postgres psql -U rdcs -d rdcs -f /docker-entrypoint-initdb.d/002_seed_data.sql

# 更新测试密码
docker exec -it rdcs-postgres psql -U rdcs -d rdcs -c "UPDATE members SET password_hash = '\$2a\$10\$mmaNLZS/sqGHkQ5OKeRCG.o5EnYWmVDsZKop78CbD/M.8c7vZmQQC';"
```

### B. 常用测试工具

- **curl**: HTTP 请求测试
- **jq**: JSON 数据处理
- **websocat**: WebSocket 测试
- **Apache Bench (ab)**: 性能测试
- **Postman**: API 集成测试
- **go test**: Go 单元测试

### C. 问题追踪

| Issue ID | 测试用例 | 问题描述 | 状态 | 修复版本 |
|----------|---------|---------|------|---------|
| #001 | TC-AUTH-001 | 种子数据密码哈希错误 | ✅ Fixed | v0.1.1 |
| #002 | TC-AUTH-001 | JWT 密钥未配置 | 🔄 In Progress | v0.1.1 |

---

**文档维护者**: RDCS Team  
**最后更新**: 2026-06-29
