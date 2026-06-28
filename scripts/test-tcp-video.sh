#!/bin/bash
# TCP 视频传输模块测试脚本

cd "$(dirname "$0")/.."

echo "=========================================="
echo "🧪 TCP 视频传输模块测试"
echo "=========================================="
echo ""

echo "1. 编译 rdcs-transport..."
cargo build -p rdcs-transport

if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi

echo ""
echo "2. 运行单元测试..."
cargo test -p rdcs-transport tcp_video

if [ $? -ne 0 ]; then
    echo "❌ 测试失败"
    exit 1
fi

echo ""
echo "=========================================="
echo "✅ TCP 视频传输模块测试通过"
echo "=========================================="
