#!/bin/bash
# RDCS 依赖检查和安装脚本
# 会根据环境自动调整安装策略

set -e

echo "=========================================="
echo "RDCS 依赖检查脚本"
echo "=========================================="
echo ""

MISSING_TOOLS=()
INSTALLED_TOOLS=()

# 检查各个工具
echo "正在检查已安装的工具..."
echo ""

# 1. 检查 Rust
if command -v rustc &> /dev/null; then
    INSTALLED_TOOLS+=("Rust $(rustc --version)")
else
    MISSING_TOOLS+=("Rust")
fi

# 2. 检查 Cargo
if command -v cargo &> /dev/null; then
    INSTALLED_TOOLS+=("Cargo $(cargo --version)")
else
    MISSING_TOOLS+=("Cargo")
fi

# 3. 检查 Go
if command -v go &> /dev/null; then
    INSTALLED_TOOLS+=("Go $(go version)")
else
    MISSING_TOOLS+=("Go")
fi

# 4. 检查 Node.js
if command -v node &> /dev/null; then
    INSTALLED_TOOLS+=("Node.js $(node --version)")
else
    MISSING_TOOLS+=("Node.js")
fi

# 5. 检查 npm
if command -v npm &> /dev/null; then
    INSTALLED_TOOLS+=("npm $(npm --version)")
else
    MISSING_TOOLS+=("npm")
fi

# 6. 检查 Flutter
if command -v flutter &> /dev/null; then
    INSTALLED_TOOLS+=("Flutter $(flutter --version | head -1)")
else
    MISSING_TOOLS+=("Flutter")
fi

# 显示结果
echo "=========================================="
echo "✅ 已安装的工具:"
echo "=========================================="
for tool in "${INSTALLED_TOOLS[@]}"; do
    echo "  ✓ $tool"
done
echo ""

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo "=========================================="
    echo "❌ 缺失的工具:"
    echo "=========================================="
    for tool in "${MISSING_TOOLS[@]}"; do
        echo "  ✗ $tool"
    done
    echo ""
fi

# 如果在容器环境，给出提示
if [ -f /.dockerenv ] || grep -q docker /proc/1/cgroup 2>/dev/null; then
    echo "=========================================="
    echo "⚠️  检测到容器环境"
    echo "=========================================="
    echo "由于权限和网络限制，请在宿主机上安装缺失的工具。"
    echo ""
fi

# 尝试安装项目依赖
echo "=========================================="
echo "安装项目依赖"
echo "=========================================="
echo ""

cd "$(dirname "$0")"

# Node.js 依赖
if command -v npm &> /dev/null; then
    echo ">>> 安装 Web 管理后台依赖..."
    cd web/admin
    npm config set registry https://registry.npmmirror.com
    npm install --prefer-offline 2>&1 | tail -20
    echo "✓ Web 管理后台依赖已安装"
    cd ../..
    echo ""
fi

# Rust 依赖
if command -v cargo &> /dev/null; then
    echo ">>> 安装 Rust workspace 依赖..."
    cargo fetch 2>&1 | tail -20 || true
    echo "✓ Rust 依赖已获取"
    echo ""
fi

# Go 依赖
if command -v go &> /dev/null; then
    echo ">>> 安装 Go API 依赖..."
    cd services/api
    go env -w GOPROXY=https://goproxy.cn,direct 2>/dev/null || true
    go mod download 2>&1 | tail -20 || true
    echo "✓ Go 依赖已下载"
    cd ../..
    echo ""
fi

# Flutter 依赖
if command -v flutter &> /dev/null; then
    echo ">>> 安装 Flutter 客户端依赖..."
    cd client/flutter
    export PUB_HOSTED_URL=https://pub.flutter-io.cn
    export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn
    flutter pub get 2>&1 | tail -20 || true
    echo "✓ Flutter 依赖已安装"
    cd ../..
    echo ""
fi

echo "=========================================="
echo "总结"
echo "=========================================="
echo ""

if [ ${#MISSING_TOOLS[@]} -eq 0 ]; then
    echo "✅ 所有工具已安装，所有依赖已配置！"
    echo ""
    echo "快速开始:"
    echo "  • Web 管理后台: cd web/admin && npm run dev"
    echo "  • Go API 服务: cd services/api && go run main.go"
    echo "  • Flutter 客户端: cd client/flutter && flutter run"
    echo "  • Rust 编译: cargo build --release"
else
    echo "⚠️  仍有工具未安装，请手动安装:"
    echo ""

    for tool in "${MISSING_TOOLS[@]}"; do
        case $tool in
            "Rust"|"Cargo")
                echo "🔧 安装 Rust (使用国内镜像):"
                echo "   curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh"
                echo ""
                ;;
            "Go")
                echo "🔧 安装 Go (1.23.5+):"
                echo "   macOS: brew install go"
                echo "   Linux: wget https://go.dev/dl/go1.23.5.linux-amd64.tar.gz"
                echo "   然后配置: go env -w GOPROXY=https://goproxy.cn,direct"
                echo ""
                ;;
            "Flutter")
                echo "🔧 安装 Flutter (使用国内镜像):"
                echo "   export PUB_HOSTED_URL=https://pub.flutter-io.cn"
                echo "   export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn"
                echo "   git clone https://github.com/flutter/flutter.git -b stable"
                echo "   export PATH=\"\$PATH:\`pwd\`/flutter/bin\""
                echo ""
                ;;
            "Node.js"|"npm")
                echo "🔧 安装 Node.js:"
                echo "   macOS: brew install node"
                echo "   Linux: https://nodejs.org/ 或使用 nvm"
                echo ""
                ;;
        esac
    done
fi

echo ""
