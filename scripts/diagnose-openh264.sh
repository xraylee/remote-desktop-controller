#!/bin/bash
# OpenH264 集成自动诊断脚本

cd "$(dirname "$0")/.."

echo "=========================================="
echo "🔧 OpenH264 集成自动诊断"
echo "=========================================="
echo ""

# 编译并收集所有错误信息
echo "编译中..."
cargo build -p rdcs-codec --features software-encoder 2>&1 > openh264-errors-full.log

if [ $? -eq 0 ]; then
    echo "✅ 编译成功！"
    exit 0
fi

echo "❌ 编译失败，分析错误..."
echo ""

# 提取所有错误类型
echo "错误摘要:"
grep "error\[E" openh264-errors-full.log

echo ""
echo "=========================================="
echo "详细错误信息:"
echo "=========================================="
echo ""

# 针对每个错误类型提取详细信息
for error_code in E0425 E0599; do
    echo ""
    echo "--- 错误 $error_code ---"
    grep -A 10 "error\[$error_code\]" openh264-errors-full.log | head -50
done

echo ""
echo "=========================================="
echo "完整日志已保存到: openh264-errors-full.log"
echo ""
echo "请将此输出发送给 AI 助手以获取修复方案"
