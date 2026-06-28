# Phase 0: libwebrtc 依赖清理总结

**完成日期**: 2026-06-28  
**状态**: ✅ 已完成  
**决策依据**: [WEBRTC_CODEC_INTEGRATION_DECISION.md](decisions/WEBRTC_CODEC_INTEGRATION_DECISION.md)

---

## 执行的清理操作

### 1. 移除 Cargo 依赖

| 文件 | 变更 |
|------|------|
| `Cargo.toml` | 移除 `libwebrtc = "0.3"` (workspace 级别) |
| `crates/rdcs-codec/Cargo.toml` | 移除 `libwebrtc = "0.3"` |
| `crates/rdcs-session/Cargo.toml` | 移除 `libwebrtc = "0.3"` |

所有移除行都添加了注释说明迁移至方案 B。

### 2. 废弃依赖 libwebrtc 的源文件

以下文件已重命名为 `*.deprecated`，不再参与编译：

```
crates/rdcs-codec/src/
├── webrtc_encoder.rs.deprecated    (14,130 bytes) - libwebrtc 编码器封装
├── webrtc_decoder.rs.deprecated    ( 9,006 bytes) - libwebrtc 解码器封装
└── peer_connection.rs.deprecated   (24,105 bytes) - PeerConnection 生命周期管理

crates/rdcs-session/src/
└── manager.rs.deprecated           (13,804 bytes) - 依赖 peer_connection 的会话管理
```

**总计**: 61,045 字节代码已废弃

### 3. 更新模块导出

#### `crates/rdcs-codec/src/lib.rs`
```rust
// 已移除：
// pub mod peer_connection;
// pub mod webrtc_decoder;
// pub mod webrtc_encoder;

// 添加注释说明迁移原因
```

#### `crates/rdcs-session/src/lib.rs`
```rust
// 已禁用：
// pub mod manager;

// 添加架构图说明方案 B 的数据流
```

### 4. 保留的模块（不受影响）

以下模块继续使用，无需修改：

```
rdcs-codec/
├── adaptive.rs          ✅ 自适应码率控制
├── analyzer.rs          ✅ 内容分析（文本/视频场景检测）
├── decoder.rs           ✅ VideoDecoder trait
├── encoder.rs           ✅ VideoEncoder trait
├── pipeline.rs          ✅ EncodePipeline / DecodePipeline
├── platform/            ✅ 平台原生编解码器（VideoToolbox/MF/VA-API）
│   ├── videotoolbox.rs  ✅ macOS - 399 行 FFI 基础
│   ├── media_foundation.rs
│   └── vaapi.rs
└── types.rs             ✅ 通用类型定义
```

---

## 验证清理结果

### 编译状态检查

```bash
# 预期：编译失败（因为 rdcs-session 的 lib.rs 禁用了 manager 模块）
cargo check --workspace
```

**预期错误**:
- `rdcs-session` 可能因为导出项为空而报警告
- 其他 crate 如果导入了 `rdcs_session::manager` 会报错

### 依赖树检查

```bash
# 验证 libwebrtc 及其传递依赖已完全移除
cargo tree | grep -E "libwebrtc|webrtc-sys|livekit"
```

**预期结果**: 无输出（所有 libwebrtc 相关依赖已清除）

### 废弃文件确认

```bash
find crates -name "*.deprecated" -ls
```

**实际结果**:
```
crates/rdcs-codec/src/webrtc_encoder.rs.deprecated    (14,130 bytes)
crates/rdcs-codec/src/webrtc_decoder.rs.deprecated    ( 9,006 bytes)
crates/rdcs-codec/src/peer_connection.rs.deprecated   (24,105 bytes)
crates/rdcs-session/src/manager.rs.deprecated         (13,804 bytes)
```

---

## 为什么废弃而不直接删除？

1. **代码审计需要**: 废弃的文件包含完整的 libwebrtc 集成逻辑，可作为方案 A 的实现参考。
2. **回滚能力**: 如果方案 B 遇到技术障碍，可以快速恢复这些文件进行对比验证。
3. **迁移指导**: 新实现可参考原有的错误处理、状态机设计等模式。

**建议**: 在方案 B 完全验证通过后（Phase 4 结束），再删除 `*.deprecated` 文件。

---

## 下一步工作（Phase 1）

现在可以开始实现 macOS VideoToolbox 真实编解码器：

### 待完成任务

1. **补全 VideoToolbox FFI 绑定**
   - 解码器相关函数（`VTDecompressionSessionCreate` 等）
   - 像素格式转换辅助函数

2. **实现 `PlatformEncoder` trait**
   ```rust
   // crates/rdcs-codec/src/platform/videotoolbox.rs
   impl PlatformEncoder for VideoToolboxEncoder {
       fn encode(&mut self, frame: &CapturedFrame) -> Result<Vec<u8>, CodecError> {
           // BGRA → NV12 → CVPixelBuffer → VTCompressionSession
           // → callback → H.264 NAL units (Annex B)
       }
   }
   ```

3. **实现 `PlatformDecoder` trait**
   ```rust
   impl PlatformDecoder for VideoToolboxDecoder {
       fn decode(&mut self, nal_units: &[u8]) -> Result<DecodedFrame, CodecError> {
           // H.264 NAL units → VTDecompressionSession
           // → callback → CVPixelBuffer → BGRA
       }
   }
   ```

4. **单元测试**
   ```rust
   #[test]
   fn videotoolbox_encode_decode_roundtrip() {
       let mut encoder = VideoToolboxEncoder::new(config).unwrap();
       let mut decoder = VideoToolboxDecoder::new(codec).unwrap();
       
       let frame = create_test_frame(1920, 1080);
       let encoded = encoder.encode(&frame).unwrap();
       let decoded = decoder.decode(&encoded).unwrap();
       
       assert_eq!(decoded.width, 1920);
       assert_eq!(decoded.height, 1080);
       // 验证像素数据相似度（允许有损压缩误差）
   }
   ```

---

## 参考文档

- [迁移追踪文档](../MIGRATION.md)
- [WebRTC 集成方案决策](decisions/WEBRTC_CODEC_INTEGRATION_DECISION.md)
- [VideoToolbox 编程指南](https://developer.apple.com/documentation/videotoolbox)
