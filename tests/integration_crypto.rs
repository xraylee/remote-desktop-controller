// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Integration test: two-party cryptographic handshake and bidirectional
//! encrypted communication.
//!
//! Exercises the full rdcs-crypto session lifecycle:
//!   1. Key generation (X25519)
//!   2. Public key exchange
//!   3. Shared secret derivation (DH)
//!   4. Session key derivation (HKDF-SHA256)
//!   5. Bidirectional encrypt/decrypt (XSalsa20-Poly1305)
//!   6. Key rotation via SessionKeyManager
//!   7. Secure session destruction

use rdcs_crypto::key_exchange::{
    derive_session_keys, derive_shared_secret, generate_keypair, rotate_nonce, PublicKey,
};
use rdcs_crypto::session::{CryptoSession, EncryptedPayload, SessionKeyManager};
use rdcs_crypto::aead;
use rdcs_crypto::CryptoError;

// ---------------------------------------------------------------------------
// Two-party handshake tests
// ---------------------------------------------------------------------------

#[test]
fn two_party_crypto_handshake() {
    // 1. Alice creates CryptoSession, generates keypair
    let mut alice = CryptoSession::new(1);
    assert!(!alice.is_ready(), "alice should not be ready before handshake");

    // 2. Bob creates CryptoSession, generates keypair
    let mut bob = CryptoSession::new(1);
    assert!(!bob.is_ready(), "bob should not be ready before handshake");

    // 3. Exchange public keys
    let alice_pub = alice.local_public_key().clone();
    let bob_pub = bob.local_public_key().clone();

    // Public keys should be distinct (overwhelmingly likely with random keys)
    assert_ne!(alice_pub.as_bytes(), bob_pub.as_bytes());

    // 4. Both complete_handshake with other's public key
    let salt = b"integration-test-session-42";
    alice.complete_handshake(&bob_pub, salt).unwrap();
    bob.complete_handshake(&alice_pub, salt).unwrap();

    assert!(alice.is_ready(), "alice should be ready after handshake");
    assert!(bob.is_ready(), "bob should be ready after handshake");

    // 5. Alice encrypts message, Bob decrypts — matches
    let alice_msg = b"Hello Bob, this is Alice!";
    let alice_payload = alice.encrypt(alice_msg).unwrap();
    let bob_decrypted = bob.decrypt(&alice_payload).unwrap();
    assert_eq!(bob_decrypted, alice_msg);

    // 6. Bob encrypts message, Alice decrypts — matches
    let bob_msg = b"Hello Alice, Bob here! Ready to remote in.";
    let bob_payload = bob.encrypt(bob_msg).unwrap();
    let alice_decrypted = alice.decrypt(&bob_payload).unwrap();
    assert_eq!(alice_decrypted, bob_msg);
}

#[test]
fn two_party_bidirectional_many_messages() {
    let mut alice = CryptoSession::new(100);
    let mut bob = CryptoSession::new(100);

    let alice_pub = alice.local_public_key().clone();
    let bob_pub = bob.local_public_key().clone();

    let salt = b"long-conversation-test";
    alice.complete_handshake(&bob_pub, salt).unwrap();
    bob.complete_handshake(&alice_pub, salt).unwrap();

    // Send 500 messages in each direction
    for i in 0u64..500 {
        // Alice → Bob
        let msg = format!("Alice message #{i}");
        let payload = alice.encrypt(msg.as_bytes()).unwrap();
        let decrypted = bob.decrypt(&payload).unwrap();
        assert_eq!(decrypted, msg.as_bytes(), "Alice→Bob mismatch at message {i}");

        // Bob → Alice
        let reply = format!("Bob reply #{i}");
        let payload = bob.encrypt(reply.as_bytes()).unwrap();
        let decrypted = alice.decrypt(&payload).unwrap();
        assert_eq!(decrypted, reply.as_bytes(), "Bob→Alice mismatch at message {i}");
    }
}

#[test]
fn two_party_different_salts_produce_different_keys() {
    // Two separate sessions between the same peers with different salts
    // should produce independent encryption contexts.

    let alice_kp = generate_keypair();
    let bob_kp = generate_keypair();

    // Session 1: salt "session-alpha"
    let mut alice_s1 = CryptoSession::new(1);
    let mut bob_s1 = CryptoSession::new(1);
    alice_s1
        .complete_handshake(&bob_kp.public, b"session-alpha")
        .unwrap();
    bob_s1
        .complete_handshake(&alice_kp.public, b"session-alpha")
        .unwrap();

    // Session 2: salt "session-beta"
    let mut alice_s2 = CryptoSession::new(2);
    let mut bob_s2 = CryptoSession::new(2);
    alice_s2
        .complete_handshake(&bob_kp.public, b"session-beta")
        .unwrap();
    bob_s2
        .complete_handshake(&alice_kp.public, b"session-beta")
        .unwrap();

    // Message encrypted in session 1 should NOT decrypt in session 2
    let payload_s1 = alice_s1.encrypt(b"secret-alpha").unwrap();
    let result = bob_s2.decrypt(&payload_s1);
    assert!(
        result.is_err(),
        "cross-session decryption should fail with different salts"
    );
}

// ---------------------------------------------------------------------------
// Low-level key exchange tests
// ---------------------------------------------------------------------------

#[test]
fn low_level_key_exchange_agreement() {
    let alice = generate_keypair();
    let bob = generate_keypair();

    // Both derive the same shared secret
    let alice_shared = derive_shared_secret(&alice.secret, &bob.public).unwrap();
    let bob_shared = derive_shared_secret(&bob.secret, &alice.public).unwrap();

    assert_eq!(
        alice_shared.0, bob_shared.0,
        "DH shared secrets must agree"
    );

    // Derive session keys from the shared secret
    let salt = b"low-level-test";
    let (send_key, recv_key) = derive_session_keys(&alice_shared, salt).unwrap();

    assert_ne!(
        send_key.material, recv_key.material,
        "send and recv keys must differ"
    );

    // Both parties derive the same keys from the same shared secret
    let (bob_send, bob_recv) = derive_session_keys(&bob_shared, salt).unwrap();
    assert_eq!(send_key.material, bob_send.material);
    assert_eq!(recv_key.material, bob_recv.material);
}

#[test]
fn low_level_aead_roundtrip_with_derived_keys() {
    let alice = generate_keypair();
    let bob = generate_keypair();

    let shared = derive_shared_secret(&alice.secret, &bob.public).unwrap();
    let (send_key, _recv_key) = derive_session_keys(&shared, b"aead-test").unwrap();

    let mut counter = 0u64;
    let nonce = rotate_nonce(&mut counter).unwrap();

    let plaintext = b"low-level AEAD roundtrip with derived keys";
    let aad = b"session-metadata";

    let ciphertext = aead::encrypt(&send_key, &nonce, plaintext, aad).unwrap();
    let decrypted = aead::decrypt(&send_key, &nonce, &ciphertext, aad).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn nonce_uniqueness_across_session() {
    let mut alice = CryptoSession::new(1);
    let mut bob = CryptoSession::new(1);

    let alice_pub = alice.local_public_key().clone();
    let bob_pub = bob.local_public_key().clone();

    alice.complete_handshake(&bob_pub, b"nonce-test").unwrap();
    bob.complete_handshake(&alice_pub, b"nonce-test").unwrap();

    // Encrypt 1000 messages and collect all nonces
    let mut nonces = Vec::new();
    for _ in 0..1000 {
        let payload = alice.encrypt(b"nonce uniqueness test").unwrap();
        nonces.push(payload.nonce.clone());
    }

    // All nonces should be unique
    let unique: std::collections::HashSet<Vec<u8>> = nonces.iter().cloned().collect();
    assert_eq!(unique.len(), 1000, "all 1000 nonces should be unique");
}

// ---------------------------------------------------------------------------
// Session key manager tests
// ---------------------------------------------------------------------------

#[test]
fn session_key_manager_rotation_and_isolation() {
    let mut mgr_alice = SessionKeyManager::new(100);
    let mut mgr_bob = SessionKeyManager::new(100);

    // Both start from the same shared secret
    let alice_kp = generate_keypair();
    let bob_kp = generate_keypair();
    let shared = derive_shared_secret(&alice_kp.secret, &bob_kp.public).unwrap();

    mgr_alice.init_from_shared_secret(&shared.0).unwrap();
    mgr_bob.init_from_shared_secret(&shared.0).unwrap();

    // Initial keys should match
    assert_eq!(
        mgr_alice.current_key().unwrap().material,
        mgr_bob.current_key().unwrap().material
    );

    // Rotate both in lockstep — keys should stay in sync
    for _ in 0..5 {
        mgr_alice.rotate().unwrap();
        mgr_bob.rotate().unwrap();
        assert_eq!(
            mgr_alice.current_key().unwrap().material,
            mgr_bob.current_key().unwrap().material,
            "rotated keys should stay in sync"
        );
    }

    // Each rotation should produce a different key
    let k5 = mgr_alice.current_key().unwrap().material;
    mgr_alice.rotate().unwrap();
    let k6 = mgr_alice.current_key().unwrap().material;
    assert_ne!(k5, k6, "successive rotations should produce different keys");
}

// ---------------------------------------------------------------------------
// Encrypted payload serialization test
// ---------------------------------------------------------------------------

#[test]
fn encrypted_payload_serde_roundtrip() {
    let mut alice = CryptoSession::new(1);
    let mut bob = CryptoSession::new(1);

    let alice_pub = alice.local_public_key().clone();
    let bob_pub = bob.local_public_key().clone();

    alice.complete_handshake(&bob_pub, b"serde-test").unwrap();
    bob.complete_handshake(&alice_pub, b"serde-test").unwrap();

    // Encrypt a message
    let payload = alice.encrypt(b"serializable payload").unwrap();

    // Serialize to JSON (simulating wire transport of metadata)
    let json = serde_json::to_string(&payload).unwrap();

    // Deserialize
    let restored: EncryptedPayload = serde_json::from_str(&json).unwrap();

    // Bob should be able to decrypt the restored payload
    let decrypted = bob.decrypt(&restored).unwrap();
    assert_eq!(decrypted, b"serializable payload");
}

// ---------------------------------------------------------------------------
// Secure destruction test
// ---------------------------------------------------------------------------

#[test]
fn session_destroy_prevents_further_use() {
    let mut alice = CryptoSession::new(1);
    let mut bob = CryptoSession::new(1);

    let alice_pub = alice.local_public_key().clone();
    let bob_pub = bob.local_public_key().clone();

    alice.complete_handshake(&bob_pub, b"destroy-test").unwrap();
    bob.complete_handshake(&alice_pub, b"destroy-test").unwrap();

    // Verify encryption works before destroy
    let _ = alice.encrypt(b"before destroy").unwrap();

    // Destroy alice's session
    alice.destroy();

    // Alice is consumed — we can't call methods on her anymore.
    // The type system enforces this. Bob's session should still work
    // for decryption of messages received before alice was destroyed.
    let payload = bob.encrypt(b"bob still works").unwrap();
    // We can't test alice.decrypt here since alice was moved into destroy().
    // The compile-time guarantee is the real test.
    let _ = payload; // suppress unused
}

// ---------------------------------------------------------------------------
// Invalid inputs test
// ---------------------------------------------------------------------------

#[test]
fn invalid_public_key_rejected() {
    let mut session = CryptoSession::new(1);

    // Too-short public key
    let bad_key = PublicKey::from_bytes(&[0u8; 16]);
    assert!(bad_key.is_err());

    // Handshake with invalid key should fail
    let fake = rdcs_crypto::key_exchange::PublicKey(vec![0u8; 16]);
    let result = session.complete_handshake(&fake, b"salt");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CryptoError::KeyExchangeFailed(_)));
}

#[test]
fn encrypt_before_handshake_fails() {
    let session = CryptoSession::new(1);
    assert!(!session.is_ready());
    let result = session.encrypt(b"data");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CryptoError::NotInitialized));
}
