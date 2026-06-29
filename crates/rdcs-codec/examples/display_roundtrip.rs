// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 完整端到端测试：屏幕捕获 → 编码 → 解码 → 显示
//!
//! 这是 Phase 2 的验收程序，测试完整的视频流管道：
//! 1. rdcs-platform 屏幕捕获（Mock）
//! 2. rdcs-codec 编码（OpenH264）
//! 3. rdcs-codec 解码（OpenH264）
//! 4. rdcs-display 显示（SDL2）
//!
//! 运行方式：
//! ```bash
//! cargo run --example display_roundtrip --features software-encoder
//! ```
//!
//! 按 ESC 或关闭窗口退出

use rdcs_codec::platform::{NativeVideoDecoder, NativeVideoEncoder};
use rdcs_codec::types::{VideoCodec, VideoResolution};
use rdcs_display::{DisplayConfig, VideoDisplay};
use rdcs_platform::{CapturedFrame, CaptureConfig, PixelFormat, ScreenCapture};
use rdcs_platform::mock::MockScreenCapture;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("info,rdcs_codec=debug,rdcs_display=debug")
        .init();

    println!("========================================");
    println!("🎬 RDCS 端到端回环测试 (Phase 2)");
    println!("========================================");
    println!();

    // Configuration
    let width = 1280u32;
    let height = 720u32;
    let fps = 30u32;
    let bitrate = 2_000_000; // 2 Mbps
    let test_duration = Duration::from_secs(10);

    println!("📋 测试配置:");
    println!("  分辨率: {}x{}", width, height);
    println!("  帧率: {} FPS", fps);
    println!("  比特率: {} Mbps", bitrate / 1_000_000);
    println!("  测试时长: {}s", test_duration.as_secs());
    println!();

    // 1. 初始化屏幕捕获（使用动画测试帧）
    println!("1️⃣  初始化屏幕捕获（Mock with animation）...");
    let capture = create_animated_capture(width, height, fps, test_duration);
    println!("   ✓ Mock capture 创建成功");
    println!();

    // 2. 初始化编码器
    println!("2️⃣  初始化 OpenH264 编码器...");
    let resolution = VideoResolution::Custom(width, height);
    let mut encoder = NativeVideoEncoder::new(VideoCodec::H264, resolution, fps, bitrate)?;
    encoder.request_keyframe(); // 请求第一帧为关键帧
    println!("   ✓ 编码器创建成功");
    println!();

    // 3. 初始化解码器
    println!("3️⃣  初始化 OpenH264 解码器...");
    let mut decoder = NativeVideoDecoder::new(VideoCodec::H264)?;
    println!("   ✓ 解码器创建成功");
    println!();

    // 4. 初始化显示窗口
    println!("4️⃣  初始化显示窗口...");
    let display_config = DisplayConfig::default()
        .with_title("RDCS 端到端测试 - 按 ESC 退出")
        .with_size(width, height)
        .with_target_fps(fps);

    let mut display = VideoDisplay::new(display_config)?;
    println!("   ✓ 显示窗口创建成功");
    println!();

    // 5. 开始捕获
    println!("5️⃣  开始视频流处理...");
    let config = CaptureConfig::default();
    let receiver = capture.start(config)?;
    println!("   ✓ 捕获已开始");
    println!("   按 ESC 或关闭窗口可提前退出");
    println!();

    // 6. 主循环：捕获 → 编码 → 解码 → 显示
    let mut stats = PipelineStats::default();
    let start_time = Instant::now();

    loop {
        let loop_start = Instant::now();

        // Check if test duration exceeded
        if start_time.elapsed() > test_duration {
            println!("\n⏱️  测试时长达到 {}s，正常退出", test_duration.as_secs());
            break;
        }

        // Receive captured frame (with timeout)
        let captured_frame = match receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(frame) => frame,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                continue; // Try again
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                println!("\n⚠️  捕获流已断开");
                break;
            }
        };

        // Encode
        let encode_start = Instant::now();
        let encoded_data = encoder.encode_captured_frame(&captured_frame)?;
        let encode_time = encode_start.elapsed();

        // Decode
        let decode_start = Instant::now();
        let decoded_frame = decoder.decode_to_captured_frame(&encoded_data)?;
        let decode_time = decode_start.elapsed();

        // Display
        let display_start = Instant::now();
        let should_continue = display.render_frame(&decoded_frame)?;
        let display_time = display_start.elapsed();

        if !should_continue {
            println!("\n👋 用户退出");
            break;
        }

        // Update stats
        let loop_time = loop_start.elapsed();
        stats.update(
            encode_time,
            decode_time,
            display_time,
            loop_time,
            encoded_data.len(),
        );

        // Print progress every second
        if stats.frames_processed % fps == 0 {
            print_progress(&stats);
        }
    }

    // Cleanup
    capture.stop()?;
    encoder.shutdown()?;
    decoder.shutdown()?;

    println!();

    // Final report
    print_final_report(&stats, &encoder, &decoder, &display);

    Ok(())
}

/// Create animated test capture.
fn create_animated_capture(
    width: u32,
    height: u32,
    fps: u32,
    duration: Duration,
) -> MockScreenCapture {
    let total_frames = (fps as f64 * duration.as_secs_f64()) as usize;
    let mut frames = Vec::with_capacity(total_frames);

    for i in 0..total_frames {
        frames.push(generate_animated_frame(width, height, i));
    }

    MockScreenCapture::with_frames(frames)
}

/// Generate an animated test frame.
fn generate_animated_frame(width: u32, height: u32, frame_num: usize) -> CapturedFrame {
    let stride = width * 4;
    let mut data = vec![0u8; (stride * height) as usize];

    let time = frame_num as f32 * 0.1;

    for y in 0..height {
        for x in 0..width {
            let offset = (y * stride + x * 4) as usize;

            // Animated gradient with moving pattern
            let wave = ((x as f32 / 50.0 + time).sin() * 0.5 + 0.5) * 255.0;
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = ((y as f32 / height as f32) * 255.0) as u8;
            let b = wave as u8;

            // BGRA format
            data[offset] = b;
            data[offset + 1] = g;
            data[offset + 2] = r;
            data[offset + 3] = 255;
        }
    }

    CapturedFrame {
        data,
        width,
        height,
        pixel_format: PixelFormat::Bgra,
        stride,
        display_id: 0,
        timestamp_us: (frame_num as u64) * 33_333,
    }
}

/// Pipeline statistics.
#[derive(Default)]
struct PipelineStats {
    frames_processed: u64,
    total_encode_time: Duration,
    total_decode_time: Duration,
    total_display_time: Duration,
    total_loop_time: Duration,
    total_bytes_encoded: u64,
}

impl PipelineStats {
    fn update(
        &mut self,
        encode_time: Duration,
        decode_time: Duration,
        display_time: Duration,
        loop_time: Duration,
        bytes_encoded: usize,
    ) {
        self.frames_processed += 1;
        self.total_encode_time += encode_time;
        self.total_decode_time += decode_time;
        self.total_display_time += display_time;
        self.total_loop_time += loop_time;
        self.total_bytes_encoded += bytes_encoded as u64;
    }

    fn avg_encode_ms(&self) -> f64 {
        if self.frames_processed == 0 {
            return 0.0;
        }
        self.total_encode_time.as_secs_f64() * 1000.0 / self.frames_processed as f64
    }

    fn avg_decode_ms(&self) -> f64 {
        if self.frames_processed == 0 {
            return 0.0;
        }
        self.total_decode_time.as_secs_f64() * 1000.0 / self.frames_processed as f64
    }

    fn avg_display_ms(&self) -> f64 {
        if self.frames_processed == 0 {
            return 0.0;
        }
        self.total_display_time.as_secs_f64() * 1000.0 / self.frames_processed as f64
    }

    fn avg_latency_ms(&self) -> f64 {
        self.avg_encode_ms() + self.avg_decode_ms() + self.avg_display_ms()
    }

    fn avg_fps(&self) -> f64 {
        if self.total_loop_time.as_secs_f64() == 0.0 {
            return 0.0;
        }
        self.frames_processed as f64 / self.total_loop_time.as_secs_f64()
    }
}

fn print_progress(stats: &PipelineStats) {
    println!(
        "  Frame {}: {:.1} FPS, Latency {:.1}ms (E:{:.1} D:{:.1} R:{:.1})",
        stats.frames_processed,
        stats.avg_fps(),
        stats.avg_latency_ms(),
        stats.avg_encode_ms(),
        stats.avg_decode_ms(),
        stats.avg_display_ms()
    );
}

fn print_final_report(
    stats: &PipelineStats,
    encoder: &NativeVideoEncoder,
    decoder: &NativeVideoDecoder,
    display: &VideoDisplay,
) {
    println!("========================================");
    println!("📊 最终报告");
    println!("========================================");
    println!();

    println!("📈 性能指标:");
    println!("  处理帧数:     {}", stats.frames_processed);
    println!("  平均帧率:     {:.1} FPS", stats.avg_fps());
    println!("  平均延迟:     {:.1} ms", stats.avg_latency_ms());
    println!("  - 编码:       {:.1} ms", stats.avg_encode_ms());
    println!("  - 解码:       {:.1} ms", stats.avg_decode_ms());
    println!("  - 显示:       {:.1} ms", stats.avg_display_ms());
    println!();

    let encoder_stats = encoder.stats();
    println!("🎞️  编码器统计:");
    println!("  编码帧数:     {}", encoder_stats.frames_encoded);
    println!("  关键帧数:     {}", encoder_stats.keyframes_generated);
    println!("  编码字节:     {} KB", encoder_stats.bytes_encoded / 1024);
    println!(
        "  平均比特率:   {:.1} Mbps",
        (encoder_stats.bytes_encoded as f64 * 8.0)
            / (stats.total_loop_time.as_secs_f64() * 1_000_000.0)
    );
    println!();

    let decoder_stats = decoder.stats();
    println!("🎬 解码器统计:");
    println!("  解码帧数:     {}", decoder_stats.frames_decoded);
    println!("  关键帧数:     {}", decoder_stats.keyframes_received);
    println!("  解码字节:     {} KB", decoder_stats.bytes_decoded / 1024);
    println!();

    let display_stats = display.stats();
    println!("🖥️  显示统计:");
    println!("  渲染帧数:     {}", display_stats.frames_rendered);
    println!("  丢帧数:       {}", display_stats.frames_dropped);
    println!("  纹理重建:     {}", display_stats.texture_recreations);
    println!();

    println!("========================================");
    println!("✅ Phase 2 验收标准");
    println!("========================================");

    let latency_ok = stats.avg_latency_ms() < 100.0;
    let fps_ok = stats.avg_fps() >= 24.0;
    let encode_ok = stats.avg_encode_ms() < 50.0;
    let decode_ok = stats.avg_decode_ms() < 50.0;

    println!(
        "[ {} ] 端到端延迟 < 100ms ({:.1}ms)",
        if latency_ok { "✓" } else { "✗" },
        stats.avg_latency_ms()
    );
    println!(
        "[ {} ] 帧率 >= 24 FPS ({:.1})",
        if fps_ok { "✓" } else { "✗" },
        stats.avg_fps()
    );
    println!(
        "[ {} ] 编码延迟 < 50ms ({:.1}ms)",
        if encode_ok { "✓" } else { "✗" },
        stats.avg_encode_ms()
    );
    println!(
        "[ {} ] 解码延迟 < 50ms ({:.1}ms)",
        if decode_ok { "✓" } else { "✗" },
        stats.avg_decode_ms()
    );
    println!();

    if latency_ok && fps_ok && encode_ok && decode_ok {
        println!("🎉 Phase 2 端到端测试通过！");
        println!();
        println!("✨ 完整视频流管道验证成功：");
        println!("   捕获 → 编码 → 解码 → 显示");
        println!();
        println!("📋 下一步: Phase 3 - 跨网络传输测试");
    } else {
        println!("⚠️  部分指标未达标，但核心功能正常");
        println!();
        println!("💡 建议:");
        if !encode_ok || !decode_ok {
            println!("   - 考虑启用硬件加速（VideoToolbox）");
        }
        if !fps_ok {
            println!("   - 降低分辨率或帧率");
        }
    }
    println!();
}
