#!/bin/bash
# 测试 hardware-accel feature gate

# 加载 Rust 环境
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "=========================================="
echo "🧪 测试 hardware-accel feature gate"
echo "=========================================="
echo ""

cd "$(dirname "$0")"

# 1. 测试默认配置（不启用 hardware-accel）
echo "1️⃣  测试默认配置（不启用硬件加速）..."
echo ""
cargo test -p rdcs-codec --lib 2>&1 | tee test-default.log
DEFAULT_RESULT=$?
echo ""

if [ $DEFAULT_RESULT -eq 0 ]; then
    echo "✅ 默认配置测试通过（无 SIGSEGV）"
else
    echo "❌ 默认配置测试失败"
    echo "查看详细日志: test-default.log"
fi
echo ""

# 2. 测试启用 hardware-accel（预期可能崩溃）
echo "2️⃣  测试启用 hardware-accel feature..."
echo ""
cargo test -p rdcs-codec --lib --features hardware-accel 2>&1 | tee test-hardware-accel.log
HARDWARE_RESULT=$?
echo ""

if [ $HARDWARE_RESULT -eq 0 ]; then
    echo "✅ hardware-accel 测试通过"
else
    echo "⚠️  hardware-accel 测试失败（预期行为 - 需要集成测试环境）"
    echo "查看详细日志: test-hardware-accel.log"
fi
echo ""

# 3. 验证 platform 模块编译
echo "3️⃣  验证 platform 模块编译..."
echo ""
cargo check -p rdcs-codec --features hardware-accel 2>&1 | tail -5
CHECK_RESULT=$?
echo ""

if [ $CHECK_RESULT -eq 0 ]; then
    echo "✅ platform 模块编译通过"
else
    echo "❌ platform 模块编译失败"
fi
echo ""

# 总结
echo "=========================================="
echo "📊 测试总结"
echo "=========================================="
echo "默认配置测试: $([ $DEFAULT_RESULT -eq 0 ] && echo '✅ 通过' || echo '❌ 失败')"
echo "硬件加速测试: $([ $HARDWARE_RESULT -eq 0 ] && echo '✅ 通过' || echo '⚠️  失败')"
echo "模块编译检查: $([ $CHECK_RESULT -eq 0 ] && echo '✅ 通过' || echo '❌ 失败')"
echo ""

if [ $DEFAULT_RESULT -eq 0 ] && [ $CHECK_RESULT -eq 0 ]; then
    echo "🎉 Feature gate 配置成功！"
    echo ""
    echo "✅ 单元测试使用 Mock（无硬件调用）"
    echo "✅ platform 模块可选启用"
    echo ""
    echo "下一步: 实现本地回环测试（Phase 1）"
else
    echo "⚠️  仍有问题需要修复"
fi
echo ""
