//! Client leave-channel packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveChannelPacket {
    pub channel_id: u16,
}

impl Decodable for LeaveChannelPacket {
    const KIND: PacketKind = PacketKind::LeaveChannel;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            channel_id: bytes.get_u16()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_leave_channel() {
        let mut payload: &[u8] = &[0x34, 0x12];
        assert_eq!(
            LeaveChannelPacket::decode(&mut payload).unwrap().channel_id,
            0x1234
        );
    }
}
