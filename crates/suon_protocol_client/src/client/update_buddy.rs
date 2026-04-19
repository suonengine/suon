//! Client update-buddy packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Highest icon id accepted by the protocol for buddy entries.
pub const MAX_BUDDY_ICON_ID: u32 = 10;

/// Packet sent by the client to update a buddy entry.
///
/// The icon id is clamped to `MAX_BUDDY_ICON_ID` to match the upstream protocol.
///
/// # Examples
/// ```
/// use suon_protocol_client::prelude::*;
///
/// let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12, 4, 0, b'n', b'o', b't', b'e', 3, 0, 0, 0, 1];
/// let packet = UpdateBuddyPacket::decode(&mut payload).unwrap();
///
/// assert_eq!(packet.guid, 0x12345678);
/// assert_eq!(packet.description, "note");
/// assert_eq!(packet.icon_id, 3);
/// assert!(packet.notify_login);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateBuddyPacket {
    /// Guid of the buddy entry to update.
    pub guid: u32,

    /// Description associated with the buddy entry.
    pub description: String,

    /// Clamped icon id associated with the buddy entry.
    pub icon_id: u32,

    /// Whether login notifications should be enabled.
    pub notify_login: bool,
}

impl Decodable for UpdateBuddyPacket {
    const KIND: PacketKind = PacketKind::UpdateBuddy;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let guid = bytes.get_u32()?;
        let description = bytes.get_string()?;
        let icon_id = bytes.get_u32()?.min(MAX_BUDDY_ICON_ID);
        let notify_login = bytes.get_bool()?;

        Ok(Self {
            guid,
            description,
            icon_id,
            notify_login,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_update_buddy() {
        let mut payload: &[u8] = &[
            0x78, 0x56, 0x34, 0x12, 4, 0, b'n', b'o', b't', b'e', 3, 0, 0, 0, 1,
        ];

        let packet = UpdateBuddyPacket::decode(&mut payload)
            .expect("UpdateBuddy packets should decode guid, description, icon, and notify flag");

        assert_eq!(packet.guid, 0x12345678);
        assert_eq!(packet.description, "note");
        assert_eq!(packet.icon_id, 3);
        assert!(packet.notify_login);
        assert!(
            payload.is_empty(),
            "UpdateBuddy decoding should consume the whole payload"
        );
    }

    #[test]
    fn should_clamp_update_buddy_icon_id_to_protocol_maximum() {
        let mut payload: &[u8] = &[0x78, 0x56, 0x34, 0x12, 0, 0, 42, 0, 0, 0, 0];

        let packet = UpdateBuddyPacket::decode(&mut payload)
            .expect("UpdateBuddy packets should clamp icon ids above the protocol maximum");

        assert_eq!(
            packet.icon_id, MAX_BUDDY_ICON_ID,
            "UpdateBuddy packets should clamp icon ids to the protocol maximum"
        );
    }

    #[test]
    fn should_expose_update_buddy_kind_constant() {
        assert_eq!(
            UpdateBuddyPacket::KIND,
            PacketKind::UpdateBuddy,
            "UpdateBuddy packets should advertise the correct packet kind"
        );
    }
}
