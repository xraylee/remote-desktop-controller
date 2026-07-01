# 开发环境依赖安装指南

**日期**: 2026-06-29  
**适用平台**: macOS (Apple Silicon & Intel)

---

## 📋 必需依赖清单

### 1. Xcode 命令行工具 ⭐
**用途**: 编译 macOS 应用、Swift 代码  
**状态**: ❌ 未安装（导致当前错误）

#### 安装方法
```bash
# 安装 Xcode 命令行工具
xcode-select --install

# 验证安装
xcodebuild -version

# 应显示类似：
# Xcode 15.x
# Build version 15Xxx
```

#### 如果已安装但路径错误
```bash
# 重置路径
sudo xcode-select --reset

# 或手动设置（如果安装了完整 Xcode.app）
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

---

### 2. CocoaPods ⭐
**用途**: Flutter iOS/macOS 依赖管理  
**状态**: ❌ 未安装（导致当前错误）

#### 安装方法
```bash
# 使用 Homebrew 安装（推荐）
brew install cocoapods

# 或使用 Ruby gem 安装
sudo gem install cocoapods

# 验证安装
pod --version

# 应显示类似：
# 1.15.2
```

#### 初始化（首次安装）
```bash
# 初始化 CocoaPods（可能需要几分钟）
pod setup

# 验证
pod repo list
```

---

### 3. Homebrew
**用途**: macOS 包管理器  
**状态**: ⚠️ 需确认

#### 检查是否安装
```bash
brew --version
```

#### 如未安装
```bash
# 安装 Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 验证
brew doctor
```

---

### 4. Rust 工具链 ⭐
**用途**: 编译 RDCS Rust 代码  
**状态**: ✅ 已安装

#### 验证
```bash
rustc --version
cargo --version

# 应显示类似：
# rustc 1.75.0
# cargo 1.75.0
```

#### 如未安装
```bash
# 安装 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 添加到 PATH
source $HOME/.cargo/env
```

---

### 5. Flutter SDK ⭐
**用途**: 构建 Flutter 应用  
**状态**: ✅ 已安装

#### 验证
```bash
flutter --version
flutter doctor

# 应显示：
# Flutter 3.x.x
# [✓] Flutter (Channel stable)
# [✓] macOS
# [✓] Xcode
# [✓] VS Code (可选)
```

#### 如未安装
```bash
# 使用 Homebrew 安装
brew install --cask flutter

# 或手动下载
# 从 https://flutter.dev/docs/get-started/install/macos 下载

# 添加到 PATH
export PATH="$PATH:/path/to/flutter/bin"

# 运行 flutter doctor 检查
flutter doctor
```

---

## 🔧 完整安装流程

### 步骤 1: 安装 Xcode 命令行工具

```bash
# 安装
xcode-select --install

# 等待安装完成（弹窗）
# 点击"安装"并同意协议

# 验证
xcodebuild -version
```

**预期输出**:
```
Xcode 15.x
Build version 15Xxx
```

---

### 步骤 2: 安装 Homebrew（如未安装）

```bash
# 检查是否已安装
which brew

# 如果没有输出，则安装
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 验证
brew --version
```

---

### 步骤 3: 安装 CocoaPods

```bash
# 使用 Homebrew 安装
brew install cocoapods

# 验证
pod --version

# 初始化（首次安装，需要几分钟）
pod setup

# 验证仓库
pod repo list
```

**预期输出**:
```
1.15.2

master
- Type: git (master)
- URL:  https://github.com/CocoaPods/Specs.git
- Path: ~/.cocoapods/repos/master
```

---

### 步骤 4: 配置 Flutter

```bash
cd /Users/lc/Development/source/remote-desktop-controller/client/flutter

# 启用 macOS 桌面支持
flutter config --enable-macos-desktop

# 清理旧构建
flutter clean

# 获取依赖
flutter pub get

# 为项目添加 macOS 平台
flutter create --platforms=macos .

# 安装 CocoaPods 依赖
cd macos
pod install
cd ..

# 验证
flutter doctor
```

**预期输出**:
```
Doctor summary (to see all details, run flutter doctor -v):
[✓] Flutter (Channel stable, 3.x.x)
[✓] Xcode - develop for iOS and macOS (Xcode 15.x)
[✓] Chrome - develop for the web
[✓] VS Code (version 1.x.x)
[✓] Connected device (1 available)
```

---

### 步骤 5: 运行应用

```bash
# 运行 Flutter 应用
flutter run -d macos
```

---

## ⚠️ 常见问题

### 问题 1: `xcode-select: error: tool 'xcodebuild' requires Xcode`

**原因**: 只安装了命令行工具，某些功能需要完整 Xcode

**解决**:
```bash
# 从 App Store 安装完整 Xcode
# 或从 https://developer.apple.com/download/ 下载

# 安装后设置路径
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer

# 接受许可
sudo xcodebuild -license accept
```

---

### 问题 2: `pod: command not found`

**原因**: CocoaPods 未添加到 PATH

**解决**:
```bash
# 检查安装位置
which pod

# 如果使用 gem 安装，添加到 PATH
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc

# 或重新安装
brew install cocoapods
```

---

### 问题 3: `Pod install` 失败

**原因**: 依赖版本冲突或网络问题

**解决**:
```bash
cd client/flutter/macos

# 清理 CocoaPods 缓存
pod cache clean --all

# 更新仓库
pod repo update

# 重新安装
pod install --repo-update

# 如果仍失败，使用国内镜像
export COCOAPODS_MIRROR=https://mirrors.tuna.tsinghua.edu.cn/git/CocoaPods/Specs.git
pod install
```

---

### 问题 4: Flutter 插件警告

```
The following plugins do not support Swift Package Manager:
  - tray_manager
  - window_manager
```

**原因**: 这些插件尚未支持 Swift Package Manager

**解决**: 这只是警告，不影响功能。如果遇到编译问题：

```dart
// 暂时注释掉相关导入（client/flutter/lib/main.dart）
// import 'package:tray_manager/tray_manager.dart';
// import 'package:window_manager/window_manager.dart';

// 注释掉初始化代码
// await windowManager.ensureInitialized();
```

---

### 问题 5: 权限错误

**症状**: 屏幕捕获失败或鼠标/键盘注入无效

**解决**:
```bash
# 打开系统设置
open "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"

# 手动授予权限：
# macOS 设置 → 隐私与安全性 → 屏幕录制
# macOS 设置 → 隐私与安全性 → 辅助功能

# 添加你的应用并重启应用
```

---

## ✅ 环境验证清单

完成安装后，运行以下命令验证：

```bash
# 1. Xcode 命令行工具
xcodebuild -version
# 预期: Xcode 15.x

# 2. CocoaPods
pod --version
# 预期: 1.15.x

# 3. Homebrew
brew --version
# 预期: Homebrew 4.x.x

# 4. Rust
rustc --version
# 预期: rustc 1.75.x

# 5. Flutter
flutter --version
# 预期: Flutter 3.x.x

# 6. Flutter Doctor
flutter doctor
# 预期: 所有项目都是 [✓]
```

---

## 📝 完整安装脚本

**保存为 `setup_environment.sh`**:

```bash
#!/bin/bash
set -e

echo "=== RDCS 开发环境安装脚本 ==="
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 1. 检查 Xcode 命令行工具
echo -e "${YELLOW}[1/5] 检查 Xcode 命令行工具...${NC}"
if xcodebuild -version > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Xcode 命令行工具已安装${NC}"
else
    echo -e "${YELLOW}⚠️  未检测到 Xcode 命令行工具，开始安装...${NC}"
    xcode-select --install
    echo -e "${YELLOW}请在弹出窗口中完成安装，然后重新运行此脚本${NC}"
    exit 1
fi
echo ""

# 2. 检查 Homebrew
echo -e "${YELLOW}[2/5] 检查 Homebrew...${NC}"
if command -v brew > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Homebrew 已安装${NC}"
else
    echo -e "${YELLOW}⚠️  未检测到 Homebrew，开始安装...${NC}"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    echo -e "${GREEN}✅ Homebrew 安装完成${NC}"
fi
echo ""

# 3. 检查 CocoaPods
echo -e "${YELLOW}[3/5] 检查 CocoaPods...${NC}"
if command -v pod > /dev/null 2>&1; then
    echo -e "${GREEN}✅ CocoaPods 已安装 ($(pod --version))${NC}"
else
    echo -e "${YELLOW}⚠️  未检测到 CocoaPods，开始安装...${NC}"
    brew install cocoapods
    pod setup
    echo -e "${GREEN}✅ CocoaPods 安装完成${NC}"
fi
echo ""

# 4. 检查 Rust
echo -e "${YELLOW}[4/5] 检查 Rust 工具链...${NC}"
if command -v rustc > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Rust 已安装 ($(rustc --version | cut -d' ' -f2))${NC}"
else
    echo -e "${RED}❌ 未检测到 Rust${NC}"
    echo "请访问 https://rustup.rs/ 安装 Rust"
    exit 1
fi
echo ""

# 5. 检查 Flutter
echo -e "${YELLOW}[5/5] 检查 Flutter SDK...${NC}"
if command -v flutter > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Flutter 已安装 ($(flutter --version | head -1 | cut -d' ' -f2))${NC}"
else
    echo -e "${RED}❌ 未检测到 Flutter${NC}"
    echo "请访问 https://flutter.dev/docs/get-started/install/macos 安装 Flutter"
    exit 1
fi
echo ""

# 运行 flutter doctor
echo -e "${YELLOW}运行 Flutter Doctor...${NC}"
flutter doctor
echo ""

echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     环境检查完成！                     ║${NC}"
echo -e "${GREEN}║                                        ║${NC}"
echo -e "${GREEN}║  下一步:                               ║${NC}"
echo -e "${GREEN}║  cd client/flutter                     ║${NC}"
echo -e "${GREEN}║  flutter create --platforms=macos .    ║${NC}"
echo -e "${GREEN}║  cd macos && pod install && cd ..      ║${NC}"
echo -e "${GREEN}║  flutter run -d macos                  ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
```

---

## 🚀 快速启动

完成环境安装后：

```bash
cd /Users/lc/Development/source/remote-desktop-controller

# 赋予执行权限
chmod +x setup_environment.sh

# 运行环境检查脚本
./setup_environment.sh

# 配置 Flutter 项目
cd client/flutter
flutter create --platforms=macos .
cd macos
pod install
cd ..

# 运行应用
flutter run -d macos
```

---

## 📊 依赖版本推荐

| 工具 | 推荐版本 | 最低版本 |
|------|---------|----------|
| macOS | 14.x (Sonoma) | 13.0 |
| Xcode | 15.x | 14.0 |
| Xcode CLI Tools | 15.x | 14.0 |
| CocoaPods | 1.15.x | 1.12.0 |
| Homebrew | 4.x | 3.0 |
| Rust | 1.75.x | 1.70.0 |
| Flutter | 3.19.x | 3.16.0 |

---

## 💡 提示

1. **首次安装**: 整个过程可能需要 30-60 分钟
2. **网络问题**: 建议使用稳定网络，CocoaPods 和 Homebrew 需要下载大量依赖
3. **权限**: 某些安装步骤需要管理员权限（sudo）
4. **重启**: 安装 Xcode 命令行工具后可能需要重启终端

---

**创建日期**: 2026-06-29  
**维护者**: RDCS Team  
**状态**: 生产环境依赖指南 ✅
