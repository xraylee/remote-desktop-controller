// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Congestion control using GCC-inspired AIMD algorithm.
//!
//! Implements a dual-mode controller:
//! - **Window-based**: Classic congestion window (`cwnd`) for packet-level control.
//! - **Bitrate-based**: Target bitrate adjustment for media streaming,
//!   using multiplicative decrease on loss and additive increase on recovery.

/// Congestion controller state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CongestionState {
    /// Slow start — probing for available bandwidth.
    SlowStart,
    /// Congestion avoidance — probing around estimated BDP.
    CongestionAvoidance,
    /// Recovery — reducing rate after packet loss.
    Recovery,
}

/// Default initial target bitrate: 2 Mbps.
const DEFAULT_INITIAL_BITRATE: u64 = 2_000_000;
/// Minimum allowed bitrate: 100 Kbps.
const MIN_BITRATE: u64 = 100_000;
/// Maximum allowed bitrate: 50 Mbps.
const MAX_BITRATE: u64 = 50_000_000;
/// Additive increase step: 100 Kbps per round with no loss.
const INCREASE_BPS: u64 = 100_000;

/// A congestion controller combining window-based and bitrate-based control.
///
/// ## Window-based API
/// Use [`on_ack`](CongestionController::on_ack) and [`on_loss`](CongestionController::on_loss)
/// for packet-level congestion control. The [`window`](CongestionController::window)
/// method returns the current congestion window size.
///
/// ## Bitrate-based API (GCC-style)
/// Use [`on_round`](CongestionController::on_round) with a loss rate to adjust
/// the target bitrate using AIMD (Additive Increase, Multiplicative Decrease).
/// The [`target_bitrate`](CongestionController::target_bitrate) method returns
/// the current target in bits per second.
#[derive(Debug)]
pub struct CongestionController {
    state: CongestionState,
    /// Current congestion window size (in packets).
    cwnd: u32,
    /// Slow-start threshold.
    ssthresh: u32,
    /// Estimated round-trip time in microseconds.
    rtt_us: u64,
    /// Target bitrate in bits per second (GCC-style).
    target_bitrate: u64,
    /// Minimum allowed bitrate.
    min_bitrate: u64,
    /// Maximum allowed bitrate.
    max_bitrate: u64,
    /// Additive increase step per round (bps).
    increase_bps: u64,
}

impl CongestionController {
    /// Create a new congestion controller with default parameters.
    pub fn new() -> Self {
        Self {
            state: CongestionState::SlowStart,
            cwnd: 10,
            ssthresh: 64,
            rtt_us: 0,
            target_bitrate: DEFAULT_INITIAL_BITRATE,
            min_bitrate: MIN_BITRATE,
            max_bitrate: MAX_BITRATE,
            increase_bps: INCREASE_BPS,
        }
    }

    /// Create a congestion controller with a custom initial bitrate.
    pub fn with_bitrate(initial_bitrate: u64) -> Self {
        Self {
            target_bitrate: initial_bitrate.clamp(MIN_BITRATE, MAX_BITRATE),
            ..Self::new()
        }
    }

    /// Record a successful acknowledgment and update the congestion window.
    pub fn on_ack(&mut self, rtt_us: u64) {
        self.rtt_us = rtt_us;
        match self.state {
            CongestionState::SlowStart => {
                self.cwnd += 1;
                if self.cwnd >= self.ssthresh {
                    self.state = CongestionState::CongestionAvoidance;
                }
            }
            CongestionState::CongestionAvoidance => {
                // AIMD: increase by 1 per RTT (approximated per ACK)
                self.cwnd += 1;
            }
            CongestionState::Recovery => {
                self.state = CongestionState::CongestionAvoidance;
            }
        }
    }

    /// Record a packet loss event and reduce the congestion window.
    pub fn on_loss(&mut self) {
        self.ssthresh = (self.cwnd / 2).max(2);
        self.cwnd = self.ssthresh;
        self.state = CongestionState::Recovery;
    }

    /// Update the target bitrate based on the observed loss rate for one round.
    ///
    /// - **loss_rate > 0**: Multiplicative decrease — `bitrate *= (1 - loss_rate)`.
    /// - **loss_rate == 0**: Additive increase (or exponential in slow start).
    ///
    /// This implements the GCC-inspired AIMD adaptation:
    /// - 10% sustained loss causes the bitrate to drop ~41% over 5 rounds.
    /// - Zero loss causes steady additive increase.
    pub fn on_round(&mut self, loss_rate: f64) {
        let loss_rate = loss_rate.clamp(0.0, 1.0);
        if loss_rate > 0.0 {
            // Multiplicative decrease
            let factor = 1.0 - loss_rate;
            self.target_bitrate =
                ((self.target_bitrate as f64 * factor) as u64).max(self.min_bitrate);
            if self.state != CongestionState::Recovery {
                tracing::debug!(
                    state = ?self.state,
                    loss_rate,
                    new_bitrate = self.target_bitrate,
                    "congestion: loss detected, entering recovery"
                );
                self.state = CongestionState::Recovery;
            }
        } else {
            // No loss — increase bitrate
            match self.state {
                CongestionState::SlowStart => {
                    // Exponential increase during slow start
                    self.target_bitrate = (self.target_bitrate * 2).min(self.max_bitrate);
                    if self.target_bitrate >= self.max_bitrate / 2 {
                        self.state = CongestionState::CongestionAvoidance;
                    }
                }
                CongestionState::CongestionAvoidance => {
                    // Additive increase
                    self.target_bitrate =
                        (self.target_bitrate + self.increase_bps).min(self.max_bitrate);
                }
                CongestionState::Recovery => {
                    // Transition to congestion avoidance, then increase
                    self.state = CongestionState::CongestionAvoidance;
                    self.target_bitrate =
                        (self.target_bitrate + self.increase_bps).min(self.max_bitrate);
                }
            }
        }
    }

    /// Return the current congestion window size (packets).
    pub fn window(&self) -> u32 {
        self.cwnd
    }

    /// Return the current state.
    pub fn state(&self) -> CongestionState {
        self.state
    }

    /// Return the estimated RTT in microseconds.
    pub fn rtt_us(&self) -> u64 {
        self.rtt_us
    }

    /// Return the current target bitrate in bits per second.
    pub fn target_bitrate(&self) -> u64 {
        self.target_bitrate
    }
}

impl Default for CongestionController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slow_start_growth() {
        let mut cc = CongestionController::new();
        assert_eq!(cc.state(), CongestionState::SlowStart);
        cc.on_ack(1000);
        assert!(cc.window() > 10);
    }

    #[test]
    fn loss_reduces_window() {
        let mut cc = CongestionController::new();
        let before = cc.window();
        cc.on_loss();
        assert!(cc.window() < before);
        assert_eq!(cc.state(), CongestionState::Recovery);
    }

    #[test]
    fn slow_start_to_congestion_avoidance() {
        let mut cc = CongestionController::new();
        // ssthresh = 64, cwnd starts at 10
        // Each on_ack increases cwnd by 1 in slow start
        for _ in 0..54 {
            cc.on_ack(1000);
        }
        // cwnd should now be 64 = ssthresh → transition to CongestionAvoidance
        assert_eq!(cc.state(), CongestionState::CongestionAvoidance);
    }

    #[test]
    fn recovery_returns_to_avoidance() {
        let mut cc = CongestionController::new();
        cc.on_loss(); // → Recovery
        assert_eq!(cc.state(), CongestionState::Recovery);
        cc.on_ack(1000); // → CongestionAvoidance
        assert_eq!(cc.state(), CongestionState::CongestionAvoidance);
    }

    /// Acceptance test: 10% packet loss → target_bitrate drops ≥30% within 5 rounds.
    ///
    /// With multiplicative decrease factor (1 - 0.10) = 0.9 per round:
    /// After 5 rounds: 0.9^5 ≈ 0.590, so bitrate drops ~41%.
    #[test]
    fn gcc_adapt() {
        let mut cc = CongestionController::new();
        let initial = cc.target_bitrate();
        assert!(initial > 0);

        // Simulate 5 rounds with 10% loss each
        for _ in 0..5 {
            cc.on_round(0.10);
        }

        let threshold = initial * 70 / 100; // 70% of initial
        assert!(
            cc.target_bitrate() <= threshold,
            "bitrate {} should be <= {} (dropped ≥30%)",
            cc.target_bitrate(),
            threshold
        );
    }

    #[test]
    fn gcc_no_loss_increases_bitrate() {
        let mut cc = CongestionController::new();
        // Move to CongestionAvoidance first
        cc.on_loss();
        cc.on_round(0.0); // Recovery → CongestionAvoidance + increase

        let after_recovery = cc.target_bitrate();
        cc.on_round(0.0); // Another increase
        assert!(cc.target_bitrate() > after_recovery);
    }

    #[test]
    fn gcc_bitrate_clamped_to_minimum() {
        let mut cc = CongestionController::with_bitrate(MIN_BITRATE);
        // Even with 100% loss, bitrate shouldn't go below minimum
        cc.on_round(1.0);
        assert_eq!(cc.target_bitrate(), MIN_BITRATE);
    }

    #[test]
    fn gcc_bitrate_clamped_to_maximum() {
        let mut cc = CongestionController::with_bitrate(MAX_BITRATE - 1000);
        // Slow start doubles → should be clamped to max
        cc.on_round(0.0);
        assert!(cc.target_bitrate() <= MAX_BITRATE);
    }
}
