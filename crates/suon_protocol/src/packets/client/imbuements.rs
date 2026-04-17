//! Client imbuements packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to notify whether the imbuement tracker is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Imbuements {
    /// Whether the tracker state is currently enabled on the client.
    pub is_open: bool,
}

impl Decodable for Imbuements {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            is_open: bytes.get_bool()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_update_inventory_imbuements() {
        let mut payload: &[u8] = &[0];

        let packet = Imbuements::decode(PacketKind::Imbuements, &mut payload)
            .expect("Imbuements packets should decode the tracker state");

        assert!(!packet.is_open);
        assert!(payload.is_empty());
    }
}
