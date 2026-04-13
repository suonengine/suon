//! Client friend-system-action packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to perform a friend-system action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FriendSystemAction {
    /// Friend-system state/action selector.
    pub state: u8,

    /// Optional title id used by known state `0x0E`.
    pub title_id: Option<u8>,
}

impl Decodable for FriendSystemAction {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let state = bytes.get_u8()?;
        let title_id = if state == 0x0E {
            Some(bytes.get_u8()?)
        } else {
            None
        };

        Ok(Self { state, title_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_friend_system_title_action() {
        let mut payload: &[u8] = &[0x0E, 9];

        let packet = FriendSystemAction::decode(PacketKind::FriendSystemAction, &mut payload)
            .expect("FriendSystemAction packets should decode title actions");

        assert_eq!(packet.state, 0x0E);
        assert_eq!(packet.title_id, Some(9));
        assert!(payload.is_empty());
    }
}
