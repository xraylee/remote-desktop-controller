// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Error types for cryptographic operations.

use thiserror::Error;

/// Errors that can occur during cryptographic operations.
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Key exchange failed (e.g. invalid public key).
    #[error("key exchange failed: {0}")]
    KeyExchangeFailed(String),

    /// AEAD decryption failed (authentication tag mismatch).
    #[error("authentication failed")]
    AuthenticationFailed,

    /// Decryption failed — wrong key, corrupted ciphertext, or mismatched AAD.
    #[error("decryption failed")]
    DecryptionFailed,

    /// Ciphertext is too short to contain a valid tag.
    #[error("invalid ciphertext")]
    InvalidCiphertext,

    /// Session key has not been initialized.
    #[error("session key not initialized")]
    NotInitialized,

    /// Key derivation failed.
    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),

    /// Nonce counter has been exhausted (2^64 nonces used).
    #[error("nonce counter overflow")]
    NonceOverflow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        let err = CryptoError::AuthenticationFailed;
        assert_eq!(err.to_string(), "authentication failed");

        let err = CryptoError::DecryptionFailed;
        assert_eq!(err.to_string(), "decryption failed");

        let err = CryptoError::NonceOverflow;
        assert_eq!(err.to_string(), "nonce counter overflow");
    }
}
