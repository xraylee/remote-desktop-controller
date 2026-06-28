// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! ICE server configuration for NAT traversal.
//!
//! Provides STUN and TURN server configurations for different regions
//! and network conditions.

use serde::{Deserialize, Serialize};

/// Complete ICE server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServerConfig {
    /// STUN servers for NAT type detection and reflexive candidates.
    pub stun_servers: Vec<String>,

    /// TURN relay servers with authentication.
    pub turn_servers: Vec<TurnServerConfig>,
}

/// TURN server configuration with credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnServerConfig {
    /// TURN server URLs (udp/tcp transports).
    pub urls: Vec<String>,

    /// Authentication username.
    pub username: String,

    /// Authentication credential (password).
    pub credential: String,

    /// Geographic region for latency optimization.
    pub region: String,

    /// Maximum bandwidth limit (bytes/sec, for free tier).
    #[serde(default)]
    pub bandwidth_limit: Option<u64>,
}

impl Default for IceServerConfig {
    fn default() -> Self {
        Self::production_config()
    }
}

impl IceServerConfig {
    /// Production ICE server configuration with multi-region TURN servers.
    pub fn production_config() -> Self {
        Self {
            stun_servers: vec![
                "stun:stun.rdcs.io:3478".to_string(),
                "stun:stun.l.google.com:19302".to_string(), // Backup
            ],
            turn_servers: vec![
                // US West (Oregon)
                TurnServerConfig {
                    urls: vec![
                        "turn:turn-us-west.rdcs.io:3478?transport=udp".to_string(),
                        "turn:turn-us-west.rdcs.io:3478?transport=tcp".to_string(),
                    ],
                    username: "rdcs-user".to_string(),
                    credential: std::env::var("TURN_PASSWORD")
                        .unwrap_or_else(|_| "changeme".to_string()),
                    region: "us-west".to_string(),
                    bandwidth_limit: Some(2_000_000), // 2 Mbps for free tier
                },

                // US East (Virginia)
                TurnServerConfig {
                    urls: vec![
                        "turn:turn-us-east.rdcs.io:3478?transport=udp".to_string(),
                    ],
                    username: "rdcs-user".to_string(),
                    credential: std::env::var("TURN_PASSWORD")
                        .unwrap_or_else(|_| "changeme".to_string()),
                    region: "us-east".to_string(),
                    bandwidth_limit: Some(2_000_000),
                },

                // EU Central (Frankfurt)
                TurnServerConfig {
                    urls: vec![
                        "turn:turn-eu-central.rdcs.io:3478?transport=udp".to_string(),
                    ],
                    username: "rdcs-user".to_string(),
                    credential: std::env::var("TURN_PASSWORD")
                        .unwrap_or_else(|_| "changeme".to_string()),
                    region: "eu-central".to_string(),
                    bandwidth_limit: Some(2_000_000),
                },

                // AP Southeast (Singapore)
                TurnServerConfig {
                    urls: vec![
                        "turn:turn-ap-southeast.rdcs.io:3478?transport=udp".to_string(),
                    ],
                    username: "rdcs-user".to_string(),
                    credential: std::env::var("TURN_PASSWORD")
                        .unwrap_or_else(|_| "changeme".to_string()),
                    region: "ap-southeast".to_string(),
                    bandwidth_limit: Some(2_000_000),
                },
            ],
        }
    }

    /// Test configuration using local servers and public STUN.
    pub fn test_config() -> Self {
        Self {
            stun_servers: vec![
                "stun:localhost:3478".to_string(),
                "stun:stun.l.google.com:19302".to_string(),
            ],
            turn_servers: vec![
                TurnServerConfig {
                    urls: vec![
                        "turn:localhost:3478?transport=udp".to_string(),
                    ],
                    username: "test-user".to_string(),
                    credential: "test-password".to_string(),
                    region: "local".to_string(),
                    bandwidth_limit: None, // No limit in tests
                },
            ],
        }
    }

    /// Select nearest TURN servers based on client region.
    ///
    /// Returns 2-3 nearest servers to minimize latency.
    pub fn select_nearest_servers(&self, client_region: &str) -> Vec<TurnServerConfig> {
        let mut servers = self.turn_servers.clone();

        // Sort by distance (simplified - in production use GeoIP)
        servers.sort_by_key(|s| {
            region_distance(client_region, &s.region)
        });

        // Return top 3 nearest
        servers.into_iter().take(3).collect()
    }

    /// Get all STUN server URLs.
    pub fn get_stun_urls(&self) -> Vec<String> {
        self.stun_servers.clone()
    }

    /// Get all TURN server URLs (for a specific region if provided).
    pub fn get_turn_urls(&self, region: Option<&str>) -> Vec<String> {
        self.turn_servers
            .iter()
            .filter(|s| region.map_or(true, |r| s.region == r))
            .flat_map(|s| s.urls.clone())
            .collect()
    }
}

/// Calculate distance between regions (simplified).
///
/// In production, use actual GeoIP coordinates and haversine distance.
fn region_distance(from: &str, to: &str) -> u32 {
    if from == to {
        return 0;
    }

    // Simplified distance matrix
    match (from, to) {
        // US regions
        ("us-west", "us-east") | ("us-east", "us-west") => 1,
        ("us-west", "eu-central") | ("eu-central", "us-west") => 2,
        ("us-west", "ap-southeast") | ("ap-southeast", "us-west") => 2,
        ("us-east", "eu-central") | ("eu-central", "us-east") => 2,
        ("us-east", "ap-southeast") | ("ap-southeast", "us-east") => 3,

        // EU regions
        ("eu-central", "ap-southeast") | ("ap-southeast", "eu-central") => 3,

        // Default: unknown region
        _ => 10,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_config() {
        let config = IceServerConfig::production_config();

        assert!(!config.stun_servers.is_empty());
        assert_eq!(config.turn_servers.len(), 4); // 4 regions

        // Verify all regions covered
        let regions: Vec<_> = config.turn_servers.iter().map(|s| s.region.as_str()).collect();
        assert!(regions.contains(&"us-west"));
        assert!(regions.contains(&"us-east"));
        assert!(regions.contains(&"eu-central"));
        assert!(regions.contains(&"ap-southeast"));
    }

    #[test]
    fn test_test_config() {
        let config = IceServerConfig::test_config();

        assert!(!config.stun_servers.is_empty());
        assert_eq!(config.turn_servers.len(), 1);
        assert_eq!(config.turn_servers[0].region, "local");
    }

    #[test]
    fn test_select_nearest_servers() {
        let config = IceServerConfig::production_config();

        let nearest = config.select_nearest_servers("us-west");
        assert!(nearest.len() <= 3);

        // First should be us-west
        assert_eq!(nearest[0].region, "us-west");
    }

    #[test]
    fn test_region_distance() {
        assert_eq!(region_distance("us-west", "us-west"), 0);
        assert_eq!(region_distance("us-west", "us-east"), 1);
        assert!(region_distance("us-west", "ap-southeast") > 1);
    }

    #[test]
    fn test_get_stun_urls() {
        let config = IceServerConfig::production_config();
        let urls = config.get_stun_urls();

        assert!(urls.iter().any(|u| u.contains("stun.rdcs.io")));
    }

    #[test]
    fn test_get_turn_urls() {
        let config = IceServerConfig::production_config();

        // All TURN URLs
        let all_urls = config.get_turn_urls(None);
        assert!(all_urls.len() >= 4);

        // Region-specific
        let us_west_urls = config.get_turn_urls(Some("us-west"));
        assert!(us_west_urls.iter().any(|u| u.contains("turn-us-west")));
    }

    #[test]
    fn test_bandwidth_limits() {
        let config = IceServerConfig::production_config();

        for server in &config.turn_servers {
            assert!(server.bandwidth_limit.is_some());
            assert_eq!(server.bandwidth_limit.unwrap(), 2_000_000); // 2 Mbps
        }
    }
}
