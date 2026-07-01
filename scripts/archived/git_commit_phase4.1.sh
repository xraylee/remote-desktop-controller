#!/bin/bash
# Phase 4.1 - Git 提交脚本
# Usage: ./git_commit_phase4.1.sh

set -e

echo "========================================="
echo "准备提交 Phase 4.1 代码"
echo "========================================="
echo ""

echo "Step 1: 添加新文件..."

# 测试代码
git add crates/rdcs-connection/examples/hardware_encoder_test.rs

# 测试脚本
git add test_hardware_encoder.sh
git add TEST_COMMANDS.sh

# 文档
git add docs/phases/PHASE4.1_HARDWARE_ENCODER.md
git add docs/plans/PHASE4.1_HARDWARE_ENCODER_PLAN.md
git add PHASE4.1_STATUS.md

echo "✅ 文件已添加"
echo ""

echo "Step 2: 检查 Git 状态..."
git status --short
echo ""

echo "Step 3: 创建提交..."
git commit -m "feat: Phase 4.1 - VideoToolbox hardware encoder integration

Add hardware encoder performance testing framework:
- Create hardware_encoder_test.rs example
- Add automated comparison script test_hardware_encoder.sh
- Update TEST_COMMANDS.sh with new test commands
- Complete Phase 4.1 documentation

Expected performance improvements:
- Encoding latency: 45ms → 20ms (2.25x speedup)
- End-to-end latency: 79ms → 54ms (31.6% reduction)
- CPU usage: 80-100% → 10-20% (hardware acceleration)

Technical implementation:
- Use NativeVideoEncoder with feature flag control
- software-encoder feature → OpenH264 (cross-platform)
- No feature (default) → VideoToolbox (macOS hardware)

New files:
- crates/rdcs-connection/examples/hardware_encoder_test.rs
- test_hardware_encoder.sh
- docs/phases/PHASE4.1_HARDWARE_ENCODER.md
- docs/plans/PHASE4.1_HARDWARE_ENCODER_PLAN.md
- PHASE4.1_STATUS.md

Modified files:
- TEST_COMMANDS.sh

Next steps:
- Run ./test_hardware_encoder.sh to get actual performance data
- Create benchmark report with results
- Update video_e2e_test.rs to use hardware encoder by default
- Continue to Phase 4.2: Real screen capture integration"

echo "✅ 提交已创建"
echo ""

echo "Step 4: 准备推送..."
echo "运行以下命令推送到远程仓库："
echo "  git push origin main"
echo ""

echo "========================================="
echo "✅ 提交准备完成！"
echo "========================================="
