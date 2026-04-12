//! Client teleport packet.

use crate::packets::decoder::Decoder;
use suon_position::position::Position;

use super::prelude::*;

/// Packet sent by the client to request a teleport to a specific position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TeleportPacket {
    /// Destination position requested by the client.
    pub position: Position,
}

impl Decodable for TeleportPacket {
    const KIND: PacketKind = PacketKind::Teleport;

    fn accepts_kind(kind: PacketKind) -> bool {
        matches!(kind, PacketKind::Teleport | PacketKind::TeleportLegacy)
    }

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: bytes.get_position()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_teleport() {
        let mut payload: &[u8] = &[0x34, 0x12, 0x78, 0x56];

        let packet =
            TeleportPacket::decode(&mut payload).expect("Teleport packets should decode positions");

        assert_eq!(
            packet.position,
            Position {
                x: 0x1234,
                y: 0x5678,
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_accept_legacy_and_current_teleport_kinds() {
        assert!(TeleportPacket::accepts_kind(PacketKind::TeleportLegacy));
        assert!(TeleportPacket::accepts_kind(PacketKind::Teleport));
    }
}
