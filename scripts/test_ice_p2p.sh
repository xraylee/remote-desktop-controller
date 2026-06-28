#!/bin/bash
# ICE P2P 集成测试脚本

cd "$(dirname "$0")"

echo "========================================"
echo "🌐 ICE P2P 连接测试"
echo "========================================"
echo ""

echo "📝 说明："
echo "  - 此测试会创建两个 ICE Agent（模拟两个对等端）"
echo "  - 使用 Google STUN 服务器收集候选"
echo "  - 交换 SDP Offer/Answer"
echo "  - 尝试建立 P2P 连接"
echo ""

echo "⚠️  注意："
echo "  - 需要网络连接访问 STUN 服务器"
echo "  - 测试可能需要 30 秒超时"
echo "  - 如果防火墙阻止 UDP，连接可能失败"
echo ""

read -p "按 Enter 继续..."

echo ""
echo "🚀 运行 ICE P2P 测试..."
echo ""

RUST_LOG=info cargo run -p rdcs-connection --example ice_p2p_test 2>&1

TEST_RESULT=$?

echo ""
echo "========================================"

if [ $TEST_RESULT -eq 0 ]; then
    echo "✅ ICE P2P 测试完成"
    echo ""
    echo "📊 检查上方输出："
    echo "  - 候选收集数量"
    echo "  - 候选类型（Host/Srflx）"
    echo "  - 连接状态变化"
else
    echo "❌ ICE P2P 测试失败"
    echo ""
    echo "🔍 可能原因："
    echo "  - 网络无法访问 STUN 服务器"
    echo "  - 防火墙阻止 UDP 流量"
    echo "  - NAT 类型不支持（Symmetric NAT）"
    echo ""
    echo "💡 建议："
    echo "  - 检查网络连接"
    echo "  - 尝试不同的 STUN 服务器"
    echo "  - 查看详细日志：RUST_LOG=debug cargo run ..."
fi

echo "========================================"
