//! Client set-outfit packet.

use suon_position::{floor::Floor, position::Position};

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Shared appearance block that prefixes every [`UpdateOutfit`] payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitAppearance {
    /// Base outfit or creature look type id selected by the client.
    pub look_type: u16,

    /// Head color channel for the shared appearance block.
    pub look_head: u8,

    /// Body color channel for the shared appearance block.
    pub look_body: u8,

    /// Legs color channel for the shared appearance block.
    pub look_legs: u8,

    /// Feet color channel for the shared appearance block.
    pub look_feet: u8,

    /// Addon bitmask enabled for the shared appearance block.
    pub look_addons: u8,
}

/// Mount-appearance bytes embedded in outfit-window and podium updates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitMountAppearance {
    /// Mount id selected for the current branch.
    pub mount_id: u16,
    /// Mount head color channel.
    pub mount_head: u8,
    /// Mount body color channel.
    pub mount_body: u8,
    /// Mount legs color channel.
    pub mount_legs: u8,
    /// Mount feet color channel.
    pub mount_feet: u8,
}

/// Branch payload used when the client submits the regular outfit window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitWindowDetails {
    /// Mount selection and palette data submitted with the outfit window.
    pub mount: OutfitMountAppearance,
    /// Whether the client asked the server to apply the selected mount.
    pub set_mount: bool,
    /// Familiar look type carried by the outfit-window branch.
    pub familiar_look_type: u16,
    /// Whether the client requested mount randomization in the outfit window.
    pub randomize_mount: bool,
}

/// Branch payload used when the client enters the preview-only outfit branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitPreviewDetails {
    /// Opaque 32-bit value consumed by the server in the preview branch.
    pub raw_state: u32,
}

/// Podium item locator used when the client customizes a displayed creature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodiumTarget {
    /// Map coordinates of the podium item receiving the appearance update.
    pub position: Position,
    /// Floor component of the podium item coordinates.
    pub floor: Floor,
    /// Advertised podium item type present at the addressed slot.
    pub item_id: u16,
    /// Stack slot of the addressed podium item.
    pub stack_position: u8,
}

/// Branch payload used for monster-podium outfit customization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodiumOutfitDetails {
    /// Locator of the podium item whose displayed creature should change.
    pub target: PodiumTarget,
    /// Mount selection and palette bytes applied to the podium appearance.
    pub mount: OutfitMountAppearance,
    /// Direction byte for the creature displayed on the podium.
    pub direction: u8,
    /// Raw visibility byte forwarded for the displayed podium creature.
    pub podium_visibility: u8,
}

/// Branch-specific payload selected by the `outfit_type` byte of [`UpdateOutfit`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateOutfitDetails {
    /// Regular outfit-window submission with mount, familiar, and randomize flags.
    Window(OutfitWindowDetails),
    /// Preview-only update with one opaque 32-bit field.
    Preview(OutfitPreviewDetails),
    /// Monster-podium customization bound to a podium target.
    Podium(PodiumOutfitDetails),
}

/// Packet sent by the client to commit an outfit-related change.
///
/// The first byte selects which outfit flow is being used on the wire. The
/// remaining payload then branches into the corresponding variant, covering the
/// regular outfit window, preview updates, or monster-podium customization.
///
/// # Examples
///
/// ```rust
/// use suon_protocol::packets::client::prelude::{
///     Decodable, OutfitPreviewDetails, PacketKind, UpdateOutfit, UpdateOutfitDetails,
/// };
///
/// let mut payload: &[u8] = &[1, 1, 0, 2, 3, 4, 5, 6, 0x78, 0x56, 0x34, 0x12];
/// let packet = UpdateOutfit::decode(PacketKind::UpdateOutfit, &mut payload).unwrap();
///
/// assert_eq!(packet.outfit_type, 1);
/// assert!(matches!(
///     packet.details,
///     UpdateOutfitDetails::Preview(OutfitPreviewDetails {
///         raw_state: 0x12345678,
///     })
/// ));
/// assert!(payload.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOutfit {
    /// Branch discriminator byte; the current server uses `0` window, `1` preview, and `2` podium.
    pub outfit_type: u8,
    /// Shared outfit appearance block decoded before the branch-specific payload.
    pub appearance: OutfitAppearance,
    /// Branch-specific remainder of the packet selected by `outfit_type`.
    pub details: UpdateOutfitDetails,
}

impl Decodable for UpdateOutfit {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let outfit_type = bytes.get_u8()?;
        let appearance = OutfitAppearance {
            look_type: bytes.get_u16()?,
            look_head: bytes.get_u8()?,
            look_body: bytes.get_u8()?,
            look_legs: bytes.get_u8()?,
            look_feet: bytes.get_u8()?,
            look_addons: bytes.get_u8()?,
        };

        let details = match outfit_type {
            0 => UpdateOutfitDetails::Window(OutfitWindowDetails {
                mount: OutfitMountAppearance {
                    mount_id: bytes.get_u16()?,
                    mount_head: bytes.get_u8()?,
                    mount_body: bytes.get_u8()?,
                    mount_legs: bytes.get_u8()?,
                    mount_feet: bytes.get_u8()?,
                },
                set_mount: bytes.get_bool()?,
                familiar_look_type: bytes.get_u16()?,
                randomize_mount: bytes.get_bool()?,
            }),
            1 => UpdateOutfitDetails::Preview(OutfitPreviewDetails {
                raw_state: bytes.get_u32()?,
            }),
            2 => UpdateOutfitDetails::Podium(PodiumOutfitDetails {
                target: PodiumTarget {
                    position: bytes.get_position()?,
                    floor: bytes.get_floor()?,
                    item_id: bytes.get_u16()?,
                    stack_position: bytes.get_u8()?,
                },
                mount: OutfitMountAppearance {
                    mount_id: bytes.get_u16()?,
                    mount_head: bytes.get_u8()?,
                    mount_body: bytes.get_u8()?,
                    mount_legs: bytes.get_u8()?,
                    mount_feet: bytes.get_u8()?,
                },
                direction: bytes.get_u8()?,
                podium_visibility: bytes.get_u8()?,
            }),
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "outfit_type",
                    value,
                });
            }
        };

        Ok(Self {
            outfit_type,
            appearance,
            details,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_update_outfit_window_variant() {
        let mut payload: &[u8] = &[0, 1, 0, 2, 3, 4, 5, 6, 7, 0, 8, 9, 10, 11, 1, 12, 0, 1];
        let packet = UpdateOutfit::decode(PacketKind::UpdateOutfit, &mut payload).unwrap();

        assert_eq!(packet.appearance.look_type, 1);
        assert!(matches!(
            packet.details,
            UpdateOutfitDetails::Window(OutfitWindowDetails {
                mount: OutfitMountAppearance {
                    mount_id: 7,
                    mount_head: 8,
                    mount_body: 9,
                    mount_legs: 10,
                    mount_feet: 11,
                },
                set_mount: true,
                familiar_look_type: 12,
                randomize_mount: true,
            })
        ));
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_update_outfit_preview_variant() {
        let mut payload: &[u8] = &[1, 1, 0, 2, 3, 4, 5, 6, 0x78, 0x56, 0x34, 0x12];

        let packet = UpdateOutfit::decode(PacketKind::UpdateOutfit, &mut payload).unwrap();

        assert_eq!(packet.outfit_type, 1);
        assert_eq!(
            packet.details,
            UpdateOutfitDetails::Preview(OutfitPreviewDetails {
                raw_state: 0x12345678,
            })
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_update_outfit_podium_variant() {
        let mut payload: &[u8] = &[
            2, 1, 0, 2, 3, 4, 5, 6, 0x34, 0x12, 0x78, 0x56, 7, 0xBC, 0x9A, 8, 10, 0, 11, 12, 13,
            14, 2, 9,
        ];

        let packet = UpdateOutfit::decode(PacketKind::UpdateOutfit, &mut payload).unwrap();

        assert!(matches!(
            packet.details,
            UpdateOutfitDetails::Podium(PodiumOutfitDetails {
                target: PodiumTarget {
                    position: Position {
                        x: 0x1234,
                        y: 0x5678
                    },
                    floor: Floor { z: 7 },
                    item_id: 0x9ABC,
                    stack_position: 8,
                },
                mount: OutfitMountAppearance {
                    mount_id: 10,
                    mount_head: 11,
                    mount_body: 12,
                    mount_legs: 13,
                    mount_feet: 14,
                },
                direction: 2,
                podium_visibility: 9,
            })
        ));
        assert!(payload.is_empty());
    }
}
