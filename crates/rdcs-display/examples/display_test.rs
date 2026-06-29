// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Display test example: Create a window and show a test pattern.
//!
//! This example demonstrates the rdcs-display module by creating
//! a simple animated test pattern.
//!
//! Usage:
//! ```bash
//! cargo run --example display_test -p rdcs-display
//! ```

use rdcs_display::{DisplayConfig, VideoDisplay};
use rdcs_platform::{CapturedFrame, PixelFormat};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("========================================");
    println!("🖥️  RDCS Display Test");
    println!("========================================");
    println!();
    println!("Creating test window...");

    // Create display
    let config = DisplayConfig::default()
        .with_title("RDCS Display Test")
        .with_size(800, 600)
        .with_target_fps(30);

    let mut display = VideoDisplay::new(config)?;

    println!("✓ Window created");
    println!("  Press ESC or close window to exit");
    println!();

    // Generate and display frames
    let width = 800u32;
    let height = 600u32;
    let mut frame_count = 0u32;
    let start_time = Instant::now();

    loop {
        // Generate test frame with animation
        let frame = generate_test_frame(width, height, frame_count);

        // Render frame
        let should_continue = display.render_frame(&frame)?;
        if !should_continue {
            break;
        }

        frame_count += 1;

        // Print stats every 60 frames
        if frame_count % 60 == 0 {
            let elapsed = start_time.elapsed().as_secs_f64();
            let fps = frame_count as f64 / elapsed;
            let stats = display.stats();

            println!(
                "Frame {}: {:.1} FPS, {} frames rendered, {} texture recreations",
                frame_count, fps, stats.frames_rendered, stats.texture_recreations
            );
        }
    }

    // Final stats
    let elapsed = start_time.elapsed();
    let stats = display.stats();

    println!();
    println!("========================================");
    println!("📊 Final Statistics");
    println!("========================================");
    println!("Total frames:     {}", frame_count);
    println!("Total time:       {:.2}s", elapsed.as_secs_f64());
    println!("Average FPS:      {:.1}", frame_count as f64 / elapsed.as_secs_f64());
    println!("Frames rendered:  {}", stats.frames_rendered);
    println!("Frames dropped:   {}", stats.frames_dropped);
    println!("Texture recreations: {}", stats.texture_recreations);
    println!();

    Ok(())
}

/// Generate a test frame with animated pattern.
fn generate_test_frame(width: u32, height: u32, frame_num: u32) -> CapturedFrame {
    let stride = width * 4;
    let mut data = vec![0u8; (stride * height) as usize];

    let time = frame_num as f32 * 0.05;

    for y in 0..height {
        for x in 0..width {
            let offset = (y * stride + x * 4) as usize;

            // Animated gradient pattern
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = ((y as f32 / height as f32) * 255.0) as u8;
            let b = ((time.sin() * 0.5 + 0.5) * 255.0) as u8;

            // BGRA format
            data[offset] = b;
            data[offset + 1] = g;
            data[offset + 2] = r;
            data[offset + 3] = 255; // Alpha
        }
    }

    CapturedFrame {
        data,
        width,
        height,
        pixel_format: PixelFormat::Bgra,
        stride,
        display_id: 0,
        timestamp_us: (frame_num as u64) * 33_333, // ~30 FPS
    }
}
