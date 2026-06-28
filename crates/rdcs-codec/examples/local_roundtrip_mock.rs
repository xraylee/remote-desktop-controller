// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 本地回环测试：屏幕捕获 → 编码 → 解码 → 保存图片（使用 Mock）
//!
//! 这是 MVP Phase 1 的验证程序，测试：
//! 1. rdcs-platform Mock 屏幕捕获
//! 2. rdcs-codec Stub 编码器（纯软件）
//! 3. rdcs-codec Stub 解码器
//! 4. 图片保存验证
//!
//! 运行方式：
//! ```bash
//! cargo run --example local_roundtrip_mock
//! ```

use rdcs_codec::encoder::{StubEncoder, VideoEncoder, EncoderConfig, CodecType};
use rdcs_codec::decoder::{StubDecoder, VideoDecoder};
use rdcs_platform::{CapturedFrame, CaptureConfig, PixelFormat, ScreenCapture};
use rdcs_platform::mock::MockScreenCapture;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("🧪 RDCS 本地回环测试 (Phase 1 - Mock)");
    println!("========================================");
    println!();

    // 1. 初始化屏幕捕获（使用 Mock）
    println!("1️⃣  初始化屏幕捕获（Mock）...");

    // 创建测试帧
    let width = 1920u32;
    let height = 1080u32;
    let bpp = 4; // BGRA
    let test_frame = CapturedFrame {
        data: vec![128u8; (width * height * bpp) as usize], // 灰色填充
        width,
        height,
        pixel_format: PixelFormat::Bgra,
        stride: width * bpp,
        display_id: 1,
        timestamp_us: 0,
    };

    let capture = MockScreenCapture::with_frames(vec![test_frame]);
    let displays = capture.displays()?;

    println!("   ✓ 找到 {} 个显示器", displays.len());
    println!("   ✓ 使用显示器 ID: {}", displays[0].id);
    println!();

    // 2. 捕获一帧
    println!("2️⃣  捕获屏幕帧...");
    let start = Instant::now();
    let config = CaptureConfig::default();
    let receiver = capture.start(config)?;
    let captured_frame = receiver.recv()?;
    capture.stop()?;
    let capture_time = start.elapsed();

    println!("   ✓ 捕获成功");
    println!("   - 分辨率: {}x{}", captured_frame.width, captured_frame.height);
    println!("   - 像素格式: {:?}", captured_frame.pixel_format);
    println!("   - 数据大小: {} bytes", captured_frame.data.len());
    println!("   - 捕获耗时: {:.2}ms", capture_time.as_secs_f64() * 1000.0);
    println!();

    // 3. 初始化编码器（Stub）
    println!("3️⃣  初始化 Stub 编码器...");
    let encoder_config = EncoderConfig {
        codec: CodecType::H264,
        width: captured_frame.width,
        height: captured_frame.height,
        target_fps: 30,
        target_bitrate_bps: 2_000_000,
        keyframe_interval: 30,
        hardware_accel: false,
    };
    let mut encoder = StubEncoder::new();
    encoder.configure(encoder_config)?;
    println!("   ✓ 编码器创建成功");
    println!();

    // 4. 编码
    println!("4️⃣  编码帧...");
    let start = Instant::now();
    let encoded_frame = encoder.encode(&captured_frame)?;
    let encode_time = start.elapsed();

    println!("   ✓ 编码成功");
    println!("   - 编码数据: {} bytes", encoded_frame.data.len());
    println!("   - 关键帧: {}", encoded_frame.is_keyframe);
    println!("   - 编码耗时: {:.2}ms", encode_time.as_secs_f64() * 1000.0);
    println!("   - 压缩比: {:.1}x (Stub 不压缩，仅封装)",
        captured_frame.data.len() as f64 / encoded_frame.data.len() as f64);
    println!();

    // 5. 保存编码数据（可选）
    println!("5️⃣  保存编码数据...");
    let mut stub_file = File::create("output.stub")?;
    stub_file.write_all(&encoded_frame.data)?;
    println!("   ✓ 已保存: output.stub");
    println!();

    // 6. 初始化解码器（Stub）
    println!("6️⃣  初始化 Stub 解码器...");
    let decoder_config = rdcs_codec::decoder::DecoderConfig {
        codec: CodecType::H264,
        width: captured_frame.width,
        height: captured_frame.height,
    };
    let mut decoder = StubDecoder::new();
    decoder.configure(decoder_config)?;
    println!("   ✓ 解码器创建成功");
    println!();

    // 7. 解码
    println!("7️⃣  解码帧...");
    let start = Instant::now();
    let decoded_frame = decoder.decode(&encoded_frame)?;
    let decode_time = start.elapsed();

    println!("   ✓ 解码成功");
    println!("   - 分辨率: {}x{}", decoded_frame.width, decoded_frame.height);
    println!("   - 解码耗时: {:.2}ms", decode_time.as_secs_f64() * 1000.0);
    println!();

    // 8. 保存为 PPM
    println!("8️⃣  保存为 PPM 图片...");
    save_as_ppm(&decoded_frame, "output.ppm")?;
    println!("   ✓ 已保存: output.ppm");
    println!();

    // 9. 统计信息
    println!("========================================");
    println!("📊 性能统计");
    println!("========================================");
    let total_time = capture_time + encode_time + decode_time;
    println!("捕获耗时:   {:.2}ms", capture_time.as_secs_f64() * 1000.0);
    println!("编码耗时:   {:.2}ms", encode_time.as_secs_f64() * 1000.0);
    println!("解码耗时:   {:.2}ms", decode_time.as_secs_f64() * 1000.0);
    println!("总耗时:     {:.2}ms", total_time.as_secs_f64() * 1000.0);
    println!("理论 FPS:   {:.1}", 1000.0 / total_time.as_secs_f64() / 1000.0);
    println!();

    // 10. 验证
    println!("========================================");
    println!("✅ 验收标准");
    println!("========================================");

    let capture_ok = capture_time.as_millis() < 100;
    let encode_ok = encode_time.as_millis() < 50;
    let decode_ok = decode_time.as_millis() < 50;
    let resolution_ok = decoded_frame.width == captured_frame.width
        && decoded_frame.height == captured_frame.height;
    let data_ok = decoded_frame.data == captured_frame.data;

    println!("[ {} ] 捕获延迟 < 100ms", if capture_ok { "✓" } else { "✗" });
    println!("[ {} ] 编码延迟 < 50ms", if encode_ok { "✓" } else { "✗" });
    println!("[ {} ] 解码延迟 < 50ms", if decode_ok { "✓" } else { "✗" });
    println!("[ {} ] 分辨率一致", if resolution_ok { "✓" } else { "✗" });
    println!("[ {} ] 数据完整性（像素一致）", if data_ok { "✓" } else { "✗" });
    println!();

    if capture_ok && encode_ok && decode_ok && resolution_ok && data_ok {
        println!("🎉 Phase 1 本地回环测试通过（Mock）！");
        println!();
        println!("✅ 编解码流程验证成功");
        println!("✅ 数据完整性验证通过");
        println!();
        println!("下一步:");
        println!("1. 使用硬件加速编码器（VideoToolbox/MF/VA-API）");
        println!("2. Phase 2 - 本地网络传输");
    } else {
        println!("⚠️  部分指标未达标");
    }
    println!();

    Ok(())
}

/// 保存 DecodedFrame 为 PPM 图片
fn save_as_ppm(frame: &rdcs_codec::decoder::DecodedFrame, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(path)?;

    // PPM header
    writeln!(file, "P6")?;
    writeln!(file, "{} {}", frame.width, frame.height)?;
    writeln!(file, "255")?;

    // PPM data (RGB format) - 假设输入是 BGRA 格式
    let stride = frame.stride as usize;
    for y in 0..frame.height as usize {
        for x in 0..frame.width as usize {
            let offset = y * stride + x * 4;
            if offset + 3 < frame.data.len() {
                let b = frame.data[offset];
                let g = frame.data[offset + 1];
                let r = frame.data[offset + 2];
                // PPM 是 RGB 顺序
                file.write_all(&[r, g, b])?;
            }
        }
    }

    println!("   提示: 转换为 PNG: convert {} output.png", path);

    Ok(())
}
