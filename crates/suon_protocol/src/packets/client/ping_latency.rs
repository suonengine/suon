use super::prelude::*;

pub struct PingLatencyPacket;

impl Decodable for PingLatencyPacket {
    const KIND: PacketKind = PacketKind::PingLatency;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(PingLatencyPacket)
    }
}
