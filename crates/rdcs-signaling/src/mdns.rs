// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! mDNS local network device discovery.
//!
//! This module implements Multicast DNS (mDNS) service discovery for finding
//! RDCS devices on the local network without requiring a central signaling server.
//!
//! ## Service Type
//! - Service: `_rdcs._tcp.local.`
//! - TXT records: `device_code`, `device_name`, `version`
//!
//! ## Usage
//! ```no_run
//! use rdcs_signaling::mdns::{MdnsDiscovery, MdnsDevice};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let discovery = MdnsDiscovery::new("ABC123", "My Device", 8080)?;
//! discovery.start().await?;
//!
//! // Listen for nearby devices
//! let mut rx = discovery.subscribe();
//! while let Ok(device) = rx.recv().await {
//!     println!("Found device: {} at {}", device.name, device.address);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

/// RDCS mDNS service type.
const SERVICE_TYPE: &str = "_rdcs._tcp.local.";

/// Maximum number of concurrent discovery event subscribers.
const DISCOVERY_CHANNEL_SIZE: usize = 64;

/// Represents a discovered RDCS device on the local network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdnsDevice {
    /// Device code (6-digit identifier).
    pub device_code: String,
    /// Human-readable device name.
    pub device_name: String,
    /// RDCS client version.
    pub version: String,
    /// Socket address (IP + port).
    pub address: SocketAddr,
    /// Last seen timestamp (Unix seconds).
    pub last_seen: u64,
}

/// mDNS discovery service for local network device detection.
pub struct MdnsDiscovery {
    device_code: String,
    device_name: String,
    port: u16,
    version: String,
    devices: Arc<RwLock<HashMap<String, MdnsDevice>>>,
    event_tx: broadcast::Sender<MdnsDevice>,
}

impl MdnsDiscovery {
    /// Create a new mDNS discovery service.
    ///
    /// # Arguments
    /// - `device_code`: This device's 6-digit code
    /// - `device_name`: Human-readable name
    /// - `port`: Local WebSocket port for connections
    pub fn new(device_code: &str, device_name: &str, port: u16) -> Result<Self> {
        let (tx, _) = broadcast::channel(DISCOVERY_CHANNEL_SIZE);

        Ok(Self {
            device_code: device_code.to_string(),
            device_name: device_name.to_string(),
            port,
            version: env!("CARGO_PKG_VERSION").to_string(),
            devices: Arc::new(RwLock::new(HashMap::new())),
            event_tx: tx,
        })
    }

    /// Start mDNS service announcement and device discovery.
    ///
    /// This spawns background tasks for:
    /// - Broadcasting this device's service
    /// - Listening for other devices
    /// - Cleaning up stale devices
    pub async fn start(&self) -> Result<()> {
        info!(
            device_code = %self.device_code,
            device_name = %self.device_name,
            port = self.port,
            "Starting mDNS discovery service"
        );

        // Spawn service announcement task
        self.spawn_announcer().await?;

        // Spawn discovery listener task
        self.spawn_listener().await?;

        // Spawn cleanup task
        self.spawn_cleanup_task();

        Ok(())
    }

    /// Subscribe to device discovery events.
    ///
    /// Returns a receiver that yields newly discovered or updated devices.
    pub fn subscribe(&self) -> broadcast::Receiver<MdnsDevice> {
        self.event_tx.subscribe()
    }

    /// Get all currently discovered devices.
    pub async fn list_devices(&self) -> Vec<MdnsDevice> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    /// Get a specific device by device code.
    pub async fn get_device(&self, device_code: &str) -> Option<MdnsDevice> {
        let devices = self.devices.read().await;
        devices.get(device_code).cloned()
    }

    async fn spawn_announcer(&self) -> Result<()> {
        // In a real implementation, this would use the `mdns` crate to announce
        // the service. For now, we provide a stub that logs the intent.

        let device_code = self.device_code.clone();
        let device_name = self.device_name.clone();
        let version = self.version.clone();
        let port = self.port;

        tokio::spawn(async move {
            info!("mDNS announcer started");

            // TODO: Integrate `mdns` crate
            // Example pseudo-code:
            // let responder = MdnsResponder::new()?;
            // responder.register(
            //     SERVICE_TYPE,
            //     &device_code,
            //     port,
            //     &[("device_code", &device_code),
            //       ("device_name", &device_name),
            //       ("version", &version)]
            // )?;

            debug!(
                "Would announce: service={}, code={}, name={}, version={}, port={}",
                SERVICE_TYPE, device_code, device_name, version, port
            );

            // Keep the service announced
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                debug!("mDNS service announcement still active");
            }
        });

        Ok(())
    }

    async fn spawn_listener(&self) -> Result<()> {
        let devices = self.devices.clone();
        let event_tx = self.event_tx.clone();
        let self_code = self.device_code.clone();

        tokio::spawn(async move {
            info!("mDNS listener started");

            // TODO: Integrate `mdns` crate for service discovery
            // Example pseudo-code:
            // let stream = MdnsStream::new(SERVICE_TYPE)?;
            // while let Some(response) = stream.next().await {
            //     if let Some(device) = parse_mdns_response(response) {
            //         // Skip self
            //         if device.device_code == self_code {
            //             continue;
            //         }
            //
            //         devices.write().await.insert(device.device_code.clone(), device.clone());
            //         let _ = event_tx.send(device);
            //     }
            // }

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                debug!("mDNS listener still active, discovered {} devices", devices.read().await.len());
            }
        });

        Ok(())
    }

    fn spawn_cleanup_task(&self) {
        let devices = self.devices.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(120)).await;

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let mut devices = devices.write().await;
                let before_count = devices.len();

                // Remove devices not seen in 5 minutes
                devices.retain(|code, device| {
                    let keep = now - device.last_seen < 300;
                    if !keep {
                        warn!(device_code = %code, "Removing stale device");
                    }
                    keep
                });

                let removed = before_count - devices.len();
                if removed > 0 {
                    info!("Cleaned up {} stale devices", removed);
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mdns_discovery_creation() {
        let discovery = MdnsDiscovery::new("123456", "Test Device", 8080);
        assert!(discovery.is_ok());
    }

    #[tokio::test]
    async fn test_list_devices_initially_empty() {
        let discovery = MdnsDiscovery::new("123456", "Test Device", 8080).unwrap();
        let devices = discovery.list_devices().await;
        assert_eq!(devices.len(), 0);
    }

    #[tokio::test]
    async fn test_subscription() {
        let discovery = MdnsDiscovery::new("123456", "Test Device", 8080).unwrap();
        let mut rx = discovery.subscribe();

        // Spawn a task to send a test device
        let tx = discovery.event_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            let device = MdnsDevice {
                device_code: "654321".to_string(),
                device_name: "Remote Device".to_string(),
                version: "0.1.0".to_string(),
                address: "192.168.1.100:8080".parse().unwrap(),
                last_seen: 1234567890,
            };
            let _ = tx.send(device);
        });

        // Wait for the device
        let result = tokio::time::timeout(
            tokio::time::Duration::from_secs(1),
            rx.recv()
        ).await;

        assert!(result.is_ok());
        let device = result.unwrap().unwrap();
        assert_eq!(device.device_code, "654321");
    }
}
