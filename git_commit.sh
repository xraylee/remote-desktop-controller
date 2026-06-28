#!/bin/bash
# Git 提交脚本 - Phase 3.4+ 完整成果
# 使用说明: ./git_commit.sh

set -e

echo "========================================="
echo "准备提交 Phase 3.4+ 代码"
echo "========================================="
echo ""

echo "Step 1: 添加新文件..."

# 核心代码
git add crates/rdcs-connection/src/video_channel.rs
git add crates/rdcs-connection/src/frame_reassembler.rs
git add crates/rdcs-connection/examples/video_e2e_test.rs
git add crates/rdcs-connection/src/lib.rs
git add crates/rdcs-connection/Cargo.toml
git add crates/rdcs-connection/src/real_ice_agent.rs
git add crates/rdcs-connection/examples/video_datachannel_test.rs

# 测试修复
git add tests/integration_connection.rs
git add tests/transfer_integration_test.rs
git add tests/e2e_performance_test.rs

# 文档
git add docs/testing/E2E_VIDEO_STREAMING_SUCCESS.md
git add docs/testing/COMPILE_ERROR_FIX_REPORT.md
git add docs/CURRENT_PHASE.md
git add docs/MVP.md
git add docs/E2E_TEST_PLAN.md
git add docs/EXECUTION_CHECKLIST.md
git add docs/STANDARD_STRUCTURE.md
git add docs/archived/
git add docs/testing/CROSS_ARCHITECTURE_TEST.md

# 工具脚本
git add TEST_COMMANDS.sh
git add check_build.sh
git add TODO.md

# 其他
git add Cargo.lock
git add README.md
git add docs/README.md

echo "✅ 文件已添加"
echo ""

echo "Step 2: 检查 Git 状态..."
git status --short
echo ""

echo "Step 3: 创建提交..."
git commit -m "feat: Phase 3.4+ - End-to-end video streaming over DataChannel

Major achievements:
- Implement VideoChannel wrapper for DataChannel
- Implement FrameReassembler with protocol header (8 bytes + payload)
- Fix DataChannel offerer/answerer role asymmetry
- Integrate OpenH264 encoder/decoder
- Complete end-to-end video streaming test
- 100% success rate (30/30 frames)
- Average latency 79ms < 100ms target
- Fix compilation errors after Phase 3.3 DTLS changes

New files:
- crates/rdcs-connection/src/video_channel.rs
- crates/rdcs-connection/src/frame_reassembler.rs
- crates/rdcs-connection/examples/video_e2e_test.rs
- docs/testing/E2E_VIDEO_STREAMING_SUCCESS.md
- docs/testing/COMPILE_ERROR_FIX_REPORT.md
- TEST_COMMANDS.sh
- check_build.sh
- TODO.md

Modified files:
- crates/rdcs-connection/src/real_ice_agent.rs (add DataChannel support)
- crates/rdcs-connection/Cargo.toml (add dev-dependencies)
- crates/rdcs-connection/src/lib.rs (export new modules)
- crates/rdcs-connection/examples/video_datachannel_test.rs (fix ICE order)
- tests/integration_connection.rs (add fingerprint field)
- tests/transfer_integration_test.rs (remove unused mut)
- tests/e2e_performance_test.rs (prefix unused variable)

Documentation:
- Archive 50+ deprecated docs to docs/archived/
- Create standardized documentation structure
- Add comprehensive test reports
- Add TODO task list

Technical details:
- Frame protocol: 8-byte header (frame_id, is_keyframe, chunk_index, total_chunks)
- Automatic chunking at 16KB boundary
- Out-of-order reassembly support
- BGRA ↔ YUV420 color space conversion
- OpenH264 software encoding: ~45ms average
- OpenH264 software decoding: ~32ms average
- ICE P2P + DTLS encryption
- DataChannel reliable ordered transport

Performance:
- Resolution: 1280x720 @ 30fps
- Bitrate: 2 Mbps
- Encode latency: ~45ms (keyframe: ~57ms)
- Decode latency: ~32ms average
- Network latency: ~2ms (local P2P)
- End-to-end latency: ~79ms ✅
- Success rate: 100% (30/30 frames)

Next phase:
- Phase 4.1: Hardware encoder integration (VideoToolbox)
- Phase 4.2: Real screen capture integration
- Phase 4.3: Flutter UI video display"

echo "✅ 提交已创建"
echo ""

echo "Step 4: 准备推送..."
echo "请确认你的代理设置后运行："
echo "  git push origin main"
echo ""
echo "或者如果使用 Clash 代理："
echo "  git config --global http.proxy http://127.0.0.1:7890"
echo "  git config --global https.proxy http://127.0.0.1:7890"
echo "  git push origin main"
echo ""

echo "========================================="
echo "✅ 提交准备完成！"
echo "========================================="
