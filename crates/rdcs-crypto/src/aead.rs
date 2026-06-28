// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Authenticated Encryption with Associated Data (AEAD) using XSalsa20-Poly1305.
//!
//! This module implements XSalsa20-Poly1305 with AAD support, built on top of
//! the `salsa20` and `poly1305` crates from RustCrypto. The construction
//! follows the NaCl `crypto_secretbox` pattern extended with AAD authentication
//! using the RFC 8439 MAC construction:
//!
//! ```text
//! tag = Poly1305(key, pad16(aad) || pad16(ct) || le64(|aad|) || le64(|ct|))
//! ```
//!
//! XSalsa20 provides the stream cipher with 24-byte extended nonces, and
//! Poly1305 provides the one-time authenticator. The Poly1305 key is derived
//! from the first 32 bytes of the XSalsa20 keystream.

use poly1305::universal_hash::{KeyInit, UniversalHash};
use poly1305::Poly1305;
use salsa20::cipher::{KeyIvInit, StreamCipher};
use salsa20::XSalsa20;
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

use crate::error::CryptoError;

/// Nonce size in bytes for XSalsa20-Poly1305.
pub const NONCE_SIZE: usize = 24;

/// Poly1305 authentication tag size in bytes.
pub const TAG_SIZE: usize = 16;

/// Poly1305 key size in bytes (derived from XSalsa20 keystream).
const POLY_KEY_SIZE: usize = 32;

/// Encrypt `plaintext` with XSalsa20-Poly1305.
///
/// The 32-byte `key` and 24-byte `nonce` must never be reused together.
/// Additional authenticated data (`aad`) is authenticated but not encrypted.
/// Returns ciphertext with the 16-byte Poly1305 tag appended.
pub fn seal(
    key: &[u8; 32],
    nonce: &[u8; NONCE_SIZE],
    aad: &[u8],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let mut cipher = XSalsa20::new(key.into(), nonce.into());

    // Derive Poly1305 key from the first 32 bytes of keystream
    let mut poly_key = [0u8; POLY_KEY_SIZE];
    cipher.apply_keystream(&mut poly_key);

    // Encrypt plaintext in-place
    let mut ciphertext = plaintext.to_vec();
    cipher.apply_keystream(&mut ciphertext);

    // Compute Poly1305 tag over AAD and ciphertext (RFC 8439 construction)
    let tag = compute_tag(&poly_key, aad, &ciphertext);

    // Zeroize the Poly1305 key
    let mut pk = poly_key;
    pk.zeroize();

    // Append tag to ciphertext
    let mut result = ciphertext;
    result.extend_from_slice(tag.as_slice());
    Ok(result)
}

/// Decrypt `ciphertext` (with appended Poly1305 tag) using XSalsa20-Poly1305.
///
/// `aad` must match the value used during encryption. Returns an error if
/// the authentication tag does not verify (wrong key, tampered ciphertext,
/// or mismatched AAD).
pub fn open(
    key: &[u8; 32],
    nonce: &[u8; NONCE_SIZE],
    aad: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    if ciphertext.len() < TAG_SIZE {
        return Err(CryptoError::InvalidCiphertext);
    }

    let (ct_body, tag_bytes) = ciphertext.split_at(ciphertext.len() - TAG_SIZE);

    let mut cipher = XSalsa20::new(key.into(), nonce.into());

    // Derive Poly1305 key from the first 32 bytes of keystream
    let mut poly_key = [0u8; POLY_KEY_SIZE];
    cipher.apply_keystream(&mut poly_key);

    // Verify tag before decrypting (authenticate-then-decrypt)
    let expected_tag = compute_tag(&poly_key, aad, ct_body);

    // Zeroize the Poly1305 key
    let mut pk = poly_key;
    pk.zeroize();

    // Constant-time tag comparison
    let tag_arr = poly1305::Tag::from_slice(tag_bytes);
    if !bool::from(expected_tag.ct_eq(tag_arr)) {
        return Err(CryptoError::DecryptionFailed);
    }

    // Decrypt ciphertext
    let mut plaintext = ct_body.to_vec();
    cipher.apply_keystream(&mut plaintext);

    Ok(plaintext)
}

/// Compute Poly1305 tag using the RFC 8439 construction:
///
/// ```text
/// MAC_data = pad16(aad) || pad16(ciphertext) || le64(|aad|) || le64(|ct|)
/// tag = Poly1305(key, MAC_data)
/// ```
fn compute_tag(
    poly_key: &[u8; POLY_KEY_SIZE],
    aad: &[u8],
    ciphertext: &[u8],
) -> poly1305::Tag {
    let mut mac = Poly1305::new(poly_key.into());

    // Update with AAD (padded to 16-byte boundary)
    mac.update_padded(aad);

    // Update with ciphertext (padded to 16-byte boundary)
    mac.update_padded(ciphertext);

    // Append lengths as a single 16-byte block: le64(aad_len) || le64(ct_len)
    let mut len_block = poly1305::Block::default();
    len_block[..8].copy_from_slice(&(aad.len() as u64).to_le_bytes());
    len_block[8..16].copy_from_slice(&(ciphertext.len() as u64).to_le_bytes());
    mac.update(std::slice::from_ref(&len_block));

    mac.finalize()
}

/// Encrypt convenience wrapper that takes a [`SessionKey`](crate::key_exchange::SessionKey).
pub fn encrypt(
    key: &crate::key_exchange::SessionKey,
    nonce: &[u8; NONCE_SIZE],
    plaintext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    seal(&key.material, nonce, aad, plaintext)
}

/// Decrypt convenience wrapper that takes a [`SessionKey`](crate::key_exchange::SessionKey).
pub fn decrypt(
    key: &crate::key_exchange::SessionKey,
    nonce: &[u8; NONCE_SIZE],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    open(&key.material, nonce, aad, ciphertext)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngCore;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let key = [1u8; 32];
        let nonce = [2u8; NONCE_SIZE];
        let aad = b"associated data";
        let plaintext = b"hello, remote desktop";

        let ct = seal(&key, &nonce, aad, plaintext).expect("seal should succeed");
        let pt = open(&key, &nonce, aad, &ct).expect("open should succeed");
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn ciphertext_longer_than_plaintext() {
        let key = [1u8; 32];
        let nonce = [2u8; NONCE_SIZE];
        let plaintext = b"short";

        let ct = seal(&key, &nonce, b"", plaintext).expect("seal should succeed");
        // Ciphertext must include the 16-byte tag
        assert_eq!(ct.len(), plaintext.len() + TAG_SIZE);
    }

    #[test]
    fn reject_short_ciphertext() {
        let key = [1u8; 32];
        let nonce = [2u8; NONCE_SIZE];
        let result = open(&key, &nonce, b"", &[0u8; 5]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CryptoError::InvalidCiphertext));
    }

    #[test]
    fn wrong_key_rejected() {
        let correct_key = [1u8; 32];
        let wrong_key = [2u8; 32];
        let nonce = [3u8; NONCE_SIZE];
        let aad = b"test";
        let plaintext = b"secret data";

        let ct = seal(&correct_key, &nonce, aad, plaintext).expect("seal should succeed");
        let result = open(&wrong_key, &nonce, aad, &ct);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CryptoError::DecryptionFailed
        ));
    }

    #[test]
    fn wrong_aad_rejected() {
        let key = [5u8; 32];
        let nonce = [6u8; NONCE_SIZE];
        let plaintext = b"authenticated data test";

        let ct = seal(&key, &nonce, b"correct aad", plaintext).expect("seal should succeed");
        let result = open(&key, &nonce, b"wrong aad", &ct);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CryptoError::DecryptionFailed
        ));
    }

    #[test]
    fn tampered_ciphertext_rejected() {
        let key = [10u8; 32];
        let nonce = [11u8; NONCE_SIZE];
        let plaintext = b"do not tamper";

        let mut ct = seal(&key, &nonce, b"", plaintext).expect("seal should succeed");
        // Flip a bit in the ciphertext body
        ct[0] ^= 0x01;
        let result = open(&key, &nonce, b"", &ct);
        assert!(result.is_err());
    }

    #[test]
    fn tampered_tag_rejected() {
        let key = [12u8; 32];
        let nonce = [13u8; NONCE_SIZE];
        let plaintext = b"tag integrity";

        let mut ct = seal(&key, &nonce, b"aad", plaintext).expect("seal should succeed");
        // Flip a bit in the tag (last 16 bytes)
        let last = ct.len() - 1;
        ct[last] ^= 0x01;
        let result = open(&key, &nonce, b"aad", &ct);
        assert!(result.is_err());
    }

    #[test]
    fn wrong_nonce_rejected() {
        let key = [20u8; 32];
        let nonce1 = [21u8; NONCE_SIZE];
        let nonce2 = [22u8; NONCE_SIZE];
        let plaintext = b"nonce matters";

        let ct = seal(&key, &nonce1, b"", plaintext).expect("seal should succeed");
        let result = open(&key, &nonce2, b"", &ct);
        assert!(result.is_err());
    }

    #[test]
    fn empty_plaintext() {
        let key = [30u8; 32];
        let nonce = [31u8; NONCE_SIZE];
        let aad = b"only aad";

        let ct = seal(&key, &nonce, aad, b"").expect("seal should succeed for empty plaintext");
        // Should still produce a tag
        assert_eq!(ct.len(), TAG_SIZE);
        let pt = open(&key, &nonce, aad, &ct).expect("open should succeed");
        assert!(pt.is_empty());
    }

    #[test]
    fn empty_aad() {
        let key = [40u8; 32];
        let nonce = [41u8; NONCE_SIZE];
        let plaintext = b"no aad";

        let ct = seal(&key, &nonce, b"", plaintext).expect("seal should succeed");
        let pt = open(&key, &nonce, b"", &ct).expect("open should succeed");
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn large_aad() {
        let key = [50u8; 32];
        let nonce = [51u8; NONCE_SIZE];
        let aad = vec![0xABu8; 10_000];
        let plaintext = b"large aad test";

        let ct = seal(&key, &nonce, &aad, plaintext).expect("seal should succeed");
        let pt = open(&key, &nonce, &aad, &ct).expect("open should succeed");
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn large_plaintext() {
        let key = [60u8; 32];
        let nonce = [61u8; NONCE_SIZE];
        let mut plaintext = vec![0u8; 64 * 1024]; // 64 KB
        rand::thread_rng().fill_bytes(&mut plaintext);

        let ct = seal(&key, &nonce, b"aad", &plaintext).expect("seal should succeed");
        let pt = open(&key, &nonce, b"aad", &ct).expect("open should succeed");
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn roundtrip_1000_random_messages() {
        let mut rng = rand::thread_rng();
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);

        for i in 0u64..1000 {
            // Generate random nonce
            let mut nonce = [0u8; NONCE_SIZE];
            rng.fill_bytes(&mut nonce);

            // Generate random plaintext of varying size (0..4096 bytes)
            let len = (i as usize * 37) % 4097;
            let mut plaintext = vec![0u8; len];
            rng.fill_bytes(&mut plaintext);

            let aad = format!("message-{i}");
            let ct = seal(&key, &nonce, aad.as_bytes(), &plaintext).expect("seal");
            let pt = open(&key, &nonce, aad.as_bytes(), &ct).expect("open");
            assert_eq!(pt, plaintext, "roundtrip mismatch at iteration {i}");
        }
    }

    #[test]
    fn session_key_encrypt_decrypt() {
        let sk = crate::key_exchange::SessionKey {
            material: [42u8; 32],
        };
        let nonce = [7u8; NONCE_SIZE];
        let plaintext = b"session key wrapper test";
        let aad = b"aad";

        let ct = encrypt(&sk, &nonce, plaintext, aad).expect("encrypt");
        let pt = decrypt(&sk, &nonce, &ct, aad).expect("decrypt");
        assert_eq!(pt, plaintext);
    }

    #[test]
    fn different_keys_produce_different_ciphertext() {
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];
        let nonce = [3u8; NONCE_SIZE];
        let plaintext = b"same plaintext";

        let ct1 = seal(&key1, &nonce, b"", plaintext).unwrap();
        let ct2 = seal(&key2, &nonce, b"", plaintext).unwrap();

        // Different keys should produce different ciphertexts (excluding tag)
        assert_ne!(&ct1[..ct1.len() - TAG_SIZE], &ct2[..ct2.len() - TAG_SIZE]);
    }

    #[test]
    fn different_nonces_produce_different_ciphertext() {
        let key = [1u8; 32];
        let nonce1 = [2u8; NONCE_SIZE];
        let nonce2 = [3u8; NONCE_SIZE];
        let plaintext = b"same plaintext";

        let ct1 = seal(&key, &nonce1, b"", plaintext).unwrap();
        let ct2 = seal(&key, &nonce2, b"", plaintext).unwrap();

        assert_ne!(&ct1[..ct1.len() - TAG_SIZE], &ct2[..ct2.len() - TAG_SIZE]);
    }
}
