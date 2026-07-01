#!/bin/bash
# 快速修复 - Flutter FFI 连接问题
# 复制粘贴运行即可

echo "🔧 开始修复..."

# 步骤 1: 重新编译 Rust FFI 库
echo ""
echo "📦 编译 Rust FFI 库..."
cargo build -p rdcs-ffi

# 步骤 2: 验证
echo ""
echo "✅ 验证库文件..."
if [ -f "target/debug/librdcs_core.dylib" ]; then
    ls -lh target/debug/librdcs_core.dylib
    echo "✅ 库文件正确生成"
else
    echo "❌ 错误: librdcs_core.dylib 未生成"
    exit 1
fi

# 步骤 3: 清理并重新构建 Flutter
echo ""
echo "🧹 清理 Flutter 构建..."
cd client/flutter
flutter clean

echo ""
echo "🎉 修复完成！"
echo ""
echo "现在运行："
echo "  flutter run -d macos"
echo ""
echo "预期日志："
echo "  ✅ Loading from: .../Contents/Frameworks/librdcs_core.dylib"
