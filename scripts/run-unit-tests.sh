#!/bin/bash
# RDCS 单元测试

# 加载 Rust 环境
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "=========================================="
echo "🧪 RDCS 单元测试"
echo "=========================================="
echo ""

cd "$(dirname "$0")/.."

# 运行所有 Rust 单元测试（不启用 hardware-accel）
echo "运行 Rust workspace 单元测试..."
cargo test --workspace --lib -- --nocapture 2>&1 | tee unit-tests.log

TEST_RESULT=$?

echo ""
echo "=========================================="

if [ $TEST_RESULT -eq 0 ]; then
    echo "✅ 单元测试全部通过"
else
    echo "❌ 部分单元测试失败"
fi

echo ""
exit $TEST_RESULT
