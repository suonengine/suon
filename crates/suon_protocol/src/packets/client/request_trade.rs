//! Client request-trade packet.

use suon_position::prelude::*;

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to request a trade for a specific item and partner.
///
/// # Examples
/// ```
/// use suon_position::{floor::Floor, position::Position};
/// use suon_protocol::packets::client::{Decodable, prelude::RequestTradePacket};
///
/// let mut payload: &[u8] = &[
///     0x34, 0x12, 0x78, 0x56, 0x07, 0xCD, 0xAB, 0x03, 0x78, 0x56, 0x34, 0x12,
/// ];
/// let packet = RequestTradePacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.position, Position { x: 0x1234, y: 0x5678 });
/// assert_eq!(packet.floor, Floor { z: 7 });
/// assert_eq!(packet.item_id, 0xABCD);
/// assert_eq!(packet.stack_position, 3);
/// assert_eq!(packet.partner_id, 0x12345678);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestTradePacket {
    /// Tile position of the traded item.
    pub position: Position,

    /// Floor of the traded item.
    pub floor: Floor,

    /// Item id used by the protocol to identify the traded item.
    pub item_id: u16,

    /// Stack position of the traded item on the tile.
    pub stack_position: u8,

    /// Creature id of the intended trade partner.
    pub partner_id: u32,
}

impl Decodable for RequestTradePacket {
    const KIND: PacketKind = PacketKind::RequestTrade;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            position: Position {
                x: bytes.get_u16()?,
                y: bytes.get_u16()?,
            },
            floor: Floor { z: bytes.get_u8()? },
            item_id: bytes.get_u16()?,
            stack_position: bytes.get_u8()?,
            partner_id: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_request_trade() {
        let mut payload: &[u8] = &[
            0x34, 0x12, 0x78, 0x56, 0x07, 0xCD, 0xAB, 0x03, 0x78, 0x56, 0x34, 0x12,
        ];

        let packet = RequestTradePacket::decode(&mut payload).expect(
            "RequestTrade packets should decode position, item id, stack position, and partner id",
        );

        assert_eq!(
            packet.position,
            Position {
                x: 0x1234,
                y: 0x5678
            }
        );
        assert_eq!(packet.floor, Floor { z: 7 });
        assert_eq!(packet.item_id, 0xABCD);
        assert_eq!(packet.stack_position, 3);
        assert_eq!(packet.partner_id, 0x12345678);
        assert!(
            payload.is_empty(),
            "RequestTrade decoding should consume the whole payload"
        );
    }

    #[test]
    fn should_expose_request_trade_kind_constant() {
        assert_eq!(
            RequestTradePacket::KIND,
            PacketKind::RequestTrade,
            "RequestTrade packets should advertise the correct packet kind"
        );
    }
}
