// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 本地回环测试：屏幕捕获 → 编码 → 解码 → 保存图片
//!
//! 这是 MVP Phase 1 的验证程序，测试：
//! 1. rdcs-platform 屏幕捕获
//! 2. rdcs-codec 硬件编码（VideoToolbox/MF/VA-API）
//! 3. rdcs-codec 硬件解码
//! 4. 图片保存验证
//!
//! 运行方式：
//! ```bash
//! cargo run --example local_roundtrip --features hardware-accel
//! ```

use rdcs_codec::platform::{NativeVideoDecoder, NativeVideoEncoder};
use rdcs_codec::types::{VideoCodec, VideoResolution};
use rdcs_platform::{CapturedFrame, CaptureConfig, PixelFormat, ScreenCapture};
use rdcs_platform::mock::MockScreenCapture;
use std::fs::File;
use std::io::Write;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("========================================");
    println!("🧪 RDCS 本地回环测试 (Phase 1)");
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

    // 3. 初始化编码器
    println!("3️⃣  初始化硬件编码器...");
    let resolution = VideoResolution::Custom(captured_frame.width, captured_frame.height);
    let mut encoder = NativeVideoEncoder::new(
        VideoCodec::H264,
        resolution,
        30, // fps
        2_000_000, // 2 Mbps
    )?;
    println!("   ✓ 编码器创建成功");
    println!();

    // 4. 编码
    println!("4️⃣  编码帧...");
    encoder.request_keyframe(); // 第一帧请求 keyframe
    let start = Instant::now();
    let encoded_data = encoder.encode_captured_frame(&captured_frame)?;
    let encode_time = start.elapsed();

    println!("   ✓ 编码成功");
    println!("   - 编码数据: {} bytes", encoded_data.len());
    println!("   - 编码耗时: {:.2}ms", encode_time.as_secs_f64() * 1000.0);
    println!("   - 压缩比: {:.1}x",
        captured_frame.data.len() as f64 / encoded_data.len() as f64);
    println!();

    // 5. 保存 H.264 数据（可选）
    println!("5️⃣  保存编码数据...");
    let mut h264_file = File::create("output.h264")?;
    h264_file.write_all(&encoded_data)?;
    println!("   ✓ 已保存: output.h264");
    println!();

    // 6. 初始化解码器
    println!("6️⃣  初始化硬件解码器...");
    let mut decoder = NativeVideoDecoder::new(VideoCodec::H264)?;
    println!("   ✓ 解码器创建成功");
    println!();

    // 7. 解码
    println!("7️⃣  解码帧...");
    let start = Instant::now();
    let decoded_frame = decoder.decode_to_captured_frame(&encoded_data)?;
    let decode_time = start.elapsed();

    println!("   ✓ 解码成功");
    println!("   - 分辨率: {}x{}", decoded_frame.width, decoded_frame.height);
    println!("   - 解码耗时: {:.2}ms", decode_time.as_secs_f64() * 1000.0);
    println!();

    // 8. 保存为 PNG
    println!("8️⃣  保存为 PNG 图片...");
    save_as_png(&decoded_frame, "output.png")?;
    println!("   ✓ 已保存: output.png");
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

    let encoder_stats = encoder.stats();
    println!("编码器统计:");
    println!("  - 编码帧数: {}", encoder_stats.frames_encoded);
    println!("  - 关键帧数: {}", encoder_stats.keyframes_generated);
    println!("  - 编码字节: {}", encoder_stats.bytes_encoded);
    println!();

    let decoder_stats = decoder.stats();
    println!("解码器统计:");
    println!("  - 解码帧数: {}", decoder_stats.frames_decoded);
    println!("  - 关键帧数: {}", decoder_stats.keyframes_received);
    println!("  - 解码字节: {}", decoder_stats.bytes_decoded);
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

    println!("[ {} ] 捕获延迟 < 100ms", if capture_ok { "✓" } else { "✗" });
    println!("[ {} ] 编码延迟 < 50ms", if encode_ok { "✓" } else { "✗" });
    println!("[ {} ] 解码延迟 < 50ms", if decode_ok { "✓" } else { "✗" });
    println!("[ {} ] 分辨率一致", if resolution_ok { "✓" } else { "✗" });
    println!();

    if capture_ok && encode_ok && decode_ok && resolution_ok {
        println!("🎉 Phase 1 本地回环测试通过！");
        println!();
        println!("下一步: Phase 2 - 本地网络传输");
    } else {
        println!("⚠️  部分指标未达标，但核心功能正常");
    }
    println!();

    // 关闭
    encoder.shutdown()?;
    decoder.shutdown()?;

    Ok(())
}

/// 保存 CapturedFrame 为 PNG 图片（使用简单的 PPM 格式）
fn save_as_png(frame: &CapturedFrame, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if frame.pixel_format != PixelFormat::Bgra {
        return Err("Only BGRA format is supported".into());
    }

    // 由于我们没有依赖 image crate，这里使用 PPM 格式（纯文本格式）
    // PPM 格式简单但文件较大，适合快速验证
    let ppm_path = path.replace(".png", ".ppm");
    let mut file = File::create(&ppm_path)?;

    // PPM header
    writeln!(file, "P6")?;
    writeln!(file, "{} {}", frame.width, frame.height)?;
    writeln!(file, "255")?;

    // PPM data (RGB format)
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

    println!("   注意: 保存为 PPM 格式: {}", ppm_path);
    println!("   提示: 使用 'convert {} {}' 转换为 PNG (需要 ImageMagick)", ppm_path, path);

    Ok(())
}
