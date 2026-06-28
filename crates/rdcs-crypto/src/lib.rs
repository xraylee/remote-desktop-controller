// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! rdcs-crypto: Encryption layer for the RDCS remote desktop system.
//!
//! Provides X25519 key exchange, XSalsa20-Poly1305 AEAD encryption,
//! and session key lifecycle management. All secret material is zeroed
//! on drop via the `zeroize` crate.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use rdcs_crypto::session::CryptoSession;
//!
//! // Create sessions for both peers
//! let mut alice = CryptoSession::new(1);
//! let mut bob = CryptoSession::new(1);
//!
//! // Exchange public keys (out-of-band)
//! let alice_pub = alice.local_public_key().clone();
//! let bob_pub = bob.local_public_key().clone();
//!
//! // Complete handshake with shared salt
//! let salt = b"session-42-2026-01-01";
//! alice.complete_handshake(&bob_pub, salt).unwrap();
//! bob.complete_handshake(&alice_pub, salt).unwrap();
//!
//! // Encrypt and send
//! let payload = alice.encrypt(b"hello").unwrap();
//! let plaintext = bob.decrypt(&payload).unwrap();
//! assert_eq!(plaintext, b"hello");
//!
//! // Cleanup
//! alice.destroy();
//! bob.destroy();
//! ```

pub mod aead;
pub mod error;
pub mod key_exchange;
pub mod session;

// Re-export primary types at crate root for convenience.
pub use error::CryptoError;
pub use key_exchange::{
    derive_session_keys, derive_shared_secret, generate_keypair, rotate_nonce, KeyPair, PublicKey,
    SessionKey, SharedSecret,
};
pub use session::{CryptoSession, EncryptedPayload};
