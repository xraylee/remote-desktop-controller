#!/bin/bash
# Remote Desktop Controller - Test Commands
# 快速测试脚本

echo "=========================================="
echo "RDCS 测试命令集"
echo "=========================================="
echo ""

echo "1. Phase 3.4 - DataChannel 基础测试"
echo "   RUST_LOG=info cargo run -p rdcs-connection --example video_datachannel_test"
echo ""

echo "2. Phase 3.4+ - 端到端视频流测试（推荐）"
echo "   RUST_LOG=info cargo run -p rdcs-connection --example video_e2e_test"
echo ""

echo "3. ICE P2P 本地测试"
echo "   RUST_LOG=info cargo run -p rdcs-connection --example ice_p2p_test"
echo ""

echo "4. 跨网络测试 - Server 端"
echo "   RUST_LOG=info cargo run -p rdcs-connection --example ice_server"
echo ""

echo "5. 跨网络测试 - Client 端"
echo "   RUST_LOG=info cargo run -p rdcs-connection --example ice_client"
echo ""

echo "6. Phase 1 - 本地回环测试"
echo "   cargo run --example local_roundtrip --features hardware-accel"
echo ""

echo "=========================================="
echo "快速运行最新测试:"
echo "=========================================="
echo ""
echo "RUST_LOG=info cargo run -p rdcs-connection --example video_e2e_test"
