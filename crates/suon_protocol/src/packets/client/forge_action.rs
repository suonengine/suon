//! Client forge-action packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Forge action requested by the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForgeActionKind {
    /// Fuses two items, optionally using support resources.
    Fusion {
        /// Whether the fusion uses convergence mode.
        convergence: bool,
        /// Left-hand item id.
        first_item_id: u16,
        /// Tier shared by the source items.
        tier: u8,
        /// Right-hand item id.
        second_item_id: u16,
        /// Whether a core should be consumed.
        use_core: bool,
        /// Whether tier-loss reduction should be applied.
        reduce_tier_loss: bool,
    },
    /// Transfers an item tier from one item to another.
    Transfer {
        /// Whether the transfer uses convergence mode.
        convergence: bool,
        /// Donor item id.
        first_item_id: u16,
        /// Tier being transferred.
        tier: u8,
        /// Receiver item id.
        second_item_id: u16,
    },
    /// Converts forge dust into slivers.
    ConvertDustToSlivers,
    /// Converts forge slivers into cores.
    ConvertSliversToCores,
    /// Increases the forge dust limit.
    IncreaseDustLimit,
}

/// Packet sent by the client to execute a forge action.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForgeActionPacket {
    /// Forge action requested by the client.
    pub action: ForgeActionKind,
}

impl Decodable for ForgeActionPacket {
    const KIND: PacketKind = PacketKind::ForgeAction;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let action = match bytes.get_u8()? {
            0 => {
                let convergence = bytes.get_bool()?;
                let first_item_id = bytes.get_u16()?;
                let tier = bytes.get_u8()?;
                let second_item_id = bytes.get_u16()?;
                let (use_core, reduce_tier_loss) = if convergence {
                    (false, false)
                } else {
                    (bytes.get_bool()?, bytes.get_bool()?)
                };

                ForgeActionKind::Fusion {
                    convergence,
                    first_item_id,
                    tier,
                    second_item_id,
                    use_core,
                    reduce_tier_loss,
                }
            }
            1 => ForgeActionKind::Transfer {
                convergence: bytes.get_bool()?,
                first_item_id: bytes.get_u16()?,
                tier: bytes.get_u8()?,
                second_item_id: bytes.get_u16()?,
            },
            2 => ForgeActionKind::ConvertDustToSlivers,
            3 => ForgeActionKind::ConvertSliversToCores,
            4 => ForgeActionKind::IncreaseDustLimit,
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "action_type",
                    value,
                });
            }
        };

        Ok(Self { action })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_forge_fusion_action() {
        let mut payload: &[u8] = &[0, 0, 0x11, 0x11, 3, 0x22, 0x22, 1, 0];

        let packet = ForgeActionPacket::decode(&mut payload)
            .expect("ForgeAction packets should decode fusion requests");

        assert_eq!(
            packet.action,
            ForgeActionKind::Fusion {
                convergence: false,
                first_item_id: 0x1111,
                tier: 3,
                second_item_id: 0x2222,
                use_core: true,
                reduce_tier_loss: false,
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_forge_transfer_action() {
        let mut payload: &[u8] = &[1, 1, 0x34, 0x12, 4, 0x78, 0x56];

        let packet = ForgeActionPacket::decode(&mut payload)
            .expect("ForgeAction packets should decode transfer requests");

        assert_eq!(
            packet.action,
            ForgeActionKind::Transfer {
                convergence: true,
                first_item_id: 0x1234,
                tier: 4,
                second_item_id: 0x5678,
            }
        );
    }

    #[test]
    fn should_reject_unknown_forge_action_types() {
        let mut payload: &[u8] = &[9];

        let error = ForgeActionPacket::decode(&mut payload)
            .expect_err("ForgeAction packets should reject unknown action types");

        assert!(matches!(
            error,
            DecodableError::InvalidFieldValue {
                field: "action_type",
                value: 9
            }
        ));
    }
}
