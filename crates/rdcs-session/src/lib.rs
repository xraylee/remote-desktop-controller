// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! RDCS Session Orchestration
//!
//! This crate provides the high-level session orchestration layer that connects
//! the media pipeline (rdcs-codec) with the signaling protocol (rdcs-signaling)
//! and transport layer (rdcs-transport).
//!
//! # Architecture (方案 B：webrtc-rs + 平台原生编解码)
//!
//! ```text
//!  ┌─────────────────────────────────────────────────────────────┐
//!  │                     RdcsSession                              │
//!  │                                                              │
//!  │  ┌──────────────┐         ┌─────────────────────────────┐   │
//!  │  │  Signaling   │◄───────►│  EncodePipeline             │   │
//!  │  │  (WebSocket) │  SDP/ICE│  DecodePipeline             │   │
//!  │  └──────────────┘         │  (Platform Native Codecs)   │   │
//!  │                           └─────────────────────────────┘   │
//!  │                                    │                         │
//!  │                                    │ H.264 NAL units         │
//!  │                                    ▼                         │
//!  │                          ┌──────────────────┐               │
//!  │                          │  RTP Packetizer  │               │
//!  │                          │  (webrtc-rs)     │               │
//!  │                          └──────────────────┘               │
//!  │                                    │                         │
//!  │                                    │ RTP packets             │
//!  │                                    ▼                         │
//!  │                          ┌──────────────────┐               │
//!  │                          │  SRTP Encryption │               │
//!  │                          │  (webrtc-rs)     │               │
//!  │                          └──────────────────┘               │
//!  │                                    │                         │
//!  │                                    │ Encrypted packets       │
//!  │                                    ▼                         │
//!  │                          ┌──────────────────┐               │
//!  │                          │ TransportChannel │               │
//!  │                          │  (NACK/FEC/UDP)  │               │
//!  │                          └──────────────────┘               │
//!  └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Migration Status
//!
//! - ✅ Phase 0: libwebrtc 依赖已清理
//! - 🚧 Phase 1: 实现 VideoToolbox/MF/VA-API 编解码器
//! - 🚧 Phase 2: 集成 webrtc-rs RTP/SRTP
//! - 🚧 Phase 3: 对接 rdcs-transport/connection
//!
//! 原 manager.rs 已废弃，待重新实现。

// manager 模块暂时禁用 - 正在迁移到方案 B
// pub mod manager;

use thiserror::Error;

/// Session-layer errors.
#[derive(Debug, Error)]
pub enum SessionError {
    /// WebRTC/codec error.
    #[error("codec error: {0}")]
    Codec(#[from] rdcs_codec::CodecError),

    /// Connection/ICE error.
    #[error("connection error: {0}")]
    Connection(#[from] rdcs_connection::ConnectionError),

    /// Transport error.
    #[error("transport error: {0}")]
    Transport(#[from] rdcs_transport::TransportError),

    /// Signaling protocol error.
    #[error("signaling error: {0}")]
    Signaling(String),

    /// Session is not in the expected state for this operation.
    #[error("invalid session state: {0}")]
    InvalidState(String),
}

/// Convenience result type for session operations.
pub type Result<T> = std::result::Result<T, SessionError>;
