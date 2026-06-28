// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 传输层可靠性集成测试。
//!
//! 测试 FEC 恢复 + NACK 重传 + TransportChannel 的完整可靠性保障链路。

use rdcs_transport::{
    channel::{ChannelConfig, TransportChannel},
    decode_packet,
    fec::{FecDecoder, FecEncoder},
    nack::NackTracker,
    packet::PacketType,
};

// ─────────────────────────────────────────────────────────────
// FEC：XOR 恢复
// ─────────────────────────────────────────────────────────────

/// 8+2 配置，每子组丢 1 包，期望 decode_group 返回完整数据。
#[test]
fn test_fec_recovers_one_loss_per_subgroup() {
    let mut encoder = FecEncoder::new(8, 2);
    let data: Vec<Vec<u8>> = (0u8..8).map(|i| vec![i; 128]).collect();

    let mut repairs = None;
    for pkt in &data {
        repairs = encoder.feed(pkt);
    }
    let repairs = repairs.expect("第 8 包后应输出 repair");
    assert_eq!(repairs.len(), 2);

    // 丢 data[2]（子组 0）和 data[5]（子组 1）
    let received_data: Vec<Option<Vec<u8>>> = data
        .iter()
        .enumerate()
        .map(|(i, p)| if i == 2 || i == 5 { None } else { Some(p.clone()) })
        .collect();
    let received_repairs: Vec<Option<Vec<u8>>> = repairs.iter().map(|r| Some(r.clone())).collect();

    let decoder = FecDecoder::new(8, 2);
    let recovered = decoder
        .decode_group(&received_data, &received_repairs)
        .expect("FEC 应能恢复");

    assert_eq!(recovered[2], data[2], "子组 0 的丢包应被 FEC 恢复");
    assert_eq!(recovered[5], data[5], "子组 1 的丢包应被 FEC 恢复");
}

/// 同子组丢 2 包，超出恢复能力，decode_group 应返回 Err。
#[test]
fn test_fec_fails_on_two_losses_in_same_subgroup() {
    let mut encoder = FecEncoder::new(8, 2);
    let data: Vec<Vec<u8>> = (0u8..8).map(|i| vec![i; 64]).collect();

    let mut repairs = None;
    for pkt in &data {
        repairs = encoder.feed(pkt);
    }
    let repairs = repairs.unwrap();

    // 子组 0 = data[0..4]，丢 0 和 1
    let received_data: Vec<Option<Vec<u8>>> = data
        .iter()
        .enumerate()
        .map(|(i, p)| if i == 0 || i == 1 { None } else { Some(p.clone()) })
        .collect();
    let received_repairs: Vec<Option<Vec<u8>>> = repairs.iter().map(|r| Some(r.clone())).collect();

    let decoder = FecDecoder::new(8, 2);
    assert!(
        decoder.decode_group(&received_data, &received_repairs).is_err(),
        "同子组丢 2 包时应无法恢复"
    );
}

// ─────────────────────────────────────────────────────────────
// NACK
// ─────────────────────────────────────────────────────────────

#[test]
fn test_nack_detects_and_reports_gaps() {
    let mut tracker = NackTracker::new(3);
    tracker.report_gap(3, 6); // 缺 3, 4, 5

    let list = tracker.generate_nack_list();
    assert!(list.contains(&3));
    assert!(list.contains(&4));
    assert!(list.contains(&5));
    assert!(!list.contains(&6));
}

#[test]
fn test_nack_abandons_after_max_retries() {
    let mut tracker = NackTracker::new(2);
    tracker.report_gap(10, 11); // 缺序号 10

    tracker.generate_nack_list(); // 第 1 次
    tracker.generate_nack_list(); // 第 2 次（达到 max_retries）
    let list = tracker.generate_nack_list(); // 第 3 次：应被抛弃

    assert!(!list.contains(&10), "超过 max_retries 后应从列表移除");
    assert_eq!(tracker.missing_count(), 0);
}

#[test]
fn test_nack_cleared_on_retransmission() {
    let mut tracker = NackTracker::new(5);
    tracker.report_gap(20, 23); // 缺 20, 21, 22

    tracker.mark_received(21); // 重传到达

    let list = tracker.generate_nack_list();
    assert!(!list.contains(&21), "重传到达后应从 NACK 列表移除");
    assert!(list.contains(&20));
    assert!(list.contains(&22));
}

// ─────────────────────────────────────────────────────────────
// TransportChannel：发送-接收完整路径
// ─────────────────────────────────────────────────────────────

/// 发送 N 包，接收端按顺序全部到达，应无丢失无 NACK。
#[test]
fn test_channel_full_delivery_no_loss() {
    let config = ChannelConfig::default_for(rdcs_transport::channel::ChannelId::Video);
    let mut sender = TransportChannel::new(config.clone(), 1);
    let mut receiver = TransportChannel::new(config, 2);

    let payloads: Vec<Vec<u8>> = (0u8..16).map(|i| vec![i; 200]).collect();

    let mut all_raw: Vec<Vec<u8>> = Vec::new();
    for (ts, payload) in payloads.iter().enumerate() {
        let raw_pkts = sender.prepare_send(payload, ts as u32).unwrap();
        all_raw.extend(raw_pkts);
    }

    // 顺序送达所有包（含 FEC repair）
    for raw in &all_raw {
        receiver.on_receive(raw).unwrap();
    }

    // 顺序送达后不应有 NACK
    let nacks = receiver.poll_nacks();
    assert!(nacks.is_empty(), "无丢包时 NACK 列表应为空，got: {:?}", nacks);
}

/// FEC 包随数据包一起送达，接收端丢 1 包后利用 FEC 恢复，不产生 NACK。
#[test]
fn test_channel_fec_suppresses_nack_on_single_loss() {
    let config = ChannelConfig::default_for(rdcs_transport::channel::ChannelId::Video);
    let mut sender = TransportChannel::new(config.clone(), 10);
    let mut receiver = TransportChannel::new(config, 20);

    // 发送 8 包（恰好一个 FEC group）
    let payloads: Vec<Vec<u8>> = (0u8..8).map(|i| vec![i; 100]).collect();
    let mut all_raw: Vec<Vec<u8>> = Vec::new();
    for (ts, payload) in payloads.iter().enumerate() {
        let pkts = sender.prepare_send(payload, ts as u32).unwrap();
        all_raw.extend(pkts);
    }

    // 从 all_raw 中分离数据包和 FEC repair 包
    let (data_pkts, repair_pkts): (Vec<_>, Vec<_>) = all_raw.iter().partition(|raw| {
        let (hdr, _) = decode_packet(raw).unwrap();
        hdr.packet_type == PacketType::Data
    });

    // 丢弃第 3 个数据包（序号 2），但发送所有 FEC repair 包
    for (i, raw) in data_pkts.iter().enumerate() {
        if i != 2 {
            receiver.on_receive(raw).unwrap();
        }
    }
    for repair in &repair_pkts {
        receiver.on_receive(repair).unwrap();
    }

    // FEC 应能覆盖单包丢失，接收端不应产生 NACK
    let nacks = receiver.poll_nacks();
    // 注意：TransportChannel 在接收 FEC repair 后会自动填充缺失，
    // 此时 NACK 应被取消或从未产生
    // 若实现上先 NACK 再 FEC 恢复，此处验证 NACK 最终被清空
    assert!(
        nacks.is_empty(),
        "FEC 恢复后不应有残留 NACK，got: {:?}",
        nacks
    );
}

/// 发送 16 包，丢弃其中 3 包且无 FEC repair（模拟 FEC 也丢了），
/// 接收端 NACK 列表应包含这 3 个序号。
#[test]
fn test_channel_nack_triggered_without_fec() {
    let config = ChannelConfig::default_for(rdcs_transport::channel::ChannelId::Video);
    let mut sender = TransportChannel::new(config.clone(), 100);
    let mut receiver = TransportChannel::new(config, 200);

    let payloads: Vec<Vec<u8>> = (0u8..16).map(|i| vec![i; 100]).collect();
    let mut data_only: Vec<Vec<u8>> = Vec::new();

    for (ts, payload) in payloads.iter().enumerate() {
        let pkts = sender.prepare_send(payload, ts as u32).unwrap();
        for raw in pkts {
            let (hdr, _) = decode_packet(&raw).unwrap();
            if hdr.packet_type == PacketType::Data {
                data_only.push(raw);
            }
        }
    }

    // 丢弃 data[1], data[5], data[10]（跨多个 FEC 子组）
    let drop_set = [1usize, 5, 10];
    for (i, raw) in data_only.iter().enumerate() {
        if !drop_set.contains(&i) {
            receiver.on_receive(raw).unwrap();
        }
    }

    let nacks = receiver.poll_nacks();
    assert!(!nacks.is_empty(), "丢包后应产生 NACK 请求");
    // 丢失的序号应出现在 NACK 列表（具体序号依赖初始 seq 值）
    assert!(nacks.len() >= 3, "应有至少 3 个 NACK 条目，got: {:?}", nacks);
}
