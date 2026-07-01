# 邀请码生成功能联调指南

**日期**: 2026-06-30  
**功能**: Flutter 客户端生成 4 位随机邀请码  
**状态**: ✅ 代码已实现，待验证

---

## 一、功能概述

### 实现位置

| 层次 | 文件 | 实现内容 |
|------|------|---------|
| **Rust FFI** | `crates/rdcs-ffi/src/lib.rs:712-734` | `rdcs_generate_invite()` - 生成随机 4 位数字 |
| **Flutter 调用** | `client/flutter/lib/features/home/home_page.dart:199-241` | `_generateInviteCode()` - UI 交互逻辑 |
| **FFI 桥接** | `client/flutter/lib/core/ffi/engine_isolate.dart` | `generateInvite()` - 异步调用 FFI |

### 工作流程

```
用户点击「生成邀请码」
    ↓
HomePage._generateInviteCode()
    ↓
EngineIsolate.generateInvite()
    ↓
FFI: rdcs_generate_invite(handle)
    ↓
生成随机 4 位数字（0000-9999）
    ↓
显示弹窗，支持复制
```

---

## 二、服务器配置说明

### 客户端设置 → 网络选项卡

| 配置项 | 本地开发环境 | 生产环境示例 |
|--------|-------------|-------------|
| **信令服务器** | `ws://127.0.0.1:8443` | `wss://signal.yourdomain.com` |
| **中继服务器** | `turn:127.0.0.1:3478` | `turn://relay.yourdomain.com:3478` |
| **管理 API** | `http://127.0.0.1:8080` | `https://api.yourdomain.com` |

#### 说明

**信令服务器 (Signaling Server)**
- 端口: 8443
- 协议: WebSocket (ws:// 或 wss://)
- 功能: 设备注册、邀请码验证、P2P 协商
- 服务: `rdcs-signaling` (Rust)

**中继服务器 (Relay Server)**
- 端口: 3478
- 协议: TURN (turn://)
- 功能: P2P 失败时的流量中继
- 状态: ⚠️ Phase 3 计划中，目前可留空

**管理 API (Management API)**
- 端口: 8080
- 协议: HTTP/HTTPS
- 功能: 设备管理、会话历史、用户认证
- 服务: `services/api` (Go)

---

## 三、前置条件检查

### 3.1 Rust FFI 库

```bash
# 检查库文件
ls -lh target/debug/librdcs_core.dylib

# 如果不存在，编译
cargo build -p rdcs-ffi

# 验证 rand 依赖
grep "rand = " crates/rdcs-ffi/Cargo.toml
# 应输出: rand = "0.8"
```

### 3.2 Flutter 配置初始化

```bash
# 验证 main.dart 中的初始化代码
grep -A 2 "await config.init()" client/flutter/lib/main.dart
```

应该看到：
```dart
// Initialize configuration (generates device code if not exists)
final config = container.read(configProvider.notifier);
await config.init();
```

### 3.3 配置文件状态

```bash
# 首次运行前
ls ~/.rdcs/config.json
# 如果不存在 → 正常，首次运行会自动生成

# 如果需要重置配置
rm -rf ~/.rdcs/
```

---

## 四、测试步骤

### 步骤 1: 启动 Flutter 应用

```bash
cd client/flutter
flutter run -d macos
```

**预期输出**:
```
✅ Configuration initialized. Device code: 123-456-789
```

如果看到设备代码，说明配置初始化成功。

### 步骤 2: 查看主页设备代码

应用启动后，主页应该显示：
- ✅ 设备代码格式: `XXX-XXX-XXX`（9 位数字）
- ✅ 状态: "设备已就绪"（绿色指示灯）
- ✅ 可点击代码复制

**如果显示 `--- --- ---`**:
- ❌ 说明配置初始化失败
- 检查终端是否有错误日志
- 检查 `~/.rdcs/` 目录权限

### 步骤 3: 生成邀请码

1. 点击「生成邀请码」按钮
2. 观察终端输出

**成功的终端输出**:
```
✅ Generated invite code: 3847
```

**成功的 UI 表现**:
- 弹出对话框，标题「邀请码」
- 显示 4 位数字，大字体，居中
- 有「复制」和「关闭」按钮
- 说明文字: "将邀请码分享给对方..."

**失败的表现**:
- SnackBar 显示「生成邀请码失败」
- 终端输出错误:
  ```
  ❌ rdcs_generate_invite: null handle
  ❌ rdcs_generate_invite: engine is shutdown
  ```

### 步骤 4: 测试复制功能

1. 点击对话框中的「复制」按钮
2. 对话框自动关闭
3. SnackBar 显示「邀请码已复制到剪贴板」
4. 粘贴到其他应用验证

### 步骤 5: 多次生成验证随机性

1. 连续点击「生成邀请码」3-5 次
2. 每次应该生成不同的 4 位数字
3. 范围: 0000 - 9999

---

## 五、故障排查

### 问题 1: FFI 库加载失败

**症状**:
```
Error: Unable to load dynamic library 'librdcs_core.dylib'
```

**解决方案**:
```bash
# 1. 检查库文件名
ls target/debug/librdcs_core.dylib

# 2. 检查 Cargo.toml
grep 'name = "rdcs_core"' crates/rdcs-ffi/Cargo.toml

# 3. 重新编译
cargo clean -p rdcs-ffi
cargo build -p rdcs-ffi
```

### 问题 2: 生成邀请码返回 null

**症状**:
```
❌ rdcs_generate_invite: null handle
EngineException(-1, "Failed to generate invite code")
```

**原因**: Engine 未正确初始化或已销毁

**解决方案**:
```dart
// 检查 main.dart 中的引擎初始化
final engine = container.read(engineIsolateProvider);
await engine.init();  // 必须在 config.init() 之前
```

### 问题 3: 设备代码显示 "--- --- ---"

**症状**: 主页不显示真实设备代码

**原因**: `configProvider.notifier.init()` 未调用

**解决方案**:
```bash
# 验证 main.dart
grep "config.init()" client/flutter/lib/main.dart

# 如果缺失，添加：
final config = container.read(configProvider.notifier);
await config.init();
```

### 问题 4: 邀请码总是相同

**症状**: 每次生成的邀请码都是 "0000"

**原因**: 使用了旧版本的 FFI 库（占位符实现）

**解决方案**:
```bash
# 检查 Rust 代码
grep -A 5 "rdcs_generate_invite" crates/rdcs-ffi/src/lib.rs

# 应该看到:
# use rand::Rng;
# let code = format!("{:04}", rng.gen_range(0..10000));

# 重新编译
cargo build -p rdcs-ffi

# 重启 Flutter 应用
```

---

## 六、当前实现限制

### ⚠️ 已知限制

1. **无服务器验证**
   - 邀请码仅在客户端生成
   - 未注册到信令服务器
   - 无过期时间管理

2. **无冲突检测**
   - 4 位数字空间较小（10,000 种组合）
   - 可能出现重复（生日悖论）

3. **无持久化**
   - 关闭对话框后邀请码丢失
   - 无历史记录

### 🚧 长期改进方案

#### Phase 3 计划（服务器端邀请码）

```rust
// 未来实现：向信令服务器注册邀请码
pub extern "C" fn rdcs_generate_invite(handle: *mut EngineHandle) -> *mut c_char {
    // 1. 生成邀请码
    let code = generate_random_code();
    
    // 2. 调用信令服务器 API
    let response = http_client
        .post("http://localhost:8080/api/invite/generate")
        .json(&InviteRequest {
            device_code: engine.device_code.clone(),
            ttl_seconds: 300, // 5 分钟过期
        })
        .send()
        .await?;
    
    // 3. 返回服务器确认的邀请码
    let invite: InviteResponse = response.json().await?;
    string_to_cstring(&invite.code)
}
```

#### 信令服务器 API（待实现）

```http
POST /api/invite/generate
Content-Type: application/json

{
  "device_code": "123456789",
  "ttl_seconds": 300
}

---

Response 200:
{
  "code": "3847",
  "expires_at": "2026-06-30T12:05:00Z",
  "device_code": "123456789"
}
```

---

## 七、验收清单

### 基本功能（当前实现）

- [ ] Flutter 应用正常启动
- [ ] 主页显示 9 位设备代码（格式 XXX-XXX-XXX）
- [ ] 点击「生成邀请码」按钮无崩溃
- [ ] 弹出对话框显示 4 位数字
- [ ] 终端输出 `✅ Generated invite code: XXXX`
- [ ] 点击「复制」按钮成功复制到剪贴板
- [ ] 连续生成 5 次，至少出现 3 个不同数字
- [ ] 邀请码范围在 0000-9999 之间

### 错误处理

- [ ] 引擎未初始化时显示友好错误提示
- [ ] 错误信息不崩溃应用

### 性能

- [ ] 生成邀请码响应时间 < 100ms
- [ ] 无内存泄漏（多次生成后内存稳定）

---

## 八、下一步工作

### 短期（本周）

1. **验证基本功能** - 按照本文档测试所有场景
2. **添加过期提示** - UI 上显示"邀请码有效期 5 分钟"
3. **邀请码历史** - 在设置页显示最近生成的 5 个邀请码

### 中期（Phase 3）

4. **服务器端验证** - 实现信令服务器 `/api/invite/generate` API
5. **过期管理** - Redis 存储邀请码，TTL 5 分钟
6. **连接流程** - 对方输入邀请码后验证有效性

### 长期（Phase 4）

7. **邀请码格式优化** - 改为 6 位字母数字混合（避免歧义字符）
8. **多设备支持** - 同一邀请码可被多个设备使用
9. **权限控制** - 邀请码关联权限（仅查看 / 完全控制）

---

## 九、相关文档

- [CLIENT_ANALYSIS.md](CLIENT_ANALYSIS.md) - 客户端架构分析
- [CLIENT_TEST_REPORT_2026-06-29.md](docs/testing/CLIENT_TEST_REPORT_2026-06-29.md) - 测试报告
- [crates/rdcs-ffi/src/lib.rs](crates/rdcs-ffi/src/lib.rs) - FFI 实现
- [client/flutter/lib/features/home/home_page.dart](client/flutter/lib/features/home/home_page.dart) - UI 实现

---

**维护者**: AI 助手  
**最后更新**: 2026-06-30  
**验证状态**: ⏳ 待手动验证
