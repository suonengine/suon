//! Client inspect-item-details packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to inspect additional item details.
///
/// # Examples
/// ```rust
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::InspectItemDetails};
///
/// let mut payload: &[u8] = &[0x34, 0x12, 5];
/// let packet = InspectItemDetails::decode(PacketKind::InspectItemDetails, &mut payload).unwrap();
///
/// assert_eq!(packet.item_id, 0x1234);
/// assert_eq!(packet.item_tier, Some(5));
/// assert!(payload.is_empty());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectItemDetails {
    /// Item id being inspected.
    pub item_id: u16,

    /// Optional item tier, present for upgraded items.
    pub item_tier: Option<u8>,
}

impl Decodable for InspectItemDetails {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let item_id = bytes.get_u16()?;
        let item_tier = if bytes.is_empty() {
            None
        } else {
            Some(bytes.get_u8()?)
        };

        Ok(Self { item_id, item_tier })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_inspect_item_details_without_tier() {
        let mut payload: &[u8] = &[0x34, 0x12];

        let packet = InspectItemDetails::decode(PacketKind::InspectItemDetails, &mut payload)
            .expect("InspectItemDetails packets should decode base item ids");

        assert_eq!(packet.item_id, 0x1234);
        assert_eq!(packet.item_tier, None);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_inspect_item_details_with_tier() {
        let mut payload: &[u8] = &[0x34, 0x12, 5];

        let packet = InspectItemDetails::decode(PacketKind::InspectItemDetails, &mut payload)
            .expect("InspectItemDetails packets should decode optional tier values");

        assert_eq!(packet.item_id, 0x1234);
        assert_eq!(packet.item_tier, Some(5));
        assert!(payload.is_empty());
    }
}
