#!/bin/bash
# 测试 RealIceAgent 编译和基本功能

cd "$(dirname "$0")"

echo "========================================"
echo "🧪 RealIceAgent 编译测试"
echo "========================================"
echo ""

echo "1️⃣ 编译 rdcs-connection..."
cargo build -p rdcs-connection 2>&1

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ 编译失败"
    exit 1
fi

echo ""
echo "✅ 编译成功"
echo ""

echo "2️⃣ 运行单元测试..."
cargo test -p rdcs-connection --lib 2>&1

if [ $? -ne 0 ]; then
    echo ""
    echo "❌ 单元测试失败"
    exit 1
fi

echo ""
echo "✅ 单元测试通过"
echo ""

echo "========================================"
echo "✅ RealIceAgent 测试完成"
echo "========================================"
echo ""
echo "📝 下一步："
echo "  运行 ICE P2P 测试："
echo "  cargo run -p rdcs-connection --example ice_p2p_test"
