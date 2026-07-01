#!/bin/bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0
#
# 修复 Flutter 测试文件中的已知问题
# Usage: ./scripts/fix-client-tests.sh

set -e

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
cd "$PROJECT_ROOT/client/flutter"

echo "═══════════════════════════════════════════════════════════════"
echo "  修复 Flutter 测试文件"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# 1. 删除损坏的 widget_test.dart（引用不存在的 MyApp）
if [ -f "test/widget_test.dart" ]; then
  echo "▶ 删除损坏的 test/widget_test.dart (引用不存在的 MyApp)"
  rm test/widget_test.dart
  echo "✅ 已删除"
fi
echo ""

# 2. 修复 ui_integration_test.dart 中错误的包名
if [ -f "test/ui_integration_test.dart" ]; then
  echo "▶ 修复 test/ui_integration_test.dart 中错误的包名"
  echo "   rdcs_flutter → rdcs_client"
  sed -i.bak 's/package:rdcs_flutter\//package:rdcs_client\//g' test/ui_integration_test.dart
  rm test/ui_integration_test.dart.bak
  echo "✅ 已修复"
fi
echo ""

# 3. 验证修复
echo "▶ 验证修复"
echo "─────────────────────────────────────────────────────────"
if grep -q "rdcs_flutter" test/*.dart 2>/dev/null; then
  echo "❌ 仍存在 rdcs_flutter 引用"
  grep -l "rdcs_flutter" test/*.dart
  exit 1
fi
echo "✅ 所有测试文件已修复"
echo ""

echo "═══════════════════════════════════════════════════════════════"
echo "  修复完成，运行测试："
echo "  cd client/flutter && flutter test"
echo "═══════════════════════════════════════════════════════════════"
