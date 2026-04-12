//! Client save-wheel packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to persist wheel settings.
///
/// The server forwards this payload to the wheel subsystem without decoding it
/// in `ProtocolGame`, so the raw bytes are preserved here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveWheelPacket {
    /// Raw wheel-state payload.
    pub payload: Vec<u8>,
}

impl Decodable for SaveWheelPacket {
    const KIND: PacketKind = PacketKind::SaveWheel;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            payload: bytes.take_remaining().to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_save_wheel_as_raw_payload() {
        let mut payload: &[u8] = &[9, 8, 7];

        let packet = SaveWheelPacket::decode(&mut payload)
            .expect("SaveWheel packets should preserve their opaque payload");

        assert_eq!(packet.payload, vec![9, 8, 7]);
        assert!(payload.is_empty());
    }
}
