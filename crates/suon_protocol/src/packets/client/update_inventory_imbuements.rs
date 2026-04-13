//! Client update-inventory-imbuements packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to notify whether the inventory-imbuement tracker is open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateInventoryImbuements {
    /// Whether the tracker window is currently open.
    pub is_open: bool,
}

impl Decodable for UpdateInventoryImbuements {
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

        let packet =
            UpdateInventoryImbuements::decode(PacketKind::UpdateInventoryImbuements, &mut payload)
                .expect("UpdateInventoryImbuements packets should decode the tracker state");

        assert!(!packet.is_open);
        assert!(payload.is_empty());
    }
}
