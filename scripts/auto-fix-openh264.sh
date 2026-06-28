#!/bin/bash
# 自动修复编译错误的脚本

cd "$(dirname "$0")/.."

echo "🔧 开始自动修复 OpenH264 编译错误..."

# 循环编译直到成功
MAX_ATTEMPTS=5
attempt=1

while [ $attempt -le $MAX_ATTEMPTS ]; do
    echo ""
    echo "==================== 尝试 $attempt/$MAX_ATTEMPTS ===================="

    # 编译
    cargo build -p rdcs-codec --features software-encoder 2>&1 | tee build-attempt-$attempt.log

    if [ $? -eq 0 ]; then
        echo "✅ 编译成功！"
        exit 0
    fi

    echo "❌ 编译失败，分析错误..."

    # 提取错误类型
    grep "error\[E" build-attempt-$attempt.log | head -10

    attempt=$((attempt + 1))
    sleep 1
done

echo ""
echo "❌ 在 $MAX_ATTEMPTS 次尝试后仍然失败"
echo "请将 build-attempt-1.log 发送给 AI 助手"
