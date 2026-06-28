// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Application configuration loaded from environment variables.

use std::env;

use serde::Deserialize;
use thiserror::Error;

/// Configuration errors that can occur when loading from the environment.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("required environment variable `{0}` is not set")]
    MissingVar(&'static str),
    #[error("environment variable `{name}` has invalid value: {reason}")]
    InvalidValue {
        name: &'static str,
        reason: String,
    },
}

/// Static configuration for a single relay node, loaded from the
/// `RDCS_RELAY_NODES` environment variable (JSON array).
#[derive(Debug, Clone, Deserialize)]
pub struct RelayNodeConfig {
    /// Public address (hostname or IP) of the relay server.
    pub addr: String,
    /// UDP/TCP port the relay server listens on.
    pub port: u16,
    /// Geographic region identifier (e.g. `"ap-east-1"`).
    pub region: String,
    /// Maximum number of concurrent relay sessions this node supports.
    pub max_sessions: u32,
}

/// Application configuration, typically loaded from environment variables.
///
/// | Variable            | Default               | Description                    |
/// |---------------------|-----------------------|--------------------------------|
/// | `RDCS_BIND_ADDR`    | `0.0.0.0:8443`        | HTTP server listen address     |
/// | `RDCS_REDIS_URL`    | `redis://127.0.0.1:6379` | Redis connection URL        |
/// | `RDCS_HMAC_SECRET`  | *(required)*          | HMAC secret for relay tokens   |
/// | `RDCS_RELAY_NODES`  | `[]`                  | JSON array of relay node configs |
/// | `RDCS_LOG_LEVEL`    | `info`                | tracing filter directive       |
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Address the HTTP server binds to (e.g. `0.0.0.0:8443`).
    pub bind_addr: String,
    /// Redis connection URL.
    pub redis_url: String,
    /// HMAC secret used to sign relay session tokens.
    pub relay_hmac_secret: String,
    /// Static list of relay nodes available for allocation.
    pub relay_nodes: Vec<RelayNodeConfig>,
    /// tracing log-level filter directive.
    pub log_level: String,
}

impl AppConfig {
    /// Load configuration from environment variables.
    ///
    /// Optional variables fall back to sensible defaults suitable for local
    /// development. `RDCS_HMAC_SECRET` is required in every environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr = env_or("RDCS_BIND_ADDR", "0.0.0.0:8443");
        let redis_url = env_or("RDCS_REDIS_URL", "redis://127.0.0.1:6379");
        let log_level = env_or("RDCS_LOG_LEVEL", "info");

        let relay_hmac_secret = env::var("RDCS_HMAC_SECRET").map_err(|_| {
            ConfigError::MissingVar("RDCS_HMAC_SECRET")
        })?;

        if relay_hmac_secret.is_empty() {
            return Err(ConfigError::InvalidValue {
                name: "RDCS_HMAC_SECRET",
                reason: "must not be empty".into(),
            });
        }

        let relay_nodes = env::var("RDCS_RELAY_NODES")
            .ok()
            .and_then(|json| serde_json::from_str::<Vec<RelayNodeConfig>>(&json).ok())
            .unwrap_or_default();

        Ok(Self {
            bind_addr,
            redis_url,
            relay_hmac_secret,
            relay_nodes,
            log_level,
        })
    }
}

/// Read an environment variable or return `default` if it is not set.
fn env_or(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    /// Serialize tests that modify process-global environment variables.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Helper to clear all RDCS env vars before each test.
    fn clear_env() {
        env::remove_var("RDCS_BIND_ADDR");
        env::remove_var("RDCS_REDIS_URL");
        env::remove_var("RDCS_LOG_LEVEL");
        env::remove_var("RDCS_HMAC_SECRET");
        env::remove_var("RDCS_RELAY_NODES");
    }

    #[test]
    fn defaults_when_optional_vars_missing() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("RDCS_HMAC_SECRET", "test-secret");

        let cfg = AppConfig::from_env().expect("should succeed");
        assert_eq!(cfg.bind_addr, "0.0.0.0:8443");
        assert_eq!(cfg.redis_url, "redis://127.0.0.1:6379");
        assert_eq!(cfg.log_level, "info");
        assert_eq!(cfg.relay_hmac_secret, "test-secret");
        assert!(cfg.relay_nodes.is_empty());
    }

    #[test]
    fn custom_values_override_defaults() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("RDCS_BIND_ADDR", "127.0.0.1:9000");
        env::set_var("RDCS_REDIS_URL", "redis://redis.local:6380");
        env::set_var("RDCS_LOG_LEVEL", "debug");
        env::set_var("RDCS_HMAC_SECRET", "custom-secret");

        let cfg = AppConfig::from_env().expect("should succeed");
        assert_eq!(cfg.bind_addr, "127.0.0.1:9000");
        assert_eq!(cfg.redis_url, "redis://redis.local:6380");
        assert_eq!(cfg.log_level, "debug");
        assert_eq!(cfg.relay_hmac_secret, "custom-secret");
    }

    #[test]
    fn relay_nodes_parsed_from_json() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("RDCS_HMAC_SECRET", "test-secret");
        env::set_var(
            "RDCS_RELAY_NODES",
            r#"[{"addr":"relay1.example.com","port":4443,"region":"ap-east-1","max_sessions":1000}]"#,
        );

        let cfg = AppConfig::from_env().expect("should succeed");
        assert_eq!(cfg.relay_nodes.len(), 1);
        assert_eq!(cfg.relay_nodes[0].addr, "relay1.example.com");
        assert_eq!(cfg.relay_nodes[0].port, 4443);
        assert_eq!(cfg.relay_nodes[0].region, "ap-east-1");
        assert_eq!(cfg.relay_nodes[0].max_sessions, 1000);
    }

    #[test]
    fn relay_nodes_invalid_json_defaults_to_empty() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("RDCS_HMAC_SECRET", "test-secret");
        env::set_var("RDCS_RELAY_NODES", "not valid json");

        let cfg = AppConfig::from_env().expect("should succeed");
        assert!(cfg.relay_nodes.is_empty());
    }

    #[test]
    fn missing_hmac_secret_returns_error() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();

        let err = AppConfig::from_env().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("RDCS_HMAC_SECRET"), "unexpected error: {msg}");
    }

    #[test]
    fn empty_hmac_secret_returns_error() {
        let _lock = ENV_LOCK.lock().unwrap();
        clear_env();
        env::set_var("RDCS_HMAC_SECRET", "");

        let err = AppConfig::from_env().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("must not be empty"), "unexpected error: {msg}");
    }
}
