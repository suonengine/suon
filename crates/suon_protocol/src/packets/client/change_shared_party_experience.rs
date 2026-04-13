//! Client change-shared-party-experience packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to enable or disable shared party experience.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::ChangeSharedPartyExperience};
///
/// let mut payload: &[u8] = &[1];
/// let packet = ChangeSharedPartyExperience::decode(PacketKind::ChangeSharedPartyExperience, &mut payload).unwrap();
///
/// assert!(packet.is_enabled);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChangeSharedPartyExperience {
    /// Whether shared party experience should be active.
    pub is_enabled: bool,
}

impl Decodable for ChangeSharedPartyExperience {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            is_enabled: bytes.get_u8()? == 1,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_change_shared_party_experience() {
        let mut payload: &[u8] = &[1];

        let packet = ChangeSharedPartyExperience::decode(
            PacketKind::ChangeSharedPartyExperience,
            &mut payload,
        )
        .expect("ChangeSharedPartyExperience packets should decode the shared experience flag");

        assert!(packet.is_enabled);
        assert!(
            payload.is_empty(),
            "ChangeSharedPartyExperience decoding should consume the whole payload"
        );
    }
}
