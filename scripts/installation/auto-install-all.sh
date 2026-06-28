#!/bin/bash
# RDCS 完全自动安装脚本
# 此脚本会自动检测系统并安装所有依赖，无需用户干预

set -e

echo "=========================================="
echo "RDCS 完全自动安装脚本"
echo "=========================================="
echo ""

# 颜色输出
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }
log_info() { echo "→ $1"; }

# 检测操作系统
if [[ "$OSTYPE" == "darwin"* ]]; then
    OS="macos"
    ARCH=$(uname -m)
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    OS="linux"
    ARCH=$(uname -m)
else
    log_error "不支持的操作系统: $OSTYPE"
    exit 1
fi

log_success "检测到系统: $OS ($ARCH)"
echo ""

# 进入项目目录
cd "$(dirname "$0")"
PROJECT_ROOT=$(pwd)

# ============================================
# 1. 安装 Rust
# ============================================
echo "=========================================="
echo "1/4 安装 Rust 工具链"
echo "=========================================="

# 检查 Rust 是否可用（不仅是安装了，还要能运行）
RUST_WORKS=false
if command -v rustc &> /dev/null; then
    if rustc --version &> /dev/null; then
        RUST_WORKS=true
        log_success "Rust 已安装: $(rustc --version)"
    else
        log_warning "检测到 Rust 但无法运行（可能是架构不匹配）"
        log_info "正在清理并重新安装..."

        # 清理旧安装
        rm -rf "$HOME/.cargo" "$HOME/.rustup" 2>/dev/null || true
        sudo rm -f /usr/local/bin/rustc /usr/local/bin/cargo /usr/local/bin/rustup 2>/dev/null || true
    fi
fi

if [ "$RUST_WORKS" = false ]; then
    log_info "正在安装 Rust..."

    # 下载并安装 rustup
    if [[ "$OS" == "macos" ]]; then
        # macOS 优先使用 Homebrew
        if command -v brew &> /dev/null; then
            log_info "使用 Homebrew 安装..."
            brew install rustup-init
            rustup-init -y --default-toolchain stable
        else
            log_info "使用官方脚本安装..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        fi
    else
        # Linux 使用官方脚本
        log_info "使用官方脚本安装..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    fi

    # 加载 Rust 环境
    source "$HOME/.cargo/env"
    log_success "Rust 安装完成: $(rustc --version)"
fi

# 安装 Rust 项目依赖
log_info "正在获取 Rust 依赖..."
cargo fetch
log_success "Rust 依赖已获取"
echo ""

# ============================================
# 2. 安装 Go
# ============================================
echo "=========================================="
echo "2/4 安装 Go 语言环境"
echo "=========================================="

if command -v go &> /dev/null; then
    log_success "Go 已安装: $(go version)"
else
    log_info "正在安装 Go..."

    GO_VERSION="1.23.5"

    if [[ "$OS" == "macos" ]]; then
        # macOS 使用 Homebrew
        if command -v brew &> /dev/null; then
            log_info "使用 Homebrew 安装..."
            brew install go
        else
            log_error "请先安装 Homebrew: /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
            exit 1
        fi
    else
        # Linux 手动安装
        if [[ "$ARCH" == "x86_64" ]]; then
            GO_ARCH="amd64"
        elif [[ "$ARCH" == "aarch64" ]] || [[ "$ARCH" == "arm64" ]]; then
            GO_ARCH="arm64"
        else
            log_error "不支持的架构: $ARCH"
            exit 1
        fi

        log_info "下载 Go ${GO_VERSION}..."
        cd /tmp
        curl -L "https://go.dev/dl/go${GO_VERSION}.linux-${GO_ARCH}.tar.gz" -o go.tar.gz

        # 安装到用户目录（无需 root）
        log_info "安装到 $HOME/go-sdk..."
        mkdir -p "$HOME/go-sdk"
        tar -C "$HOME/go-sdk" -xzf go.tar.gz

        export PATH="$HOME/go-sdk/go/bin:$PATH"

        # 添加到 shell 配置
        for rcfile in "$HOME/.bashrc" "$HOME/.zshrc"; do
            if [[ -f "$rcfile" ]]; then
                if ! grep -q "go-sdk/go/bin" "$rcfile"; then
                    echo 'export PATH="$HOME/go-sdk/go/bin:$PATH"' >> "$rcfile"
                    echo 'export PATH="$HOME/go/bin:$PATH"' >> "$rcfile"
                fi
            fi
        done

        cd "$PROJECT_ROOT"
    fi

    log_success "Go 安装完成: $(go version)"
fi

# 配置 Go 代理
log_info "配置 Go 镜像代理..."
go env -w GOPROXY=https://goproxy.cn,direct
go env -w GOSUMDB=sum.golang.google.cn
log_success "Go 代理已配置"

# 安装 Go 项目依赖
log_info "正在下载 Go 依赖..."
cd "$PROJECT_ROOT/services/api"
go mod download
cd "$PROJECT_ROOT"
log_success "Go 依赖已下载"
echo ""

# ============================================
# 3. 安装 Node.js
# ============================================
echo "=========================================="
echo "3/4 检查 Node.js 环境"
echo "=========================================="

if command -v node &> /dev/null; then
    log_success "Node.js 已安装: $(node --version)"
    log_success "npm 已安装: $(npm --version)"
else
    log_error "Node.js 未安装"

    if [[ "$OS" == "macos" ]]; then
        if command -v brew &> /dev/null; then
            log_info "使用 Homebrew 安装..."
            brew install node
        else
            log_error "请先安装 Homebrew 或从 https://nodejs.org/ 下载"
            exit 1
        fi
    else
        log_error "请从 https://nodejs.org/ 下载安装，或使用 nvm"
        exit 1
    fi
fi

# 安装 npm 依赖
log_info "正在安装 Web 管理后台依赖..."
cd "$PROJECT_ROOT/web/admin"
npm config set registry https://registry.npmmirror.com
npm install
cd "$PROJECT_ROOT"
log_success "Web 管理后台依赖已安装"
echo ""

# ============================================
# 4. 安装 Flutter
# ============================================
echo "=========================================="
echo "4/4 安装 Flutter SDK"
echo "=========================================="

if command -v flutter &> /dev/null; then
    log_success "Flutter 已安装: $(flutter --version | head -1)"
else
    log_info "正在安装 Flutter..."

    # 配置 Flutter 国内镜像
    export PUB_HOSTED_URL=https://pub.flutter-io.cn
    export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn

    if [[ "$OS" == "macos" ]]; then
        # macOS 使用 Homebrew
        if command -v brew &> /dev/null; then
            log_info "使用 Homebrew 安装..."
            brew install flutter
        else
            log_error "请先安装 Homebrew"
            exit 1
        fi
    else
        # Linux 使用 git clone
        log_info "克隆 Flutter 仓库（稳定版）..."
        cd "$HOME"
        if [[ ! -d "flutter" ]]; then
            git clone https://github.com/flutter/flutter.git -b stable --depth 1
        else
            log_warning "Flutter 目录已存在，跳过克隆"
        fi

        export PATH="$HOME/flutter/bin:$PATH"

        # 添加到 shell 配置
        for rcfile in "$HOME/.bashrc" "$HOME/.zshrc"; do
            if [[ -f "$rcfile" ]]; then
                if ! grep -q "flutter/bin" "$rcfile"; then
                    echo 'export PUB_HOSTED_URL=https://pub.flutter-io.cn' >> "$rcfile"
                    echo 'export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn' >> "$rcfile"
                    echo 'export PATH="$HOME/flutter/bin:$PATH"' >> "$rcfile"
                fi
            fi
        done

        cd "$PROJECT_ROOT"
    fi

    log_success "Flutter 安装完成"

    # 运行 flutter doctor
    log_info "运行 flutter doctor..."
    flutter doctor || log_warning "Flutter doctor 检测到一些问题（可能需要安装额外的工具）"
fi

# 安装 Flutter 依赖
log_info "正在获取 Flutter 依赖..."
cd "$PROJECT_ROOT/client/flutter"
flutter pub get
cd "$PROJECT_ROOT"
log_success "Flutter 依赖已安装"
echo ""

# ============================================
# 最终验证
# ============================================
echo "=========================================="
echo "验证安装"
echo "=========================================="

echo ""
echo "工具链版本："
echo "  Rust:    $(rustc --version 2>/dev/null || echo '未安装')"
echo "  Cargo:   $(cargo --version 2>/dev/null || echo '未安装')"
echo "  Go:      $(go version 2>/dev/null || echo '未安装')"
echo "  Node.js: $(node --version 2>/dev/null || echo '未安装')"
echo "  npm:     $(npm --version 2>/dev/null || echo '未安装')"
echo "  Flutter: $(flutter --version 2>/dev/null | head -1 || echo '未安装')"
echo ""

# ============================================
# 完成
# ============================================
echo "=========================================="
log_success "所有依赖安装完成！"
echo "=========================================="
echo ""
echo "快速开始："
echo ""
echo "  Web 管理后台（开发服务器）："
echo "    cd web/admin && npm run dev"
echo ""
echo "  Go API 服务："
echo "    cd services/api && go run main.go"
echo ""
echo "  Flutter 桌面客户端："
if [[ "$OS" == "macos" ]]; then
    echo "    cd client/flutter && flutter run -d macos"
else
    echo "    cd client/flutter && flutter run -d linux"
fi
echo ""
echo "  Rust 编译（Release）："
echo "    cargo build --release"
echo ""
echo "详细文档请查看 SETUP.md"
echo ""
