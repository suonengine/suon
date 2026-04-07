//! Client leave-npc-channel packet.

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveNpcChannelPacket;

impl Decodable for LeaveNpcChannelPacket {
    const KIND: PacketKind = PacketKind::LeaveNpcChannel;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_leave_npc_channel() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            LeaveNpcChannelPacket::decode(&mut payload).unwrap(),
            LeaveNpcChannelPacket
        ));
    }
}
