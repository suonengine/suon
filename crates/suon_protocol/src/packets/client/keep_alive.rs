//! Client keep-alive packet.

use super::prelude::*;

/// Packet sent by the client to keep the session active without payload data.
pub struct KeepAlivePacket;

impl Decodable for KeepAlivePacket {
    const KIND: PacketKind = PacketKind::KeepAlive;

    fn decode(_: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(KeepAlivePacket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_keep_alive_from_empty_payload() {
        let mut payload: &[u8] = &[];

        let packet = KeepAlivePacket::decode(&mut payload)
            .expect("KeepAlive packets should decode without payload bytes");

        assert!(matches!(packet, KeepAlivePacket));

        assert!(
            payload.is_empty(),
            "KeepAlive decoding should not consume any payload bytes"
        );
    }

    #[test]
    fn should_expose_keep_alive_kind_constant() {
        assert_eq!(
            KeepAlivePacket::KIND,
            PacketKind::KeepAlive,
            "KeepAlive packets should advertise the correct packet kind"
        );
    }
}
