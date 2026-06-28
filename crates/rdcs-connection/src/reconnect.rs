// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Automatic reconnection with exponential backoff.

use std::time::Duration;

// ---------------------------------------------------------------------------
// Trait interface
// ---------------------------------------------------------------------------

/// Strategy for computing reconnection delays with exponential backoff.
pub trait ReconnectStrategy: Send {
    /// Return the delay before the next reconnection attempt. Each call
    /// advances the internal state so the following call returns a longer
    /// delay (up to the cap).
    fn next_delay(&mut self) -> Duration;

    /// Reset the strategy to its initial state (e.g. after a successful
    /// reconnection).
    fn reset(&mut self);

    /// Return the number of attempts still available before exhaustion.
    /// Returns 0 when all attempts have been used.
    fn attempts_remaining(&self) -> u32;
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Reconnection strategy configuration.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Initial delay before the first reconnection attempt.
    pub initial_delay: Duration,
    /// Maximum delay between reconnection attempts (the cap).
    pub max_delay: Duration,
    /// Multiplier applied to the delay after each failed attempt.
    pub backoff_multiplier: f64,
    /// Maximum number of reconnection attempts (0 = unlimited).
    pub max_attempts: u32,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            max_attempts: 10,
        }
    }
}

// ---------------------------------------------------------------------------
// Exponential-backoff reconnect manager
// ---------------------------------------------------------------------------

/// Manages automatic reconnection attempts with exponential backoff.
///
/// The delay sequence follows: `initial_delay * multiplier^n`, capped at
/// `max_delay`.  With the default configuration the sequence is:
///
/// ```text
/// [1s, 2s, 4s, 8s, 16s, 30s, 30s, 30s, 30s, 30s]
/// ```
#[derive(Debug)]
pub struct ReconnectManager {
    config: ReconnectConfig,
    attempts: u32,
    current_delay: Duration,
}

impl ReconnectManager {
    /// Create a new reconnection manager.
    pub fn new(config: ReconnectConfig) -> Self {
        let current_delay = config.initial_delay;
        Self {
            config,
            attempts: 0,
            current_delay,
        }
    }

    /// Return the delay before the next reconnection attempt.
    /// Returns `None` if the maximum number of attempts has been reached.
    pub fn next_delay_or_none(&mut self) -> Option<Duration> {
        if self.config.max_attempts > 0 && self.attempts >= self.config.max_attempts {
            return None;
        }

        let delay = self.current_delay;
        self.attempts += 1;

        // Apply exponential backoff.
        let next = Duration::from_secs_f64(delay.as_secs_f64() * self.config.backoff_multiplier);
        self.current_delay = next.min(self.config.max_delay);

        Some(delay)
    }

    /// Return the number of attempts made so far.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    /// Return whether all attempts have been exhausted.
    pub fn exhausted(&self) -> bool {
        self.config.max_attempts > 0 && self.attempts >= self.config.max_attempts
    }
}

impl ReconnectStrategy for ReconnectManager {
    fn next_delay(&mut self) -> Duration {
        // When exhausted, keep returning the max delay rather than panicking.
        // Callers should check `attempts_remaining()` to decide whether to retry.
        if self.config.max_attempts > 0 && self.attempts >= self.config.max_attempts {
            return self.config.max_delay;
        }

        let delay = self.current_delay;
        self.attempts += 1;

        let next = Duration::from_secs_f64(delay.as_secs_f64() * self.config.backoff_multiplier);
        self.current_delay = next.min(self.config.max_delay);

        delay
    }

    fn reset(&mut self) {
        self.attempts = 0;
        self.current_delay = self.config.initial_delay;
    }

    fn attempts_remaining(&self) -> u32 {
        if self.config.max_attempts == 0 {
            // Unlimited.
            u32::MAX
        } else {
            self.config.max_attempts.saturating_sub(self.attempts)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_manager() -> ReconnectManager {
        ReconnectManager::new(ReconnectConfig::default())
    }

    // --- ReconnectStrategy trait tests ---

    #[test]
    fn backoff_sequence_default() {
        let mut mgr = default_manager();
        // Expected: [1, 2, 4, 8, 16, 30, 30, 30, 30, 30]
        let expected = [
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(30),
            Duration::from_secs(30),
            Duration::from_secs(30),
            Duration::from_secs(30),
            Duration::from_secs(30),
        ];
        for &exp in &expected {
            assert_eq!(mgr.next_delay(), exp);
        }
    }

    #[test]
    fn attempts_remaining_decreases() {
        let mut mgr = ReconnectManager::new(ReconnectConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            max_attempts: 3,
        });
        assert_eq!(mgr.attempts_remaining(), 3);

        mgr.next_delay();
        assert_eq!(mgr.attempts_remaining(), 2);

        mgr.next_delay();
        assert_eq!(mgr.attempts_remaining(), 1);

        mgr.next_delay();
        assert_eq!(mgr.attempts_remaining(), 0);
    }

    #[test]
    fn reset_restores_initial_state() {
        let mut mgr = default_manager();
        let _ = mgr.next_delay();
        let _ = mgr.next_delay();
        mgr.reset();
        assert_eq!(mgr.attempts_remaining(), 10);
        assert_eq!(mgr.next_delay(), Duration::from_secs(1));
    }

    #[test]
    fn unlimited_attempts() {
        let mut mgr = ReconnectManager::new(ReconnectConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            max_attempts: 0, // unlimited
        });
        assert_eq!(mgr.attempts_remaining(), u32::MAX);

        // Can keep going forever (well, test a few).
        for _ in 0..100 {
            let _ = mgr.next_delay();
        }
        assert_eq!(mgr.attempts_remaining(), u32::MAX);
    }

    // --- Concrete ReconnectManager method tests ---

    #[test]
    fn next_delay_or_none_returns_none_when_exhausted() {
        let mut mgr = ReconnectManager::new(ReconnectConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            max_attempts: 3,
        });
        assert_eq!(mgr.next_delay_or_none(), Some(Duration::from_secs(1)));
        assert_eq!(mgr.next_delay_or_none(), Some(Duration::from_secs(2)));
        assert_eq!(mgr.next_delay_or_none(), Some(Duration::from_secs(4)));
        assert_eq!(mgr.next_delay_or_none(), None); // exhausted
    }

    #[test]
    fn attempts_count_increments() {
        let mut mgr = default_manager();
        assert_eq!(mgr.attempts(), 0);
        mgr.next_delay();
        assert_eq!(mgr.attempts(), 1);
        mgr.next_delay();
        assert_eq!(mgr.attempts(), 2);
    }

    #[test]
    fn exhausted_flag() {
        let mut mgr = ReconnectManager::new(ReconnectConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            max_attempts: 2,
        });
        assert!(!mgr.exhausted());
        mgr.next_delay();
        assert!(!mgr.exhausted());
        mgr.next_delay();
        assert!(mgr.exhausted());
    }

    #[test]
    fn delay_capped_at_max() {
        let mut mgr = ReconnectManager::new(ReconnectConfig {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            max_attempts: 5,
        });
        // 1, 2, 4, 8, 10 (capped)
        assert_eq!(mgr.next_delay(), Duration::from_secs(1));
        assert_eq!(mgr.next_delay(), Duration::from_secs(2));
        assert_eq!(mgr.next_delay(), Duration::from_secs(4));
        assert_eq!(mgr.next_delay(), Duration::from_secs(8));
        assert_eq!(mgr.next_delay(), Duration::from_secs(10)); // capped
    }

    #[test]
    fn custom_backoff_multiplier() {
        let mut mgr = ReconnectManager::new(ReconnectConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 3.0,
            max_attempts: 5,
        });
        // 100ms, 300ms, 900ms, 2700ms, 5000ms (capped)
        assert_eq!(mgr.next_delay(), Duration::from_millis(100));
        assert_eq!(mgr.next_delay(), Duration::from_millis(300));
        assert_eq!(mgr.next_delay(), Duration::from_millis(900));
        assert_eq!(mgr.next_delay(), Duration::from_millis(2700));
        assert_eq!(mgr.next_delay(), Duration::from_secs(5)); // capped
    }
}
