//! Client close-container packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CloseContainer {
    pub container_id: u8,
}

impl Decodable for CloseContainer {
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
    fn should_decode_close_container() {
        let mut payload: &[u8] = &[3];
        assert_eq!(
            CloseContainer::decode(PacketKind::CloseContainer, &mut payload)
                .unwrap()
                .container_id,
            3
        );
    }
}
