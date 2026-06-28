// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! TTL constants and convenience helpers for common Redis key-lifecycle patterns.
//!
//! All durations are expressed in **seconds**.

use redis::AsyncCommands;

use super::RedisPool;

// ---------------------------------------------------------------------------
// TTL constants
// ---------------------------------------------------------------------------

/// Heartbeat TTL for the `device:{code}:online` key.
///
/// Devices must refresh this key at least once per minute to remain "online".
pub const DEVICE_ONLINE_TTL: u64 = 60;

/// Expiry for a pending device-invite code (`device_invite:{code}`).
pub const INVITE_TTL: u64 = 600; // 10 minutes

/// Expiry for a one-shot relay-session token.
pub const RELAY_TOKEN_TTL: u64 = 30;

/// Expiry for session metadata (`session:{id}`).
///
/// Sessions are kept for 24 hours after creation; the relay layer may
/// refresh this TTL while the session is active.
pub const SESSION_TTL: u64 = 86_400; // 24 hours

/// Duration of a temporary lockout after repeated authentication failures.
pub const LOCKOUT_TTL: u64 = 1800; // 30 minutes

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Write `value` at `key` with an expiry of `ttl` seconds (`SET key value EX ttl`).
///
/// # Errors
///
/// Propagates any [`redis::RedisError`] returned by the server.
pub async fn set_with_ttl(
    pool: &mut RedisPool,
    key: &str,
    value: &str,
    ttl: u64,
) -> Result<(), redis::RedisError> {
    pool.set_ex(key, value, ttl).await
}

/// Reset the TTL on an existing `key` to `ttl` seconds (`EXPIRE key ttl`).
///
/// Returns `true` if the key exists and the timeout was set, `false` if the key
/// does not exist.
///
/// # Errors
///
/// Propagates any [`redis::RedisError`] returned by the server.
pub async fn refresh_ttl(
    pool: &mut RedisPool,
    key: &str,
    ttl: u64,
) -> Result<bool, redis::RedisError> {
    pool.expire(key, ttl as i64).await
}

/// Delete a key from Redis (`DEL key`).
///
/// Returns the number of keys actually removed (0 or 1).
///
/// # Errors
///
/// Propagates any [`redis::RedisError`] returned by the server.
pub async fn del_key(
    pool: &mut RedisPool,
    key: &str,
) -> Result<u64, redis::RedisError> {
    pool.del(key).await
}

/// Serialize `value` as JSON and store it at `key` with the given TTL.
///
/// This is a convenience wrapper around [`set_with_ttl`] that handles
/// JSON serialization.
///
/// # Errors
///
/// Returns a [`serde_json::Error`] if serialization fails, or a
/// [`redis::RedisError`] if the Redis command fails.
pub async fn set_json_with_ttl<T: serde::Serialize>(
    pool: &mut RedisPool,
    key: &str,
    value: &T,
    ttl: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let json = serde_json::to_string(value)?;
    set_with_ttl(pool, key, &json, ttl).await?;
    Ok(())
}

/// Add `member` to the set stored at `key` (`SADD key member`).
///
/// Returns the number of elements added (0 if the member was already present).
///
/// # Errors
///
/// Propagates any [`redis::RedisError`] returned by the server.
pub async fn sadd(
    pool: &mut RedisPool,
    key: &str,
    member: &str,
) -> Result<u64, redis::RedisError> {
    pool.sadd(key, member).await
}

/// Remove `member` from the set stored at `key` (`SREM key member`).
///
/// Returns the number of elements removed (0 if the member was not present).
///
/// # Errors
///
/// Propagates any [`redis::RedisError`] returned by the server.
pub async fn srem(
    pool: &mut RedisPool,
    key: &str,
    member: &str,
) -> Result<u64, redis::RedisError> {
    pool.srem(key, member).await
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Constant sanity checks (no Redis needed) --------------------------

    #[test]
    fn device_online_ttl_is_one_minute() {
        assert_eq!(DEVICE_ONLINE_TTL, 60);
    }

    #[test]
    fn invite_ttl_is_ten_minutes() {
        assert_eq!(INVITE_TTL, 600);
    }

    #[test]
    fn relay_token_ttl_is_thirty_seconds() {
        assert_eq!(RELAY_TOKEN_TTL, 30);
    }

    #[test]
    fn lockout_ttl_is_thirty_minutes() {
        assert_eq!(LOCKOUT_TTL, 1800);
    }

    // -- Integration tests (require a running Redis at the default URL) ----

    const TEST_REDIS_URL: &str = "redis://127.0.0.1:6379";

    /// Helper: build a throwaway pool for integration tests.
    async fn test_pool() -> RedisPool {
        crate::redis::create_pool(TEST_REDIS_URL)
            .await
            .expect("failed to connect to local Redis for integration test")
    }

    /// Unique key prefix so parallel test runs don't collide.
    fn test_key(suffix: &str) -> String {
        format!("rdcs_test:ttl:{suffix}:{}", std::process::id())
    }

    #[tokio::test]
    #[ignore = "requires a running Redis server at redis://127.0.0.1:6379"]
    async fn set_with_ttl_writes_key_that_auto_expires() {
        let mut pool = test_pool().await;
        let key = test_key("set_with_ttl");

        // Clean up any leftover from a previous run.
        let _: Result<u64, _> = pool.del(&key).await;

        set_with_ttl(&mut pool, &key, "hello", 60).await.unwrap();

        let val: Option<String> = pool.get(&key).await.unwrap();
        assert_eq!(val.as_deref(), Some("hello"));

        let ttl: i64 = pool.ttl(&key).await.unwrap();
        assert!(ttl > 0 && ttl <= 60, "expected TTL in (0, 60], got {ttl}");

        // Cleanup.
        let _: Result<u64, _> = pool.del(&key).await;
    }

    #[tokio::test]
    #[ignore = "requires a running Redis server at redis://127.0.0.1:6379"]
    async fn refresh_ttl_extends_existing_key() {
        let mut pool = test_pool().await;
        let key = test_key("refresh_ttl");

        let _: Result<u64, _> = pool.del(&key).await;

        set_with_ttl(&mut pool, &key, "v", 10).await.unwrap();

        let ok = refresh_ttl(&mut pool, &key, 300).await.unwrap();
        assert!(ok, "refresh_ttl should return true for existing key");

        let ttl: i64 = pool.ttl(&key).await.unwrap();
        assert!(ttl > 10, "TTL should have been extended beyond 10s, got {ttl}");
        assert!(ttl <= 300, "TTL should not exceed 300s, got {ttl}");

        let _: Result<u64, _> = pool.del(&key).await;
    }

    #[tokio::test]
    #[ignore = "requires a running Redis server at redis://127.0.0.1:6379"]
    async fn refresh_ttl_returns_false_for_missing_key() {
        let mut pool = test_pool().await;
        let key = test_key("refresh_ttl_missing");

        let _: Result<u64, _> = pool.del(&key).await;

        let ok = refresh_ttl(&mut pool, &key, 60).await.unwrap();
        assert!(!ok, "refresh_ttl should return false when key does not exist");
    }

    #[tokio::test]
    #[ignore = "requires a running Redis server at redis://127.0.0.1:6379"]
    async fn del_key_removes_key() {
        let mut pool = test_pool().await;
        let key = test_key("del_key");

        let _: Result<u64, _> = pool.del(&key).await;

        set_with_ttl(&mut pool, &key, "v", 60).await.unwrap();

        let removed = del_key(&mut pool, &key).await.unwrap();
        assert_eq!(removed, 1);

        let val: Option<String> = pool.get(&key).await.unwrap();
        assert!(val.is_none(), "key should be gone after del_key");

        // Deleting again should report 0 removed.
        let removed_again = del_key(&mut pool, &key).await.unwrap();
        assert_eq!(removed_again, 0);
    }

    // -- Unit tests for new helpers (no Redis needed) -----------------------

    #[test]
    fn set_json_with_ttl_serializes_correctly() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Info {
            platform: String,
            version: String,
        }

        let info = Info {
            platform: "linux".into(),
            version: "0.1.0".into(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: Info = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, info);
    }

    // -- Integration tests for set operations (require Redis) ---------------

    #[tokio::test]
    #[ignore = "requires a running Redis server at redis://127.0.0.1:6379"]
    async fn sadd_adds_and_srem_removes_member() {
        let mut pool = test_pool().await;
        let key = test_key("sadd_srem");

        // Clean up.
        let _: Result<u64, _> = pool.del(&key).await;

        let added = sadd(&mut pool, &key, "DEV1").await.unwrap();
        assert_eq!(added, 1);

        // Adding same member again should return 0.
        let added_again = sadd(&mut pool, &key, "DEV1").await.unwrap();
        assert_eq!(added_again, 0);

        // Adding a different member.
        let added2 = sadd(&mut pool, &key, "DEV2").await.unwrap();
        assert_eq!(added2, 1);

        // Check cardinality.
        let count: u64 = pool.scard(&key).await.unwrap();
        assert_eq!(count, 2);

        // Remove DEV1.
        let removed = srem(&mut pool, &key, "DEV1").await.unwrap();
        assert_eq!(removed, 1);

        // DEV1 should be gone.
        let is_member: bool = pool.sismember(&key, "DEV1").await.unwrap();
        assert!(!is_member);

        // DEV2 should still be there.
        let is_member2: bool = pool.sismember(&key, "DEV2").await.unwrap();
        assert!(is_member2);

        // Cleanup.
        let _: Result<u64, _> = pool.del(&key).await;
    }

    #[tokio::test]
    #[ignore = "requires a running Redis server at redis://127.0.0.1:6379"]
    async fn set_json_with_ttl_stores_and_expires() {
        use serde::{Deserialize, Serialize};

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct OnlineInfo {
            platform: String,
            version: String,
            team_id: Option<String>,
        }

        let mut pool = test_pool().await;
        let key = test_key("set_json");

        let _: Result<u64, _> = pool.del(&key).await;

        let info = OnlineInfo {
            platform: "macos".into(),
            version: "1.0.0".into(),
            team_id: Some("team-42".into()),
        };

        set_json_with_ttl(&mut pool, &key, &info, 60)
            .await
            .unwrap();

        let val: String = pool.get(&key).await.unwrap();
        let parsed: OnlineInfo = serde_json::from_str(&val).unwrap();
        assert_eq!(parsed, info);

        let ttl: i64 = pool.ttl(&key).await.unwrap();
        assert!(ttl > 0 && ttl <= 60);

        // Cleanup.
        let _: Result<u64, _> = pool.del(&key).await;
    }
}
