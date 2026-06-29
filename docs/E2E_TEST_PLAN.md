# E2E 测试计划

**版本**: 2.0  
**更新日期**: 2026-06-28  
**原则**: Superpowers — 垂直切片，MVP 优先，单一信息源

---

## 前置说明

### 机器角色

| 机器 | 架构 | 角色 | 备注 |
|------|------|------|------|
| 当前开发机 | Apple Silicon (ARM) | 主力调试机 | 所有调试以此为准 |
| 辅助机 | Intel (x86_64) | 辅助测试机 | 用于跨架构验证 |

### 应用架构

同一个 binary，两种启动模式：

```
rdcs-desktop serve              # 被控端：捕获屏幕并接受控制
rdcs-desktop connect <IP>       # 主控端：显示远端屏幕并发送输入
```

### 测试前提：Phase 依赖关系

```
T1（单机）→ T2（双进程）→ T3（局域网）→ T4（跨架构）→ T5（验收）
  必须通过才能进入下一层
```

---

## T1 — 单机回环测试

**验证**: 视频管道在本地完整工作  
**阶段前提**: Phase 2 编解码 + 显示已就绪  
**执行机器**: Apple Silicon Mac（开发机）

### 执行方式

```bash
cargo run --example display_roundtrip --features software-encoder --release
```

### 通过标准

| 指标 | 要求 |
|------|------|
| 能看到动画画面 | ✅ 必须 |
| 帧率 | ≥ 24 FPS |
| 端到端延迟（编码+解码+显示） | < 100ms |
| 无崩溃 | 运行 60s 无异常 |

### 当前状态

`display_roundtrip` 示例已存在，待在实机执行。

---

## T2 — 双进程本地测试

**验证**: serve/connect 两种模式能在同一台机器上通信  
**阶段前提**: `rdcs-desktop` binary 可编译运行  
**执行机器**: Apple Silicon Mac（开发机）

### 执行方式

```bash
# Terminal 1 — 被控端模式
./target/release/rdcs-desktop serve --port 7000

# Terminal 2 — 主控端模式
./target/release/rdcs-desktop connect 127.0.0.1 --port 7000
```

### 通过标准

| 指标 | 要求 |
|------|------|
| 两个进程能握手建立连接 | ✅ 必须 |
| Controller 窗口显示视频 | ✅ 必须 |
| 帧率 | ≥ 24 FPS |
| 端到端延迟 | < 100ms |
| 稳定运行 | 5 分钟无断线 |

### 阻塞项

- `rdcs-desktop` binary 骨架已创建，agent/controller 逻辑待实现
- 需要完成 ICE 信令握手流程

---

## T3 — 局域网双机测试

**验证**: 两台 Mac 在同一局域网下远程桌面可用  
**阶段前提**: T2 通过  
**执行机器**: 两台 Mac，同一 Wi-Fi/有线网络

### 部署方式

```bash
# Intel Mac — 被控端
scp target/release/rdcs-desktop user@intel-mac:~/
ssh user@intel-mac
./rdcs-desktop serve --port 7000

# Apple Silicon Mac — 主控端（开发机）
./target/release/rdcs-desktop connect <Intel-Mac-IP> --port 7000
```

### 通过标准

| 指标 | 要求 |
|------|------|
| 能看到 Intel Mac 的桌面 | ✅ 必须 |
| 帧率 | ≥ 24 FPS |
| 端到端延迟（含网络） | < 150ms |
| 稳定运行 | 10 分钟无断线 |
| 断线后能重连 | ✅ 必须 |

---

## T4 — 跨架构兼容性测试

**验证**: ARM 和 x86_64 之间编解码、字节序无兼容问题  
**阶段前提**: T3 通过  
**执行机器**: 同 T3，额外增加反向测试

### 测试矩阵

| 被控端（serve） | 主控端（connect） | 必须通过 |
|---------------|-----------------|---------|
| Intel Mac | Apple Silicon Mac | ✅ 正向 |
| Apple Silicon Mac | Intel Mac | ✅ 反向 |

### 通过标准

两个方向均满足 T3 通过标准，且互相编解码无花屏/音视频不同步。

---

## T5 — MVP 验收测试

**验证**: 满足 MVP 定义："用户能从 Apple Silicon Mac 远程查看和控制 Intel Mac 的桌面"  
**阶段前提**: T4 通过，Phase 3（输入控制）完成  
**执行者**: 开发者扮演真实用户，按使用手册操作

### 场景脚本

1. 打开 Intel Mac，不做任何配置，启动 `rdcs-desktop serve`
2. 在 Apple Silicon Mac 启动 `rdcs-desktop connect <IP>`
3. 能看到 Intel Mac 的屏幕
4. 移动鼠标：Intel Mac 光标跟随移动
5. 点击应用：Intel Mac 响应点击
6. 输入文字：Intel Mac 接收键盘输入
7. 主动断开连接：两端均干净退出

### 通过标准

| 场景步骤 | 要求 |
|---------|------|
| 1–3 连接建立 | < 10 秒 |
| 屏幕延迟 | < 300ms |
| 帧率 | ≥ 24 FPS |
| 鼠标响应延迟 | < 100ms |
| 键盘响应 | 无丢字 |
| CPU 使用率（每台机器） | < 60% |
| 内存占用 | < 500MB |
| 稳定运行 | 30 分钟无崩溃 |

---

## 当前状态总览

| 测试层 | 状态 | 阻塞原因 |
|--------|------|---------|
| T1 单机回环 | 🔄 待执行 | 示例已就绪，需在实机跑 |
| T2 双进程 | ❌ 不可执行 | rdcs-desktop agent/controller 逻辑未实现 |
| T3 局域网 | ❌ 不可执行 | 依赖 T2 |
| T4 跨架构 | ❌ 不可执行 | 依赖 T3 |
| T5 MVP 验收 | ❌ 不可执行 | 依赖 Phase 3 输入控制 |

---

## 执行顺序建议

```
本周: T1 → 修复问题 → 推进 rdcs-desktop 实现
下周: T2 → T3
三周后: T4
Phase 3 完成后: T5
```

---

**维护原则**: 每个测试层通过后在此文档记录结果和日期，不通过记录失败原因和修复 PR。
