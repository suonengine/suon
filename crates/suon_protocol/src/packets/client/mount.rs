//! Client mount packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to enable or disable mount usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mount {
    /// Whether the client wants the character to stay mounted.
    pub mounted: bool,
}

impl Decodable for Mount {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            mounted: bytes.get_bool()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_set_mount_state() {
        let mut payload: &[u8] = &[1];

        let packet = Mount::decode(PacketKind::Mount, &mut payload)
            .expect("Mount packets should decode the requested mount flag");

        assert!(packet.mounted);
        assert!(payload.is_empty());
    }
}
