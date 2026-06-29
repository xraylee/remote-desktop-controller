// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! SDL2-based video renderer.

use crate::error::{DisplayError, Result};
use rdcs_platform::{CapturedFrame, PixelFormat};
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::time::Instant;
use tracing::{debug, trace, warn};

/// SDL2-based video renderer.
///
/// Handles texture creation, frame uploading, and rendering to the canvas.
pub struct VideoRenderer {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    texture: Option<Texture>,
    current_width: u32,
    current_height: u32,
    stats: RenderStats,
}

/// Rendering statistics.
#[derive(Debug, Clone, Default)]
pub struct RenderStats {
    pub frames_rendered: u64,
    pub total_render_time_ms: u64,
    pub frames_dropped: u64,
    pub texture_recreations: u32,
}

impl VideoRenderer {
    /// Create a new video renderer from an SDL2 canvas.
    pub fn new(canvas: Canvas<Window>) -> Result<Self> {
        let texture_creator = canvas.texture_creator();

        Ok(Self {
            canvas,
            texture_creator,
            texture: None,
            current_width: 0,
            current_height: 0,
            stats: RenderStats::default(),
        })
    }

    /// Render a captured frame to the display.
    ///
    /// If the frame dimensions change, the texture is automatically recreated.
    pub fn render_frame(&mut self, frame: &CapturedFrame) -> Result<()> {
        let start = Instant::now();

        // Validate pixel format
        if frame.pixel_format != PixelFormat::Bgra {
            return Err(DisplayError::InvalidFrameFormat(format!(
                "expected BGRA format, got {:?}",
                frame.pixel_format
            )));
        }

        // Recreate texture if dimensions changed
        if self.texture.is_none()
            || frame.width != self.current_width
            || frame.height != self.current_height
        {
            self.create_texture(frame.width, frame.height)?;
        }

        // Get texture and upload frame data
        let texture = self.texture.as_mut().ok_or_else(|| {
            DisplayError::RenderFailed("texture not initialized".into())
        })?;

        // SDL2 expects BGRA as ARGB8888 on little-endian systems
        // Our CapturedFrame is already in BGRA byte order
        let pitch = frame.stride as usize;
        texture
            .update(None, &frame.data, pitch)
            .map_err(|e| DisplayError::RenderFailed(format!("texture update failed: {}", e)))?;

        // Clear canvas
        self.canvas.clear();

        // Calculate destination rectangle (preserve aspect ratio)
        let src_rect = Rect::new(0, 0, frame.width, frame.height);
        let dst_rect = self.calculate_display_rect(frame.width, frame.height);

        // Copy texture to canvas
        self.canvas
            .copy(texture, Some(src_rect), Some(dst_rect))
            .map_err(|e| DisplayError::RenderFailed(format!("canvas copy failed: {}", e)))?;

        // Present
        self.canvas.present();

        // Update stats
        let render_time = start.elapsed();
        self.stats.frames_rendered += 1;
        self.stats.total_render_time_ms += render_time.as_millis() as u64;

        trace!(
            "Rendered frame: {}x in {:.2}ms",
            frame.width,
            frame.height,
            render_time.as_secs_f64() * 1000.0
        );

        Ok(())
    }

    /// Create or recreate the rendering texture.
    fn create_texture(&mut self, width: u32, height: u32) -> Result<()> {
        debug!(
            "Creating texture: {}x{} (previous: {}x{})",
            width, height, self.current_width, self.current_height
        );

        // SDL2 PixelFormatEnum::ARGB8888 matches BGRA byte order on little-endian
        let texture = self
            .texture_creator
            .create_texture_streaming(PixelFormatEnum::ARGB8888, width, height)
            .map_err(|e| {
                DisplayError::TextureCreationFailed(format!(
                    "failed to create {}x{} texture: {}",
                    width, height, e
                ))
            })?;

        self.texture = Some(texture);
        self.current_width = width;
        self.current_height = height;
        self.stats.texture_recreations += 1;

        Ok(())
    }

    /// Calculate destination rectangle preserving aspect ratio.
    fn calculate_display_rect(&self, frame_width: u32, frame_height: u32) -> Rect {
        let (window_width, window_height) = self.canvas.window().size();

        // Calculate scaling factors
        let scale_x = window_width as f32 / frame_width as f32;
        let scale_y = window_height as f32 / frame_height as f32;

        // Use the smaller scale to fit within window
        let scale = scale_x.min(scale_y);

        // Calculate scaled dimensions
        let scaled_width = (frame_width as f32 * scale) as u32;
        let scaled_height = (frame_height as f32 * scale) as u32;

        // Center in window
        let x = (window_width - scaled_width) / 2;
        let y = (window_height - scaled_height) / 2;

        Rect::new(x as i32, y as i32, scaled_width, scaled_height)
    }

    /// Get rendering statistics.
    pub fn stats(&self) -> RenderStats {
        self.stats.clone()
    }

    /// Clear the display.
    pub fn clear(&mut self) -> Result<()> {
        self.canvas.clear();
        self.canvas.present();
        Ok(())
    }

    /// Get the window size.
    pub fn window_size(&self) -> (u32, u32) {
        self.canvas.window().size()
    }
}
