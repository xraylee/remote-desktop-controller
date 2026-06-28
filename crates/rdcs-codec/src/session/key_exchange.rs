// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 简化的密钥交换模块（模拟 DTLS）。
//!
//! 注意：这是原型实现，用于测试和开发。生产环境应使用完整的 DTLS 握手。
//!
//! # 设计
//!
//! 使用简单的密钥协商流程：
//! 1. 生成临时密钥对
//! 2. 交换公钥
//! 3. 派生共享密钥
//! 4. 生成 SRTP 密钥材料

use crate::rtp::{SrtpConfig, SrtpProfile};
use rand::RngCore;
use sha2::{Digest, Sha256};
use thiserror::Error;

/// 密钥交换错误。
#[derive(Debug, Error)]
pub enum KeyExchangeError {
    /// 密钥协商失败。
    #[error("key negotiation failed: {0}")]
    NegotiationFailed(String),

    /// 无效的对端公钥。
    #[error("invalid peer public key")]
    InvalidPeerKey,

    /// 密钥派生失败。
    #[error("key derivation failed: {0}")]
    DerivationFailed(String),
}

/// 密钥交换结果。
pub type Result<T> = std::result::Result<T, KeyExchangeError>;

/// 简化的密钥交换上下文。
pub struct KeyExchange {
    /// 本地私钥（32 字节）。
    private_key: [u8; 32],
    /// 本地公钥（32 字节）。
    public_key: [u8; 32],
    /// 对端公钥（可选）。
    peer_public_key: Option<[u8; 32]>,
}

impl KeyExchange {
    /// 创建新的密钥交换上下文。
    ///
    /// 生成临时密钥对。
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let mut private_key = [0u8; 32];
        let mut public_key = [0u8; 32];

        rng.fill_bytes(&mut private_key);

        // 简化：公钥 = SHA256(私钥)（在真实 DTLS 中使用 ECDH）
        let hash = Sha256::digest(&private_key);
        public_key.copy_from_slice(&hash);

        Self {
            private_key,
            public_key,
            peer_public_key: None,
        }
    }

    /// 获取本地公钥。
    pub fn public_key(&self) -> &[u8; 32] {
        &self.public_key
    }

    /// 设置对端公钥。
    pub fn set_peer_public_key(&mut self, peer_key: [u8; 32]) -> Result<()> {
        // 验证对端公钥不为全零
        if peer_key.iter().all(|&b| b == 0) {
            return Err(KeyExchangeError::InvalidPeerKey);
        }

        self.peer_public_key = Some(peer_key);
        Ok(())
    }

    /// 派生 SRTP 密钥材料。
    ///
    /// 使用 HKDF 从共享密钥派生 SRTP master key 和 salt。
    pub fn derive_srtp_keys(&self, profile: SrtpProfile) -> Result<SrtpConfig> {
        let peer_key = self
            .peer_public_key
            .ok_or_else(|| KeyExchangeError::NegotiationFailed("peer key not set".into()))?;

        // 计算共享密钥：SHA256(private_key || peer_public_key)
        let mut hasher = Sha256::new();
        hasher.update(&self.private_key);
        hasher.update(&peer_key);
        let shared_secret = hasher.finalize();

        // 使用 HKDF 派生密钥材料
        let (master_key, master_salt) = self.hkdf_expand(&shared_secret)?;

        Ok(SrtpConfig {
            master_key: master_key.to_vec(),
            master_salt: master_salt.to_vec(),
            profile,
        })
    }

    /// 简化的 HKDF 扩展。
    ///
    /// 从共享密钥派生 16 字节 master key 和 14 字节 master salt。
    fn hkdf_expand(&self, shared_secret: &[u8]) -> Result<([u8; 16], [u8; 14])> {
        // HKDF-Expand 简化版：
        // master_key = SHA256(shared_secret || "srtp_key")[:16]
        // master_salt = SHA256(shared_secret || "srtp_salt")[:14]

        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        hasher.update(b"srtp_key");
        let key_hash = hasher.finalize();

        let mut hasher = Sha256::new();
        hasher.update(shared_secret);
        hasher.update(b"srtp_salt");
        let salt_hash = hasher.finalize();

        let mut master_key = [0u8; 16];
        let mut master_salt = [0u8; 14];

        master_key.copy_from_slice(&key_hash[..16]);
        master_salt.copy_from_slice(&salt_hash[..14]);

        Ok((master_key, master_salt))
    }
}

impl Default for KeyExchange {
    fn default() -> Self {
        Self::new()
    }
}

/// 密钥交换协调器（用于测试）。
///
/// 模拟完整的握手流程。
pub struct KeyExchangeCoordinator {
    /// 发起方。
    pub initiator: KeyExchange,
    /// 响应方。
    pub responder: KeyExchange,
}

impl KeyExchangeCoordinator {
    /// 创建新的协调器。
    pub fn new() -> Self {
        Self {
            initiator: KeyExchange::new(),
            responder: KeyExchange::new(),
        }
    }

    /// 执行完整的密钥交换。
    ///
    /// 返回双方的 SRTP 配置。
    pub fn perform_handshake(&mut self, profile: SrtpProfile) -> Result<(SrtpConfig, SrtpConfig)> {
        // 1. 交换公钥
        let initiator_pub = *self.initiator.public_key();
        let responder_pub = *self.responder.public_key();

        self.initiator.set_peer_public_key(responder_pub)?;
        self.responder.set_peer_public_key(initiator_pub)?;

        // 2. 派生 SRTP 密钥
        let initiator_srtp = self.initiator.derive_srtp_keys(profile)?;
        let responder_srtp = self.responder.derive_srtp_keys(profile)?;

        // 验证双方派生的密钥相同
        assert_eq!(
            initiator_srtp.master_key, responder_srtp.master_key,
            "derived keys must match"
        );
        assert_eq!(
            initiator_srtp.master_salt, responder_srtp.master_salt,
            "derived salts must match"
        );

        Ok((initiator_srtp, responder_srtp))
    }
}

impl Default for KeyExchangeCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_exchange_creation() {
        let kex = KeyExchange::new();

        // 公钥不应为全零
        assert!(!kex.public_key.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_set_peer_public_key() {
        let mut kex = KeyExchange::new();

        let peer_key = [0x42; 32];
        kex.set_peer_public_key(peer_key).unwrap();

        assert_eq!(kex.peer_public_key, Some(peer_key));
    }

    #[test]
    fn test_invalid_peer_public_key() {
        let mut kex = KeyExchange::new();

        // 全零密钥应该被拒绝
        let zero_key = [0u8; 32];
        let result = kex.set_peer_public_key(zero_key);

        assert!(result.is_err());
    }

    #[test]
    fn test_derive_srtp_keys_without_peer() {
        let kex = KeyExchange::new();

        // 未设置对端密钥时应该失败
        let result = kex.derive_srtp_keys(SrtpProfile::Aead_Aes128Gcm);
        assert!(result.is_err());
    }

    #[test]
    fn test_derive_srtp_keys() {
        let mut kex1 = KeyExchange::new();
        let mut kex2 = KeyExchange::new();

        // 交换公钥
        let pub1 = *kex1.public_key();
        let pub2 = *kex2.public_key();

        kex1.set_peer_public_key(pub2).unwrap();
        kex2.set_peer_public_key(pub1).unwrap();

        // 派生 SRTP 密钥
        let srtp1 = kex1
            .derive_srtp_keys(SrtpProfile::Aead_Aes128Gcm)
            .unwrap();
        let srtp2 = kex2
            .derive_srtp_keys(SrtpProfile::Aead_Aes128Gcm)
            .unwrap();

        // 双方应该派生出相同的密钥
        assert_eq!(srtp1.master_key, srtp2.master_key);
        assert_eq!(srtp1.master_salt, srtp2.master_salt);
        assert_eq!(srtp1.profile, srtp2.profile);

        // 验证密钥长度
        assert_eq!(srtp1.master_key.len(), 16);
        assert_eq!(srtp1.master_salt.len(), 14);
    }

    #[test]
    fn test_coordinator_handshake() {
        let mut coordinator = KeyExchangeCoordinator::new();

        let (initiator_srtp, responder_srtp) = coordinator
            .perform_handshake(SrtpProfile::Aead_Aes128Gcm)
            .unwrap();

        // 验证双方密钥相同
        assert_eq!(initiator_srtp.master_key, responder_srtp.master_key);
        assert_eq!(initiator_srtp.master_salt, responder_srtp.master_salt);
    }

    #[test]
    fn test_different_key_pairs_produce_different_keys() {
        let mut kex1a = KeyExchange::new();
        let mut kex1b = KeyExchange::new();

        let mut kex2a = KeyExchange::new();
        let mut kex2b = KeyExchange::new();

        // 配对 1: kex1a <-> kex1b
        kex1a.set_peer_public_key(*kex1b.public_key()).unwrap();
        kex1b.set_peer_public_key(*kex1a.public_key()).unwrap();

        let srtp1a = kex1a
            .derive_srtp_keys(SrtpProfile::Aead_Aes128Gcm)
            .unwrap();

        // 配对 2: kex2a <-> kex2b
        kex2a.set_peer_public_key(*kex2b.public_key()).unwrap();
        kex2b.set_peer_public_key(*kex2a.public_key()).unwrap();

        let srtp2a = kex2a
            .derive_srtp_keys(SrtpProfile::Aead_Aes128Gcm)
            .unwrap();

        // 不同的密钥对应该产生不同的 SRTP 密钥
        assert_ne!(srtp1a.master_key, srtp2a.master_key);
    }

    #[test]
    fn test_hkdf_expand_deterministic() {
        let kex = KeyExchange::new();
        let shared_secret = [0x42; 32];

        let (key1, salt1) = kex.hkdf_expand(&shared_secret).unwrap();
        let (key2, salt2) = kex.hkdf_expand(&shared_secret).unwrap();

        // 相同的输入应该产生相同的输出
        assert_eq!(key1, key2);
        assert_eq!(salt1, salt2);
    }
}
