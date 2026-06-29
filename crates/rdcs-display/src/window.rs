// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Video display window management.

use crate::error::{DisplayError, Result};
use crate::renderer::{RenderStats, VideoRenderer};
use rdcs_platform::CapturedFrame;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::{Window, WindowBuilder};
use sdl2::EventPump;
use sdl2::Sdl;
use std::time::{Duration, Instant};
use tracing::{debug, info};

/// Configuration for video display.
#[derive(Debug, Clone)]
pub struct DisplayConfig {
    /// Window title.
    pub title: String,
    /// Initial window width.
    pub width: u32,
    /// Initial window height.
    pub height: u32,
    /// Whether the window is resizable.
    pub resizable: bool,
    /// Whether to use VSync.
    pub vsync: bool,
    /// Target frame rate (0 = unlimited).
    pub target_fps: u32,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            title: "RDCS Remote Desktop".to_string(),
            width: 1280,
            height: 720,
            resizable: true,
            vsync: true,
            target_fps: 60,
        }
    }
}

impl DisplayConfig {
    /// Set window title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set window size.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Set resizable flag.
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Set VSync flag.
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self
    }

    /// Set target FPS.
    pub fn with_target_fps(mut self, fps: u32) -> Self {
        self.target_fps = fps;
        self
    }
}

/// Video display window with SDL2 rendering.
pub struct VideoDisplay {
    _sdl_context: Sdl,
    renderer: VideoRenderer,
    event_pump: EventPump,
    config: DisplayConfig,
    frame_interval: Duration,
    last_frame_time: Instant,
    should_quit: bool,
}

impl VideoDisplay {
    /// Create a new video display window.
    pub fn new(config: DisplayConfig) -> Result<Self> {
        info!(
            "Creating video display: {}x{}, title: '{}'",
            config.width, config.height, config.title
        );

        // Initialize SDL2
        let sdl_context = sdl2::init().map_err(|e| DisplayError::SdlInitFailed(e))?;
        let video_subsystem = sdl_context
            .video()
            .map_err(|e| DisplayError::SdlInitFailed(format!("video subsystem: {}", e)))?;

        // Create window
        let mut window_builder = video_subsystem.window(&config.title, config.width, config.height);

        if config.resizable {
            window_builder.resizable();
        }

        let window = window_builder
            .position_centered()
            .build()
            .map_err(|e| DisplayError::WindowCreationFailed(e.to_string()))?;

        // Create canvas
        let mut canvas_builder = window.into_canvas();

        if config.vsync {
            canvas_builder = canvas_builder.present_vsync();
        }

        let canvas = canvas_builder
            .accelerated()
            .build()
            .map_err(|e| DisplayError::RendererCreationFailed(e.to_string()))?;

        debug!("SDL2 canvas created with acceleration");

        // Create renderer
        let renderer = VideoRenderer::new(canvas)?;

        // Create event pump
        let event_pump = sdl_context
            .event_pump()
            .map_err(|e| DisplayError::SdlInitFailed(format!("event pump: {}", e)))?;

        // Calculate frame interval
        let frame_interval = if config.target_fps > 0 {
            Duration::from_secs_f64(1.0 / config.target_fps as f64)
        } else {
            Duration::from_secs(0)
        };

        info!("Video display created successfully");

        Ok(Self {
            _sdl_context: sdl_context,
            renderer,
            event_pump,
            config,
            frame_interval,
            last_frame_time: Instant::now(),
            should_quit: false,
        })
    }

    /// Render a video frame to the display.
    ///
    /// Returns `Ok(true)` if the window should continue running,
    /// `Ok(false)` if the user requested quit.
    pub fn render_frame(&mut self, frame: &CapturedFrame) -> Result<bool> {
        // Process events
        self.process_events();

        if self.should_quit {
            return Ok(false);
        }

        // Frame rate limiting
        if self.config.target_fps > 0 {
            let elapsed = self.last_frame_time.elapsed();
            if elapsed < self.frame_interval {
                let sleep_duration = self.frame_interval - elapsed;
                std::thread::sleep(sleep_duration);
            }
        }

        // Render frame
        self.renderer.render_frame(frame)?;
        self.last_frame_time = Instant::now();

        Ok(!self.should_quit)
    }

    /// Process SDL2 events.
    fn process_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    debug!("Quit event received");
                    self.should_quit = true;
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    debug!("Escape key pressed, quitting");
                    self.should_quit = true;
                }
                Event::Window { .. } => {
                    // Window events (resize, focus, etc.)
                    // Currently no special handling needed
                }
                _ => {}
            }
        }
    }

    /// Check if the display should quit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Clear the display.
    pub fn clear(&mut self) -> Result<()> {
        self.renderer.clear()
    }

    /// Get rendering statistics.
    pub fn stats(&self) -> RenderStats {
        self.renderer.stats()
    }

    /// Get window size.
    pub fn window_size(&self) -> (u32, u32) {
        self.renderer.window_size()
    }

    /// Wait for events (blocking).
    ///
    /// Useful for keeping the window open without rendering frames.
    pub fn wait_for_quit(&mut self) {
        info!("Waiting for quit event...");
        while !self.should_quit {
            self.process_events();
            std::thread::sleep(Duration::from_millis(16)); // ~60 Hz polling
        }
        info!("Quit event received");
    }
}
