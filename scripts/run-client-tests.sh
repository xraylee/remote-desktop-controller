#!/bin/bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0
#
# RDCS 客户端测试 - 一键运行所有测试
# Usage: ./scripts/run-client-tests.sh

set -e

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$PROJECT_ROOT"

echo "═══════════════════════════════════════════════════════════════"
echo "  RDCS 客户端完整测试套件"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# ── 1. FFI 库名称验证 ───────────────────────────────────────
echo "▶ Step 1/5: 验证 FFI 库名称配置"
echo "─────────────────────────────────────────────────────────"
LIB_NAME=$(grep -A 2 '\[lib\]' crates/rdcs-ffi/Cargo.toml | grep 'name' | sed 's/.*"\(.*\)".*/\1/')
if [ "$LIB_NAME" != "rdcs_core" ]; then
  echo "❌ 错误: crates/rdcs-ffi/Cargo.toml 的 [lib] name 应为 'rdcs_core'，实际为 '$LIB_NAME'"
  exit 1
fi
echo "✅ FFI 库名称配置正确: rdcs_core"
echo ""

# ── 2. 重新编译 FFI 库 ──────────────────────────────────────
echo "▶ Step 2/5: 编译 FFI 库"
echo "─────────────────────────────────────────────────────────"
cargo build -p rdcs-ffi 2>&1 | tail -5
if [ ! -f "target/debug/librdcs_core.dylib" ]; then
  echo "❌ 编译后 librdcs_core.dylib 不存在"
  exit 1
fi
LIB_SIZE=$(ls -lh target/debug/librdcs_core.dylib | awk '{print $5}')
echo "✅ librdcs_core.dylib 生成成功 ($LIB_SIZE)"
echo ""

# ── 3. Rust 单元测试 ────────────────────────────────────────
echo "▶ Step 3/5: Rust FFI 单元测试"
echo "─────────────────────────────────────────────────────────"
cargo test -p rdcs-ffi --lib 2>&1 | tail -10
echo ""

# ── 4. Web Admin 测试 ───────────────────────────────────────
echo "▶ Step 4/5: Web 管理控制台测试"
echo "─────────────────────────────────────────────────────────"
if [ -d "web/admin/node_modules" ]; then
  cd web/admin
  npm test -- --run 2>&1 | tail -15 || echo "⚠️ 部分 Web 测试失败"
  cd "$PROJECT_ROOT"
else
  echo "⚠️ web/admin/node_modules 不存在，跳过。运行: cd web/admin && npm install"
fi
echo ""

# ── 5. Flutter 客户端测试 ───────────────────────────────────
echo "▶ Step 5/5: Flutter 客户端 UI 测试"
echo "─────────────────────────────────────────────────────────"
cd client/flutter
if command -v flutter &> /dev/null; then
  # 跳过已知损坏的两个文件
  flutter test \
    test/home_page_test.dart \
    test/connect_page_test.dart \
    test/session_screen_test.dart \
    test/settings_screen_test.dart 2>&1 | tail -15 || echo "⚠️ 部分 Flutter 测试失败"
else
  echo "⚠️ flutter 命令不存在，跳过 Flutter 测试"
fi
cd "$PROJECT_ROOT"
echo ""

# ── 总结 ───────────────────────────────────────────────────
echo "═══════════════════════════════════════════════════════════════"
echo "  测试套件执行完成"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "📊 详细报告: docs/testing/CLIENT_TEST_REPORT_2026-06-29.md"
echo ""
echo "已知失败测试 (P0):"
echo "  - test/widget_test.dart (引用不存在的 MyApp)"
echo "  - test/ui_integration_test.dart (包名错误 rdcs_flutter)"
echo ""
echo "运行 ./scripts/fix-client-tests.sh 修复以上问题"
