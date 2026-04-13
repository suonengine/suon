//! Client leave-channel packet.

use super::prelude::*;
use crate::packets::decoder::Decoder;

/// Packet sent by the client to leave an open chat channel by its numeric identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeaveChannel {
    /// Numeric channel identifier that the client wants to leave.
    pub channel_id: u16,
}

impl Decodable for LeaveChannel {
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
    fn should_decode_leave_channel() {
        let mut payload: &[u8] = &[0x34, 0x12];
        assert_eq!(
            LeaveChannel::decode(PacketKind::LeaveChannel, &mut payload)
                .unwrap()
                .channel_id,
            0x1234
        );
    }
}
