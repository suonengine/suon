//! Client open-wheel packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to open the wheel for a specific owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenWheel {
    /// Creature id whose wheel should be opened.
    pub owner_id: u32,
}

impl Decodable for OpenWheel {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            owner_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_wheel() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];

        let packet = OpenWheel::decode(PacketKind::OpenWheel, &mut payload)
            .expect("OpenWheel packets should decode the owner id");

        assert_eq!(packet.owner_id, 0x12345678);
        assert!(payload.is_empty());
    }
}
