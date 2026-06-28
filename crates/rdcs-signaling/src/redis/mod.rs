// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Redis connection pool and shared utilities for the signaling server.
//!
//! This module provides:
//! - [`RedisPool`]: a type alias for [`redis::aio::ConnectionManager`], an
//!   auto-reconnecting, cloneable async connection suitable for sharing across
//!   tasks.
//! - [`create_pool`]: constructor that opens a connection to the given Redis URL.
//! - [`keys`]: functions that generate canonical Redis key strings.
//! - [`ttl`]: TTL constants and helper functions for common write/expire patterns.

pub mod keys;
pub mod ttl;

/// Shared Redis connection pool.
///
/// [`redis::aio::ConnectionManager`] is cloneable, auto-reconnects on transient
/// failures, and can be cheaply cloned to hand out to multiple async tasks.
pub type RedisPool = redis::aio::ConnectionManager;

/// Create a new Redis connection pool (a single [`ConnectionManager`]).
///
/// # Errors
///
/// Returns a [`redis::RedisError`] if the connection cannot be established.
pub async fn create_pool(url: &str) -> Result<RedisPool, redis::RedisError> {
    let client = redis::Client::open(url)?;
    RedisPool::new(client).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redis_pool_is_clone() {
        // Compile-time check: RedisPool must implement Clone.
        fn assert_clone<T: Clone>() {}
        assert_clone::<RedisPool>();
    }
}
