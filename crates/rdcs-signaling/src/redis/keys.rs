// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Canonical Redis key-generation functions.
//!
//! Every key used by the signaling server is produced by exactly one function
//! in this module, ensuring a single source of truth for the key namespace
//! layout documented in the architecture spec (Section 3.3).

/// Key tracking whether a device is currently online.
///
/// Format: `device:{code}:online`
pub fn device_online_key(code: &str) -> String {
    format!("device:{code}:online")
}

/// Sorted-set key holding the set of online device codes for a team.
///
/// Format: `team:{team_id}:online_devices`
pub fn team_online_key(team_id: &str) -> String {
    format!("team:{team_id}:online_devices")
}

/// Hash key storing metadata for a single remote-control session.
///
/// Format: `session:{session_id}`
pub fn session_key(id: &str) -> String {
    format!("session:{id}")
}

/// Key for a pending device-invite code awaiting claim.
///
/// Format: `device_invite:{code}`
pub fn invite_key(code: &str) -> String {
    format!("device_invite:{code}")
}

/// Pub/Sub channel name for broadcasting team-scoped events.
///
/// Format: `team:{team_id}:events`
pub fn team_events_channel(team_id: &str) -> String {
    format!("team:{team_id}:events")
}

/// Pub/Sub channel name for sending signaling messages to a specific device.
///
/// Format: `device:{code}:signals`
pub fn device_signals_channel(code: &str) -> String {
    format!("device:{code}:signals")
}

/// Key used for temporary account/IP lockout after repeated failures.
///
/// Format: `lockout:{kind}:{id}`
///
/// * `kind` — discriminator such as `"ip"` or `"user"`.
/// * `id`   — the specific identifier (e.g. an IP address or user ID).
pub fn lockout_key(kind: &str, id: &str) -> String {
    format!("lockout:{kind}:{id}")
}

/// Key for a one-shot relay session token.
///
/// Format: `relay_token:{token_hex}`
///
/// The value stored at this key is the session ID the token was issued for.
/// It expires after [`super::ttl::RELAY_TOKEN_TTL`] seconds.
pub fn relay_token_key(token_hex: &str) -> String {
    format!("relay_token:{token_hex}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_online_key_format() {
        assert_eq!(device_online_key("ABC123"), "device:ABC123:online");
    }

    #[test]
    fn device_online_key_empty_code() {
        assert_eq!(device_online_key(""), "device::online");
    }

    #[test]
    fn team_online_key_format() {
        assert_eq!(team_online_key("team-42"), "team:team-42:online_devices");
    }

    #[test]
    fn session_key_format() {
        assert_eq!(session_key("sess-xyz-001"), "session:sess-xyz-001");
    }

    #[test]
    fn invite_key_uses_device_invite_prefix() {
        // The architecture doc specifies "device_invite:" — not "invite:".
        assert_eq!(invite_key("INV001"), "device_invite:INV001");
        assert!(invite_key("X").starts_with("device_invite:"));
    }

    #[test]
    fn team_events_channel_format() {
        assert_eq!(team_events_channel("team-42"), "team:team-42:events");
    }

    #[test]
    fn device_signals_channel_format() {
        assert_eq!(device_signals_channel("ABC123"), "device:ABC123:signals");
    }

    #[test]
    fn lockout_key_format_ip() {
        assert_eq!(lockout_key("ip", "1.2.3.4"), "lockout:ip:1.2.3.4");
    }

    #[test]
    fn lockout_key_format_user() {
        assert_eq!(lockout_key("user", "u-99"), "lockout:user:u-99");
    }

    #[test]
    fn relay_token_key_format() {
        assert_eq!(
            relay_token_key("abc123def456"),
            "relay_token:abc123def456"
        );
    }

    #[test]
    fn relay_token_key_empty() {
        assert_eq!(relay_token_key(""), "relay_token:");
    }
}
