//! Client wheel-gem packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to forward a wheel-gem payload.
///
/// The server forwards this payload to the wheel subsystem without decoding it
/// inside `ProtocolGame`, so this packet preserves the raw bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelGem {
    /// Raw payload forwarded to the wheel-gem handler.
    pub payload: Vec<u8>,
}

impl Decodable for WheelGem {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            payload: bytes.take_remaining().to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_wheel_gem_action_as_raw_payload() {
        let mut payload: &[u8] = &[1, 2, 3, 4];

        let packet = WheelGem::decode(PacketKind::WheelGem, &mut payload)
            .expect("WheelGem packets should preserve their opaque payload");

        assert_eq!(packet.payload, vec![1, 2, 3, 4]);
        assert!(payload.is_empty());
    }
}
