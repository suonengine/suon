//! Client leave-npc-channel packet.

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveNpcChannel;

impl Decodable for LeaveNpcChannel {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
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
            LeaveNpcChannel::decode(PacketKind::LeaveNpcChannel, &mut payload).unwrap(),
            LeaveNpcChannel
        ));
    }
}
