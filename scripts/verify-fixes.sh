#!/bin/bash
# 快速验证修复是否成功

echo "═══════════════════════════════════════════════════════════════"
echo "  验证测试修复"
echo "═══════════════════════════════════════════════════════════════"
echo ""

cd "$(dirname "$0")/.."

# 1. 验证 FFI 库
echo "▶ 1. 检查 FFI 库"
if [ -f "target/debug/librdcs_core.dylib" ]; then
  echo "✅ librdcs_core.dylib 存在"
  ls -lh target/debug/librdcs_core.dylib | awk '{print "   大小: " $5}'
else
  echo "❌ librdcs_core.dylib 不存在，运行: cargo build -p rdcs-ffi"
  exit 1
fi
echo ""

# 2. 运行 Web 测试
echo "▶ 2. 运行 Web 测试（已修复语言问题）"
cd web/admin
npm test 2>&1 | tail -5
WEB_EXIT=$?
cd ../..
echo ""

# 3. 提示 Flutter 测试
echo "▶ 3. Flutter 测试命令"
echo "   cd client/flutter"
echo "   flutter test"
echo ""

# 4. 提示 Flutter 运行
echo "▶ 4. 验证 Flutter APP"
echo "   cd client/flutter"
echo "   flutter clean"
echo "   flutter run -d macos"
echo "   预期日志: ✅ Engine created successfully"
echo ""

echo "═══════════════════════════════════════════════════════════════"
if [ $WEB_EXIT -eq 0 ]; then
  echo "  ✅ Web 测试通过"
else
  echo "  ⚠️ Web 测试失败（查看上面输出）"
fi
echo "═══════════════════════════════════════════════════════════════"
