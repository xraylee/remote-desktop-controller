// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Integration of mDNS discovery with WebSocket signaling.
//!
//! This module bridges local mDNS device discovery with the WebSocket
//! signaling protocol, allowing devices to receive `nearby_update` notifications
//! when new devices appear on the local network.

use crate::mdns::{MdnsDevice, MdnsDiscovery};
use crate::ws::{SessionManager, WsMessage};
use crate::ws::message::DeviceInfo;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Start mDNS discovery and forward updates to connected WebSocket clients.
///
/// This spawns a background task that:
/// 1. Subscribes to mDNS discovery events
/// 2. Broadcasts `nearby_update` messages to all connected devices
///
/// # Arguments
/// - `discovery`: The mDNS discovery service
/// - `session_manager`: WebSocket session manager for broadcasting
pub fn start_mdns_broadcaster(
    discovery: Arc<MdnsDiscovery>,
    session_manager: Arc<SessionManager>,
) {
    let mut rx = discovery.subscribe();

    tokio::spawn(async move {
        info!("mDNS broadcaster started");

        while let Ok(device) = rx.recv().await {
            debug!(
                device_code = %device.device_code,
                device_name = %device.device_name,
                address = %device.address,
                "Broadcasting nearby device update"
            );

            // Create nearby_update message
            let device_info = DeviceInfo {
                code: device.device_code.clone(),
                name: device.device_name.clone(),
                platform: String::new(), // mDNS doesn't provide platform info
                online: true,
            };
            let message = WsMessage::NearbyUpdate {
                devices: vec![device_info],
            };

            // Broadcast to all connected sessions
            let broadcast_count = session_manager.broadcast_all(message).await;
            debug!(
                "Broadcasted nearby_update to {} sessions",
                broadcast_count
            );
        }

        info!("mDNS broadcaster stopped");
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ws::SessionManager;

    #[tokio::test]
    async fn test_broadcaster_setup() {
        let discovery = Arc::new(MdnsDiscovery::new("123456", "Test", 8080).unwrap());
        let session_manager = Arc::new(SessionManager::new());

        // Just verify we can start the broadcaster without panicking
        start_mdns_broadcaster(discovery, session_manager);

        // Give it a moment to spawn
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}
