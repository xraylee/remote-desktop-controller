# 🔧 Apple Silicon (M1/M2/M3) 安装说明

## ⚠️ 问题诊断

你遇到了 "Bad CPU type in executable" 错误，这是因为：

- 你的 Mac 是 **ARM64 架构**（Apple Silicon）
- 但安装的 Rust 是 **x86_64 架构**（Intel）
- 两者不兼容

---

## ✅ 解决方案（30秒搞定）

**在 Mac 终端运行：**

```bash
cd /Users/lc/Development/source/remote-desktop-controller
./install-apple-silicon.sh
```

这个脚本会：
1. ✅ 清理旧的 x86_64 Rust
2. ✅ 安装 ARM64 原生 Rust
3. ✅ 安装 Go 和 Flutter（如果缺失）
4. ✅ 配置所有国内镜像
5. ✅ 安装所有项目依赖
6. ✅ 验证安装结果

**全自动，2-3 分钟完成！**

---

## 🎯 安装后验证

所有命令应该正常工作：

```bash
rustc --version   # 应显示版本号，不报错
cargo --version   # 应显示版本号，不报错
go version        # 应显示版本号
flutter --version # 应显示版本号
```

---

## 📊 预期输出

```
工具版本：
  Rust:    rustc 1.xx.x (xxxxxx)
  Cargo:   cargo 1.xx.x (xxxxxx)
  Go:      go version go1.23.x darwin/arm64
  Node.js: v22.22.3
  npm:     10.9.8
  Flutter: Flutter 3.xx.x • channel stable
```

---

## 🚀 安装完成后

```bash
# Web 管理后台
cd web/admin && npm run dev

# Go API 服务
cd services/api && go run main.go

# Flutter 桌面客户端
cd client/flutter && flutter run -d macos

# Rust 编译
cargo build --release
```

---

## 🆘 如果脚本失败

### 1. 确保有 Homebrew

```bash
brew --version
```

如果没有，先安装：

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### 2. 手动清理和安装

```bash
# 清理旧 Rust
rm -rf ~/.cargo ~/.rustup
sudo rm -f /usr/local/bin/rustc /usr/local/bin/cargo

# 安装 ARM64 原生 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source ~/.cargo/env

# 验证
rustc --version  # 应该正常显示版本
```

### 3. 安装其他工具

```bash
brew install go flutter

# 配置代理
go env -w GOPROXY=https://goproxy.cn,direct
```

### 4. 安装项目依赖

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# Rust
cargo fetch

# Go
cd services/api && go mod download && cd ../..

# Flutter
cd client/flutter && flutter pub get && cd ../..
```

---

## 💡 技术说明

### 为什么会有架构不匹配？

- Apple Silicon (M1/M2/M3) 使用 **ARM64** 架构
- 老的 Intel Mac 使用 **x86_64** 架构
- 二进制文件必须匹配系统架构才能运行

### Rosetta 2 为什么不能解决？

- Rosetta 2 可以运行 x86_64 程序
- 但如果 Rust 安装在系统路径（`/usr/local/bin`），可能被误识别为原生程序
- 最好的方案是直接安装 ARM64 原生版本

---

**立即运行：`./install-apple-silicon.sh`** 🚀
