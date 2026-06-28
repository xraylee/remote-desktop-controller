# 测试文档索引

本目录包含项目的测试相关文档。

## 📋 测试规范

- [TESTING_GUIDELINES.md](TESTING_GUIDELINES.md) - 测试规范和开发流程

## 🧪 集成测试

- [CROSS_ARCHITECTURE_TEST.md](CROSS_ARCHITECTURE_TEST.md) - **跨架构 ICE 连接测试指南** ⭐ 推荐
  - ARM ↔ Intel Mac P2P 连接测试
  - STUN 穿透验证
  - 详细操作步骤和问题排查

## 📊 阶段完成报告

### Phase 1 - 编解码
- [PHASE1_COMPLETION_REPORT.md](PHASE1_COMPLETION_REPORT.md) - Phase 1 编解码完成报告

### Phase 2 - 网络传输
- [PHASE2_COMPLETION_REPORT.md](PHASE2_COMPLETION_REPORT.md) - Phase 2 完成报告
- [PHASE2_TEST_SUMMARY.md](PHASE2_TEST_SUMMARY.md) - Phase 2 测试总结
- [TCP_TRANSPORT_IMPLEMENTATION.md](TCP_TRANSPORT_IMPLEMENTATION.md) - TCP 传输实现
- [OPENH264_INTEGRATION_REPORT.md](OPENH264_INTEGRATION_REPORT.md) - OpenH264 集成报告

### Phase 3 - NAT 穿透和加密
- [PHASE3_NAT_IMPLEMENTATION.md](PHASE3_NAT_IMPLEMENTATION.md) - NAT 穿透实现
- [PHASE3_ICE_SUCCESS_REPORT.md](PHASE3_ICE_SUCCESS_REPORT.md) - ICE 成功报告
- [PHASE3_DTLS_SUCCESS_REPORT.md](PHASE3_DTLS_SUCCESS_REPORT.md) - DTLS 加密成功报告
- [PHASE3_NETWORK_TEST_PLAN.md](PHASE3_NETWORK_TEST_PLAN.md) - 网络测试计划
- [PHASE3_NETWORK_TEST_RESULTS.md](PHASE3_NETWORK_TEST_RESULTS.md) - 网络测试结果
- [PHASE3_VIDEO_DATACHANNEL_IMPLEMENTATION.md](PHASE3_VIDEO_DATACHANNEL_IMPLEMENTATION.md) - 视频数据通道实现

## 🔍 问题诊断

- [VIDEOTOOLBOX_CRASH_DIAGNOSIS.md](VIDEOTOOLBOX_CRASH_DIAGNOSIS.md) - VideoToolbox 崩溃诊断

## 🚀 快速开始

### 当前推荐测试：跨架构 ICE 连接

这是验证项目核心功能的最佳时机！

**在 Intel Mac 上**：
```bash
cd /path/to/rdcs
./scripts/ice-test-server.sh
```

**在 Apple Silicon Mac 上**：
```bash
cd /Users/lc/Development/source/remote-desktop-controller
./scripts/ice-test-client.sh
```

详细步骤见：[CROSS_ARCHITECTURE_TEST.md](CROSS_ARCHITECTURE_TEST.md)

快速参考：[../../ICE_TEST_QUICK_REF.md](../../ICE_TEST_QUICK_REF.md)

## 📈 测试进展

| 阶段 | 功能 | 状态 |
|------|------|------|
| Phase 1 | 编解码器（Mock） | ✅ 完成 |
| Phase 1 | 硬件加速 | ⚠️ 待修复 |
| Phase 2 | TCP 传输 | ✅ 完成 |
| Phase 2 | OpenH264 集成 | ✅ 完成 |
| Phase 3 | ICE NAT 穿透 | ✅ 完成 |
| Phase 3 | DTLS 加密 | ✅ 完成 |
| Phase 3 | 跨架构测试 | 🔄 进行中 |

## 🔗 相关资源

- [脚本工具](../../scripts/README.md)
- [项目结构](../../PROJECT_STRUCTURE.md)
- [开发指南](../DEVELOPMENT.md)

---

**最后更新**: 2026-06-28
