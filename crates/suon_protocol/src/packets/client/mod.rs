use thiserror::Error;

mod accept_market_offer;
mod accept_trade;
mod browse_field;
mod browse_market;
mod cancel_market_offer;
mod cancel_steps;
mod cancel_target_and_trail;
mod change_podium;
mod change_shared_party_experience;
mod channels;
mod close_container;
mod close_trade;
mod create_buddy;
mod create_market_offer;
mod create_private_channel;
mod delete_buddy;
mod equip_item;
mod face;
mod inspect_trade;
mod invite_private_channel;
mod invite_to_party;
mod invite_to_private_channel;
mod join_channel;
mod join_party;
mod keep_alive;
mod leave_channel;
mod leave_market;
mod leave_npc_channel;
mod leave_party;
mod leave_npc_shop;
mod logout;
mod look_at;
mod look_in_battle_list;
mod look_in_npc_shop;
mod modal_window_answer;
mod move_up_container;
mod offer_trade;
mod pass_party_leadership;
mod ping_latency;
mod purchase_npc_shop;
mod refresh_container;
mod remove_from_private_channel;
mod revoke_party_invite;
mod rotate_item;
mod rule_violation_report;
mod say;
mod seek_in_container;
mod sell_npc_shop;
mod step;
mod steps;
mod submit_house_window;
mod submit_text_window;
mod target;
mod throw_item;
mod trail;
mod update_buddy;
mod update_fight_modes;
mod update_outfit;
mod use_item;
mod use_item_with_creature;
mod use_item_with_target;
mod wrap_item;

pub mod prelude {
    pub use super::{
        Decodable, DecodableError, PacketKind,
        accept_market_offer::AcceptMarketOfferPacket,
        accept_trade::AcceptTradePacket,
        browse_field::BrowseFieldPacket,
        browse_market::BrowseMarketPacket,
        cancel_market_offer::CancelMarketOfferPacket,
        cancel_steps::CancelStepsPacket,
        cancel_target_and_trail::CancelTargetAndTrailPacket,
        change_podium::ChangePodiumPacket,
        change_shared_party_experience::ChangeSharedPartyExperiencePacket,
        channels::ChannelsPacket,
        close_container::CloseContainerPacket,
        close_trade::CloseTradePacket,
        create_buddy::CreateBuddyPacket,
        create_market_offer::{CreateMarketOfferPacket, MarketOfferKind},
        create_private_channel::CreatePrivateChannelPacket,
        delete_buddy::DeleteBuddyPacket,
        equip_item::EquipItemPacket,
        face::FacePacket,
        inspect_trade::InspectTradePacket,
        invite_private_channel::InvitePrivateChannelPacket,
        invite_to_party::InviteToPartyPacket,
        invite_to_private_channel::InviteToPrivateChannelPacket,
        join_channel::JoinChannelPacket,
        join_party::JoinPartyPacket,
        keep_alive::KeepAlivePacket,
        leave_channel::LeaveChannelPacket,
        leave_market::LeaveMarketPacket,
        leave_npc_channel::LeaveNpcChannelPacket,
        leave_party::LeavePartyPacket,
        leave_npc_shop::LeaveNpcShopPacket,
        logout::LogoutPacket,
        look_at::LookAtPacket,
        look_in_battle_list::LookInBattleListPacket,
        look_in_npc_shop::LookInNpcShopPacket,
        modal_window_answer::ModalWindowAnswerPacket,
        move_up_container::MoveUpContainerPacket,
        offer_trade::OfferTradePacket,
        pass_party_leadership::PassPartyLeadershipPacket,
        ping_latency::PingLatencyPacket,
        purchase_npc_shop::PurchaseNpcShopPacket,
        refresh_container::RefreshContainerPacket,
        remove_from_private_channel::RemoveFromPrivateChannelPacket,
        revoke_party_invite::RevokePartyInvitePacket,
        rotate_item::RotateItemPacket,
        rule_violation_report::RuleViolationReportPacket,
        say::{SayPacket, SpeakClass},
        seek_in_container::SeekInContainerPacket,
        sell_npc_shop::SellNpcShopPacket,
        step::StepPacket,
        steps::StepsPacket,
        submit_house_window::SubmitHouseWindowPacket,
        submit_text_window::SubmitTextWindowPacket,
        target::TargetPacket,
        throw_item::ThrowItemPacket,
        trail::TrailPacket,
        update_buddy::UpdateBuddyPacket,
        update_fight_modes::{ChaseMode, FightMode, SecureMode, UpdateFightModesPacket},
        update_outfit::{
            OutfitAppearance, OutfitMountAppearance, OutfitPreviewDetails, OutfitWindowDetails,
            PodiumOutfitDetails, PodiumTarget, UpdateOutfitDetails, UpdateOutfitPacket,
        },
        use_item::UseItemPacket,
        use_item_with_creature::UseItemWithCreaturePacket,
        use_item_with_target::UseItemWithTargetPacket,
        wrap_item::WrapItemPacket,
    };
}

/// Errors that can occur while decoding a packet.
#[derive(Debug, Error)]
pub enum DecodableError {
    /// Wraps a lower-level decoding error.
    #[error("failed to decode packet: {0}")]
    Decoder(#[from] crate::packets::decoder::DecoderError),

    /// The payload contained an unsupported value for a typed packet field.
    #[error("invalid value {value} for field '{field}'")]
    InvalidFieldValue {
        /// Logical field name being decoded.
        field: &'static str,
        /// Raw value received from the wire.
        value: u8,
    },
}

/// Represents a packet that can be decoded from a binary buffer.
///
/// This trait defines how a packet is reconstructed from raw bytes received
/// over a network or read from storage. Each packet type has a unique [`PacketKind`]
/// that identifies it and allows the system to dispatch the correct decoding logic.
///
/// # Associated Constant
/// - [`Self::KIND`]: The unique [`PacketKind`] that identifies this packet type.
///
/// # Methods
/// - [`Self::decode`]: Decodes the packet instance from a raw byte slice.
///
/// # Example
/// ```
/// use suon_protocol::packets::client::{Decodable, DecodableError, PacketKind};
///
/// struct LoginPacket {
///     username: String,
/// }
///
/// impl Decodable for LoginPacket {
///     const KIND: PacketKind = PacketKind::Login;
///
///     fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError> {
///         use suon_protocol::packets::decoder::Decoder;
///
///         let username = (&mut *bytes).get_string()?;
///         Ok(LoginPacket { username })
///     }
/// }
///
/// let mut buffer: &[u8] = &[5, 0, b'A', b'l', b'i', b'c', b'e'];
/// let packet = LoginPacket::decode(&mut buffer).unwrap();
///
/// assert_eq!(packet.username, "Alice");
/// ```
///
/// This trait is typically paired with the server-side
/// [`crate::packets::server::Encodable`] trait to allow symmetric serialization
/// and deserialization of packet types.
pub trait Decodable: Sized {
    /// Unique kind identifier for this packet type.
    const KIND: PacketKind;

    /// Returns whether this packet type can be decoded from the provided wire kind.
    fn accepts_kind(kind: PacketKind) -> bool {
        kind == Self::KIND
    }

    /// Decodes the packet instance from a raw byte slice.
    ///
    /// Implementers should read the buffer according to the expected packet structure.
    /// Returns an error if the buffer is incomplete or contains invalid data.
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError>;

    /// Decodes the packet instance while preserving the originating wire kind.
    fn decode_with_kind(_: PacketKind, bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Self::decode(bytes)
    }
}

/// Defines the possible kinds or categories of network packets.
///
/// Each [`PacketKind`] corresponds to a specific packet type that implements
/// the [`Decodable`] trait. This allows the system to determine how to
/// deserialize and distinguish different packet variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketKind {
    /// Internal packet sent by the client as the **first message**.
    ServerName = 0,

    /// Sent when a client attempts to log in.
    Login = 10,
    /// Sent when a client logs out.
    Logout = 20,

    /// Requests a multi-step path.
    Steps = 100,
    /// Requests a one-tile step to the north.
    StepNorth = 101,
    /// Requests a one-tile step to the east.
    StepEast = 102,
    /// Requests a one-tile step to the south.
    StepSouth = 103,
    /// Requests a one-tile step to the west.
    StepWest = 104,
    /// Requests that any active step sequence be canceled.
    CancelSteps = 105,
    /// Requests a one-tile step to the north-east.
    StepNorthEast = 106,
    /// Requests a one-tile step to the south-east.
    StepSouthEast = 107,
    /// Requests a one-tile step to the south-west.
    StepSouthWest = 108,
    /// Requests a one-tile step to the north-west.
    StepNorthWest = 109,

    /// Faces north.
    FaceNorth = 111,
    /// Faces east.
    FaceEast = 112,
    /// Faces south.
    FaceSouth = 113,
    /// Faces west.
    FaceWest = 114,

    /// Sent to measure latency between client and server.
    PingLatency = 29,
    /// Keeps the connection alive.
    KeepAlive = 30,

    /// Equips an item directly from the client interface.
    EquipItem = 119,
    /// Throws or moves an item from one tile to another.
    ThrowItem = 120,
    /// Looks at an item shown in the NPC shop window.
    LookInNpcShop = 121,
    /// Purchases an item from an NPC shop.
    PurchaseNpcShop = 122,
    /// Sells an item to an NPC shop.
    SellNpcShop = 123,
    /// Leaves the NPC shop window.
    LeaveNpcShop = 124,

    /// Offers an item for trade to another player.
    OfferTrade = 125,
    /// Inspects one of the items shown in the trade window.
    InspectTrade = 126,
    /// Accepts the current trade.
    AcceptTrade = 127,
    /// Closes the current trade.
    CloseTrade = 128,

    /// Uses an item directly.
    UseItem = 130,
    /// Uses an item on another target.
    UseItemWithTarget = 131,
    /// Uses an item on a creature.
    UseItemWithCreature = 132,
    /// Rotates an item.
    RotateItem = 133,
    /// Opens podium editing for an item.
    ChangePodium = 134,
    /// Closes a container.
    CloseContainer = 135,
    /// Moves up one level in a container view.
    MoveUpContainer = 136,
    /// Submits a text window.
    SubmitTextWindow = 137,
    /// Submits a house window.
    SubmitHouseWindow = 138,
    /// Wraps an item.
    WrapItem = 139,
    /// Looks at a thing on the map.
    LookAt = 140,
    /// Looks at a creature from the battle list.
    LookInBattleList = 141,

    /// Sends spoken text.
    Say = 150,
    /// Lists the available channels.
    Channels = 151,
    /// Opens a channel.
    JoinChannel = 152,
    /// Leaves a channel.
    LeaveChannel = 153,
    /// Invites or opens a private conversation with a receiver.
    InvitePrivateChannel = 154,
    /// Leaves the NPC channel.
    LeaveNpcChannel = 158,
    /// Changes fight modes.
    UpdateFightModes = 160,
    /// Targets a creature for attack.
    Target = 161,
    /// Trails a creature.
    Trail = 162,

    /// Creates a buddy entry.
    CreateBuddy = 220,
    /// Deletes a buddy entry.
    DeleteBuddy = 221,
    /// Updates a buddy entry.
    UpdateBuddy = 222,

    /// Invites a player to the party.
    InviteToParty = 163,
    /// Joins a party through a target player's invitation.
    JoinParty = 164,
    /// Revokes a previously sent party invite.
    RevokePartyInvite = 165,
    /// Passes party leadership to another player.
    PassPartyLeadership = 166,
    /// Leaves the current party.
    LeaveParty = 167,
    /// Changes the shared party experience state.
    ChangeSharedPartyExperience = 168,
    /// Creates a private channel.
    CreatePrivateChannel = 170,
    /// Invites a player to a private channel.
    InviteToPrivateChannel = 171,
    /// Removes a player from a private channel.
    RemoveFromPrivateChannel = 172,

    /// Cancels both target and trail states.
    CancelTargetAndTrail = 190,
    /// Refreshes an open container.
    RefreshContainer = 202,
    /// Browses a field.
    BrowseField = 203,
    /// Seeks to an index inside a container.
    SeekInContainer = 204,
    /// Changes outfit or podium appearance.
    UpdateOutfit = 211,
    /// Reports a rule violation.
    RuleViolationReport = 242,

    /// Leaves the market view.
    LeaveMarket = 244,
    /// Browses a market category, own offers, or own history.
    BrowseMarket = 245,
    /// Creates a market offer.
    CreateMarketOffer = 246,
    /// Cancels a market offer.
    CancelMarketOffer = 247,
    /// Accepts a market offer.
    AcceptMarketOffer = 248,

    /// Answers a modal window.
    ModalWindowAnswer = 249,
}

impl TryFrom<u8> for PacketKind {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::ServerName),

            10 => Ok(Self::Login),
            20 => Ok(Self::Logout),

            29 => Ok(Self::PingLatency),
            30 => Ok(Self::KeepAlive),
            119 => Ok(Self::EquipItem),
            120 => Ok(Self::ThrowItem),
            121 => Ok(Self::LookInNpcShop),
            122 => Ok(Self::PurchaseNpcShop),
            123 => Ok(Self::SellNpcShop),
            124 => Ok(Self::LeaveNpcShop),

            100 => Ok(Self::Steps),
            101 => Ok(Self::StepNorth),
            102 => Ok(Self::StepEast),
            103 => Ok(Self::StepSouth),
            104 => Ok(Self::StepWest),
            105 => Ok(Self::CancelSteps),
            106 => Ok(Self::StepNorthEast),
            107 => Ok(Self::StepSouthEast),
            108 => Ok(Self::StepSouthWest),
            109 => Ok(Self::StepNorthWest),

            111 => Ok(Self::FaceNorth),
            112 => Ok(Self::FaceEast),
            113 => Ok(Self::FaceSouth),
            114 => Ok(Self::FaceWest),

            125 => Ok(Self::OfferTrade),
            126 => Ok(Self::InspectTrade),
            127 => Ok(Self::AcceptTrade),
            128 => Ok(Self::CloseTrade),
            130 => Ok(Self::UseItem),
            131 => Ok(Self::UseItemWithTarget),
            132 => Ok(Self::UseItemWithCreature),
            133 => Ok(Self::RotateItem),
            134 => Ok(Self::ChangePodium),
            135 => Ok(Self::CloseContainer),
            136 => Ok(Self::MoveUpContainer),
            137 => Ok(Self::SubmitTextWindow),
            138 => Ok(Self::SubmitHouseWindow),
            139 => Ok(Self::WrapItem),
            140 => Ok(Self::LookAt),
            141 => Ok(Self::LookInBattleList),
            150 => Ok(Self::Say),
            151 => Ok(Self::Channels),
            152 => Ok(Self::JoinChannel),
            153 => Ok(Self::LeaveChannel),
            154 => Ok(Self::InvitePrivateChannel),
            158 => Ok(Self::LeaveNpcChannel),
            160 => Ok(Self::UpdateFightModes),
            161 => Ok(Self::Target),
            162 => Ok(Self::Trail),

            163 => Ok(Self::InviteToParty),
            164 => Ok(Self::JoinParty),
            165 => Ok(Self::RevokePartyInvite),
            166 => Ok(Self::PassPartyLeadership),
            167 => Ok(Self::LeaveParty),
            168 => Ok(Self::ChangeSharedPartyExperience),
            170 => Ok(Self::CreatePrivateChannel),
            171 => Ok(Self::InviteToPrivateChannel),
            172 => Ok(Self::RemoveFromPrivateChannel),
            190 => Ok(Self::CancelTargetAndTrail),
            202 => Ok(Self::RefreshContainer),
            203 => Ok(Self::BrowseField),
            204 => Ok(Self::SeekInContainer),
            211 => Ok(Self::UpdateOutfit),

            220 => Ok(Self::CreateBuddy),
            221 => Ok(Self::DeleteBuddy),
            222 => Ok(Self::UpdateBuddy),
            242 => Ok(Self::RuleViolationReport),

            244 => Ok(Self::LeaveMarket),
            245 => Ok(Self::BrowseMarket),
            246 => Ok(Self::CreateMarketOffer),
            247 => Ok(Self::CancelMarketOffer),
            248 => Ok(Self::AcceptMarketOffer),

            249 => Ok(Self::ModalWindowAnswer),

            _ => Err(value),
        }
    }
}

impl std::fmt::Display for PacketKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} (0x{:02X})", self, *self as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Packet;

    impl Decodable for Packet {
        const KIND: PacketKind = PacketKind::PingLatency;

        fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError> {
            if bytes.is_empty() {
                Err(DecodableError::Decoder(
                    crate::packets::decoder::DecoderError::Incomplete {
                        expected: 1,
                        available: 0,
                    },
                ))
            } else {
                Ok(Packet)
            }
        }
    }

    #[test]
    fn decode_packet_returns_error_on_empty_buffer() {
        let mut buffer: &[u8] = &[];

        let error = Packet::decode(&mut buffer)
            .expect_err("Expected DecoderError::Incomplete for empty buffer");

        match error {
            DecodableError::Decoder(crate::packets::decoder::DecoderError::Incomplete {
                expected,
                available,
            }) => {
                assert!(
                    expected == 1,
                    "Expected 1 byte to be required, got {}",
                    expected
                );
                assert!(
                    available == 0,
                    "Expected 0 bytes available, got {}",
                    available
                );
            }
            other => {
                panic!("Unexpected error variant: {:?}", other);
            }
        }
    }

    #[test]
    fn decode_packet_succeeds_with_non_empty_buffer() {
        const PAYLOAD: &[u8] = &[42];

        let mut buffer: &[u8] = PAYLOAD;

        let packet_result = Packet::decode(&mut buffer);
        assert!(
            packet_result.is_ok(),
            "Decoding should succeed with non-empty buffer"
        );

        let packet = packet_result.unwrap();
        assert!(
            matches!(packet, Packet),
            "Decoded packet should be of type Packet"
        );
    }

    #[test]
    fn packet_kind_should_convert_from_wire_values_and_format_for_logs() {
        assert_eq!(
            PacketKind::try_from(30),
            Ok(PacketKind::KeepAlive),
            "Wire value 30 should decode to the keep-alive client packet kind"
        );
        assert_eq!(
            PacketKind::try_from(101),
            Ok(PacketKind::StepNorth),
            "Wire value 101 should decode to the step-north client packet kind"
        );
        assert_eq!(
            PacketKind::PingLatency.to_string(),
            "PingLatency (0x1D)",
            "Display should include both the variant name and hexadecimal id"
        );
        assert_eq!(
            PacketKind::try_from(222),
            Ok(PacketKind::UpdateBuddy),
            "Wire value 222 should decode to the update-buddy client packet kind"
        );
    }
}
