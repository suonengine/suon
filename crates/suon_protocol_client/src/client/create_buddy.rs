//! Client create-buddy packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to create a buddy entry by name.
///
/// # Examples
/// ```
/// use suon_protocol_client::prelude::*;
///
/// let mut payload: &[u8] = &[4, 0, b'J', b'o', b'h', b'n'];
/// let packet = CreateBuddyPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.name, "John");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateBuddyPacket {
    /// Name of the player to add to the buddy list.
    pub name: String,
}

impl Decodable for CreateBuddyPacket {
    const KIND: PacketKind = PacketKind::CreateBuddy;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            name: bytes.get_string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_create_buddy() {
        let mut payload: &[u8] = &[4, 0, b'J', b'o', b'h', b'n'];

        let packet = CreateBuddyPacket::decode(&mut payload)
            .expect("CreateBuddy packets should decode a player name");

        assert_eq!(packet.name, "John");
        assert!(
            payload.is_empty(),
            "CreateBuddy decoding should consume the whole payload"
        );
    }

    #[test]
    fn should_expose_create_buddy_kind_constant() {
        assert_eq!(
            CreateBuddyPacket::KIND,
            PacketKind::CreateBuddy,
            "CreateBuddy packets should advertise the correct packet kind"
        );
    }
}
