//! Client extended-opcode packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to forward an extended opcode string payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedOpcodePacket {
    /// Extended opcode identifier.
    pub opcode: u8,

    /// String payload associated with the extended opcode.
    pub payload: String,
}

impl Decodable for ExtendedOpcodePacket {
    const KIND: PacketKind = PacketKind::ExtendedOpcode;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            opcode: bytes.get_u8()?,
            payload: bytes.get_string()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_extended_opcode() {
        let mut payload: &[u8] = &[9, 4, 0, b't', b'e', b's', b't'];

        let packet = ExtendedOpcodePacket::decode(&mut payload)
            .expect("ExtendedOpcode packets should decode opcode and string payload");

        assert_eq!(packet.opcode, 9);
        assert_eq!(packet.payload, "test");
        assert!(payload.is_empty());
    }
}
