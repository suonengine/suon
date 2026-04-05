//! Client market-browse packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to browse market data.
///
/// The `request_kind` identifies the kind of market request. Some request kinds
/// are standalone, while others are followed by a `sprite_id`.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::BrowseMarketPacket};
///
/// let mut payload: &[u8] = &[3, 0x2A, 0x00];
/// let packet = BrowseMarketPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.request_kind, 3);
/// assert_eq!(packet.sprite_id, Some(42));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowseMarketPacket {
    /// Identifies the requested market browse action.
    pub request_kind: u8,

    /// Sprite id when the browse action targets a specific item.
    pub sprite_id: Option<u16>,
}

impl Decodable for BrowseMarketPacket {
    const KIND: PacketKind = PacketKind::BrowseMarket;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let request_kind = bytes.get_u8()?;
        let sprite_id = if bytes.is_empty() {
            None
        } else {
            Some(bytes.get_u16()?)
        };

        Ok(Self {
            request_kind,
            sprite_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_browse_market_without_sprite_id() {
        let mut payload: &[u8] = &[0x00];

        let packet = BrowseMarketPacket::decode(&mut payload)
            .expect("BrowseMarket packets should decode browse-only payloads");

        assert_eq!(packet.request_kind, 0x00);
        assert_eq!(packet.sprite_id, None);
        assert!(
            payload.is_empty(),
            "Browse-only market requests should consume the whole payload"
        );
    }

    #[test]
    fn should_decode_browse_market_with_sprite_id() {
        let mut payload: &[u8] = &[0x03, 0x2A, 0x00];

        let packet = BrowseMarketPacket::decode(&mut payload)
            .expect("BrowseMarket packets should decode item-browse payloads");

        assert_eq!(packet.request_kind, 0x03);
        assert_eq!(packet.sprite_id, Some(42));
        assert!(
            payload.is_empty(),
            "Item-browse market requests should consume the whole payload"
        );
    }

    #[test]
    fn should_expose_browse_market_kind_constant() {
        assert_eq!(
            BrowseMarketPacket::KIND,
            PacketKind::BrowseMarket,
            "BrowseMarket packets should advertise the correct packet kind"
        );
    }
}
