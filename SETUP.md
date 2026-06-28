# RDCS 开发环境配置指南

本文档帮助你在本地机器上配置 RDCS 项目的完整开发环境。

## 📋 环境要求

RDCS 是一个多语言项目，需要以下工具：

- **Rust** (Cargo) - 核心库和底层组件
- **Go** 1.23+ - API 服务
- **Node.js** 18+ - Web 管理后台
- **Flutter** 3.4+ - 跨平台客户端

## ✅ 当前状态

已安装：
- ✅ Node.js v22.22.3
- ✅ npm 10.9.8
- ✅ Web 管理后台依赖（已配置国内镜像）

待安装：
- ⏳ Rust 工具链
- ⏳ Go 语言环境
- ⏳ Flutter SDK

---

## 🚀 快速安装（推荐）

### macOS 用户

```bash
# 1. 安装 Homebrew（如果还没有）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. 安装所有工具
brew install rustup-init go node flutter

# 3. 初始化 Rust
rustup-init -y

# 4. 配置镜像源（可选，但推荐）
# Rust 镜像（项目已配置）
# Go 镜像
go env -w GOPROXY=https://goproxy.cn,direct

# Flutter 镜像
export PUB_HOSTED_URL=https://pub.flutter-io.cn
export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn
echo 'export PUB_HOSTED_URL=https://pub.flutter-io.cn' >> ~/.zshrc
echo 'export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn' >> ~/.zshrc

# 5. 安装项目依赖
cd /Users/lc/Development/source/remote-desktop-controller
./check-and-install.sh
```

### Linux 用户

```bash
# 1. 安装 Rust（使用国内镜像）
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh
source $HOME/.cargo/env

# 2. 安装 Go
GO_VERSION="1.23.5"
wget https://mirrors.aliyun.com/golang/go${GO_VERSION}.linux-amd64.tar.gz
sudo tar -C /usr/local -xzf go${GO_VERSION}.linux-amd64.tar.gz
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
echo 'export PATH=$PATH:$HOME/go/bin' >> ~/.bashrc
source ~/.bashrc

# 配置 Go 镜像
go env -w GOPROXY=https://goproxy.cn,direct
go env -w GOSUMDB=sum.golang.google.cn

# 3. 安装 Node.js（如果还没有）
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs

# 4. 安装 Flutter
export PUB_HOSTED_URL=https://pub.flutter-io.cn
export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn
cd ~
git clone https://github.com/flutter/flutter.git -b stable --depth 1
echo 'export PATH="$PATH:$HOME/flutter/bin"' >> ~/.bashrc
echo 'export PUB_HOSTED_URL=https://pub.flutter-io.cn' >> ~/.bashrc
echo 'export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn' >> ~/.bashrc
source ~/.bashrc

# 5. 运行 Flutter doctor
flutter doctor

# 6. 安装项目依赖
cd /Users/lc/Development/source/remote-desktop-controller
./check-and-install.sh
```

---

## 📦 手动安装各组件依赖

如果你已经安装了工具链，只需要安装项目依赖：

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 1. Rust 依赖
cargo fetch
cargo build --workspace

# 2. Go 依赖
cd services/api
go mod download
cd ../..

# 3. Node.js 依赖（已完成）
cd web/admin
npm install
cd ../..

# 4. Flutter 依赖
cd client/flutter
flutter pub get
cd ../..
```

---

## 🔍 验证安装

运行检查脚本验证所有工具是否正确安装：

```bash
./check-and-install.sh
```

或手动检查：

```bash
rustc --version    # 应显示 Rust 版本
cargo --version    # 应显示 Cargo 版本
go version         # 应显示 Go 1.23+
node --version     # 应显示 Node.js 18+
npm --version      # 应显示 npm 版本
flutter --version  # 应显示 Flutter 3.4+
```

---

## 🏃 运行项目

### Web 管理后台（开发模式）

```bash
cd web/admin
npm run dev

# 浏览器访问 http://localhost:5173
```

### Go API 服务

```bash
cd services/api
go run main.go

# API 默认运行在 http://localhost:8080
```

### Flutter 桌面客户端

```bash
cd client/flutter
flutter run -d macos  # macOS
flutter run -d linux  # Linux
flutter run -d windows  # Windows
```

### Rust 编译（Release 模式）

```bash
cargo build --release

# 编译产物在 target/release/ 目录
```

---

## 🛠️ 常见问题

### Q: Rust 安装失败，提示网络错误

A: 使用国内镜像源：
```bash
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh
```

### Q: Go 依赖下载很慢

A: 配置 Go 代理：
```bash
go env -w GOPROXY=https://goproxy.cn,direct
go env -w GOSUMDB=sum.golang.google.cn
```

### Q: Flutter pub get 失败

A: 配置 Flutter 国内镜像：
```bash
export PUB_HOSTED_URL=https://pub.flutter-io.cn
export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn
```

### Q: npm install 很慢

A: 已自动配置为国内镜像 `registry.npmmirror.com`，如需手动设置：
```bash
npm config set registry https://registry.npmmirror.com
```

---

## 📚 项目结构

```
remote-desktop-controller/
├── crates/              # Rust 核心库（多个 crates）
├── services/api/        # Go API 服务
├── web/admin/          # React 管理后台 ✅
├── client/flutter/     # Flutter 桌面客户端
├── docs/               # 项目文档
├── Cargo.toml          # Rust workspace 配置
├── .cargo/config.toml  # Rust 镜像配置 ✅
└── check-and-install.sh # 依赖检查脚本
```

---

## 🤝 需要帮助？

- 查看项目文档：`docs/`
- 提交 Issue：GitHub Issues
- 查看贡献指南：`CONTRIBUTING.md`

---

**最后更新**: 2026-06-27
