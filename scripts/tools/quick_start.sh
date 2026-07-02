#!/bin/bash
# 简化版：构建 Rust + 运行 Flutter（自动复制库到运行时位置）

set -e

echo "=== RDCS 快速启动脚本 ==="
echo ""

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# 步骤 1: 编译 Rust FFI 库
echo -e "${YELLOW}[1/3] 编译 Rust FFI 库...${NC}"
cd crates/rdcs-ffi
cargo build --lib --features software-encoder
cd "$PROJECT_ROOT"

RUST_LIB="$PROJECT_ROOT/target/debug/librdcs_core.dylib"

if [ ! -f "$RUST_LIB" ]; then
    echo -e "${RED}❌ 编译失败${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Rust 库编译成功: $(ls -lh $RUST_LIB | awk '{print $5}')${NC}"
echo ""

# 步骤 2: 启动 Flutter（后台编译）
echo -e "${YELLOW}[2/3] 启动 Flutter 编译...${NC}"
cd client/flutter

# 后台启动 flutter run
flutter run -d macos &
FLUTTER_PID=$!

echo -e "${YELLOW}Flutter 正在编译（PID: $FLUTTER_PID）...${NC}"
echo -e "${YELLOW}等待构建产物生成...${NC}"
echo ""

# 步骤 3: 监控构建目录并复制库
echo -e "${YELLOW}[3/3] 监控构建并自动复制库...${NC}"

APP_PATH="$PROJECT_ROOT/client/flutter/build/macos/Build/Products/Debug/rdcs_client.app"
APP_FW="$APP_PATH/Contents/Frameworks"
DEST_LIB="$APP_FW/librdcs_core.dylib"

# 等待应用包创建
for i in {1..60}; do
    if [ -d "$APP_PATH" ]; then
        echo -e "${GREEN}✅ 应用包已创建${NC}"
        break
    fi
    sleep 1
    if [ $i -eq 60 ]; then
        echo -e "${RED}❌ 等待超时${NC}"
        kill $FLUTTER_PID 2>/dev/null || true
        exit 1
    fi
done

# 复制库
mkdir -p "$APP_FW"
cp "$RUST_LIB" "$DEST_LIB"
echo -e "${GREEN}✅ 库已复制到: $DEST_LIB${NC}"
ls -lh "$DEST_LIB"

# 修复 install_name
install_name_tool -id "@rpath/librdcs_core.dylib" "$DEST_LIB" 2>/dev/null || true

echo ""
echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║     ✅ 设置完成！                      ║${NC}"
echo -e "${GREEN}║                                        ║${NC}"
echo -e "${GREEN}║  Flutter 正在运行中...                 ║${NC}"
echo -e "${GREEN}║  如果应用黑屏，请在 Flutter 终端      ║${NC}"
echo -e "${GREEN}║  按 'R' 键热重启                       ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"

# 等待 flutter run 进程
wait $FLUTTER_PID
