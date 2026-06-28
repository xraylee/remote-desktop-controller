// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! X25519 key exchange primitives.
//!
//! Provides keypair generation, shared-secret derivation via Curve25519
//! Diffie-Hellman, and session-key derivation using HKDF-SHA256.
//! All secret material is zeroized on drop.

use serde::{Deserialize, Serialize};
use x25519_dalek::{PublicKey as DalekPublicKey, StaticSecret};
use zeroize::Zeroize;

use crate::error::CryptoError;

/// An X25519 public key (32 bytes).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PublicKey(#[serde(with = "serde_bytes")] pub Vec<u8>);

impl PublicKey {
    /// Construct from a 32-byte slice. Returns an error if the length is wrong.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::KeyExchangeFailed(format!(
                "public key must be 32 bytes, got {}",
                bytes.len()
            )));
        }
        Ok(Self(bytes.to_vec()))
    }

    /// Return the raw bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// An X25519 secret key (32 bytes). Zeroized on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SecretKey(pub [u8; 32]);

impl std::fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretKey(<redacted>)")
    }
}

/// A key pair for X25519 Diffie-Hellman.
#[derive(Debug)]
pub struct KeyPair {
    pub public: PublicKey,
    pub secret: SecretKey,
}

impl KeyPair {
    /// Generate a new random key pair using the OS random number generator.
    pub fn generate() -> Result<Self, CryptoError> {
        use rand::RngCore;
        let mut seed = [0u8; 32];
        rand::thread_rng()
            .try_fill_bytes(&mut seed)
            .map_err(|e| CryptoError::KeyExchangeFailed(format!("RNG failure: {e}")))?;
        let secret = StaticSecret::from(seed);
        let public = DalekPublicKey::from(&secret);
        let keypair = Self {
            public: PublicKey(public.as_bytes().to_vec()),
            secret: SecretKey(secret.to_bytes()),
        };
        // Zeroize the temporary seed
        seed.zeroize();
        Ok(keypair)
    }
}

/// A shared secret derived from X25519 DH. Zeroized on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SharedSecret(pub [u8; 32]);

impl std::fmt::Debug for SharedSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SharedSecret(<redacted>)")
    }
}

/// A session encryption key. Zeroized on drop.
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SessionKey {
    /// The raw 32-byte key material.
    pub material: [u8; 32],
}

impl std::fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionKey(<redacted>)")
    }
}

/// Generate a new X25519 key pair.
pub fn generate_keypair() -> KeyPair {
    KeyPair::generate().expect("key generation should not fail with OS RNG")
}

/// Perform X25519 Diffie-Hellman to derive a shared secret.
///
/// Given our secret key and the remote party's public key, computes the
/// shared secret that both parties can independently derive.
pub fn derive_shared_secret(
    our_secret: &SecretKey,
    their_public: &PublicKey,
) -> Result<SharedSecret, CryptoError> {
    let secret = StaticSecret::from(our_secret.0);
    let public_bytes: [u8; 32] =
        their_public
            .0
            .as_slice()
            .try_into()
            .map_err(|_| {
                CryptoError::KeyExchangeFailed(format!(
                    "invalid public key length: expected 32, got {}",
                    their_public.0.len()
                ))
            })?;
    let public = DalekPublicKey::from(public_bytes);
    let shared = secret.diffie_hellman(&public);
    Ok(SharedSecret(shared.to_bytes()))
}

/// Derive a pair of session keys (send, receive) from a shared secret using
/// HKDF-SHA256.
///
/// The `salt` parameter provides domain separation and should include session
/// metadata to prevent cross-session key reuse.
pub fn derive_session_keys(
    shared: &SharedSecret,
    salt: &[u8],
) -> Result<(SessionKey, SessionKey), CryptoError> {
    use hkdf::Hkdf;
    use sha2::Sha256;

    let hk = Hkdf::<Sha256>::new(Some(salt), &shared.0);

    let mut send_material = [0u8; 32];
    let mut recv_material = [0u8; 32];

    hk.expand(b"rdcs-session-send-key-v1", &mut send_material)
        .map_err(|e| CryptoError::KeyDerivationFailed(format!("HKDF expand send: {e}")))?;
    hk.expand(b"rdcs-session-recv-key-v1", &mut recv_material)
        .map_err(|e| CryptoError::KeyDerivationFailed(format!("HKDF expand recv: {e}")))?;

    Ok((
        SessionKey {
            material: send_material,
        },
        SessionKey {
            material: recv_material,
        },
    ))
}

/// Rotate a nonce counter and return a unique 24-byte nonce.
///
/// The counter is encoded as a little-endian u64 in the first 8 bytes,
/// with the remaining 16 bytes set to zero. Each call increments the
/// counter, guaranteeing uniqueness for up to 2^64 calls.
///
/// Returns an error if the counter would overflow.
pub fn rotate_nonce(counter: &mut u64) -> Result<[u8; 24], CryptoError> {
    if *counter == u64::MAX {
        return Err(CryptoError::NonceOverflow);
    }
    let mut nonce = [0u8; 24];
    nonce[..8].copy_from_slice(&counter.to_le_bytes());
    *counter += 1;
    Ok(nonce)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_key_pair() {
        let kp = KeyPair::generate().expect("key generation should succeed");
        assert_eq!(kp.public.0.len(), 32);
        // Secret key should not be all zeros (overwhelmingly unlikely with real RNG)
        assert_ne!(kp.secret.0, [0u8; 32]);
    }

    #[test]
    fn generate_keypair_free_function() {
        let kp = generate_keypair();
        assert_eq!(kp.public.0.len(), 32);
    }

    #[test]
    fn shared_secret_agreement() {
        // Two parties generate keypairs
        let alice = KeyPair::generate().unwrap();
        let bob = KeyPair::generate().unwrap();

        // Both derive the shared secret
        let alice_shared = derive_shared_secret(&alice.secret, &bob.public).unwrap();
        let bob_shared = derive_shared_secret(&bob.secret, &alice.public).unwrap();

        // They must agree on the same shared secret
        assert_eq!(alice_shared.0, bob_shared.0);
    }

    #[test]
    fn different_peers_different_secrets() {
        let alice = KeyPair::generate().unwrap();
        let bob = KeyPair::generate().unwrap();
        let carol = KeyPair::generate().unwrap();

        let ab = derive_shared_secret(&alice.secret, &bob.public).unwrap();
        let ac = derive_shared_secret(&alice.secret, &carol.public).unwrap();

        // Shared secrets with different peers should differ
        assert_ne!(ab.0, ac.0);
    }

    #[test]
    fn derive_session_keys_deterministic() {
        let shared = SharedSecret([42u8; 32]);
        let salt = b"test-session-salt";

        let (send1, recv1) = derive_session_keys(&shared, salt).unwrap();
        let (send2, recv2) = derive_session_keys(&shared, salt).unwrap();

        // Same inputs produce same outputs
        assert_eq!(send1.material, send2.material);
        assert_eq!(recv1.material, recv2.material);
    }

    #[test]
    fn derive_session_keys_send_recv_differ() {
        let shared = SharedSecret([7u8; 32]);
        let salt = b"session";

        let (send, recv) = derive_session_keys(&shared, salt).unwrap();

        // Send and receive keys must be different
        assert_ne!(send.material, recv.material);
    }

    #[test]
    fn derive_session_keys_different_salt() {
        let shared = SharedSecret([99u8; 32]);

        let (send1, _) = derive_session_keys(&shared, b"salt-a").unwrap();
        let (send2, _) = derive_session_keys(&shared, b"salt-b").unwrap();

        // Different salts produce different keys
        assert_ne!(send1.material, send2.material);
    }

    #[test]
    fn invalid_public_key_length() {
        let secret = SecretKey([1u8; 32]);
        let bad_public = PublicKey(vec![0u8; 16]); // wrong length
        let result = derive_shared_secret(&secret, &bad_public);
        assert!(result.is_err());
    }

    #[test]
    fn rotate_nonce_sequential() {
        let mut counter = 0u64;
        let n0 = rotate_nonce(&mut counter).unwrap();
        let n1 = rotate_nonce(&mut counter).unwrap();
        assert_eq!(counter, 2);
        assert_ne!(n0, n1);
        // First nonce should have 0 in first 8 bytes (LE)
        assert_eq!(&n0[..8], &0u64.to_le_bytes());
        assert_eq!(&n1[..8], &1u64.to_le_bytes());
    }

    #[test]
    fn rotate_nonce_overflow() {
        let mut counter = u64::MAX;
        let result = rotate_nonce(&mut counter);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::NonceOverflow));
    }

    #[test]
    fn rotate_nonce_uniqueness_large_range() {
        // Verify uniqueness over 100k sequential nonces
        let mut counter = 0u64;
        let mut seen = std::collections::HashSet::new();
        for _ in 0..100_000 {
            let nonce = rotate_nonce(&mut counter).unwrap();
            assert!(seen.insert(nonce), "nonce collision detected");
        }
        assert_eq!(seen.len(), 100_000);
    }

    #[test]
    fn public_key_from_bytes() {
        let bytes = [42u8; 32];
        let pk = PublicKey::from_bytes(&bytes).unwrap();
        assert_eq!(pk.as_bytes(), &bytes);

        let bad = PublicKey::from_bytes(&[0u8; 16]);
        assert!(bad.is_err());
    }
}
