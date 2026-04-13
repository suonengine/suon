//! Client member-finder-action packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Action requested by the member-finder window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemberFinderActionKind {
    /// Requests the available team-finder list.
    OpenList,

    /// Requests to join a leader listing.
    JoinRequest {
        /// Target leader guid.
        leader_guid: u32,
    },

    /// Cancels a previously sent join request.
    CancelRequest {
        /// Target leader guid.
        leader_guid: u32,
    },
}

/// Packet sent by the client to manage the member-finder window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemberFinderAction {
    /// Member-finder action requested by the client.
    pub action: MemberFinderActionKind,
}

impl Decodable for MemberFinderAction {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let raw_action = bytes.get_u8()?;
        let action = match raw_action {
            0 => MemberFinderActionKind::OpenList,
            1 => MemberFinderActionKind::JoinRequest {
                leader_guid: bytes.get_u32()?,
            },
            _ => MemberFinderActionKind::CancelRequest {
                leader_guid: bytes.get_u32()?,
            },
        };

        Ok(Self { action })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_member_finder_open_list() {
        let mut payload: &[u8] = &[0];

        let packet = MemberFinderAction::decode(PacketKind::MemberFinderAction, &mut payload)
            .expect("MemberFinderAction packets should decode list requests");

        assert_eq!(packet.action, MemberFinderActionKind::OpenList);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_member_finder_cancel_request() {
        let mut payload: &[u8] = &[2, 0x78, 0x56, 0x34, 0x12];

        let packet = MemberFinderAction::decode(PacketKind::MemberFinderAction, &mut payload)
            .expect("MemberFinderAction packets should decode cancel requests");

        assert_eq!(
            packet.action,
            MemberFinderActionKind::CancelRequest {
                leader_guid: 0x12345678,
            }
        );
        assert!(payload.is_empty());
    }
}
