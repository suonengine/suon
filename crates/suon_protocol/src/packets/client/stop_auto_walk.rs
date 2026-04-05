//! Client stop-auto-walk packet.

use super::prelude::*;

/// Packet sent by the client to stop auto-walk without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, prelude::StopAutoWalkPacket};
///
/// let mut payload: &[u8] = &[];
/// let packet = StopAutoWalkPacket::decode(&mut payload).unwrap();
///
/// assert!(matches!(packet, StopAutoWalkPacket));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopAutoWalkPacket;

impl Decodable for StopAutoWalkPacket {
    const KIND: PacketKind = PacketKind::StopAutoWalk;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(StopAutoWalkPacket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_stop_auto_walk_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = StopAutoWalkPacket::decode(&mut payload)
            .expect("StopAutoWalk packets should decode without payload bytes");

        assert!(matches!(packet, StopAutoWalkPacket));
        assert!(
            payload.is_empty(),
            "StopAutoWalk decoding should not consume any payload bytes"
        );
    }
}
