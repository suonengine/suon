//! Client delete-buddy packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to delete a buddy entry by guid.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::DeleteBuddyPacket};
///
/// let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];
/// let packet = DeleteBuddyPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.guid, 0x12345678);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteBuddyPacket {
    /// Guid of the buddy entry to delete.
    pub guid: u32,
}

impl Decodable for DeleteBuddyPacket {
    const KIND: PacketKind = PacketKind::DeleteBuddy;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            guid: bytes.get_u32()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_delete_buddy() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12];

        let packet = DeleteBuddyPacket::decode(&mut payload)
            .expect("DeleteBuddy packets should decode a guid");

        assert_eq!(packet.guid, 0x12345678);
        assert!(
            payload.is_empty(),
            "DeleteBuddy decoding should consume the whole payload"
        );
    }

    #[test]
    fn should_expose_delete_buddy_kind_constant() {
        assert_eq!(
            DeleteBuddyPacket::KIND,
            PacketKind::DeleteBuddy,
            "DeleteBuddy packets should advertise the correct packet kind"
        );
    }
}
