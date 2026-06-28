#!/bin/bash
# TCP 视频传输端到端测试

cd "$(dirname "$0")/.."

echo "=========================================="
echo "🎬 TCP 视频传输端到端测试"
echo "=========================================="
echo ""

echo "运行测试..."
cargo run -p rdcs-transport --example tcp_video_e2e --features software-encoder

if [ $? -eq 0 ]; then
    echo ""
    echo "=========================================="
    echo "✅ 测试成功"
    echo "=========================================="
    echo ""

    if [ -f "tcp_output.ppm" ]; then
        echo "输出文件: tcp_output.ppm"
        ls -lh tcp_output.ppm
    fi
else
    echo ""
    echo "❌ 测试失败"
    exit 1
fi
