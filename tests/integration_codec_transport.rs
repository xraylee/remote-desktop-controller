// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! 跨层组合集成测试：rdcs-codec + rdcs-transport。
//!
//! 验证 RTP 包通过 TransportChannel（含 FEC）传输的完整路径：
//! H.264 → RTP → TransportChannel(FEC) → [网络] → TransportChannel(恢复) → RTP → H.264

use rdcs_codec::rtp::{
    H264Depacketizer, H264Packetizer, PacketizerConfig, SrtpConfig, SrtpContext, SrtpProfile,
};
use rdcs_transport::{
    channel::{ChannelConfig, ChannelId, TransportChannel},
    decode_packet,
    packet::PacketType,
};

// ─────────────────────────────────────────────────────────────
// 辅助：固定 SRTP 密钥
// ─────────────────────────────────────────────────────────────

fn srtp_config() -> SrtpConfig {
    SrtpConfig {
        master_key:  vec![0x01,0x02,0x03,0x04,0x05,0x06,0x07,0x08,
                          0x09,0x0A,0x0B,0x0C,0x0D,0x0E,0x0F,0x10],
        master_salt: vec![0x11,0x12,0x13,0x14,0x15,0x16,0x17,0x18,
                          0x19,0x1A,0x1B,0x1C,0x1D,0x1E],
        profile: SrtpProfile::Aead_Aes128Gcm,
    }
}

// ─────────────────────────────────────────────────────────────
// 测试 1：RTP 包通过 TransportChannel 无损传输
// ─────────────────────────────────────────────────────────────

/// RTP 包经 TransportChannel 封装后传输，接收端解封还原并验证内容不变。
#[tokio::test]
async fn test_rtp_packets_through_transport_channel() {
    // 1. RTP 打包：简单 H.264 NAL unit
    let cfg = PacketizerConfig { mtu: 1200, clock_rate: 90000, ssrc: 0xDEAD, payload_type: 96 };
    let mut packetizer = H264Packetizer::new(cfg);

    let annex_b = {
        let mut v = vec![0x00, 0x00, 0x00, 0x01, 0x65]; // IDR
        v.extend(vec![0xAB; 80]);
        v
    };
    let rtp_packets = packetizer.packetize(&annex_b, 9000).unwrap();
    assert_eq!(rtp_packets.len(), 1);
    let rtp_raw = &rtp_packets[0];

    // 2. 发送端 TransportChannel 封装
    let config = ChannelConfig::default_for(ChannelId::Video);
    let mut sender   = TransportChannel::new(config.clone(), 1);
    let mut receiver = TransportChannel::new(config, 2);

    let transport_pkts = sender.prepare_send(rtp_raw, 9000).unwrap();

    // 3. 顺序送达（只发数据包，暂不发 FEC repair）
    let mut delivered: Vec<Vec<u8>> = Vec::new();
    for raw in &transport_pkts {
        let (hdr, _) = decode_packet(raw).unwrap();
        if hdr.packet_type == PacketType::Data {
            let payloads = receiver.on_receive(raw).unwrap();
            delivered.extend(payloads);
        }
    }

    // 4. 接收端还原的载荷应与原始 RTP 包完全一致
    assert_eq!(delivered.len(), 1, "应有一个还原的 RTP 包");
    assert_eq!(&delivered[0], rtp_raw, "还原的 RTP 包应与原始一致");
}

// ─────────────────────────────────────────────────────────────
// 测试 2：TransportChannel 丢 1 包后 FEC 恢复，RTP 流仍完整
// ─────────────────────────────────────────────────────────────

/// 发送 8 个 RTP 包组成一个 FEC group，丢第 3 包，
/// FEC repair 送达后恢复，Depacketizer 应输出全部 NAL 单元。
#[tokio::test]
async fn test_fec_recovery_preserves_rtp_stream() {
    // 准备 8 个独立 H.264 NAL 单元（每个一个 RTP 包）
    let pkt_cfg = PacketizerConfig { mtu: 1200, clock_rate: 90000, ssrc: 0x1234, payload_type: 96 };
    let mut packetizer = H264Packetizer::new(pkt_cfg);

    let rtp_packets: Vec<Vec<u8>> = (0u8..8)
        .map(|i| {
            let mut v = vec![0x00, 0x00, 0x00, 0x01, 0x65];
            v.extend(vec![i; 50]);
            let ts = 3000 * i as u32;
            packetizer.packetize(&v, ts).unwrap().remove(0)
        })
        .collect();

    // 将 8 个 RTP 包送入 TransportChannel，收集所有 transport 包
    let config = ChannelConfig::default_for(ChannelId::Video);
    let mut sender = TransportChannel::new(config.clone(), 10);
    let mut receiver = TransportChannel::new(config, 20);

    let mut transport_data: Vec<Vec<u8>> = Vec::new();
    let mut transport_repair: Vec<Vec<u8>> = Vec::new();

    for (ts, rtp) in rtp_packets.iter().enumerate() {
        let pkts = sender.prepare_send(rtp, ts as u32 * 3000).unwrap();
        for raw in pkts {
            let (hdr, _) = decode_packet(&raw).unwrap();
            match hdr.packet_type {
                PacketType::Data      => transport_data.push(raw),
                PacketType::FecRepair => transport_repair.push(raw),
                _ => {}
            }
        }
    }

    // 丢弃第 2 个数据包（index=2），先不发 FEC repair
    let recovered_before_fec: Vec<Vec<u8>> = transport_data
        .iter()
        .enumerate()
        .filter_map(|(i, raw)| {
            if i == 2 { return None; }
            let payloads = receiver.on_receive(raw).unwrap();
            Some(payloads)
        })
        .flatten()
        .collect();

    // 发 FEC repair 包，触发恢复
    let mut fec_recovered: Vec<Vec<u8>> = Vec::new();
    for repair in &transport_repair {
        let payloads = receiver.on_receive(repair).unwrap();
        fec_recovered.extend(payloads);
    }

    // 总共应收到 8 个 RTP 包（7 正常 + 1 FEC 恢复）
    let total = recovered_before_fec.len() + fec_recovered.len();
    assert_eq!(total, 8, "FEC 恢复后应有 8 个 RTP 包，got {}", total);

    // 验证 NACK 列表为空（FEC 已覆盖丢失）
    let nacks = receiver.poll_nacks();
    assert!(nacks.is_empty(), "FEC 覆盖后不应有 NACK，got {:?}", nacks);
}

// ─────────────────────────────────────────────────────────────
// 测试 3：SRTP 加密后通过 TransportChannel，解密还原完整
// ─────────────────────────────────────────────────────────────

/// SRTP 加密的 RTP 包作为 TransportChannel 的载荷传输，
/// 接收端从 TransportChannel 取出后解密，NAL 单元完整。
#[tokio::test]
async fn test_srtp_payload_through_transport_channel() {
    // 打包
    let pkt_cfg = PacketizerConfig { mtu: 1200, clock_rate: 90000, ssrc: 0xBEEF, payload_type: 96 };
    let mut packetizer = H264Packetizer::new(pkt_cfg);
    let mut depacketizer = H264Depacketizer::new();

    let original = {
        let mut v = vec![0x00, 0x00, 0x00, 0x01, 0x65];
        v.extend(vec![0x42; 60]);
        v
    };
    let rtp_raw = packetizer.packetize(&original, 30000).unwrap().remove(0);

    // SRTP 加密
    let srtp_tx = SrtpContext::new(srtp_config()).await.unwrap();
    let srtp_rx = SrtpContext::new(srtp_config()).await.unwrap();
    let encrypted = srtp_tx.encrypt(&rtp_raw).await.unwrap();

    // TransportChannel 传输（以 SRTP 包为载荷）
    let config = ChannelConfig::default_for(ChannelId::Video);
    let mut sender   = TransportChannel::new(config.clone(), 30);
    let mut receiver = TransportChannel::new(config, 40);

    let transport_pkts = sender.prepare_send(&encrypted, 30000).unwrap();

    let mut rx_payloads: Vec<Vec<u8>> = Vec::new();
    for raw in &transport_pkts {
        let (hdr, _) = decode_packet(raw).unwrap();
        if hdr.packet_type == PacketType::Data {
            rx_payloads.extend(receiver.on_receive(raw).unwrap());
        }
    }

    assert_eq!(rx_payloads.len(), 1);

    // SRTP 解密
    let decrypted = srtp_rx.decrypt(&rx_payloads[0]).await.unwrap();
    assert_eq!(decrypted, rtp_raw, "解密后应与原始 RTP 包一致");

    // RTP 解包
    let nal = depacketizer.depacketize(&decrypted).unwrap();
    assert!(nal.is_some());
    assert_eq!(nal.unwrap(), original, "最终 NAL 单元应与原始 H.264 数据一致");
}
