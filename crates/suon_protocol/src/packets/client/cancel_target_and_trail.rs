//! Client cancel-target-and-trail packet.

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelTargetAndTrailPacket;

impl Decodable for CancelTargetAndTrailPacket {
    const KIND: PacketKind = PacketKind::CancelTargetAndTrail;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_decode_cancel_target_and_trail() {
        let mut payload: &[u8] = &[];
        assert!(matches!(
            CancelTargetAndTrailPacket::decode(&mut payload).unwrap(),
            CancelTargetAndTrailPacket
        ));
    }
}
