//! Client change-map-aware-range packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to update its map-aware range dimensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangeMapAwareRangePacket {
    /// Horizontal map-aware range requested by the client.
    pub x_range: u8,

    /// Vertical map-aware range requested by the client.
    pub y_range: u8,
}

impl Decodable for ChangeMapAwareRangePacket {
    const KIND: PacketKind = PacketKind::ChangeMapAwareRange;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            x_range: bytes.get_u8()?,
            y_range: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_change_map_aware_range() {
        let mut payload: &[u8] = &[18, 14];

        let packet = ChangeMapAwareRangePacket::decode(&mut payload)
            .expect("ChangeMapAwareRange packets should decode x and y dimensions");

        assert_eq!(
            packet,
            ChangeMapAwareRangePacket {
                x_range: 18,
                y_range: 14,
            }
        );
        assert!(payload.is_empty());
    }
}
