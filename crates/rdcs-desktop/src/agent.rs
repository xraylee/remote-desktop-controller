// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Agent mode — 被控端：捕获屏幕 → 编码 → 发送

use anyhow::Result;
use tracing::info;

pub async fn run(port: u16, name: String) -> Result<()> {
    info!(%name, port, "RDCS Agent starting");

    // TODO Phase 3: 集成 rdcs-platform 屏幕捕获 + rdcs-codec 编码器
    //               + rdcs-connection ICE 监听

    // Placeholder — 保持进程运行直到 Ctrl-C
    println!("🖥️  RDCS Agent");
    println!("   Device: {}", name);
    println!("   Listening on port {}", port);
    println!("   Waiting for controller connections... (Ctrl-C to stop)");

    tokio::signal::ctrl_c().await?;
    info!("Agent shutting down");
    Ok(())
}
