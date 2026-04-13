//! Client browse-transaction-history packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Wire layout used by the transaction-history browse packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Current layout used by newer clients: `u32 page` + `u8 entries_per_page`.
    Current,
    /// Legacy layout used by older clients: `u16 page` + `u32 entries_per_page`.
    Legacy,
}

/// Packet sent by the client to browse a page from transaction history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionHistory {
    /// Decoded wire layout variant.
    pub format: Format,

    /// Page number selected by the client.
    pub page: u32,

    /// Page size selected by the client.
    pub entries_per_page: u32,
}

impl Decodable for TransactionHistory {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        match bytes.len() {
            5 => Ok(Self {
                format: Format::Current,
                page: bytes.get_u32()?,
                entries_per_page: u32::from(bytes.get_u8()?),
            }),
            6 => Ok(Self {
                format: Format::Legacy,
                page: u32::from(bytes.get_u16()?),
                entries_per_page: bytes.get_u32()?,
            }),
            _ => Err(DecodableError::Decoder(
                crate::packets::decoder::DecoderError::Incomplete {
                    expected: 5,
                    available: bytes.len(),
                },
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_current_transaction_history_browse() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12, 25];

        let packet = TransactionHistory::decode(PacketKind::BrowseTransactionHistory, &mut payload)
            .expect("BrowseTransactionHistory packets should decode the current format");

        assert_eq!(packet.format, Format::Current);
        assert_eq!(packet.page, 0x12345678);
        assert_eq!(packet.entries_per_page, 25);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_legacy_transaction_history_browse() {
        let mut payload: &[u8] = &[0x34, 0x12, 25, 0, 0, 0];

        let packet = TransactionHistory::decode(PacketKind::BrowseTransactionHistory, &mut payload)
            .expect("BrowseTransactionHistory packets should decode the legacy format");

        assert_eq!(packet.format, Format::Legacy);
        assert_eq!(packet.page, 0x1234);
        assert_eq!(packet.entries_per_page, 25);
        assert!(payload.is_empty());
    }
}
