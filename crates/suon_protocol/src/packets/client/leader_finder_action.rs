//! Client leader-finder-action packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Team-finder activity kind configured by the leader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TeamFinderActivity {
    /// Boss team listing.
    Boss { boss_id: u16 },

    /// Hunt team listing.
    Hunt {
        /// Hunt type selector.
        hunt_type: u16,
        /// Hunt area selector.
        hunt_area: u16,
    },

    /// Quest team listing.
    Quest { quest_id: u16 },

    /// Unknown team listing kind kept as a raw value.
    Other { team_type: u8 },
}

/// Team-finder listing definition sent by the leader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeamFinderListing {
    /// Minimum level allowed.
    pub minimum_level: u16,
    /// Maximum level allowed.
    pub maximum_level: u16,
    /// Packed vocation mask.
    pub vocation_ids: u8,
    /// Maximum team size.
    pub team_slots: u16,
    /// Remaining free slots.
    pub free_slots: u16,
    /// Whether the current party should be imported.
    pub include_current_party: bool,
    /// Listing timestamp sent by the client.
    pub timestamp: u32,
    /// Team-finder activity kind.
    pub activity: TeamFinderActivity,
}

/// Action requested by the leader-finder window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaderFinderActionKind {
    /// Requests the current leader-finder state.
    Open,
    /// Resets or removes the current listing.
    Reset,
    /// Updates an existing member request status.
    UpdateMemberStatus {
        /// Target member guid.
        member_guid: u32,
        /// New member status byte.
        member_status: u8,
    },
    /// Creates or replaces the current team-finder listing.
    CreateListing {
        /// Listing data sent by the client.
        listing: TeamFinderListing,
    },
}

/// Packet sent by the client to manage the leader-finder window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaderFinderAction {
    /// Action requested by the team leader.
    pub action: LeaderFinderActionKind,
}

impl Decodable for LeaderFinderAction {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let action = match bytes.get_u8()? {
            0 => LeaderFinderActionKind::Open,
            1 => LeaderFinderActionKind::Reset,
            2 => LeaderFinderActionKind::UpdateMemberStatus {
                member_guid: bytes.get_u32()?,
                member_status: bytes.get_u8()?,
            },
            3 => {
                let minimum_level = bytes.get_u16()?;
                let maximum_level = bytes.get_u16()?;
                let vocation_ids = bytes.get_u8()?;
                let team_slots = bytes.get_u16()?;
                let free_slots = bytes.get_u16()?;
                let include_current_party = bytes.get_u8()? != 0;
                let timestamp = bytes.get_u32()?;
                let team_type = bytes.get_u8()?;
                let activity = match team_type {
                    1 => TeamFinderActivity::Boss {
                        boss_id: bytes.get_u16()?,
                    },
                    2 => TeamFinderActivity::Hunt {
                        hunt_type: bytes.get_u16()?,
                        hunt_area: bytes.get_u16()?,
                    },
                    3 => TeamFinderActivity::Quest {
                        quest_id: bytes.get_u16()?,
                    },
                    other => TeamFinderActivity::Other { team_type: other },
                };

                LeaderFinderActionKind::CreateListing {
                    listing: TeamFinderListing {
                        minimum_level,
                        maximum_level,
                        vocation_ids,
                        team_slots,
                        free_slots,
                        include_current_party,
                        timestamp,
                        activity,
                    },
                }
            }
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "action",
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
    fn should_decode_leader_finder_member_status_update() {
        let mut payload: &[u8] = &[2, 0x78, 0x56, 0x34, 0x12, 3];

        let packet = LeaderFinderAction::decode(PacketKind::LeaderFinderAction, &mut payload)
            .expect("LeaderFinderAction packets should decode status updates");

        assert_eq!(
            packet.action,
            LeaderFinderActionKind::UpdateMemberStatus {
                member_guid: 0x12345678,
                member_status: 3,
            }
        );
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_leader_finder_hunt_listing() {
        let mut payload: &[u8] = &[
            3, 10, 0, 40, 0, 7, 5, 0, 2, 0, 1, 0x78, 0x56, 0x34, 0x12, 2, 9, 0, 4, 0,
        ];

        let packet = LeaderFinderAction::decode(PacketKind::LeaderFinderAction, &mut payload)
            .expect("LeaderFinderAction packets should decode listing creation");

        assert_eq!(
            packet.action,
            LeaderFinderActionKind::CreateListing {
                listing: TeamFinderListing {
                    minimum_level: 10,
                    maximum_level: 40,
                    vocation_ids: 7,
                    team_slots: 5,
                    free_slots: 2,
                    include_current_party: true,
                    timestamp: 0x12345678,
                    activity: TeamFinderActivity::Hunt {
                        hunt_type: 9,
                        hunt_area: 4,
                    },
                },
            }
        );
        assert!(payload.is_empty());
    }
}
