// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 连接层集成测试。
//!
//! 测试 PriorityPathSelector 的 L1→L2→L3 降级策略。

use rdcs_connection::{
    path::{CandidateType, ConnectionPath, PathCandidate, PathSelector, PathType, PriorityPathSelector},
    ConnectionError,
};
use std::net::SocketAddr;

fn candidate(tier: PathType, addr: &str, rtt: Option<f64>) -> PathCandidate {
    PathCandidate {
        path_type: tier,
        addr: addr.parse().unwrap(),
        rtt_ms: rtt,
    }
}

// ─────────────────────────────────────────────────────────────
// 路径选择
// ─────────────────────────────────────────────────────────────

/// 有 L1/L2/L3 候选时，应优先选 L1。
#[test]
fn test_path_selector_prefers_l1() {
    let mut selector = PriorityPathSelector::new();
    let candidates = vec![
        candidate(PathType::L3Relay,    "1.2.3.4:3478",  Some(80.0)),
        candidate(PathType::L2Punch,    "5.6.7.8:9000",  Some(40.0)),
        candidate(PathType::L1Direct,   "192.168.1.2:9",  Some(2.0)),
    ];

    let path = selector.select_path(&candidates).unwrap();
    assert_eq!(path.path_type(), PathType::L1Direct);
}

/// L1 失败后，应自动降级到 L2。
#[test]
fn test_path_selector_degrades_l1_to_l2() {
    let mut selector = PriorityPathSelector::new();
    let candidates = vec![
        candidate(PathType::L1Direct, "192.168.1.2:9", Some(2.0)),
        candidate(PathType::L2Punch,  "5.6.7.8:9000",  Some(45.0)),
        candidate(PathType::L3Relay,  "1.2.3.4:3478",  Some(80.0)),
    ];

    let path = selector.select_path(&candidates).unwrap();
    assert_eq!(path.path_type(), PathType::L1Direct);

    let fallback = selector.on_path_failed(path).unwrap();
    assert_eq!(fallback.path_type(), PathType::L2Punch, "L1 失败后应降级到 L2");
}

/// L1 和 L2 都失败后，应降级到 L3。
#[test]
fn test_path_selector_degrades_to_l3() {
    let mut selector = PriorityPathSelector::new();
    let candidates = vec![
        candidate(PathType::L1Direct, "192.168.1.2:9", Some(2.0)),
        candidate(PathType::L2Punch,  "5.6.7.8:9000",  Some(45.0)),
        candidate(PathType::L3Relay,  "1.2.3.4:3478",  Some(80.0)),
    ];

    let path = selector.select_path(&candidates).unwrap();
    let l2 = selector.on_path_failed(path).unwrap();
    assert_eq!(l2.path_type(), PathType::L2Punch);

    let l3 = selector.on_path_failed(l2).unwrap();
    assert_eq!(l3.path_type(), PathType::L3Relay, "L1+L2 失败后应降级到 L3");
}

/// 所有路径失败后，on_path_failed 应返回 None。
#[test]
fn test_path_selector_returns_none_when_all_failed() {
    let mut selector = PriorityPathSelector::new();
    let candidates = vec![
        candidate(PathType::L1Direct, "192.168.1.2:9", None),
        candidate(PathType::L2Punch,  "5.6.7.8:9000",  None),
        candidate(PathType::L3Relay,  "1.2.3.4:3478",  None),
    ];

    let path = selector.select_path(&candidates).unwrap();
    let p2 = selector.on_path_failed(path).unwrap();
    let p3 = selector.on_path_failed(p2).unwrap();
    let none = selector.on_path_failed(p3);

    assert!(none.is_none(), "三层全部失败后应返回 None");
}

/// 同一层有多个候选时，应选 RTT 最低的。
#[test]
fn test_path_selector_picks_lowest_rtt_within_tier() {
    let mut selector = PriorityPathSelector::new();
    let candidates = vec![
        candidate(PathType::L1Direct, "192.168.1.10:9", Some(15.0)),
        candidate(PathType::L1Direct, "192.168.1.20:9", Some(5.0)),  // ← 最低
        candidate(PathType::L1Direct, "192.168.1.30:9", Some(10.0)),
    ];

    let path = selector.select_path(&candidates).unwrap();
    assert_eq!(path.addr().ip().to_string(), "192.168.1.20");
}

/// 仅有 L3 候选时，应直接返回 L3，不报 NoViablePath。
#[test]
fn test_path_selector_relay_only() {
    let mut selector = PriorityPathSelector::new();
    let candidates = vec![candidate(PathType::L3Relay, "relay.example.com:3478", Some(100.0))];

    let path = selector.select_path(&candidates).unwrap();
    assert_eq!(path.path_type(), PathType::L3Relay);
}

/// 候选列表为空时，应返回 NoViablePath。
#[test]
fn test_path_selector_no_candidates_returns_error() {
    let mut selector = PriorityPathSelector::new();
    let result = selector.select_path(&[]);
    assert!(
        matches!(result, Err(ConnectionError::NoViablePath)),
        "空候选列表应返回 NoViablePath"
    );
}
