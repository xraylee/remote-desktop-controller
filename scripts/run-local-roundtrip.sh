#!/bin/bash
# 运行本地回环测试（Phase 1）

# 加载 Rust 环境
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "=========================================="
echo "🚀 运行本地回环测试"
echo "=========================================="
echo ""

cd "$(dirname "$0")"

# 检查 macOS 屏幕录制权限
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "📋 macOS 权限检查..."
    echo "   请确保已授予屏幕录制权限："
    echo "   系统设置 → 隐私与安全性 → 屏幕录制"
    echo ""
fi

# 编译并运行
echo "🔨 编译测试程序..."
cargo build -p rdcs-codec --example local_roundtrip --features hardware-accel 2>&1 | tail -10
BUILD_RESULT=$?

if [ $BUILD_RESULT -ne 0 ]; then
    echo ""
    echo "❌ 编译失败"
    exit 1
fi

echo ""
echo "✅ 编译成功"
echo ""
echo "=========================================="
echo ""

# 运行测试
cargo run -p rdcs-codec --example local_roundtrip --features hardware-accel

RUN_RESULT=$?

echo ""
echo "=========================================="

if [ $RUN_RESULT -eq 0 ]; then
    echo "🎉 测试完成！"
    echo ""
    echo "生成的文件："
    if [ -f "output.h264" ]; then
        ls -lh output.h264
    fi
    if [ -f "output.ppm" ]; then
        ls -lh output.ppm
        echo ""
        echo "转换为 PNG (需要 ImageMagick):"
        echo "  brew install imagemagick"
        echo "  convert output.ppm output.png"
    fi
else
    echo "❌ 测试失败"
    echo ""
    echo "常见问题："
    echo "1. 权限不足: 需要屏幕录制权限"
    echo "2. 硬件不支持: 确认系统支持硬件加速"
    echo "3. 依赖缺失: 运行 ./install-china-mirror.sh"
fi

echo ""
