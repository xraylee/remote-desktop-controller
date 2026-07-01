#!/bin/bash
# RDCS 开发环境安装脚本

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
    xcodebuild -version | head -1
else
    echo -e "${RED}❌ 未检测到 Xcode 命令行工具${NC}"
    echo ""
    echo "请执行以下命令安装："
    echo "  xcode-select --install"
    echo ""
    echo "安装完成后重新运行此脚本"
    exit 1
fi
echo ""

# 2. 检查 Homebrew
echo -e "${YELLOW}[2/5] 检查 Homebrew...${NC}"
if command -v brew > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Homebrew 已安装${NC}"
    brew --version | head -1
else
    echo -e "${RED}❌ 未检测到 Homebrew${NC}"
    echo ""
    echo "请执行以下命令安装："
    echo '  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"'
    echo ""
    echo "安装完成后重新运行此脚本"
    exit 1
fi
echo ""

# 3. 检查 CocoaPods
echo -e "${YELLOW}[3/5] 检查 CocoaPods...${NC}"
if command -v pod > /dev/null 2>&1; then
    echo -e "${GREEN}✅ CocoaPods 已安装 ($(pod --version))${NC}"
else
    echo -e "${YELLOW}⚠️  未检测到 CocoaPods，开始安装...${NC}"
    brew install cocoapods
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ CocoaPods 安装成功${NC}"
        echo -e "${YELLOW}正在初始化 CocoaPods（可能需要几分钟）...${NC}"
        pod setup
        echo -e "${GREEN}✅ CocoaPods 初始化完成${NC}"
    else
        echo -e "${RED}❌ CocoaPods 安装失败${NC}"
        exit 1
    fi
fi
echo ""

# 4. 检查 Rust
echo -e "${YELLOW}[4/5] 检查 Rust 工具链...${NC}"
if command -v rustc > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Rust 已安装 ($(rustc --version | cut -d' ' -f2))${NC}"
    echo -e "${GREEN}✅ Cargo 已安装 ($(cargo --version | cut -d' ' -f2))${NC}"
else
    echo -e "${RED}❌ 未检测到 Rust${NC}"
    echo ""
    echo "请执行以下命令安装："
    echo '  curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh'
    echo '  source $HOME/.cargo/env'
    echo ""
    echo "安装完成后重新运行此脚本"
    exit 1
fi
echo ""

# 5. 检查 Flutter
echo -e "${YELLOW}[5/5] 检查 Flutter SDK...${NC}"
if command -v flutter > /dev/null 2>&1; then
    echo -e "${GREEN}✅ Flutter 已安装${NC}"
    flutter --version | head -1
else
    echo -e "${RED}❌ 未检测到 Flutter${NC}"
    echo ""
    echo "请访问以下网址安装 Flutter："
    echo "  https://flutter.dev/docs/get-started/install/macos"
    echo ""
    echo "安装完成后重新运行此脚本"
    exit 1
fi
echo ""

# 运行 flutter doctor
echo -e "${YELLOW}运行 Flutter Doctor 检查...${NC}"
echo ""
flutter doctor
echo ""

# 检查 Flutter macOS 支持
echo -e "${YELLOW}检查 Flutter macOS 支持...${NC}"
if flutter config | grep -q "enable-macos-desktop: true"; then
    echo -e "${GREEN}✅ macOS 桌面支持已启用${NC}"
else
    echo -e "${YELLOW}⚠️  macOS 桌面支持未启用，正在启用...${NC}"
    flutter config --enable-macos-desktop
    echo -e "${GREEN}✅ macOS 桌面支持已启用${NC}"
fi
echo ""

# 总结
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     ✅ 环境检查完成！                  ║${NC}"
echo -e "${GREEN}║                                        ║${NC}"
echo -e "${GREEN}║  所有必需依赖已安装：                  ║${NC}"
echo -e "${GREEN}║  • Xcode 命令行工具                    ║${NC}"
echo -e "${GREEN}║  • Homebrew                            ║${NC}"
echo -e "${GREEN}║  • CocoaPods                           ║${NC}"
echo -e "${GREEN}║  • Rust + Cargo                        ║${NC}"
echo -e "${GREEN}║  • Flutter SDK                         ║${NC}"
echo -e "${GREEN}║                                        ║${NC}"
echo -e "${GREEN}║  下一步：配置 Flutter 项目             ║${NC}"
echo -e "${GREEN}║                                        ║${NC}"
echo -e "${GREEN}║  cd client/flutter                     ║${NC}"
echo -e "${GREEN}║  flutter create --platforms=macos .    ║${NC}"
echo -e "${GREEN}║  cd macos && pod install && cd ..      ║${NC}"
echo -e "${GREEN}║  flutter run -d macos                  ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
