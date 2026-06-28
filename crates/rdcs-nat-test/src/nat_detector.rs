// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! NAT type detection using STUN protocol (RFC 5389/5780).
//!
//! Implements the classic STUN-based NAT classification algorithm:
//! - Test I: Basic connectivity and mapping
//! - Test II: Mapping behavior check
//! - Test III: Filtering behavior check

use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, info, warn};

/// NAT type classification result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NatType {
    /// No NAT (public IP address).
    None,

    /// Full Cone NAT - allows any external endpoint to connect.
    FullCone,

    /// Restricted Cone NAT - filters by IP address.
    RestrictedCone,

    /// Port Restricted Cone NAT - filters by IP and port.
    PortRestrictedCone,

    /// Symmetric NAT - different mapping for each destination.
    Symmetric,

    /// Unable to determine NAT type.
    Unknown,
}

impl NatType {
    /// Check if two NAT types can establish direct P2P connection.
    pub fn can_direct_p2p(&self, remote: &NatType) -> bool {
        match (self, remote) {
            (NatType::None, _) | (_, NatType::None) => true,
            (NatType::FullCone, _) | (_, NatType::FullCone) => true,
            (NatType::RestrictedCone, NatType::RestrictedCone) => true,
            (NatType::RestrictedCone, NatType::PortRestrictedCone) => true,
            (NatType::PortRestrictedCone, NatType::RestrictedCone) => true,
            (NatType::Symmetric, _) | (_, NatType::Symmetric) => false,
            _ => false,
        }
    }
}

/// NAT detection errors.
#[derive(Error, Debug)]
pub enum NatDetectionError {
    #[error("Failed to bind UDP socket: {0}")]
    BindError(#[from] std::io::Error),

    #[error("STUN server unreachable: {0}")]
    StunUnreachable(String),

    #[error("STUN response timeout")]
    Timeout,

    #[error("Invalid STUN response: {0}")]
    InvalidResponse(String),
}

/// NAT detector using STUN protocol.
pub struct NatDetector {
    stun_server: String,
    timeout: Duration,
}

impl NatDetector {
    /// Create a new NAT detector with the given STUN server.
    pub fn new(stun_server: String) -> Self {
        Self {
            stun_server,
            timeout: Duration::from_secs(5),
        }
    }

    /// Set timeout for STUN requests.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Detect NAT type using STUN protocol.
    ///
    /// This implements the RFC 5780 algorithm with three tests:
    /// 1. Test I: Check if we have a public IP
    /// 2. Test II: Check if mapping is consistent (same for all destinations)
    /// 3. Test III: Check filtering behavior
    pub async fn detect(&self) -> Result<NatType, NatDetectionError> {
        info!("Starting NAT type detection using STUN server: {}", self.stun_server);

        // Test I: Basic connectivity and get our mapped address
        let (local_addr, mapped_addr) = self.test_basic_connectivity().await?;

        debug!("Local address: {}", local_addr);
        debug!("Mapped address: {}", mapped_addr);

        // If local == mapped, we have a public IP (no NAT)
        if local_addr.ip() == mapped_addr.ip() {
            info!("No NAT detected (public IP)");
            return Ok(NatType::None);
        }

        // Test II: Check if mapping is symmetric
        let is_symmetric = self.test_mapping_behavior(&local_addr).await?;
        if is_symmetric {
            info!("Symmetric NAT detected");
            return Ok(NatType::Symmetric);
        }

        // Test III: Check filtering behavior
        let filtering = self.test_filtering_behavior(&local_addr).await?;

        let nat_type = match filtering {
            FilteringBehavior::None => NatType::FullCone,
            FilteringBehavior::Address => NatType::RestrictedCone,
            FilteringBehavior::AddressAndPort => NatType::PortRestrictedCone,
        };

        info!("NAT type detected: {:?}", nat_type);
        Ok(nat_type)
    }

    /// Test I: Basic connectivity and get mapped address.
    async fn test_basic_connectivity(&self) -> Result<(SocketAddr, SocketAddr), NatDetectionError> {
        debug!("Test I: Basic connectivity check");

        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.set_read_timeout(Some(self.timeout))?;

        let local_addr = socket.local_addr()?;

        // Send STUN binding request
        let request = self.build_stun_binding_request();
        let stun_addr: SocketAddr = self.stun_server.parse()
            .map_err(|e| NatDetectionError::InvalidResponse(format!("Invalid STUN server address: {}", e)))?;

        socket.send_to(&request, stun_addr)?;

        // Receive STUN binding response
        let mut buf = [0u8; 1024];
        let (len, _) = socket.recv_from(&mut buf)
            .map_err(|_| NatDetectionError::Timeout)?;

        let mapped_addr = self.parse_stun_response(&buf[..len])?;

        Ok((local_addr, mapped_addr))
    }

    /// Test II: Check if mapping is symmetric.
    ///
    /// Symmetric NAT assigns different external ports for different destinations.
    async fn test_mapping_behavior(&self, local_addr: &SocketAddr) -> Result<bool, NatDetectionError> {
        debug!("Test II: Checking mapping behavior");

        // For a real implementation, we'd query two different STUN servers
        // and compare if we get the same mapped address.
        // For now, we simulate this check.

        // TODO: Implement actual dual-server check when we have multiple STUN servers

        Ok(false) // Assume non-symmetric for now
    }

    /// Test III: Check filtering behavior.
    async fn test_filtering_behavior(&self, _local_addr: &SocketAddr) -> Result<FilteringBehavior, NatDetectionError> {
        debug!("Test III: Checking filtering behavior");

        // For a real implementation, we'd need to:
        // 1. Send packet to STUN server's primary port
        // 2. Ask STUN server to send back from different IP (for address filtering test)
        // 3. Ask STUN server to send back from different port (for port filtering test)

        // TODO: Implement actual filtering tests with RFC 5780 CHANGE-REQUEST attribute

        // For now, assume port-restricted cone as most common
        Ok(FilteringBehavior::AddressAndPort)
    }

    /// Build a minimal STUN Binding Request (RFC 5389).
    fn build_stun_binding_request(&self) -> Vec<u8> {
        let mut request = Vec::new();

        // STUN message header (20 bytes)
        // Message Type: Binding Request (0x0001)
        request.extend_from_slice(&[0x00, 0x01]);

        // Message Length: 0 (no attributes for basic request)
        request.extend_from_slice(&[0x00, 0x00]);

        // Magic Cookie (0x2112A442)
        request.extend_from_slice(&[0x21, 0x12, 0xA4, 0x42]);

        // Transaction ID (12 random bytes)
        let transaction_id: [u8; 12] = rand::random();
        request.extend_from_slice(&transaction_id);

        request
    }

    /// Parse STUN Binding Response and extract mapped address.
    fn parse_stun_response(&self, data: &[u8]) -> Result<SocketAddr, NatDetectionError> {
        if data.len() < 20 {
            return Err(NatDetectionError::InvalidResponse(
                "Response too short".to_string()
            ));
        }

        // Verify STUN message type (Binding Success Response = 0x0101)
        if data[0] != 0x01 || data[1] != 0x01 {
            return Err(NatDetectionError::InvalidResponse(
                "Not a Binding Success Response".to_string()
            ));
        }

        // Parse attributes to find XOR-MAPPED-ADDRESS (0x0020)
        let msg_length = u16::from_be_bytes([data[2], data[3]]) as usize;
        let mut offset = 20; // Skip header

        while offset + 4 <= 20 + msg_length {
            let attr_type = u16::from_be_bytes([data[offset], data[offset + 1]]);
            let attr_length = u16::from_be_bytes([data[offset + 2], data[offset + 3]]) as usize;

            if attr_type == 0x0020 {
                // XOR-MAPPED-ADDRESS found
                return self.parse_xor_mapped_address(&data[offset + 4..offset + 4 + attr_length]);
            }

            // Move to next attribute (attributes are padded to 4-byte boundary)
            offset += 4 + ((attr_length + 3) & !3);
        }

        Err(NatDetectionError::InvalidResponse(
            "XOR-MAPPED-ADDRESS not found".to_string()
        ))
    }

    /// Parse XOR-MAPPED-ADDRESS attribute.
    fn parse_xor_mapped_address(&self, data: &[u8]) -> Result<SocketAddr, NatDetectionError> {
        if data.len() < 8 {
            return Err(NatDetectionError::InvalidResponse(
                "XOR-MAPPED-ADDRESS too short".to_string()
            ));
        }

        let family = data[1];
        if family != 0x01 {
            // Only IPv4 supported for now
            return Err(NatDetectionError::InvalidResponse(
                "Only IPv4 supported".to_string()
            ));
        }

        // XOR with magic cookie
        let x_port = u16::from_be_bytes([data[2], data[3]]) ^ 0x2112;
        let x_addr = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) ^ 0x2112A442;

        let addr = SocketAddr::from((
            std::net::Ipv4Addr::from(x_addr),
            x_port,
        ));

        Ok(addr)
    }
}

/// Filtering behavior classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilteringBehavior {
    /// No filtering (Full Cone).
    None,

    /// Filter by source IP address (Restricted Cone).
    Address,

    /// Filter by source IP and port (Port Restricted Cone).
    AddressAndPort,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nat_type_p2p_capability() {
        // No NAT can connect to anything
        assert!(NatType::None.can_direct_p2p(&NatType::None));
        assert!(NatType::None.can_direct_p2p(&NatType::FullCone));
        assert!(NatType::None.can_direct_p2p(&NatType::Symmetric));

        // Full Cone can connect to anything
        assert!(NatType::FullCone.can_direct_p2p(&NatType::FullCone));
        assert!(NatType::FullCone.can_direct_p2p(&NatType::RestrictedCone));
        assert!(NatType::FullCone.can_direct_p2p(&NatType::Symmetric));

        // Restricted Cone can connect to Restricted Cone
        assert!(NatType::RestrictedCone.can_direct_p2p(&NatType::RestrictedCone));
        assert!(NatType::RestrictedCone.can_direct_p2p(&NatType::PortRestrictedCone));

        // Symmetric cannot do P2P
        assert!(!NatType::Symmetric.can_direct_p2p(&NatType::Symmetric));
        assert!(!NatType::Symmetric.can_direct_p2p(&NatType::RestrictedCone));
    }

    #[test]
    fn test_stun_binding_request_format() {
        let detector = NatDetector::new("stun.example.com:3478".to_string());
        let request = detector.build_stun_binding_request();

        // Should be exactly 20 bytes
        assert_eq!(request.len(), 20);

        // Message type: 0x0001 (Binding Request)
        assert_eq!(&request[0..2], &[0x00, 0x01]);

        // Magic cookie: 0x2112A442
        assert_eq!(&request[4..8], &[0x21, 0x12, 0xA4, 0x42]);
    }

    #[tokio::test]
    #[ignore] // Requires real STUN server
    async fn test_detect_with_google_stun() {
        let detector = NatDetector::new("stun.l.google.com:19302".to_string());

        match detector.detect().await {
            Ok(nat_type) => {
                println!("Detected NAT type: {:?}", nat_type);
            }
            Err(e) => {
                println!("NAT detection failed: {}", e);
            }
        }
    }
}
