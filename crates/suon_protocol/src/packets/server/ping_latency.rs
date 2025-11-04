use super::prelude::*;

pub struct PingLatencyPacket;

impl Encodable for PingLatencyPacket {
    const KIND: PacketKind = PacketKind::PingLatency;
}
