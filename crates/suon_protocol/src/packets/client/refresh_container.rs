//! Client refresh-container packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefreshContainerPacket {
    pub container_id: u8,
}

impl Decodable for RefreshContainerPacket {
    const KIND: PacketKind = PacketKind::RefreshContainer;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            container_id: bytes.get_u8()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_refresh_container() {
        let mut payload: &[u8] = &[3];
        assert_eq!(
            RefreshContainerPacket::decode(&mut payload)
                .unwrap()
                .container_id,
            3
        );
    }
}
