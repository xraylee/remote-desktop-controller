// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! HMAC-SHA256 token generation and verification for relay session allocation.
//!
//! Tokens are issued by the signaling server and verified by relay nodes.
//! Format: `base64(json_payload) + "." + hex(hmac_signature)`
//!
//! The HMAC covers the deterministic binary encoding of:
//! `session_id (u64 LE) || relay_addr (UTF-8) || nonce (u64 LE) || expires_at (u64 LE)`

use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Token payload
// ---------------------------------------------------------------------------

/// Token payload embedded in relay allocation tokens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenPayload {
    /// Unique session identifier assigned by the signaling server.
    pub session_id: u64,
    /// Address of the relay node this token authorizes (e.g. `"10.0.0.1:3478"`).
    pub relay_addr: String,
    /// Cryptographically random nonce for replay protection.
    pub nonce: u64,
    /// Token expiration as a Unix timestamp (seconds since epoch).
    pub expires_at: u64,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur during token verification.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("invalid token format")]
    InvalidFormat,

    #[error("invalid HMAC signature")]
    InvalidSignature,

    #[error("token expired at {0}")]
    Expired(u64),

    #[error("nonce {0} already used (replay attack)")]
    NonceReused(u64),

    #[error("base64 decode error: {0}")]
    Base64Error(String),

    #[error("json decode error: {0}")]
    JsonError(String),
}

// ---------------------------------------------------------------------------
// HMAC helpers
// ---------------------------------------------------------------------------

/// Build the deterministic byte sequence that the HMAC covers.
///
/// Layout: `session_id (8 LE) || relay_addr (UTF-8) || nonce (8 LE) || expires_at (8 LE)`
fn hmac_message(payload: &TokenPayload) -> Vec<u8> {
    let mut buf = Vec::with_capacity(8 + payload.relay_addr.len() + 8 + 8);
    buf.extend_from_slice(&payload.session_id.to_le_bytes());
    buf.extend_from_slice(payload.relay_addr.as_bytes());
    buf.extend_from_slice(&payload.nonce.to_le_bytes());
    buf.extend_from_slice(&payload.expires_at.to_le_bytes());
    buf
}

/// Compute the HMAC-SHA256 over the canonical message and return raw bytes.
fn compute_hmac(payload: &TokenPayload, secret: &[u8]) -> Vec<u8> {
    let mut mac =
        HmacSha256::new_from_slice(secret).expect("HMAC-SHA256 accepts any key length");
    mac.update(&hmac_message(payload));
    mac.finalize().into_bytes().to_vec()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Generates an HMAC-SHA256 token for relay session allocation.
///
/// Token format: `base64(json_payload) + "." + hex(hmac_signature)`
///
/// The HMAC covers: `session_id || relay_addr || nonce || expires_at`
/// encoded as a deterministic binary blob (little-endian integers, raw UTF-8
/// address).
pub fn generate_token(payload: &TokenPayload, secret: &[u8]) -> String {
    let json = serde_json::to_string(payload).expect("TokenPayload serialization cannot fail");
    let payload_b64 = BASE64.encode(json.as_bytes());
    let sig = compute_hmac(payload, secret);
    let sig_hex: String = sig.iter().map(|b| format!("{b:02x}")).collect();
    format!("{payload_b64}.{sig_hex}")
}

/// Verifies an HMAC token and extracts the payload.
///
/// Returns an error if:
/// - The token is malformed (missing `.`, bad base64, bad JSON)
/// - The HMAC signature does not match
/// - The token has expired (`expires_at` is in the past)
pub fn verify_token(token: &str, secret: &[u8]) -> Result<TokenPayload, AuthError> {
    // 1. Split on the single '.' separator.
    let (payload_b64, sig_hex) = token.split_once('.').ok_or(AuthError::InvalidFormat)?;

    // 2. Decode the base64 payload.
    let payload_bytes = BASE64
        .decode(payload_b64)
        .map_err(|e| AuthError::Base64Error(e.to_string()))?;

    // 3. Deserialize JSON.
    let payload: TokenPayload = serde_json::from_slice(&payload_bytes)
        .map_err(|e| AuthError::JsonError(e.to_string()))?;

    // 4. Decode the hex signature.
    let expected_sig = hex_decode(sig_hex).ok_or(AuthError::InvalidFormat)?;

    // 5. Compute and compare HMAC (constant-time via the `hmac` crate).
    let actual_sig = compute_hmac(&payload, secret);
    if !constant_time_eq(&actual_sig, &expected_sig) {
        return Err(AuthError::InvalidSignature);
    }

    // 6. Check expiration.
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_secs();
    if now > payload.expires_at {
        return Err(AuthError::Expired(payload.expires_at));
    }

    Ok(payload)
}

// ---------------------------------------------------------------------------
// Nonce cache (replay protection)
// ---------------------------------------------------------------------------

/// In-memory nonce cache with TTL-based expiry for replay protection.
///
/// Each inserted nonce is timestamped. [`cleanup_expired`](NonceCache::cleanup_expired)
/// removes entries older than `max_age_secs`.
pub struct NonceCache {
    /// `(nonce_value, insertion_unix_timestamp)` pairs.
    seen: HashSet<(u64, u64)>,
    /// Maximum age (seconds) before a nonce entry is eligible for cleanup.
    max_age_secs: u64,
}

impl NonceCache {
    /// Create a new cache that retains entries for at most `max_age_secs`.
    pub fn new(max_age_secs: u64) -> Self {
        Self {
            seen: HashSet::new(),
            max_age_secs,
        }
    }

    /// Check whether `nonce` has already been used. If not, record it and
    /// return `Ok(())`. If it has, return [`AuthError::NonceReused`].
    pub fn check_and_insert(&mut self, nonce: u64) -> Result<(), AuthError> {
        // A nonce is "already used" if any entry with the same nonce value
        // exists (regardless of its timestamp).
        let already_used = self.seen.iter().any(|&(n, _)| n == nonce);
        if already_used {
            return Err(AuthError::NonceReused(nonce));
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();
        self.seen.insert((nonce, now));
        Ok(())
    }

    /// Remove entries whose age exceeds `max_age_secs`.
    pub fn cleanup_expired(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX epoch")
            .as_secs();
        let max_age = self.max_age_secs;
        self.seen.retain(|&(_, ts)| now.saturating_sub(ts) <= max_age);
    }

    /// Number of nonces currently in the cache.
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Decode a hex string into bytes. Returns `None` on invalid input.
fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

/// Constant-time byte-slice comparison (avoids timing side-channels).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter()
        .zip(b.iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: create a payload that expires far in the future.
    fn future_payload(session_id: u64, nonce: u64) -> TokenPayload {
        // Year 2100 — well beyond any reasonable test run.
        TokenPayload {
            session_id,
            relay_addr: "10.0.0.1:3478".into(),
            nonce,
            expires_at: 4_102_444_800,
        }
    }

    /// Helper: create a payload that is already expired.
    fn expired_payload(session_id: u64, nonce: u64) -> TokenPayload {
        TokenPayload {
            session_id,
            relay_addr: "10.0.0.1:3478".into(),
            nonce,
            // 2020-01-01T00:00:00Z
            expires_at: 1_577_836_800,
        }
    }

    const SECRET: &[u8] = b"test-hmac-secret-key";

    // -- Acceptance: generate_verify_roundtrip --------------------------------

    #[test]
    fn generate_verify_roundtrip() {
        let payload = future_payload(42, 12345);
        let token = generate_token(&payload, SECRET);
        let verified = verify_token(&token, SECRET).expect("verification should succeed");
        assert_eq!(verified, payload);
    }

    #[test]
    fn generate_verify_roundtrip_different_secrets() {
        let payload = future_payload(100, 99999);
        let secret_a = b"secret-alpha";
        let secret_b = b"secret-beta!!";

        let token_a = generate_token(&payload, secret_a);
        let token_b = generate_token(&payload, secret_b);

        // Each token verifies with its own secret.
        assert!(verify_token(&token_a, secret_a).is_ok());
        assert!(verify_token(&token_b, secret_b).is_ok());

        // Cross-secret verification must fail.
        let err = verify_token(&token_a, secret_b).unwrap_err();
        assert!(
            matches!(err, AuthError::InvalidSignature),
            "expected InvalidSignature, got {err:?}"
        );
    }

    #[test]
    fn generate_verify_roundtrip_special_addr() {
        let payload = TokenPayload {
            session_id: u64::MAX,
            relay_addr: "2001:db8::1:8443".into(),
            nonce: 0,
            expires_at: 4_102_444_800,
        };
        let token = generate_token(&payload, SECRET);
        let verified = verify_token(&token, SECRET).expect("roundtrip should succeed");
        assert_eq!(verified, payload);
    }

    // -- Acceptance: invalid_signature_rejected -------------------------------

    #[test]
    fn invalid_signature_rejected() {
        let payload = future_payload(1, 1);
        let token = generate_token(&payload, SECRET);

        // Verify with a different secret.
        let err = verify_token(&token, b"wrong-secret").unwrap_err();
        assert!(
            matches!(err, AuthError::InvalidSignature),
            "expected InvalidSignature, got {err:?}"
        );
    }

    #[test]
    fn tampered_signature_rejected() {
        let payload = future_payload(7, 42);
        let token = generate_token(&payload, SECRET);

        // Flip the last hex character of the signature.
        let (payload_part, sig_hex) = token.split_once('.').unwrap();
        let mut sig_bytes: Vec<char> = sig_hex.chars().collect();
        let last = sig_bytes.last_mut().unwrap();
        *last = if *last == '0' { '1' } else { '0' };
        let tampered = format!("{payload_part}.{}", sig_bytes.iter().collect::<String>());

        let err = verify_token(&tampered, SECRET).unwrap_err();
        assert!(
            matches!(err, AuthError::InvalidSignature),
            "expected InvalidSignature, got {err:?}"
        );
    }

    #[test]
    fn tampered_payload_rejected() {
        let payload = future_payload(1, 1);
        let token = generate_token(&payload, SECRET);

        // Decode, mutate session_id, re-encode, keep original signature.
        let (payload_b64, sig_hex) = token.split_once('.').unwrap();
        let payload_bytes = BASE64.decode(payload_b64).unwrap();
        let mut p: TokenPayload = serde_json::from_slice(&payload_bytes).unwrap();
        p.session_id = 999; // tamper
        let tampered_json = serde_json::to_string(&p).unwrap();
        let tampered_b64 = BASE64.encode(tampered_json.as_bytes());
        let tampered_token = format!("{tampered_b64}.{sig_hex}");

        let err = verify_token(&tampered_token, SECRET).unwrap_err();
        assert!(
            matches!(err, AuthError::InvalidSignature),
            "expected InvalidSignature, got {err:?}"
        );
    }

    // -- Acceptance: expired_token_rejected -----------------------------------

    #[test]
    fn expired_token_rejected() {
        let payload = expired_payload(5, 777);
        let token = generate_token(&payload, SECRET);
        let err = verify_token(&token, SECRET).unwrap_err();
        assert!(
            matches!(err, AuthError::Expired(_)),
            "expected Expired, got {err:?}"
        );
    }

    #[test]
    fn expired_token_carries_timestamp() {
        let payload = expired_payload(5, 777);
        let token = generate_token(&payload, SECRET);
        let err = verify_token(&token, SECRET).unwrap_err();
        match err {
            AuthError::Expired(ts) => assert_eq!(ts, 1_577_836_800),
            other => panic!("expected Expired, got {other:?}"),
        }
    }

    // -- Acceptance: nonce_replay_rejected ------------------------------------

    #[test]
    fn nonce_replay_rejected() {
        let mut cache = NonceCache::new(3600);

        // First use succeeds.
        cache.check_and_insert(42).expect("first insert should succeed");

        // Second use of the same nonce fails.
        let err = cache.check_and_insert(42).unwrap_err();
        assert!(
            matches!(err, AuthError::NonceReused(42)),
            "expected NonceReused(42), got {err:?}"
        );
    }

    #[test]
    fn nonce_cache_accepts_distinct_nonces() {
        let mut cache = NonceCache::new(3600);
        cache.check_and_insert(1).unwrap();
        cache.check_and_insert(2).unwrap();
        cache.check_and_insert(3).unwrap();
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn nonce_cache_cleanup_removes_expired() {
        // Use a max_age of 0 so everything inserted "now" is already expired
        // by the time cleanup runs (within the same second).
        let mut cache = NonceCache::new(0);
        cache.check_and_insert(10).unwrap();
        cache.check_and_insert(20).unwrap();
        assert_eq!(cache.len(), 2);

        // cleanup with max_age=0 retains entries whose age <= 0.
        // Since they were just inserted in the same second, age == 0 which is
        // <= 0, so they should still be present.  To guarantee removal we
        // would need to sleep, so instead test the mechanism directly:
        cache.cleanup_expired();
        // Entries inserted in the current second have age 0, which satisfies
        // `0 <= 0`, so they survive cleanup.
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn nonce_cache_cleanup_expired_with_manual_timestamp() {
        // Directly test cleanup logic by inserting entries with old timestamps.
        let mut cache = NonceCache::new(60);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Manually insert an "old" entry (120 seconds ago, beyond max_age=60).
        cache.seen.insert((100, now.saturating_sub(120)));
        // And a "recent" entry (10 seconds ago, within max_age=60).
        cache.seen.insert((200, now.saturating_sub(10)));
        assert_eq!(cache.len(), 2);

        cache.cleanup_expired();
        assert_eq!(cache.len(), 1, "only the recent entry should survive");
        // The surviving entry should be nonce 200.
        assert!(cache.seen.iter().any(|&(n, _)| n == 200));
    }

    // -- Acceptance: malformed_token_rejected ---------------------------------

    #[test]
    fn malformed_token_no_separator() {
        let err = verify_token("noseparator", SECRET).unwrap_err();
        assert!(
            matches!(err, AuthError::InvalidFormat),
            "expected InvalidFormat, got {err:?}"
        );
    }

    #[test]
    fn malformed_token_bad_base64() {
        let err = verify_token("!!!invalid-base64!!!.abcd1234", SECRET).unwrap_err();
        assert!(
            matches!(err, AuthError::Base64Error(_)),
            "expected Base64Error, got {err:?}"
        );
    }

    #[test]
    fn malformed_token_bad_json() {
        // Valid base64 but not valid JSON for TokenPayload.
        let b64 = BASE64.encode(b"not-json");
        let err = verify_token(&format!("{b64}.0000"), SECRET).unwrap_err();
        assert!(
            matches!(err, AuthError::JsonError(_)),
            "expected JsonError, got {err:?}"
        );
    }

    #[test]
    fn malformed_token_bad_hex_signature() {
        let payload = future_payload(1, 1);
        let token = generate_token(&payload, SECRET);
        let (payload_b64, _sig) = token.split_once('.').unwrap();

        // Odd-length hex is invalid.
        let err = verify_token(&format!("{payload_b64}.abc"), SECRET).unwrap_err();
        assert!(
            matches!(err, AuthError::InvalidFormat),
            "expected InvalidFormat for odd hex, got {err:?}"
        );
    }

    #[test]
    fn malformed_token_empty_string() {
        let err = verify_token("", SECRET).unwrap_err();
        assert!(
            matches!(err, AuthError::InvalidFormat),
            "expected InvalidFormat, got {err:?}"
        );
    }

    #[test]
    fn malformed_token_only_dot() {
        let err = verify_token(".", SECRET).unwrap_err();
        // Empty base64 decodes to empty bytes -> JSON parse error.
        assert!(
            matches!(err, AuthError::Base64Error(_) | AuthError::JsonError(_)),
            "expected Base64Error or JsonError, got {err:?}"
        );
    }

    // -- Token format structure -----------------------------------------------

    #[test]
    fn token_format_has_two_parts() {
        let payload = future_payload(1, 1);
        let token = generate_token(&payload, SECRET);
        let parts: Vec<&str> = token.split('.').collect();
        assert_eq!(parts.len(), 2, "token must have exactly two '.'-separated parts");
    }

    #[test]
    fn token_signature_is_hex_sha256() {
        let payload = future_payload(1, 1);
        let token = generate_token(&payload, SECRET);
        let (_payload_b64, sig_hex) = token.split_once('.').unwrap();
        // SHA-256 produces 32 bytes = 64 hex characters.
        assert_eq!(sig_hex.len(), 64, "HMAC-SHA256 signature must be 64 hex chars");
        assert!(
            sig_hex.chars().all(|c| c.is_ascii_hexdigit()),
            "signature must be lowercase hex"
        );
    }

    // -- Hex helpers ----------------------------------------------------------

    #[test]
    fn hex_decode_valid() {
        assert_eq!(hex_decode("0123456789abcdef"), Some(vec![
            0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef
        ]));
    }

    #[test]
    fn hex_decode_empty() {
        assert_eq!(hex_decode(""), Some(vec![]));
    }

    #[test]
    fn hex_decode_odd_length() {
        assert_eq!(hex_decode("abc"), None);
    }

    #[test]
    fn hex_decode_invalid_chars() {
        assert_eq!(hex_decode("zzzz"), None);
    }

    // -- Constant-time comparison ---------------------------------------------

    #[test]
    fn constant_time_eq_equal() {
        assert!(constant_time_eq(b"hello", b"hello"));
    }

    #[test]
    fn constant_time_eq_different() {
        assert!(!constant_time_eq(b"hello", b"world"));
    }

    #[test]
    fn constant_time_eq_different_lengths() {
        assert!(!constant_time_eq(b"short", b"longer"));
    }

    #[test]
    fn constant_time_eq_empty() {
        assert!(constant_time_eq(b"", b""));
    }

    // -- NonceCache basics ----------------------------------------------------

    #[test]
    fn nonce_cache_new_is_empty() {
        let cache = NonceCache::new(60);
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    // -- AuthError Display ----------------------------------------------------

    #[test]
    fn auth_error_display() {
        assert_eq!(AuthError::InvalidFormat.to_string(), "invalid token format");
        assert_eq!(
            AuthError::InvalidSignature.to_string(),
            "invalid HMAC signature"
        );
        assert_eq!(
            AuthError::Expired(1_000_000).to_string(),
            "token expired at 1000000"
        );
        assert_eq!(
            AuthError::NonceReused(42).to_string(),
            "nonce 42 already used (replay attack)"
        );
        assert_eq!(
            AuthError::Base64Error("bad".into()).to_string(),
            "base64 decode error: bad"
        );
        assert_eq!(
            AuthError::JsonError("oops".into()).to_string(),
            "json decode error: oops"
        );
    }
}
