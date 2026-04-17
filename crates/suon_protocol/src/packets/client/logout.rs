//! Client logout packet.

use super::prelude::*;

/// Packet sent by the client to log out without payload data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Logout;

impl Decodable for Logout {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Logout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_logout_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = Logout::decode(PacketKind::Logout, &mut payload)
            .expect("Logout packets should decode without payload bytes");

        assert!(matches!(packet, Logout));
        assert!(
            payload.is_empty(),
            "Logout decoding should not consume any payload bytes"
        );
    }
}
