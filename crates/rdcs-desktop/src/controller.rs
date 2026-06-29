// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Controller mode — 主控端：接收 → 解码 → SDL2 显示

use anyhow::Result;
use tracing::info;

pub async fn run(host: String, port: u16) -> Result<()> {
    info!(%host, port, "RDCS Controller starting");

    // TODO Phase 3: 集成 rdcs-connection ICE 连接
    //               + rdcs-codec 解码器 + rdcs-display SDL2 窗口

    println!("🎮 RDCS Controller");
    println!("   Connecting to {}:{} ...", host, port);
    println!("   (Not yet implemented — SDL2 display ready, ICE WIP)");

    tokio::signal::ctrl_c().await?;
    info!("Controller shutting down");
    Ok(())
}
