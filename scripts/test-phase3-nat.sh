#!/bin/bash
# Phase 3 NAT 穿透编译和测试脚本

cd "$(dirname "$0")/.."

echo "=========================================="
echo "🌐 Phase 3: NAT 穿透测试"
echo "=========================================="
echo ""

echo "1️⃣  编译 rdcs-connection (with webrtc-rs)..."
cargo build -p rdcs-connection

if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi
echo "✅ 编译成功"
echo ""

echo "2️⃣  运行单元测试..."
cargo test -p rdcs-connection --lib

if [ $? -ne 0 ]; then
    echo "❌ 单元测试失败"
    exit 1
fi
echo "✅ 单元测试通过"
echo ""

echo "3️⃣  运行 NAT 检测测试..."
cargo test -p rdcs-nat-test

if [ $? -ne 0 ]; then
    echo "⚠️  NAT 检测测试失败（可能需要网络连接）"
else
    echo "✅ NAT 检测测试通过"
fi
echo ""

echo "=========================================="
echo "✅ Phase 3 NAT 穿透编译测试完成"
echo "=========================================="
echo ""
echo "📝 注意事项:"
echo "  - ICE P2P 测试需要真实网络环境"
echo "  - 运行端到端测试: cargo run -p rdcs-connection --example ice_p2p_test"
echo "  - 需要 STUN 服务器可访问（使用 Google STUN）"
echo ""
echo "🔧 下一步:"
echo "  - 运行 ICE P2P 测试验证真实网络连接"
echo "  - 部署 TURN 服务器用于中继"
echo "  - 集成到视频传输层"
