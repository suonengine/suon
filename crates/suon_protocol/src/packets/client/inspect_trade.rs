//! Client inspect-trade packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to inspect an item shown in the trade window.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::InspectTrade};
///
/// let mut payload: &[u8] = &[1, 7];
/// let packet = InspectTrade::decode(PacketKind::InspectTrade, &mut payload).unwrap();
///
/// assert!(packet.is_counter_offer);
/// assert_eq!(packet.index, 7);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InspectTrade {
    /// Whether the inspected item belongs to the counter-offer side of the trade.
    pub is_counter_offer: bool,

    /// Zero-based item index within the selected trade view.
    pub index: u8,
}

impl Decodable for InspectTrade {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            is_counter_offer: bytes.get_u8()? == 1,
            index: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_inspect_trade() {
        let mut payload: &[u8] = &[1, 7];

        let packet = InspectTrade::decode(PacketKind::InspectTrade, &mut payload)
            .expect("InspectTrade packets should decode the side flag and item index");

        assert!(packet.is_counter_offer);
        assert_eq!(packet.index, 7);
        assert!(
            payload.is_empty(),
            "InspectTrade decoding should consume the whole payload"
        );
    }
}
