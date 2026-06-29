#!/bin/bash
# Phase 4.1 - 硬件编码器性能对比测试
# Usage: ./test_hardware_encoder.sh

set -o pipefail

echo "========================================="
echo "Phase 4.1 - 硬件编码器性能测试"
echo "========================================="
echo ""

echo "测试环境:"
echo "  平台: macOS"
echo "  分辨率: 1280x720 @ 30fps"
echo "  码率: 2 Mbps"
echo "  测试帧数: 60 (2秒)"
echo ""

echo "----------------------------------------"
echo "Test 1: OpenH264 软件编码器 (基线)"
echo "----------------------------------------"
echo ""
RUST_LOG=info cargo run -p rdcs-connection --example hardware_encoder_test --features software-encoder 2>&1 | tee /tmp/software_encoder_result.txt

echo ""
echo "----------------------------------------"
echo "Test 2: VideoToolbox 硬件编码器"
echo "----------------------------------------"
echo ""
RUST_LOG=info cargo run -p rdcs-connection --example hardware_encoder_test 2>&1 | tee /tmp/hardware_encoder_result.txt

echo ""
echo "========================================="
echo "性能对比"
echo "========================================="
echo ""

# 提取结果 (格式: "INFO hardware_encoder_test: Average encode time: 44.30ms")
SOFTWARE_TIME=$(grep "Average encode time:" /tmp/software_encoder_result.txt | awk '{print $NF}' | sed 's/ms//')
HARDWARE_TIME=$(grep "Average encode time:" /tmp/hardware_encoder_result.txt | awk '{print $NF}' | sed 's/ms//')

echo "软件编码器平均延迟: ${SOFTWARE_TIME}ms"
echo "硬件编码器平均延迟: ${HARDWARE_TIME}ms"
echo ""

# 计算加速比
if [ -n "$SOFTWARE_TIME" ] && [ -n "$HARDWARE_TIME" ]; then
    SPEEDUP=$(echo "scale=2; $SOFTWARE_TIME / $HARDWARE_TIME" | bc)
    REDUCTION=$(echo "scale=2; $SOFTWARE_TIME - $HARDWARE_TIME" | bc)
    echo "性能提升: ${SPEEDUP}x 倍"
    echo "延迟降低: ${REDUCTION}ms"
    echo ""

    # 端到端影响
    SOFTWARE_E2E=$(echo "scale=2; $SOFTWARE_TIME + 32 + 2" | bc)
    HARDWARE_E2E=$(echo "scale=2; $HARDWARE_TIME + 32 + 2" | bc)
    E2E_IMPROVEMENT=$(echo "scale=2; $SOFTWARE_E2E - $HARDWARE_E2E" | bc)
    PERCENTAGE=$(echo "scale=1; $E2E_IMPROVEMENT / $SOFTWARE_E2E * 100" | bc)

    echo "端到端延迟对比:"
    echo "  软件: ${SOFTWARE_E2E}ms"
    echo "  硬件: ${HARDWARE_E2E}ms"
    echo "  改进: ${E2E_IMPROVEMENT}ms (${PERCENTAGE}%)"
fi

echo ""
echo "========================================="
echo "✅ 测试完成！"
echo "========================================="
echo ""
echo "详细结果保存在:"
echo "  软件: /tmp/software_encoder_result.txt"
echo "  硬件: /tmp/hardware_encoder_result.txt"
