//! Client create-buddy packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to create a buddy entry by name.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::CreateBuddy};
///
/// let mut payload: &[u8] = &[4, 0, b'J', b'o', b'h', b'n'];
/// let packet = CreateBuddy::decode(PacketKind::CreateBuddy, &mut payload).unwrap();
///
/// assert_eq!(packet.name, "John");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateBuddy {
    /// Name of the player to add to the buddy list.
    pub name: String,
}

impl Decodable for CreateBuddy {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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

        let packet = CreateBuddy::decode(PacketKind::CreateBuddy, &mut payload)
            .expect("CreateBuddy packets should decode a player name");

        assert_eq!(packet.name, "John");
        assert!(
            payload.is_empty(),
            "CreateBuddy decoding should consume the whole payload"
        );
    }
}
