// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! Prometheus metrics collection for the relay node.
//!
//! Tracks active sessions, bytes forwarded, slots allocated/reclaimed,
//! packets dropped, and uptime. All counters use lock-free atomic
//! operations so they can be updated from hot paths without contention.

#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------------
// RelayMetrics
// ---------------------------------------------------------------------------

/// Lock-free metrics collector for a single relay node.
///
/// Every field is an [`AtomicU64`] updated with [`Ordering::Relaxed`] —
/// exact ordering across counters is not required; eventual consistency
/// is sufficient for monitoring dashboards.
pub struct RelayMetrics {
    /// Number of currently active relay sessions (gauge).
    pub active_sessions: AtomicU64,
    /// Cumulative bytes forwarded through this relay (counter).
    pub total_bytes_forwarded: AtomicU64,
    /// Cumulative port-pair slots allocated (counter).
    pub slots_allocated: AtomicU64,
    /// Cumulative port-pair slots reclaimed (counter).
    pub slots_reclaimed: AtomicU64,
    /// Cumulative packets that could not be forwarded (counter).
    pub packets_dropped: AtomicU64,
    /// Seconds since the relay started (gauge, updated externally).
    pub uptime_secs: AtomicU64,
}

impl RelayMetrics {
    /// Create a new collector with all counters at zero.
    pub fn new() -> Self {
        Self {
            active_sessions: AtomicU64::new(0),
            total_bytes_forwarded: AtomicU64::new(0),
            slots_allocated: AtomicU64::new(0),
            slots_reclaimed: AtomicU64::new(0),
            packets_dropped: AtomicU64::new(0),
            uptime_secs: AtomicU64::new(0),
        }
    }

    /// Increment active session count by one.
    pub fn inc_session(&self) {
        self.active_sessions.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active session count by one.
    pub fn dec_session(&self) {
        self.active_sessions.fetch_sub(1, Ordering::Relaxed);
    }

    /// Add `bytes` to the total-bytes-forwarded counter.
    pub fn add_bytes(&self, bytes: u64) {
        self.total_bytes_forwarded
            .fetch_add(bytes, Ordering::Relaxed);
    }

    /// Increment the slots-allocated counter by one.
    pub fn inc_slot_alloc(&self) {
        self.slots_allocated.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the slots-reclaimed counter by one.
    pub fn inc_slot_reclaim(&self) {
        self.slots_reclaimed.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment the packets-dropped counter by one.
    pub fn inc_dropped(&self) {
        self.packets_dropped.fetch_add(1, Ordering::Relaxed);
    }

    /// Take a point-in-time snapshot of all counters.
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            active_sessions: self.active_sessions.load(Ordering::Relaxed),
            total_bytes_forwarded: self.total_bytes_forwarded.load(Ordering::Relaxed),
            slots_allocated: self.slots_allocated.load(Ordering::Relaxed),
            slots_reclaimed: self.slots_reclaimed.load(Ordering::Relaxed),
            packets_dropped: self.packets_dropped.load(Ordering::Relaxed),
            uptime_secs: self.uptime_secs.load(Ordering::Relaxed),
        }
    }
}

impl Default for RelayMetrics {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MetricsSnapshot
// ---------------------------------------------------------------------------

/// Point-in-time snapshot of all relay metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetricsSnapshot {
    pub active_sessions: u64,
    pub total_bytes_forwarded: u64,
    pub slots_allocated: u64,
    pub slots_reclaimed: u64,
    pub packets_dropped: u64,
    pub uptime_secs: u64,
}

// ---------------------------------------------------------------------------
// Prometheus text format
// ---------------------------------------------------------------------------

/// Render a [`MetricsSnapshot`] in the Prometheus text exposition format.
///
/// Each metric is emitted as:
/// ```text
/// # HELP <name> <description>
/// # TYPE <name> <gauge|counter>
/// <name> <value>
/// ```
pub fn format_prometheus(snapshot: &MetricsSnapshot) -> String {
    let mut out = String::with_capacity(1024);

    emit_gauge(
        &mut out,
        "rdcs_active_sessions",
        "Active relay sessions",
        snapshot.active_sessions,
    );
    emit_counter(
        &mut out,
        "rdcs_total_bytes_forwarded",
        "Total bytes forwarded through relay",
        snapshot.total_bytes_forwarded,
    );
    emit_counter(
        &mut out,
        "rdcs_slots_allocated",
        "Total port-pair slots allocated",
        snapshot.slots_allocated,
    );
    emit_counter(
        &mut out,
        "rdcs_slots_reclaimed",
        "Total port-pair slots reclaimed",
        snapshot.slots_reclaimed,
    );
    emit_counter(
        &mut out,
        "rdcs_packets_dropped",
        "Total packets that could not be forwarded",
        snapshot.packets_dropped,
    );
    emit_gauge(
        &mut out,
        "rdcs_uptime_secs",
        "Seconds since relay started",
        snapshot.uptime_secs,
    );

    out
}

fn emit_gauge(out: &mut String, name: &str, help: &str, value: u64) {
    out.push_str(&format!("# HELP {name} {help}\n"));
    out.push_str(&format!("# TYPE {name} gauge\n"));
    out.push_str(&format!("{name} {value}\n"));
}

fn emit_counter(out: &mut String, name: &str, help: &str, value: u64) {
    out.push_str(&format!("# HELP {name} {help}\n"));
    out.push_str(&format!("# TYPE {name} counter\n"));
    out.push_str(&format!("{name} {value}\n"));
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_new_all_zero() {
        let m = RelayMetrics::new();
        let snap = m.snapshot();
        assert_eq!(snap.active_sessions, 0);
        assert_eq!(snap.total_bytes_forwarded, 0);
        assert_eq!(snap.slots_allocated, 0);
        assert_eq!(snap.slots_reclaimed, 0);
        assert_eq!(snap.packets_dropped, 0);
        assert_eq!(snap.uptime_secs, 0);
    }

    #[test]
    fn metrics_increment_session() {
        let m = RelayMetrics::new();
        m.inc_session();
        m.inc_session();
        m.inc_session();
        assert_eq!(m.snapshot().active_sessions, 3);
    }

    #[test]
    fn metrics_decrement_session() {
        let m = RelayMetrics::new();
        m.inc_session();
        m.inc_session();
        m.dec_session();
        assert_eq!(m.snapshot().active_sessions, 1);
    }

    #[test]
    fn metrics_add_bytes() {
        let m = RelayMetrics::new();
        m.add_bytes(1024);
        m.add_bytes(2048);
        assert_eq!(m.snapshot().total_bytes_forwarded, 3072);
    }

    #[test]
    fn metrics_slot_alloc_and_reclaim() {
        let m = RelayMetrics::new();
        m.inc_slot_alloc();
        m.inc_slot_alloc();
        m.inc_slot_reclaim();
        let snap = m.snapshot();
        assert_eq!(snap.slots_allocated, 2);
        assert_eq!(snap.slots_reclaimed, 1);
    }

    #[test]
    fn metrics_dropped_packets() {
        let m = RelayMetrics::new();
        m.inc_dropped();
        m.inc_dropped();
        m.inc_dropped();
        assert_eq!(m.snapshot().packets_dropped, 3);
    }

    #[test]
    fn metrics_uptime() {
        let m = RelayMetrics::new();
        m.uptime_secs.store(3600, Ordering::Relaxed);
        assert_eq!(m.snapshot().uptime_secs, 3600);
    }

    #[test]
    fn metrics_default_same_as_new() {
        let m = RelayMetrics::default();
        let snap = m.snapshot();
        assert_eq!(snap.active_sessions, 0);
        assert_eq!(snap.total_bytes_forwarded, 0);
    }

    // -- Acceptance: prometheus_format ----------------------------------------

    #[test]
    fn prometheus_format_empty() {
        let snap = MetricsSnapshot {
            active_sessions: 0,
            total_bytes_forwarded: 0,
            slots_allocated: 0,
            slots_reclaimed: 0,
            packets_dropped: 0,
            uptime_secs: 0,
        };
        let output = format_prometheus(&snap);

        assert!(output.contains("# HELP rdcs_active_sessions Active relay sessions"));
        assert!(output.contains("# TYPE rdcs_active_sessions gauge"));
        assert!(output.contains("rdcs_active_sessions 0"));

        assert!(output.contains("# HELP rdcs_total_bytes_forwarded Total bytes forwarded through relay"));
        assert!(output.contains("# TYPE rdcs_total_bytes_forwarded counter"));
        assert!(output.contains("rdcs_total_bytes_forwarded 0"));
    }

    #[test]
    fn prometheus_format_with_values() {
        let snap = MetricsSnapshot {
            active_sessions: 5,
            total_bytes_forwarded: 1_048_576,
            slots_allocated: 10,
            slots_reclaimed: 7,
            packets_dropped: 3,
            uptime_secs: 86400,
        };
        let output = format_prometheus(&snap);

        assert!(output.contains("rdcs_active_sessions 5"));
        assert!(output.contains("rdcs_total_bytes_forwarded 1048576"));
        assert!(output.contains("rdcs_slots_allocated 10"));
        assert!(output.contains("rdcs_slots_reclaimed 7"));
        assert!(output.contains("rdcs_packets_dropped 3"));
        assert!(output.contains("rdcs_uptime_secs 86400"));
    }

    #[test]
    fn prometheus_format_all_gauge_and_counter_types() {
        let snap = MetricsSnapshot {
            active_sessions: 1,
            total_bytes_forwarded: 1,
            slots_allocated: 1,
            slots_reclaimed: 1,
            packets_dropped: 1,
            uptime_secs: 1,
        };
        let output = format_prometheus(&snap);

        // Gauges
        assert!(output.contains("# TYPE rdcs_active_sessions gauge"));
        assert!(output.contains("# TYPE rdcs_uptime_secs gauge"));

        // Counters
        assert!(output.contains("# TYPE rdcs_total_bytes_forwarded counter"));
        assert!(output.contains("# TYPE rdcs_slots_allocated counter"));
        assert!(output.contains("# TYPE rdcs_slots_reclaimed counter"));
        assert!(output.contains("# TYPE rdcs_packets_dropped counter"));
    }

    #[test]
    fn prometheus_format_each_metric_has_help_and_type() {
        let snap = MetricsSnapshot {
            active_sessions: 0,
            total_bytes_forwarded: 0,
            slots_allocated: 0,
            slots_reclaimed: 0,
            packets_dropped: 0,
            uptime_secs: 0,
        };
        let output = format_prometheus(&snap);

        // 6 metrics, each with HELP + TYPE + value line = 18 lines minimum
        let lines: Vec<&str> = output.lines().collect();
        assert!(
            lines.len() >= 18,
            "expected at least 18 lines, got {}",
            lines.len()
        );
    }

    #[test]
    fn metrics_snapshot_eq() {
        let a = MetricsSnapshot {
            active_sessions: 1,
            total_bytes_forwarded: 2,
            slots_allocated: 3,
            slots_reclaimed: 4,
            packets_dropped: 5,
            uptime_secs: 6,
        };
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn metrics_concurrent_updates() {
        use std::sync::Arc;
        use std::thread;

        let m = Arc::new(RelayMetrics::new());
        let mut handles = Vec::new();

        // Spawn 10 threads each incrementing sessions 100 times.
        for _ in 0..10 {
            let m = Arc::clone(&m);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    m.inc_session();
                    m.add_bytes(64);
                }
            }));
        }

        for h in handles {
            h.join().unwrap();
        }

        let snap = m.snapshot();
        assert_eq!(snap.active_sessions, 1000);
        assert_eq!(snap.total_bytes_forwarded, 64_000);
    }
}
