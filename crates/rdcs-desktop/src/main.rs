// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! RDCS Desktop — unified client.
//!
//! One binary, two modes:
//!   rdcs-desktop serve          # 被控端（接受控制）
//!   rdcs-desktop connect <IP>   # 主控端（发起控制）

use anyhow::Result;
use clap::{Parser, Subcommand};

mod agent;
mod controller;

#[derive(Parser)]
#[command(name = "rdcs-desktop", version, about = "RDCS Remote Desktop")]
struct Cli {
    /// Log level: error | warn | info | debug | trace
    #[arg(long, default_value = "info", global = true)]
    log_level: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Start as agent — accept incoming control sessions
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "7000")]
        port: u16,

        /// Human-readable device name (defaults to hostname)
        #[arg(long)]
        name: Option<String>,
    },

    /// Start as controller — connect to a remote agent
    Connect {
        /// Target IP or hostname
        host: String,

        /// Target port
        #[arg(long, default_value = "7000")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialise logging
    tracing_subscriber::fmt()
        .with_env_filter(&cli.log_level)
        .init();

    match cli.command {
        Command::Serve { port, name } => {
            let name = name.unwrap_or_else(|| {
                hostname::get()
                    .ok()
                    .and_then(|s| s.into_string().ok())
                    .unwrap_or_else(|| "rdcs-device".into())
            });
            agent::run(port, name).await
        }
        Command::Connect { host, port } => {
            controller::run(host, port).await
        }
    }
}
