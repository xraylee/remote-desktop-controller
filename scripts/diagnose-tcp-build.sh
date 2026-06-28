#!/bin/bash
# TCP 视频传输层编译测试（仅编译，不运行测试）

cd "$(dirname "$0")/.."

echo "=========================================="
echo "🔧 TCP 视频传输层编译测试"
echo "=========================================="
echo ""

echo "检查文件结构..."
echo ""
echo "新增文件:"
ls -lh crates/rdcs-transport/src/tcp_video.rs 2>/dev/null && echo "✅ tcp_video.rs"
ls -lh crates/rdcs-transport/examples/tcp_video_e2e.rs 2>/dev/null && echo "✅ tcp_video_e2e.rs"

echo ""
echo "=========================================="
echo "编译 rdcs-transport..."
echo "=========================================="
echo ""

cargo build -p rdcs-transport 2>&1 | tee /tmp/tcp-build.log

if [ ${PIPESTATUS[0]} -eq 0 ]; then
    echo ""
    echo "=========================================="
    echo "✅ 编译成功"
    echo "=========================================="
    exit 0
else
    echo ""
    echo "=========================================="
    echo "❌ 编译失败"
    echo "=========================================="
    echo ""
    echo "错误摘要:"
    grep "error" /tmp/tcp-build.log | head -20
    echo ""
    echo "完整日志: /tmp/tcp-build.log"
    exit 1
fi
