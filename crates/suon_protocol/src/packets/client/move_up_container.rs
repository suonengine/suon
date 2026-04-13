//! Client move-up-container packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

/// Packet sent by the client to navigate one level up in the container hierarchy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoveUpContainer {
    /// Numeric identifier of the container view whose parent should be opened.
    pub container_id: u8,
}

impl Decodable for MoveUpContainer {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            container_id: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_move_up_container() {
        let mut payload: &[u8] = &[3];
        assert_eq!(
            MoveUpContainer::decode(PacketKind::MoveUpContainer, &mut payload)
                .unwrap()
                .container_id,
            3
        );
    }
}
