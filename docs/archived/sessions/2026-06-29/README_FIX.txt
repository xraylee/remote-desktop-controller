═══════════════════════════════════════════════════════════════
  Flutter APP "engine not create" 问题 - 快速修复指南
═══════════════════════════════════════════════════════════════

问题：Flutter APP 无法启动，报错 "engine not create"
原因：FFI 库文件名不匹配

────────────────────────────────────────────────────────────
  ⚡ 一键修复（推荐）
────────────────────────────────────────────────────────────

在项目根目录运行：

  ./quick-fix.sh

或者手动执行：

  # 1. 重新编译
  cargo build -p rdcs-ffi

  # 2. 清理 Flutter
  cd client/flutter && flutter clean

  # 3. 重新运行
  flutter run -d macos

────────────────────────────────────────────────────────────
  ✅ 验证修复
────────────────────────────────────────────────────────────

成功日志应显示：

  flutter: ✅ Loading from: .../librdcs_core.dylib
  flutter: ✅ Engine created successfully

失败日志（修复前）：

  Exception: Failed to load librdcs_core.dylib
  [ERROR] Unhandled exception

────────────────────────────────────────────────────────────
  📚 详细文档
────────────────────────────────────────────────────────────

• 完整诊断报告：SIGNALING_CONNECTION_DIAGNOSIS.md
• 修复操作指南：docs/troubleshooting/FLUTTER_FFI_FIX.md  
• 修复总结：FIX_SUMMARY.md

────────────────────────────────────────────────────────────
  🔧 故障排查
────────────────────────────────────────────────────────────

如果仍然失败：

1. 检查库文件是否存在：
   ls -la target/debug/*.dylib

2. 检查 Cargo.toml 配置：
   grep "name = " crates/rdcs-ffi/Cargo.toml
   应该看到: name = "rdcs_core"

3. 完整重新构建：
   cargo clean
   cargo build -p rdcs-ffi
   cd client/flutter
   flutter clean
   flutter run -d macos 2>&1 | tee debug.log

4. 查看详细日志：
   cat debug.log | grep -A 10 "librdcs"

────────────────────────────────────────────────────────────

修复日期：2026-06-29
修复状态：✅ 已完成

═══════════════════════════════════════════════════════════════
