#!/bin/bash
# 运行本地回环测试（Mock 版本）

# 加载 Rust 环境
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "=========================================="
echo "🚀 运行本地回环测试（Mock 版本）"
echo "=========================================="
echo ""

cd "$(dirname "$0")"

echo "🔨 编译测试程序..."
cargo build -p rdcs-codec --example local_roundtrip_mock 2>&1 | tail -10
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
cargo run -p rdcs-codec --example local_roundtrip_mock

RUN_RESULT=$?

echo ""
echo "=========================================="

if [ $RUN_RESULT -eq 0 ]; then
    echo "🎉 测试完成！"
    echo ""
    echo "生成的文件："
    if [ -f "output.stub" ]; then
        ls -lh output.stub
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
fi

echo ""
