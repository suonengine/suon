//! Client cancel-steps packet.

use super::prelude::*;

/// Packet sent by the client to cancel an active step sequence without payload data.
///
/// # Examples
/// ```
/// use suon_protocol_client::prelude::*;
///
/// let mut payload: &[u8] = &[];
/// let packet = CancelStepsPacket::decode(&mut payload).unwrap();
///
/// assert!(matches!(packet, CancelStepsPacket));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CancelStepsPacket;

impl Decodable for CancelStepsPacket {
    const KIND: PacketKind = PacketKind::CancelSteps;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(CancelStepsPacket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_cancel_steps_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = CancelStepsPacket::decode(&mut payload)
            .expect("CancelSteps packets should decode without payload bytes");

        assert!(matches!(packet, CancelStepsPacket));
        assert!(
            payload.is_empty(),
            "CancelSteps decoding should not consume any payload bytes"
        );
    }
}
