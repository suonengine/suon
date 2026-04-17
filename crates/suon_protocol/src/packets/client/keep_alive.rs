//! Client keep-alive packet.

use super::prelude::*;

/// Packet sent by the client to keep the session active without payload data.
pub struct KeepAlive;

impl Decodable for KeepAlive {
    fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(KeepAlive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_keep_alive_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = KeepAlive::decode(PacketKind::KeepAlive, &mut payload)
            .expect("KeepAlive packets should decode without payload bytes");

        assert!(matches!(packet, KeepAlive));

        assert!(
            payload.is_empty(),
            "KeepAlive decoding should not consume any payload bytes"
        );
    }
}
