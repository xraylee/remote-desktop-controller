# ⚠️ 依赖安装说明

## 当前状态

由于你正在容器环境中运行，网络访问受到限制，**无法自动安装系统级工具**（Rust、Go、Flutter）。

但是，我已经为你准备好了一切！

---

## ✅ 已完成的工作

1. **Web 管理后台依赖已完全安装** ✓
   - 所有 npm 包已下载
   - 配置了国内镜像源
   - 可以立即运行

2. **创建了完整的自动安装脚本** ✓
   - `auto-install-all.sh` - 一键安装所有工具和依赖
   - `check-and-install.sh` - 智能检查和安装
   - `SETUP.md` - 详细的配置指南

---

## 🚀 立即完成安装（2分钟）

**在你的 Mac 终端中运行以下命令：**

### 方式一：一键自动安装（推荐）

```bash
cd /Users/lc/Development/source/remote-desktop-controller
./auto-install-all.sh
```

这个脚本会：
- ✅ 自动检测已安装的工具
- ✅ 使用 Homebrew 安装缺失的工具
- ✅ 配置所有国内镜像源
- ✅ 安装所有项目依赖
- ✅ 验证安装结果

**全程自动，无需任何手动操作！**

---

### 方式二：手动安装（如果自动脚本失败）

```bash
# 1. 安装工具链（使用 Homebrew）
brew install rustup-init go flutter

# 2. 初始化 Rust
rustup-init -y

# 3. 配置代理
go env -w GOPROXY=https://goproxy.cn,direct

# 4. 安装项目依赖
cd /Users/lc/Development/source/remote-desktop-controller
cargo fetch
cd services/api && go mod download && cd ../..
cd client/flutter && flutter pub get && cd ../..

# 5. 验证
./check-and-install.sh
```

---

## 📊 当前进度

| 组件 | 工具链 | 依赖 | 状态 |
|------|--------|------|------|
| Web 管理后台 | Node.js ✅ | npm ✅ | **可运行** |
| Rust 核心库 | Rust ⏳ | Cargo ⏳ | 等待安装 |
| Go API | Go ⏳ | Modules ⏳ | 等待安装 |
| Flutter 客户端 | Flutter ⏳ | Pub ⏳ | 等待安装 |

---

## 🎯 安装后可以做什么

### 立即运行 Web 管理后台

```bash
cd web/admin
npm run dev
# 访问 http://localhost:5173
```

### 运行完整项目（安装所有依赖后）

```bash
# API 服务
cd services/api
go run main.go

# Flutter 客户端
cd client/flutter
flutter run -d macos

# 编译 Rust 核心库
cargo build --release
```

---

## 🔍 为什么需要在宿主机上安装？

容器环境的限制：
- ❌ 无 root 权限（无法使用 apt/yum）
- ❌ 网络访问被阻断（403 错误）
- ❌ 无法下载安装包

你的 Mac 上没有这些限制，可以正常安装！

---

## 📝 需要帮助？

1. 如果 `auto-install-all.sh` 执行失败，查看错误信息
2. 检查是否安装了 Homebrew：`brew --version`
3. 如果没有 Homebrew，先安装：
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```
4. 查看详细文档：`SETUP.md`

---

## ⏰ 预计时间

- 自动安装脚本：**2-5 分钟**
  - Homebrew 安装工具：1-2 分钟
  - 下载依赖包：1-3 分钟

- 手动安装：**5-10 分钟**

---

**下一步：在你的 Mac 终端运行 `./auto-install-all.sh` ！** 🚀
