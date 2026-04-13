//! Client cancel-steps packet.

use super::prelude::*;

/// Packet sent by the client to cancel an active step sequence without payload data.
///
/// # Examples
/// ```
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::CancelSteps};
///
/// let mut payload: &[u8] = &[];
/// let packet = CancelSteps::decode(PacketKind::CancelSteps, &mut payload).unwrap();
///
/// assert!(matches!(packet, CancelSteps));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelSteps;

impl Decodable for CancelSteps {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(CancelSteps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_cancel_steps_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = CancelSteps::decode(PacketKind::CancelSteps, &mut payload)
            .expect("CancelSteps packets should decode without payload bytes");

        assert!(matches!(packet, CancelSteps));
        assert!(
            payload.is_empty(),
            "CancelSteps decoding should not consume any payload bytes"
        );
    }
}
