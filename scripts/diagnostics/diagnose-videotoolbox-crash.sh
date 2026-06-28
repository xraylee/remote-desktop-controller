#!/bin/bash
# 诊断 VideoToolbox 崩溃问题

# 加载 Rust 环境
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "=========================================="
echo "🔍 诊断 VideoToolbox FFI 崩溃"
echo "=========================================="
echo ""

cd "$(dirname "$0")"

# 1. 编译测试程序（带调试符号）
echo "1️⃣  编译测试程序（debug 模式）..."
RUST_BACKTRACE=1 cargo build -p rdcs-codec --example local_roundtrip --features hardware-accel 2>&1 | tail -20
BUILD_RESULT=$?

if [ $BUILD_RESULT -ne 0 ]; then
    echo ""
    echo "❌ 编译失败"
    exit 1
fi

echo ""
echo "✅ 编译成功"
echo ""

# 2. 使用 lldb 运行并捕获崩溃信息
echo "2️⃣  使用 lldb 运行测试..."
echo ""

cat > /tmp/lldb_commands.txt <<'EOF'
# 设置断点
b rust_panic
b rdcs_codec::platform::videotoolbox::VideoToolboxEncoder::new
b rdcs_codec::platform::videotoolbox::VideoToolboxEncoder::encode
b rdcs_codec::platform::videotoolbox::VideoToolboxEncoder::create_pixel_buffer

# 运行
run

# 崩溃时打印堆栈
bt

# 打印寄存器
register read

# 继续
continue
EOF

echo "LLDB 断点设置:"
cat /tmp/lldb_commands.txt
echo ""
echo "运行测试..."
echo ""

# 运行 lldb
lldb -s /tmp/lldb_commands.txt target/debug/examples/local_roundtrip 2>&1 | tee lldb_output.log

# 3. 分析崩溃日志
echo ""
echo "=========================================="
echo "📊 崩溃分析"
echo "=========================================="
echo ""

if grep -q "SIGSEGV" lldb_output.log; then
    echo "❌ 检测到 SIGSEGV (Segmentation Fault)"
    echo ""

    # 提取崩溃位置
    echo "崩溃位置:"
    grep -A 5 "SIGSEGV" lldb_output.log | head -10
    echo ""

    # 提取堆栈跟踪
    echo "堆栈跟踪:"
    sed -n '/^frame #/p' lldb_output.log | head -20
    echo ""
else
    echo "✅ 未检测到 SIGSEGV"
fi

# 清理
rm -f /tmp/lldb_commands.txt

echo ""
echo "完整日志已保存到: lldb_output.log"
echo ""
