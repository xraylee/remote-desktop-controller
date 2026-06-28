// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! mDNS peer discovery for LAN connections.

use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::ConnectionError;

// ---------------------------------------------------------------------------
// Trait interface
// ---------------------------------------------------------------------------

/// Trait for mDNS service discovery.
///
/// Implementations register services on the local network and browse for
/// remote peers, emitting [`MdnsEvent`]s as services appear and disappear.
pub trait MdnsDiscovery: Send + Sync {
    /// Register a service on the local network.
    fn register(&self, service: MdnsService) -> Result<(), ConnectionError>;

    /// Start browsing for remote services. Returns a channel receiver that
    /// yields [`MdnsEvent::Found`] when a service appears and
    /// [`MdnsEvent::Lost`] when one disappears.
    fn browse(&self) -> Result<mpsc::Receiver<MdnsEvent>, ConnectionError>;

    /// Unregister all previously registered services.
    fn unregister_all(&self) -> Result<(), ConnectionError>;
}

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A service advertised via mDNS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MdnsService {
    /// Human-readable service name (e.g. "MacBook-Pro").
    pub name: String,
    /// mDNS service type (e.g. "_rdcs._tcp.local.").
    pub service_type: String,
    /// Port the service is listening on.
    pub port: u16,
    /// Optional IP address. When `None`, the implementation uses the
    /// machine's default interface addresses.
    pub addr: Option<IpAddr>,
}

/// Events emitted during mDNS browsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MdnsEvent {
    /// A new service was discovered.
    Found(MdnsService),
    /// A previously discovered service was lost (identified by name).
    Lost(String),
}

/// A discovered peer on the local network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    /// Human-readable device name.
    pub name: String,
    /// Device code for connection.
    pub device_code: String,
    /// Socket addresses the peer is listening on.
    pub addresses: Vec<std::net::SocketAddr>,
    /// Service type (e.g. "_rdcs._tcp.local.").
    pub service_type: String,
}

// ---------------------------------------------------------------------------
// Mock implementation (for unit / integration testing)
// ---------------------------------------------------------------------------

/// A mock mDNS discovery implementation that stores registered services in
/// memory and emits pre-configured events via [`browse`](MdnsDiscovery::browse).
#[derive(Debug)]
pub struct MockMdnsDiscovery {
    services: Arc<Mutex<Vec<MdnsService>>>,
    events: Arc<Mutex<Vec<MdnsEvent>>>,
}

impl MockMdnsDiscovery {
    /// Create a new, empty mock discovery instance.
    pub fn new() -> Self {
        Self {
            services: Arc::new(Mutex::new(Vec::new())),
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Inject events that will be delivered on the next [`browse`] call.
    pub fn inject_events(&self, events: Vec<MdnsEvent>) {
        let mut store = self.events.lock().unwrap();
        store.extend(events);
    }

    /// Return a snapshot of the currently registered services.
    pub fn registered_services(&self) -> Vec<MdnsService> {
        self.services.lock().unwrap().clone()
    }
}

impl Default for MockMdnsDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

impl MdnsDiscovery for MockMdnsDiscovery {
    fn register(&self, service: MdnsService) -> Result<(), ConnectionError> {
        self.services.lock().unwrap().push(service);
        Ok(())
    }

    fn browse(&self) -> Result<mpsc::Receiver<MdnsEvent>, ConnectionError> {
        let (tx, rx) = mpsc::channel(64);
        let events = {
            let mut store = self.events.lock().unwrap();
            std::mem::take(&mut *store)
        };
        // Spawn a task that delivers queued events.
        tokio::spawn(async move {
            for event in events {
                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });
        Ok(rx)
    }

    fn unregister_all(&self) -> Result<(), ConnectionError> {
        self.services.lock().unwrap().clear();
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// mDNS broadcaster (concrete helper, wraps advertise / stop lifecycle)
// ---------------------------------------------------------------------------

/// High-level mDNS broadcaster that wraps the [`MdnsDiscovery`] trait to
/// manage the lifecycle of a single RDCS service advertisement.
#[derive(Debug)]
pub struct MdnsBroadcaster {
    service_name: String,
    service_type: String,
    port: u16,
}

impl MdnsBroadcaster {
    /// Create a new broadcaster for the given service parameters.
    pub fn new(service_name: String, service_type: String, port: u16) -> Self {
        Self {
            service_name,
            service_type,
            port,
        }
    }

    /// Start advertising this device on the local network.
    pub fn advertise<D: MdnsDiscovery>(
        &self,
        discovery: &D,
        device_code: &str,
    ) -> Result<(), ConnectionError> {
        let service = MdnsService {
            name: format!("{}-{}", self.service_name, device_code),
            service_type: self.service_type.clone(),
            port: self.port,
            addr: None,
        };
        discovery.register(service)
    }

    /// Stop advertising by unregistering all services.
    pub fn stop<D: MdnsDiscovery>(&self, discovery: &D) -> Result<(), ConnectionError> {
        discovery.unregister_all()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_register_service() {
        let discovery = MockMdnsDiscovery::new();
        let svc = MdnsService {
            name: "test-device".into(),
            service_type: "_rdcs._tcp.local.".into(),
            port: 21115,
            addr: None,
        };
        discovery.register(svc).unwrap();
        assert_eq!(discovery.registered_services().len(), 1);
    }

    #[test]
    fn mock_unregister_all() {
        let discovery = MockMdnsDiscovery::new();
        discovery
            .register(MdnsService {
                name: "a".into(),
                service_type: "_rdcs._tcp.local.".into(),
                port: 1,
                addr: None,
            })
            .unwrap();
        discovery
            .register(MdnsService {
                name: "b".into(),
                service_type: "_rdcs._tcp.local.".into(),
                port: 2,
                addr: None,
            })
            .unwrap();
        assert_eq!(discovery.registered_services().len(), 2);

        discovery.unregister_all().unwrap();
        assert_eq!(discovery.registered_services().len(), 0);
    }

    #[tokio::test]
    async fn mock_browse_emits_found_and_lost_events() {
        let discovery = MockMdnsDiscovery::new();
        discovery.inject_events(vec![
            MdnsEvent::Found(MdnsService {
                name: "peer-a".into(),
                service_type: "_rdcs._tcp.local.".into(),
                port: 21115,
                addr: Some("192.168.1.10".parse().unwrap()),
            }),
            MdnsEvent::Lost("peer-a".into()),
        ]);

        let mut rx = discovery.browse().unwrap();
        let first = rx.recv().await.expect("expected Found event");
        assert!(matches!(first, MdnsEvent::Found(_)));

        let second = rx.recv().await.expect("expected Lost event");
        assert!(matches!(second, MdnsEvent::Lost(name) if name == "peer-a"));
    }

    #[tokio::test]
    async fn mock_browse_empty_events() {
        let discovery = MockMdnsDiscovery::new();
        let mut rx = discovery.browse().unwrap();
        // No events injected — channel should close immediately.
        let result = rx.recv().await;
        assert!(result.is_none());
    }

    #[test]
    fn broadcaster_advertise_and_stop() {
        let discovery = MockMdnsDiscovery::new();
        let broadcaster = MdnsBroadcaster::new("myhost".into(), "_rdcs._tcp.local.".into(), 21115);

        broadcaster.advertise(&discovery, "CODE123").unwrap();
        assert_eq!(discovery.registered_services().len(), 1);
        assert_eq!(
            discovery.registered_services()[0].name,
            "myhost-CODE123"
        );

        broadcaster.stop(&discovery).unwrap();
        assert_eq!(discovery.registered_services().len(), 0);
    }
}
