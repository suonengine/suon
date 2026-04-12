//! Client browse-forge-history packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to request a forge-history page.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::BrowseForgeHistoryPacket};
///
/// let mut payload: &[u8] = &[7];
/// let packet = BrowseForgeHistoryPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.page, 7);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BrowseForgeHistoryPacket {
    /// Zero-based forge-history page requested by the client.
    pub page: u8,
}

impl Decodable for BrowseForgeHistoryPacket {
    const KIND: PacketKind = PacketKind::BrowseForgeHistory;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            page: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_browse_forge_history() {
        let mut payload: &[u8] = &[7];

        let packet = BrowseForgeHistoryPacket::decode(&mut payload)
            .expect("BrowseForgeHistory packets should decode the requested page");

        assert_eq!(packet.page, 7);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_expose_browse_forge_history_kind_constant() {
        assert_eq!(
            BrowseForgeHistoryPacket::KIND,
            PacketKind::BrowseForgeHistory
        );
    }
}
