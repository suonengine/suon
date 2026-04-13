//! Client open-transaction-history packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to open the store transaction-history view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenTransactionHistory {
    /// Number of entries requested for the first page.
    pub entries_per_page: u8,
}

impl Decodable for OpenTransactionHistory {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            entries_per_page: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_open_transaction_history() {
        let mut payload: &[u8] = &[25];

        let packet =
            OpenTransactionHistory::decode(PacketKind::OpenTransactionHistory, &mut payload)
                .expect("OpenTransactionHistory packets should decode page size");

        assert_eq!(packet.entries_per_page, 25);
        assert!(payload.is_empty());
    }
}
