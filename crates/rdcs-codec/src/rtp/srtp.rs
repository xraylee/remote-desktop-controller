// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! SRTP (Secure RTP) encryption/decryption layer.
//!
//! Provides AES-128-GCM encryption for RTP packets using the webrtc-srtp crate.
//!
//! # Architecture
//!
//! ```text
//! Sender:   RTP packet → encrypt() → SRTP packet → Network
//! Receiver: SRTP packet → decrypt() → RTP packet → Depacketizer
//! ```
//!
//! # Key Management
//!
//! Keys are derived from DTLS handshake (in production) or configured directly (for testing).

use super::{Result, RtpError};
use webrtc_srtp::{config::Config as SrtpLibConfig, context::Context, protection_profile::ProtectionProfile};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, trace, warn};

/// SRTP configuration.
#[derive(Debug, Clone)]
pub struct SrtpConfig {
    /// Master key (16 bytes for AES-128).
    pub master_key: Vec<u8>,
    /// Master salt (14 bytes for AES-128).
    pub master_salt: Vec<u8>,
    /// Protection profile (encryption algorithm).
    pub profile: SrtpProfile,
}

/// SRTP protection profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SrtpProfile {
    /// AES-128-CM with HMAC-SHA1-80 (legacy, less secure).
    Aes128CmHmacSha1_80,
    /// AES-128-GCM (modern, recommended).
    Aead_Aes128Gcm,
}

impl SrtpProfile {
    fn to_webrtc_profile(self) -> ProtectionProfile {
        match self {
            Self::Aes128CmHmacSha1_80 => ProtectionProfile::Aes128CmHmacSha1_80,
            Self::Aead_Aes128Gcm => ProtectionProfile::AeadAes128Gcm,
        }
    }
}

impl Default for SrtpConfig {
    fn default() -> Self {
        Self {
            master_key: vec![0u8; 16],
            master_salt: vec![0u8; 14],
            profile: SrtpProfile::Aead_Aes128Gcm,
        }
    }
}

/// SRTP encryption/decryption context.
///
/// Wraps webrtc-srtp's Context with thread-safe access.
pub struct SrtpContext {
    /// Sender context (for encrypting outbound RTP → SRTP).
    tx_context: Arc<Mutex<Context>>,
    /// Receiver context (for decrypting inbound SRTP → RTP).
    rx_context: Arc<Mutex<Context>>,
    /// Statistics.
    stats: Arc<Mutex<SrtpStats>>,
}

/// SRTP statistics.
#[derive(Debug, Clone, Default)]
pub struct SrtpStats {
    pub packets_encrypted: u64,
    pub packets_decrypted: u64,
    pub bytes_encrypted: u64,
    pub bytes_decrypted: u64,
    pub encryption_errors: u32,
    pub decryption_errors: u32,
    pub replay_errors: u32,
}

impl SrtpContext {
    /// Create a new SRTP context with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns error if key/salt lengths are invalid or context creation fails.
    pub async fn new(config: SrtpConfig) -> Result<Self> {
        // Validate key material
        if config.master_key.len() != 16 {
            return Err(RtpError::SrtpError(format!(
                "invalid master key length: {} (expected 16)",
                config.master_key.len()
            )));
        }
        if config.master_salt.len() != 14 {
            return Err(RtpError::SrtpError(format!(
                "invalid master salt length: {} (expected 14)",
                config.master_salt.len()
            )));
        }

        let profile = config.profile.to_webrtc_profile();

        // Create sender context
        let tx_config = SrtpLibConfig {
            profile,
            keys: webrtc_srtp::config::ConfigKeyMaterial {
                master_key: config.master_key.clone(),
                master_salt: config.master_salt.clone(),
            },
            ..Default::default()
        };

        let tx_context = Context::new(&config.master_key, &config.master_salt, profile, None, None)
            .map_err(|e| RtpError::SrtpError(format!("failed to create TX context: {}", e)))?;

        // Create receiver context
        let rx_context = Context::new(&config.master_key, &config.master_salt, profile, None, None)
            .map_err(|e| RtpError::SrtpError(format!("failed to create RX context: {}", e)))?;

        debug!(
            "SRTP context created with profile {:?}",
            config.profile
        );

        Ok(Self {
            tx_context: Arc::new(Mutex::new(tx_context)),
            rx_context: Arc::new(Mutex::new(rx_context)),
            stats: Arc::new(Mutex::new(SrtpStats::default())),
        })
    }

    /// Encrypt an RTP packet to SRTP.
    ///
    /// # Arguments
    ///
    /// * `rtp_packet` - Plain RTP packet data
    ///
    /// # Returns
    ///
    /// Encrypted SRTP packet (larger than input due to auth tag).
    pub async fn encrypt(&self, rtp_packet: &[u8]) -> Result<Vec<u8>> {
        if rtp_packet.len() < 12 {
            return Err(RtpError::InvalidPacket(format!(
                "packet too short for RTP header: {} bytes",
                rtp_packet.len()
            )));
        }

        let mut srtp_packet = rtp_packet.to_vec();

        // Encrypt in-place (webrtc-srtp appends auth tag)
        let mut ctx = self.tx_context.lock().await;
        ctx.encrypt_rtp(&mut srtp_packet)
            .map_err(|e| {
                let mut stats = futures_util::executor::block_on(self.stats.lock());
                stats.encryption_errors += 1;
                RtpError::SrtpError(format!("encryption failed: {}", e))
            })?;

        // Update stats
        let mut stats = self.stats.lock().await;
        stats.packets_encrypted += 1;
        stats.bytes_encrypted += srtp_packet.len() as u64;

        trace!(
            "Encrypted RTP packet: {} → {} bytes",
            rtp_packet.len(),
            srtp_packet.len()
        );

        Ok(srtp_packet)
    }

    /// Decrypt an SRTP packet to RTP.
    ///
    /// # Arguments
    ///
    /// * `srtp_packet` - Encrypted SRTP packet data
    ///
    /// # Returns
    ///
    /// Decrypted RTP packet (smaller than input, auth tag removed).
    pub async fn decrypt(&self, srtp_packet: &[u8]) -> Result<Vec<u8>> {
        if srtp_packet.len() < 12 {
            return Err(RtpError::InvalidPacket(format!(
                "packet too short for SRTP header: {} bytes",
                srtp_packet.len()
            )));
        }

        let mut rtp_packet = srtp_packet.to_vec();

        // Decrypt in-place (webrtc-srtp removes auth tag)
        let mut ctx = self.rx_context.lock().await;
        ctx.decrypt_rtp(&mut rtp_packet)
            .map_err(|e| {
                let error_str = e.to_string();
                let mut stats = futures_util::executor::block_on(self.stats.lock());

                if error_str.contains("replay") {
                    stats.replay_errors += 1;
                } else {
                    stats.decryption_errors += 1;
                }

                RtpError::SrtpError(format!("decryption failed: {}", e))
            })?;

        // Update stats
        let mut stats = self.stats.lock().await;
        stats.packets_decrypted += 1;
        stats.bytes_decrypted += rtp_packet.len() as u64;

        trace!(
            "Decrypted SRTP packet: {} → {} bytes",
            srtp_packet.len(),
            rtp_packet.len()
        );

        Ok(rtp_packet)
    }

    /// Get current SRTP statistics.
    pub async fn stats(&self) -> SrtpStats {
        self.stats.lock().await.clone()
    }

    /// Reset statistics counters.
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.lock().await;
        *stats = SrtpStats::default();
    }
}

/// Generate a random SRTP key pair for testing.
///
/// **WARNING**: Do NOT use this in production. Keys must be derived from DTLS handshake.
#[cfg(test)]
pub fn generate_test_keys() -> SrtpConfig {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    let master_key: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
    let master_salt: Vec<u8> = (0..14).map(|_| rng.gen()).collect();

    SrtpConfig {
        master_key,
        master_salt,
        profile: SrtpProfile::Aead_Aes128Gcm,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtp::{RtpHeader, H264Packetizer, PacketizerConfig};

    #[tokio::test]
    async fn test_srtp_encrypt_decrypt_roundtrip() {
        let config = generate_test_keys();
        let srtp = SrtpContext::new(config).await.unwrap();

        // Create a test RTP packet
        let mut packetizer = H264Packetizer::new(PacketizerConfig::default());
        let annex_b = vec![0, 0, 0, 1, 0x65, 0xAA, 0xBB, 0xCC]; // Small IDR NAL
        let rtp_packets = packetizer.packetize(&annex_b, 12345).unwrap();
        let original_rtp = &rtp_packets[0];

        // Encrypt
        let srtp_packet = srtp.encrypt(original_rtp).await.unwrap();
        assert!(srtp_packet.len() > original_rtp.len()); // Auth tag added

        // Decrypt
        let decrypted_rtp = srtp.decrypt(&srtp_packet).await.unwrap();
        assert_eq!(decrypted_rtp, original_rtp);

        // Check stats
        let stats = srtp.stats().await;
        assert_eq!(stats.packets_encrypted, 1);
        assert_eq!(stats.packets_decrypted, 1);
    }

    #[tokio::test]
    async fn test_srtp_invalid_key_length() {
        let invalid_config = SrtpConfig {
            master_key: vec![0u8; 10], // Wrong length
            master_salt: vec![0u8; 14],
            profile: SrtpProfile::Aead_Aes128Gcm,
        };

        let result = SrtpContext::new(invalid_config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid master key length"));
    }

    #[tokio::test]
    async fn test_srtp_tampered_packet() {
        let config = generate_test_keys();
        let srtp = SrtpContext::new(config).await.unwrap();

        // Create and encrypt a packet
        let mut packetizer = H264Packetizer::new(PacketizerConfig::default());
        let annex_b = vec![0, 0, 0, 1, 0x65, 0xAA];
        let rtp_packets = packetizer.packetize(&annex_b, 12345).unwrap();
        let mut srtp_packet = srtp.encrypt(&rtp_packets[0]).await.unwrap();

        // Tamper with the packet
        srtp_packet[15] ^= 0xFF;

        // Decryption should fail
        let result = srtp.decrypt(&srtp_packet).await;
        assert!(result.is_err());

        let stats = srtp.stats().await;
        assert_eq!(stats.decryption_errors, 1);
    }

    #[tokio::test]
    async fn test_srtp_replay_protection() {
        let config = generate_test_keys();
        let srtp = SrtpContext::new(config).await.unwrap();

        // Create and encrypt a packet
        let mut packetizer = H264Packetizer::new(PacketizerConfig::default());
        let annex_b = vec![0, 0, 0, 1, 0x65, 0xAA];
        let rtp_packets = packetizer.packetize(&annex_b, 12345).unwrap();
        let srtp_packet = srtp.encrypt(&rtp_packets[0]).await.unwrap();

        // First decryption should succeed
        let result1 = srtp.decrypt(&srtp_packet).await;
        assert!(result1.is_ok());

        // Replay should fail (webrtc-srtp tracks sequence numbers)
        let result2 = srtp.decrypt(&srtp_packet).await;
        assert!(result2.is_err());

        let stats = srtp.stats().await;
        assert!(stats.replay_errors > 0 || stats.decryption_errors > 0);
    }

    #[tokio::test]
    async fn test_srtp_packet_too_short() {
        let config = generate_test_keys();
        let srtp = SrtpContext::new(config).await.unwrap();

        let short_packet = vec![0u8; 5];

        let result = srtp.encrypt(&short_packet).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[tokio::test]
    async fn test_multiple_packets() {
        let config = generate_test_keys();
        let srtp = SrtpContext::new(config).await.unwrap();

        let mut packetizer = H264Packetizer::new(PacketizerConfig::default());

        // Encrypt and decrypt multiple packets
        for i in 0..10 {
            let annex_b = vec![0, 0, 0, 1, 0x65, i as u8];
            let rtp_packets = packetizer.packetize(&annex_b, 1000 + i * 100).unwrap();

            let srtp_packet = srtp.encrypt(&rtp_packets[0]).await.unwrap();
            let decrypted = srtp.decrypt(&srtp_packet).await.unwrap();

            assert_eq!(decrypted, rtp_packets[0]);
        }

        let stats = srtp.stats().await;
        assert_eq!(stats.packets_encrypted, 10);
        assert_eq!(stats.packets_decrypted, 10);
    }
}
