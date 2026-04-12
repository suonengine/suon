//! Client buddy-group-action packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Group-management action requested for buddy groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuddyGroupAction {
    /// Creates a new buddy group.
    Create {
        /// Display name of the new group.
        name: String,
    },
    /// Renames an existing buddy group.
    Rename {
        /// Group identifier being renamed.
        group_id: u8,
        /// New display name for the group.
        name: String,
    },
    /// Removes an existing buddy group.
    Remove {
        /// Group identifier being removed.
        group_id: u8,
    },
}

/// Packet sent by the client to manage buddy groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuddyGroupActionPacket {
    /// Group-management action requested by the client.
    pub action: BuddyGroupAction,
}

impl Decodable for BuddyGroupActionPacket {
    const KIND: PacketKind = PacketKind::BuddyGroupAction;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let action = match bytes.get_u8()? {
            1 => BuddyGroupAction::Create {
                name: bytes.get_string()?,
            },
            2 => BuddyGroupAction::Rename {
                group_id: bytes.get_u8()?,
                name: bytes.get_string()?,
            },
            3 => BuddyGroupAction::Remove {
                group_id: bytes.get_u8()?,
            },
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "action",
                    value,
                });
            }
        };

        Ok(Self { action })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_create_buddy_group() {
        let mut payload: &[u8] = &[1, 4, 0, b'T', b'e', b'a', b'm'];

        let packet = BuddyGroupActionPacket::decode(&mut payload)
            .expect("BuddyGroupAction packets should decode group creation");

        assert_eq!(
            packet.action,
            BuddyGroupAction::Create {
                name: "Team".to_string(),
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_remove_buddy_group() {
        let mut payload: &[u8] = &[3, 8];

        let packet = BuddyGroupActionPacket::decode(&mut payload)
            .expect("BuddyGroupAction packets should decode group removal");

        assert_eq!(packet.action, BuddyGroupAction::Remove { group_id: 8 });
    }
}
