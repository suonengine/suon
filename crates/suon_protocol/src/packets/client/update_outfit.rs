//! Client set-outfit packet.

use suon_position::{floor::Floor, position::Position};

use crate::packets::decoder::Decoder;

use super::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitAppearance {
    pub look_type: u16,
    pub look_head: u8,
    pub look_body: u8,
    pub look_legs: u8,
    pub look_feet: u8,
    pub look_addons: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitMountAppearance {
    pub mount_id: u16,
    pub mount_head: Option<u8>,
    pub mount_body: Option<u8>,
    pub mount_legs: Option<u8>,
    pub mount_feet: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitWindowDetails {
    pub mount: OutfitMountAppearance,
    pub familiar_look_type: Option<u16>,
    pub randomize_mount: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutfitPreviewDetails {
    pub mount_head: u8,
    pub mount_body: u8,
    pub mount_legs: u8,
    pub mount_feet: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodiumTarget {
    pub position: Position,
    pub floor: Floor,
    pub item_id: u16,
    pub stack_position: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PodiumOutfitDetails {
    pub target: PodiumTarget,
    pub mount: OutfitMountAppearance,
    pub direction: u8,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateOutfitDetails {
    Window(OutfitWindowDetails),
    Preview(OutfitPreviewDetails),
    Podium(PodiumOutfitDetails),
    Unknown(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOutfitPacket {
    pub outfit_scope: u8,
    pub appearance: OutfitAppearance,
    pub details: UpdateOutfitDetails,
}

impl Decodable for UpdateOutfitPacket {
    const KIND: PacketKind = PacketKind::UpdateOutfit;

    fn decode(mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let outfit_scope = bytes.get_u8()?;
        let appearance = OutfitAppearance {
            look_type: bytes.get_u16()?,
            look_head: bytes.get_u8()?,
            look_body: bytes.get_u8()?,
            look_legs: bytes.get_u8()?,
            look_feet: bytes.get_u8()?,
            look_addons: bytes.get_u8()?,
        };

        let details = match outfit_scope {
            0 => UpdateOutfitDetails::Window(OutfitWindowDetails {
                mount: OutfitMountAppearance {
                    mount_id: bytes.get_u16()?,
                    mount_head: Some(bytes.get_u8()?),
                    mount_body: Some(bytes.get_u8()?),
                    mount_legs: Some(bytes.get_u8()?),
                    mount_feet: Some(bytes.get_u8()?),
                },
                familiar_look_type: Some(bytes.get_u16()?),
                randomize_mount: bytes.get_bool()?,
            }),
            1 => UpdateOutfitDetails::Preview(OutfitPreviewDetails {
                mount_head: bytes.get_u8()?,
                mount_body: bytes.get_u8()?,
                mount_legs: bytes.get_u8()?,
                mount_feet: bytes.get_u8()?,
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
                    mount_head: Some(bytes.get_u8()?),
                    mount_body: Some(bytes.get_u8()?),
                    mount_legs: Some(bytes.get_u8()?),
                    mount_feet: Some(bytes.get_u8()?),
                },
                direction: bytes.get_u8()?,
                visible: bytes.get_bool()?,
            }),
            _ => UpdateOutfitDetails::Unknown(outfit_scope),
        };

        Ok(Self {
            outfit_scope,
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
        let mut payload: &[u8] = &[0, 1, 0, 2, 3, 4, 5, 6, 7, 0, 8, 9, 10, 11, 12, 0, 1];
        let packet = UpdateOutfitPacket::decode(&mut payload).unwrap();
        assert_eq!(packet.appearance.look_type, 1);
        assert!(matches!(
            packet.details,
            UpdateOutfitDetails::Window(OutfitWindowDetails {
                mount: OutfitMountAppearance { mount_id: 7, .. },
                randomize_mount: true,
                ..
            })
        ));
    }
}
