#!/bin/bash
# Phase 2 TCP 传输层完整测试套件

cd "$(dirname "$0")/.."

echo "=========================================="
echo "📦 Phase 2: TCP 视频传输层测试套件"
echo "=========================================="
echo ""

# 1. 编译测试
echo "1️⃣  编译 rdcs-transport..."
cargo build -p rdcs-transport

if [ $? -ne 0 ]; then
    echo "❌ 编译失败"
    exit 1
fi
echo "✅ 编译成功"
echo ""

# 2. 单元测试
echo "2️⃣  运行单元测试..."
cargo test -p rdcs-transport tcp_video --lib

if [ $? -ne 0 ]; then
    echo "❌ 单元测试失败"
    exit 1
fi
echo "✅ 单元测试通过"
echo ""

# 3. 端到端测试
echo "3️⃣  运行端到端测试..."
cargo run -p rdcs-transport --example tcp_video_e2e --features software-encoder

if [ $? -ne 0 ]; then
    echo "❌ 端到端测试失败"
    exit 1
fi
echo "✅ 端到端测试通过"
echo ""

# 检查输出文件
if [ -f "tcp_output.ppm" ]; then
    echo "📄 生成的文件:"
    ls -lh tcp_output.ppm
    echo ""
fi

echo "=========================================="
echo "✅ Phase 2 TCP 传输层测试全部通过"
echo "=========================================="
echo ""
echo "📊 测试摘要:"
echo "   ✅ 编译测试"
echo "   ✅ TCP 协议单元测试"
echo "   ✅ 编码→传输→解码端到端测试"
echo ""
echo "📁 新增模块:"
echo "   - crates/rdcs-transport/src/tcp_video.rs"
echo "   - crates/rdcs-transport/examples/tcp_video_e2e.rs"
echo ""
echo "🎯 下一步: Phase 2 Go API 基础服务"
