# RDCS Scripts 目录说明

**更新日期**: 2026-06-29  
**状态**: 已整理

本目录包含 RDCS 项目的所有脚本工具，按功能分类。

---

## 📁 目录结构

```
scripts/
├── build/          # 构建相关脚本（5个）
├── testing/        # 测试相关脚本（7个）
├── deployment/     # 部署运维脚本（4个）
├── diagnostics/    # 诊断工具（3个）
├── tools/          # 开发工具（2个）
├── installation/   # 安装脚本（3个）
├── e2e/            # E2E 测试（已有）
├── validation/     # 验证脚本（已有）
└── archived/       # 归档脚本（2个）
```

---

## 🏗️ build/ - 构建脚本

| 脚本 | 用途 |
|------|------|
| `build_and_run.sh` | Flutter + Rust FFI 完整构建 |
| `build_ffi.sh` | 构建并复制 Rust FFI 库到 Flutter |
| `build_ice_tools.sh` | 编译 ICE 跨网络测试工具 |
| `check_build.sh` | 快速构建检查 |
| `setup_xcode.sh` | 配置 Xcode 项目自动复制 FFI 库 |

**常用命令**:
```bash
# 完整构建
./scripts/build/build_and_run.sh

# 仅构建 FFI 库
./scripts/build/build_ffi.sh

# 快速检查构建状态
./scripts/build/check_build.sh
```

---

## 🧪 testing/ - 测试脚本

| 脚本 | 用途 |
|------|------|
| `test_api.sh` | API 完整测试 |
| `test_controller.sh` | Apple Silicon Mac（主控端）测试 |
| `test_target.sh` | Intel Mac（被控端）测试 |
| `test_hardware_encoder.sh` | 硬件编码器性能对比测试 |
| `run_real_screen_capture_test.sh` | 真实屏幕捕获测试 |
| `verify-test-docs.sh` | 验证测试文档完整性 |
| `TEST_COMMANDS.sh` | 测试命令快速参考（可 source） |

**常用命令**:
```bash
# API 测试
./scripts/testing/test_api.sh

# 主控端测试
./scripts/testing/test_controller.sh

# 被控端测试
./scripts/testing/test_target.sh

# 查看所有测试命令
cat ./scripts/testing/TEST_COMMANDS.sh
```

---

## 🚀 deployment/ - 部署脚本

| 脚本 | 用途 |
|------|------|
| `deploy_backend.sh` | 后端服务快速部署（Docker Compose） |
| `deploy_minimal.sh` | 最小化部署（仅启动数据库） |
| `logs_backend.sh` | 查看后端服务日志 |
| `stop_backend.sh` | 停止后端服务 |

**常用命令**:
```bash
# 启动后端服务
./scripts/deployment/deploy_backend.sh

# 查看日志
./scripts/deployment/logs_backend.sh

# 停止服务
./scripts/deployment/stop_backend.sh
```

---

## 🔧 diagnostics/ - 诊断工具

| 脚本 | 用途 |
|------|------|
| `diagnose_auth.sh` | 认证系统全面诊断 |
| `quick-fix.sh` | Flutter FFI 连接快速修复 |
| （其他已有脚本） | OpenH264、TCP 等诊断工具 |

**常用命令**:
```bash
# 诊断认证问题
./scripts/diagnostics/diagnose_auth.sh

# 快速修复 FFI 连接
./scripts/diagnostics/quick-fix.sh
```

---

## 🛠️ tools/ - 开发工具

| 脚本 | 用途 |
|------|------|
| `setup_environment.sh` | 完整开发环境安装 |
| `quick_start.sh` | 快速启动（构建 Rust + 运行 Flutter） |

**常用命令**:
```bash
# 首次安装环境
./scripts/tools/setup_environment.sh

# 日常快速启动
./scripts/tools/quick_start.sh
```

---

## 📦 installation/ - 安装脚本

已有的安装相关脚本，按需使用。

---

## 🗄️ archived/ - 归档脚本

| 脚本 | 原因 |
|------|------|
| `git_commit.sh` | 会话临时提交脚本 |
| `git_commit_phase4.1.sh` | Phase 4.1 临时提交脚本 |

这些脚本已完成历史使命，保留作为参考。

---

## 💡 使用建议

### 新手入门
1. 安装环境：`./scripts/tools/setup_environment.sh`
2. 快速启动：`./scripts/tools/quick_start.sh`
3. 运行测试：`./scripts/testing/test_api.sh`

### 日常开发
1. 构建 FFI：`./scripts/build/build_ffi.sh`
2. 运行测试：`./scripts/testing/test_controller.sh`
3. 查看日志：`./scripts/deployment/logs_backend.sh`

### 问题排查
1. 认证问题：`./scripts/diagnostics/diagnose_auth.sh`
2. FFI 问题：`./scripts/diagnostics/quick-fix.sh`
3. 构建检查：`./scripts/build/check_build.sh`

---

## 📝 维护规范

1. **新增脚本** - 按功能放入对应子目录
2. **临时脚本** - 完成后移入 archived/
3. **脚本命名** - 使用小写 + 下划线或连字符
4. **添加注释** - 脚本顶部说明用途和用法
5. **可执行权限** - 记得 `chmod +x`

---

**整理日期**: 2026-06-29  
**整理依据**: Superpowers 规范 - 脚本按功能分类  
**整理人**: Claude (Superpowers Agent)
