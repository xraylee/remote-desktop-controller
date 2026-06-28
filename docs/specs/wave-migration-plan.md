# Wave 执行计划设计 — 弃用 OpenSpec 迁移至 Superpowers

**版本**: v1.0
**日期**: 2026-06-26
**状态**: 已确认

---

## 0. 范围排除

以下功能明确不在本次 MVP 实施范围内（推迟至 V1.1+）：

- macOS AudioCapture / SystemNotify（保留 stub，不影响远程控制核心流程）
- 二维码连接（随微信小程序上线）
- Windows / Linux 平台实现（MVP 仅 macOS）
- 移动端（iOS / Android）
- 多人协作模式、远程打印
- 手机投屏、Android 免 Root 被控

Wave 1 Task 1.5 因此从计划中移除。

---

## 1. 背景

项目当前使用 OpenSpec 管理变更，但存在以下问题：
- `remote-desktop-system` change 的 design.md 和 tasks.md 始终未生成
- 10 个 spec 中仅 9 个有内容（`security-privacy` 为空）
- OpenSpec CLI 流程与实际开发节奏脱节

迁移至 superpowers 工作流后，每个子系统按 Wave 推进：brainstorming → design → writing-plans → 实施 → 验证，并在每轮 Wave 结束时执行进度检查循环。

---

## 2. Wave 划分

### Wave 1：集成修复（关键路径，阻塞后续 Wave）

| Task | 内容 | 预估 |
|------|------|------|
| 1.1 | 修复 FFI 命名不一致：Flutter `bindings.dart` 中 `engine_create` 等函数名对齐 Rust 侧 `rdcs_engine_create` 导出 | 0.5d |
| 1.2 | 补写 `integration_transport.rs`：可靠 UDP 传输、NACK、FEC 端到端测试 | 1d |
| 1.3 | 补写 `integration_connection.rs`：ICE 协商、路径选择、心跳重连端到端测试 | 1d |
| 1.4 | 补写 `integration_transfer.rs`：文件分块传输、SHA-256 校验、断点续传端到端测试 | 1d |

**依赖**: 无（独立可执行）
**预估工期**: ~3.5d
**完成标准**: `cargo test --workspace` 全部通过，FFI 调用链可跑通端到端 smoke test

### Wave 2：Flutter 客户端（最大缺口）

| Task | 内容 | 预估 |
|------|------|------|
| 2.1 | 会话界面：远程画面渲染、工具栏（画质/全屏/断开）、延迟/帧率显示 | 3d |
| 2.2 | 连接确认对话框：被控端入站请求弹窗、并发请求队列 | 1d |
| 2.3 | 被控端浮动条 + 会话结束 toast（不参与屏幕捕获） | 1d |
| 2.4 | 设置页面：安全（改密码/TOTP）、网络（代理/中继）、关于 | 2d |
| 2.5 | 系统托盘常驻 + 开机自启（macOS Launch Agent / Windows Registry） | 1.5d |
| 2.6 | Riverpod 全局状态：connectionProvider、deviceProvider、settingsProvider | 2d |
| 2.7 | Widget 测试覆盖关键页面（主页、会话页、设置页） | 1.5d |

**依赖**: Wave 1（FFI 命名修复后才可联调）
**完成标准**: `flutter test` 通过；macOS 上可发起/接受远程控制完整流程

### Wave 3：React Web 控制台

| Task | 内容 | 预估 |
|------|------|------|
| 3.1 | API client 层：TanStack Query hooks 对接 Go API 全部端点 | 2d |
| 3.2 | Dashboard 仪表盘：统计卡片、在线设备图表、最近会话列表 | 2d |
| 3.3 | 设备管理页：表格/搜索/分组/详情/批量操作 | 2.5d |
| 3.4 | 会话/审计页：连接记录表格、CSV 导出、审计日志搜索 | 2d |
| 3.5 | 录制回放页：视频播放器、时间轴、搜索 | 2d |
| 3.6 | 成员管理 + 系统设置页：用户 CRUD、TOTP 设置、系统配置 | 2d |

**依赖**: Wave 1（API 集成修复），与 Wave 2 可并行
**完成标准**: `npm run build` 无错误；所有页面可完成 CRUD 操作

---

## 3. 进度检查循环

每个 Wave 结束后执行完整检查循环：

```
1. cargo test --workspace       → Rust 层全部测试
2. go test ./... (services/api) → Go API 测试
3. flutter test (client/flutter)→ Flutter 测试
4. npm run build (web/admin)    → Web 构建检查
5. 统计各 Wave task 完成数 / 总数
6. 与 baseline（当前 44/62 = 71%）对比
7. 输出进度报告（格式：Wave N 完成 X/Y tasks | 整体完成度 Z% | 下一 Wave 启动条件 ✓/✗），决定下一 Wave 启动
```

Wave 内部每完成一个 task 做轻量检查：仅运行该 task 涉及 crate/module 的测试。

---

## 4. 目录结构

```
docs/specs/
├── architecture-design.md          # 已有
├── prd-v1.md                       # 已有
├── prd-review-report.md            # 已有
├── wave-migration-plan.md          # ← 本文档
└── waves/
    ├── wave-1-integration-fixes.md  # Wave 1 设计 + 实施计划
    ├── wave-2-flutter-client.md     # Wave 2 设计 + 实施计划
    └── wave-3-web-console.md        # Wave 3 设计 + 实施计划
```

每个 wave 文件包含：设计决策 + task 列表 + 验收标准，一个文件覆盖 brainstorming → plan → 实施追踪全流程。

---

## 5. 执行顺序

```
Wave 1 (集成修复, ~3.5d)
    │
    ├──► Wave 2 (Flutter, ~12d)  ─┐
    │                               ├──► 最终进度检查 → MVP 就绪
    └──► Wave 3 (React, ~12.5d) ─┘
```

Wave 2 和 Wave 3 无互相依赖，可并行推进。
