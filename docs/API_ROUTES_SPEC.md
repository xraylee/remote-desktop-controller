# RDCS API 路由规范

## 概述

RDCS API 采用 RESTful 设计，所有资源路由都需要 `teamID` 作为路径参数，实现多租户隔离。

## 基础信息

- **Base URL**: `http://localhost:8080/api/v1`
- **认证方式**: JWT Bearer Token (除公开端点外)
- **Content-Type**: `application/json`

## 路由结构

### 公开端点（无需认证）

```
POST /api/v1/auth/login
```

### 受保护端点（需要 JWT）

所有受保护的资源端点都遵循以下模式：

```
/api/v1/teams/{teamID}/<resource>
```

其中 `teamID` 是用户所属团队的 UUID。

## 端点列表

### 1. 认证 (Authentication)

#### 登录
```http
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123",
  "totp_code": "123456"  // 可选，启用 2FA 时必填
}
```

**响应**:
```json
{
  "access_token": "eyJhbGc...",
  "refresh_token": "eyJhbGc...",
  "expires_in": 3600,
  "member": {
    "id": "uuid",
    "team_id": "uuid",
    "name": "John Doe",
    "email": "user@example.com",
    "role": "admin",
    "created_at": "2026-01-01T00:00:00Z"
  }
}
```

#### TOTP 管理
```http
POST /api/v1/auth/totp/setup
POST /api/v1/auth/totp/verify
POST /api/v1/auth/totp/disable
```

### 2. Dashboard (仪表板)

#### 获取统计数据
```http
GET /api/v1/teams/{teamID}/dashboard/stats
Authorization: Bearer {access_token}
```

**响应**:
```json
{
  "online_devices": 10,
  "active_sessions": 3,
  "total_members": 5,
  "today_connections": 15
}
```

#### 获取连接趋势
```http
GET /api/v1/teams/{teamID}/dashboard/trends?days=7
Authorization: Bearer {access_token}
```

**响应**:
```json
[
  { "date": "2026-01-01", "count": 5 },
  { "date": "2026-01-02", "count": 8 }
]
```

#### 获取最近活动
```http
GET /api/v1/teams/{teamID}/dashboard/activities?limit=10
Authorization: Bearer {access_token}
```

**响应**:
```json
[
  {
    "id": "uuid",
    "type": "connection",
    "device_code": "ABC123",
    "device_name": "MacBook Pro",
    "timestamp": 1609459200,
    "user_name": "John Doe"
  }
]
```

### 3. Devices (设备管理)

#### 列出设备
```http
GET /api/v1/teams/{teamID}/devices?status=online
Authorization: Bearer {access_token}
```

#### 创建设备
```http
POST /api/v1/teams/{teamID}/devices
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "device_code": "ABC123",
  "device_name": "MacBook Pro",
  "platform": "macos"
}
```

#### 获取设备详情
```http
GET /api/v1/teams/{teamID}/devices/{deviceCode}
Authorization: Bearer {access_token}
```

#### 删除设备
```http
DELETE /api/v1/teams/{teamID}/devices/{deviceCode}
Authorization: Bearer {access_token}
```

#### 设备操作
```http
POST /api/v1/teams/{teamID}/devices/{deviceCode}/kick
POST /api/v1/teams/{teamID}/devices/{deviceCode}/disable
POST /api/v1/teams/{teamID}/devices/{deviceCode}/enable
Authorization: Bearer {access_token}
```

### 4. Members (成员管理)

#### 列出成员
```http
GET /api/v1/teams/{teamID}/members
Authorization: Bearer {access_token}
```

#### 邀请成员
```http
POST /api/v1/teams/{teamID}/invite
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "email": "newuser@example.com",
  "role": "operator",
  "name": "Jane Doe"
}
```

#### 更新成员
```http
PUT /api/v1/teams/{teamID}/members/{memberID}
Authorization: Bearer {access_token}
Content-Type: application/json

{
  "role": "admin"
}
```

#### 删除成员
```http
DELETE /api/v1/teams/{teamID}/members/{memberID}
Authorization: Bearer {access_token}
```

### 5. Sessions (会话管理)

#### 列出会话
```http
GET /api/v1/teams/{teamID}/sessions?page=1&page_size=20
Authorization: Bearer {access_token}
```

#### 导出会话 CSV
```http
GET /api/v1/teams/{teamID}/sessions/export?time_range=week
Authorization: Bearer {access_token}
```

### 6. Connection Records (连接记录)

#### 列出连接记录
```http
GET /api/v1/teams/{teamID}/records?page=1&page_size=20&time_range=week&search=ABC123
Authorization: Bearer {access_token}
```

**查询参数**:
- `page`: 页码（默认 1）
- `page_size`: 每页大小（默认 20，最大 100）
- `time_range`: 时间范围 (`today` | `week` | `month`)
- `search`: 搜索关键词（设备代码）

**响应**:
```json
{
  "records": [
    {
      "id": "uuid",
      "controller_code": "ABC123",
      "controlled_code": "DEF456",
      "path": "direct",
      "started_at": "2026-01-01T10:00:00Z",
      "ended_at": "2026-01-01T10:30:00Z",
      "duration_sec": 1800,
      "bytes_transferred": 1048576
    }
  ],
  "total": 100,
  "page": 1,
  "page_size": 20
}
```

#### 导出连接记录 CSV
```http
GET /api/v1/teams/{teamID}/records/export?time_range=week&search=ABC123
Authorization: Bearer {access_token}
```

### 7. Audit Logs (审计日志)

#### 列出审计日志
```http
GET /api/v1/teams/{teamID}/audit-logs?page=1&page_size=20&action=device.connect
Authorization: Bearer {access_token}
```

**查询参数**:
- `page`: 页码
- `page_size`: 每页大小
- `action`: 操作类型过滤
- `start_time`: 开始时间 (RFC3339)
- `end_time`: 结束时间 (RFC3339)

### 8. WebSocket (实时事件推送)

```http
GET /api/v1/ws?token={access_token}
```

WebSocket 连接用于实时接收以下事件：
- 设备上线/离线
- 会话开始/结束
- 成员操作通知

## 前端集成指南

### 1. 使用 `buildTeamPath()` 工具函数

```typescript
import { buildTeamPath } from '@/api/teamApi'

// 自动注入当前用户的 team_id
const response = await apiClient.get(buildTeamPath('dashboard/stats'))
// 实际请求: GET /api/v1/teams/{currentTeamID}/dashboard/stats
```

### 2. 获取当前 team_id

```typescript
import { getCurrentTeamId } from '@/api/teamApi'

const teamId = getCurrentTeamId()
```

### 3. 完整示例

```typescript
// DashboardPage.tsx
import { useQuery } from '@tanstack/react-query'
import { apiClient } from '@/api/client'
import { buildTeamPath } from '@/api/teamApi'

const { data: stats } = useQuery({
  queryKey: ['dashboard-stats'],
  queryFn: async () => {
    const res = await apiClient.get(buildTeamPath('dashboard/stats'))
    return res.data
  },
})
```

## 错误处理

### 通用错误响应

```json
{
  "error": "error_code",
  "message": "Human readable error message",
  "details": {}
}
```

### 常见错误码

- `400 Bad Request`: 请求参数无效
- `401 Unauthorized`: 未认证或 token 无效
- `403 Forbidden`: 权限不足
- `404 Not Found`: 资源不存在
- `429 Too Many Requests`: 请求频率超限
- `500 Internal Server Error`: 服务器内部错误

## 版本历史

- **v1** (2026-01-01): 初始版本
  - 多租户架构，所有资源按 team 隔离
  - JWT 认证
  - TOTP 二步验证支持
