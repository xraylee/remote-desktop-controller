#!/bin/bash
# RDCS 项目整体编译检查

# 加载 Rust 环境
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

echo "=========================================="
echo "🔍 RDCS 编译检查"
echo "=========================================="
echo ""

cd "$(dirname "$0")/.."

# 1. 检查所有 Rust crates
echo "1️⃣  检查 Rust workspace..."
cargo check --workspace 2>&1 | tee check.log
RUST_CHECK=$?

if [ $RUST_CHECK -eq 0 ]; then
    echo "✅ Rust workspace 编译通过"
else
    echo "❌ Rust workspace 编译失败"
fi
echo ""

# 2. 检查 Go API
echo "2️⃣  检查 Go API 服务..."
cd api
go build ./... 2>&1 | tee ../go-check.log
GO_CHECK=$?
cd ..

if [ $GO_CHECK -eq 0 ]; then
    echo "✅ Go API 编译通过"
else
    echo "❌ Go API 编译失败"
fi
echo ""

# 3. 检查 Flutter 客户端
echo "3️⃣  检查 Flutter 客户端..."
cd client
flutter analyze 2>&1 | tee ../flutter-check.log
FLUTTER_CHECK=$?
cd ..

if [ $FLUTTER_CHECK -eq 0 ]; then
    echo "✅ Flutter 客户端分析通过"
else
    echo "❌ Flutter 客户端分析失败"
fi
echo ""

# 4. 检查 Web 管理后台
echo "4️⃣  检查 Web 管理后台..."
cd web/admin
npm run lint 2>&1 | tee ../../web-check.log
WEB_CHECK=$?
cd ../..

if [ $WEB_CHECK -eq 0 ]; then
    echo "✅ Web 管理后台检查通过"
else
    echo "⚠️  Web 管理后台检查失败（非阻塞）"
fi
echo ""

# 总结
echo "=========================================="
echo "📊 编译检查总结"
echo "=========================================="
echo "Rust workspace: $([ $RUST_CHECK -eq 0 ] && echo '✅' || echo '❌')"
echo "Go API:         $([ $GO_CHECK -eq 0 ] && echo '✅' || echo '❌')"
echo "Flutter:        $([ $FLUTTER_CHECK -eq 0 ] && echo '✅' || echo '❌')"
echo "Web Admin:      $([ $WEB_CHECK -eq 0 ] && echo '✅' || echo '⚠️')"
echo ""

if [ $RUST_CHECK -eq 0 ] && [ $GO_CHECK -eq 0 ] && [ $FLUTTER_CHECK -eq 0 ]; then
    echo "🎉 核心模块编译检查通过！"
    exit 0
else
    echo "❌ 部分模块编译检查失败，请查看日志"
    exit 1
fi
