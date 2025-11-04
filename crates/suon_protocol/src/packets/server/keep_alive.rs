use super::prelude::*;

pub struct KeepAlivePacket;

impl Encodable for KeepAlivePacket {
    const KIND: PacketKind = PacketKind::KeepAlive;
}
