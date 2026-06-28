#!/bin/bash
# 测试 openh264 集成编译

# 加载 Rust 环境
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

cd "$(dirname "$0")/.."

echo "=========================================="
echo "🔨 测试 OpenH264 集成编译"
echo "=========================================="
echo ""

echo "编译 rdcs-codec (software-encoder feature)..."
cargo build -p rdcs-codec --features software-encoder 2>&1 | tee openh264-build.log

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ 编译成功"
else
    echo ""
    echo "❌ 编译失败，查看错误:"
    echo ""
    grep "error\[E" openh264-build.log | head -20
fi
