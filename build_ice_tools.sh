#!/bin/bash
# 编译 ICE 跨网络测试工具

cd "$(dirname "$0")"

echo "========================================"
echo "🔨 编译 ICE 跨网络测试工具"
echo "========================================"
echo ""

echo "1️⃣  编译 ice_server..."
cargo build -p rdcs-connection --example ice_server

if [ $? -ne 0 ]; then
    echo "❌ ice_server 编译失败"
    exit 1
fi
echo "✅ ice_server 编译成功"
echo ""

echo "2️⃣  编译 ice_client..."
cargo build -p rdcs-connection --example ice_client

if [ $? -ne 0 ]; then
    echo "❌ ice_client 编译失败"
    exit 1
fi
echo "✅ ice_client 编译成功"
echo ""

echo "========================================"
echo "✅ 编译完成"
echo "========================================"
echo ""
echo "📝 使用说明:"
echo ""
echo "【测试跨网络连接】"
echo "  需要两台机器或一台机器 + 手机热点"
echo ""
echo "  机器 A (Server):"
echo "    cargo run -p rdcs-connection --example ice_server"
echo ""
echo "  机器 B (Client):"
echo "    cargo run -p rdcs-connection --example ice_client"
echo ""
echo "  按照屏幕提示交换 JSON"
echo ""
echo "【如果只有一台机器】"
echo "  可以跳过此测试，直接进入 Phase 3.3 (DTLS 加密)"
echo ""
echo "详细说明: ./test_ice_cross_network.sh"
echo ""
