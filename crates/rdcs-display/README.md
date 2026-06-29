# rdcs-display

视频显示和渲染模块，用于 RDCS 远程桌面系统。

## 功能特性

- ✅ 跨平台窗口创建（macOS, Windows, Linux）
- ✅ 基于 SDL2 的硬件加速渲染
- ✅ BGRA 格式帧渲染
- ✅ 自动缩放和宽高比保持
- ✅ 帧率限制和 VSync 支持
- ✅ 性能监控（FPS、丢帧统计）

## 构建

### 依赖

需要安装 SDL2 开发库：

**macOS:**
```bash
brew install sdl2
```

**Ubuntu/Debian:**
```bash
sudo apt-get install libsdl2-dev
```

**Windows:**
SDL2 会自动通过 `bundled` feature 编译。

### 编译

```bash
cargo build -p rdcs-display
```

## 示例

### 基础显示测试

显示动画测试图案：

```bash
cargo run --example display_test -p rdcs-display
```

### 端到端回环测试

完整视频管道测试（捕获→编码→解码→显示）：

```bash
cargo run --example display_roundtrip --features software-encoder
```

## 使用方法

```rust
use rdcs_display::{VideoDisplay, DisplayConfig};
use rdcs_platform::CapturedFrame;

// 创建显示窗口
let config = DisplayConfig::default()
    .with_title("Remote Desktop")
    .with_size(1920, 1080)
    .with_target_fps(30);

let mut display = VideoDisplay::new(config)?;

// 渲染循环
loop {
    let frame: CapturedFrame = // ... 从网络接收或解码
    
    let should_continue = display.render_frame(&frame)?;
    if !should_continue {
        break; // 用户关闭窗口或按 ESC
    }
}
```

## API 文档

### `DisplayConfig`

显示窗口配置：

```rust
pub struct DisplayConfig {
    pub title: String,        // 窗口标题
    pub width: u32,          // 初始宽度
    pub height: u32,         // 初始高度
    pub resizable: bool,     // 是否可调整大小
    pub vsync: bool,         // 是否启用 VSync
    pub target_fps: u32,     // 目标帧率（0 = 无限制）
}
```

### `VideoDisplay`

主显示窗口类：

- `new(config: DisplayConfig) -> Result<Self>` - 创建窗口
- `render_frame(&mut self, frame: &CapturedFrame) -> Result<bool>` - 渲染帧
- `stats(&self) -> RenderStats` - 获取渲染统计
- `should_quit(&self) -> bool` - 检查是否应退出
- `clear(&mut self) -> Result<()>` - 清空显示

### `RenderStats`

渲染统计信息：

```rust
pub struct RenderStats {
    pub frames_rendered: u64,      // 已渲染帧数
    pub total_render_time_ms: u64, // 总渲染时间
    pub frames_dropped: u64,       // 丢帧数
    pub texture_recreations: u32,  // 纹理重建次数
}
```

## 架构

```
CapturedFrame (BGRA from decoder)
       ↓
VideoDisplay::render_frame()
       ↓
VideoRenderer::render_frame()
       ↓
SDL2 Texture Update
       ↓
SDL2 Canvas Copy & Present
       ↓
显示在屏幕上
```

## 性能

在 Apple Silicon Mac 上的性能（1920x1080@30fps）：

- 渲染延迟: ~5-10ms
- CPU 使用率: ~5%
- 支持 60fps 无丢帧

## 许可证

Apache-2.0
