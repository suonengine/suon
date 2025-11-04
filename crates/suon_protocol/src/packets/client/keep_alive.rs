use super::prelude::*;

pub struct KeepAlivePacket;

impl Decodable for KeepAlivePacket {
    const KIND: PacketKind = PacketKind::KeepAlive;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(KeepAlivePacket)
    }
}
