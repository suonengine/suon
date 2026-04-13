//! Client join-channel packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JoinChannel {
    pub channel_id: u16,
}

impl Decodable for JoinChannel {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            channel_id: bytes.get_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_decode_join_channel() {
        let mut payload: &[u8] = &[0x34, 0x12];
        assert_eq!(
            JoinChannel::decode(PacketKind::JoinChannel, &mut payload)
                .unwrap()
                .channel_id,
            0x1234
        );
    }
}
