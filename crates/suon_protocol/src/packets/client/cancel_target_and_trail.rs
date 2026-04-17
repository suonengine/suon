//! Client cancel-target-and-trail packet.

use super::prelude::*;

/// Packet sent by the client to cancel both the active attack target and any creature trail simultaneously.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelTargetAndTrail;

impl Decodable for CancelTargetAndTrail {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
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
            CancelTargetAndTrail::decode(PacketKind::CancelTargetAndTrail, &mut payload).unwrap(),
            CancelTargetAndTrail
        ));
    }
}
