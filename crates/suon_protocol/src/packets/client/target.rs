//! Client target packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to set the current combat target.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::TargetPacket};
///
/// let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];
/// let packet = TargetPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.creature_id, 0x12345678);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetPacket {
    /// Creature id selected as the current target.
    pub creature_id: u32,
}

impl Decodable for TargetPacket {
    const KIND: PacketKind = PacketKind::Target;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            creature_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_target() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];

        let packet =
            TargetPacket::decode(&mut payload).expect("Target packets should decode the target id");

        assert_eq!(packet.creature_id, 0x12345678);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_expose_target_kind_constant() {
        assert_eq!(TargetPacket::KIND, PacketKind::Target);
    }
}
