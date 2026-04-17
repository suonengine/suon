use thiserror::Error;

mod accept_market_offer;
mod accept_trade_offer;
mod add_imbuement;
mod aim_at_target;
mod browse_market;
mod buddy_group_action;
mod bug_report;
mod buy_charm_rune;
mod buy_store_offer;
mod cancel_market_offer;
mod cancel_rule_violation;
mod cancel_steps;
mod cancel_target_and_trail;
mod change_podium;
mod change_shared_party_experience;
mod channels;
mod character;
mod clear_imbuement;
mod close_container;
mod close_imbuing_window;
mod close_rule_violation;
mod close_trade;
mod collect_reward_chest;
mod configure_boss_slot;
mod create_buddy;
mod create_market_offer;
mod create_private_channel;
mod cyclopedia_house_auction;
mod delete_buddy;
mod disband_party;
mod enter_game;
mod equip_item;
mod exiva_restrictions;
mod extended_opcode;
mod face;
mod fight_modes;
mod forge_action;
mod forge_history;
mod friend_system_action;
mod get_reward_daily;
mod imbuements;
mod inspect_item_details;
mod inspect_object;
mod inspect_offer;
mod inspect_trade;
mod invite_private_channel;
mod invite_to_party;
mod invite_to_private_channel;
mod join_channel;
mod join_party;
mod keep_alive;
mod leader_finder_action;
mod leave_channel;
mod leave_market;
mod leave_npc_channel;
mod leave_npc_shop;
mod leave_party;
mod login;
mod logout;
mod look_at;
mod look_in_battle_list;
mod look_in_npc_shop;
mod loot_container;
mod map_aware_range;
mod member_finder_action;
mod modal_window_answer;
mod mount;
mod move_up_container;
mod offer_trade;
mod open_bestiary;
mod open_bestiary_overview;
mod open_bless_dialog;
mod open_bosstiary;
mod open_outfit_dialog;
mod open_parent_container;
mod open_prey_dialog;
mod open_quest_line;
mod open_quest_log;
mod open_reward_history;
mod open_reward_wall;
mod open_rule_violation;
mod open_store;
mod open_tracked_quest_log;
mod open_transaction_history;
mod open_wheel;
mod party_analyzer_action;
mod pass_party_leadership;
mod ping_latency;
mod prey_action;
mod purchase_npc_shop;
mod query_boss_slot_info;
mod query_depot_search_item;
mod query_highscores;
mod quick_loot;
mod quick_loot_filter;
mod remove_from_private_channel;
mod retrieve_depot_search;
mod revoke_party_invite;
mod rotate_item;
mod rule_violation_report;
mod save_wheel;
mod say;
mod search_bestiary;
mod seek_in_container;
mod sell_npc_shop;
mod server_name;
mod set_monster_podium;
mod set_typing_state;
mod stash_action;
mod step;
mod steps;
mod store;
mod submit_house_window;
mod submit_text_window;
mod target;
mod task_hunting_action;
mod teleport;
mod throw_item;
mod tile;
mod trail;
mod transaction_history;
mod transfer_coins;
mod update_buddy;
mod update_monster_tracker;
mod update_outfit;
mod use_item;
mod use_item_with_creature;
mod use_item_with_target;
mod wheel_gem;
mod wrap;

pub mod prelude {
    pub use super::{
        Decodable, DecodableError, PacketKind,
        accept_market_offer::AcceptMarketOffer,
        accept_trade_offer::AcceptTradeOffer,
        add_imbuement::AddImbuement,
        aim_at_target::AimAtTarget,
        browse_market::BrowseMarket,
        buddy_group_action::{BuddyGroup, BuddyGroupActionKind},
        bug_report::BugReport,
        buy_charm_rune::BuyCharmRune,
        buy_store_offer::BuyStoreOffer,
        cancel_market_offer::CancelMarketOffer,
        cancel_rule_violation::CancelRuleViolation,
        cancel_steps::CancelSteps,
        cancel_target_and_trail::CancelTargetAndTrail,
        change_podium::ChangePodium,
        change_shared_party_experience::ChangeSharedPartyExperience,
        channels::Channels,
        character::{CharacterInfo, Kind},
        clear_imbuement::RemoveImbuement,
        close_container::CloseContainer,
        close_imbuing_window::CloseImbuingWindow,
        close_rule_violation::CloseRuleViolation,
        close_trade::CloseTrade,
        collect_reward_chest::CollectRewardChest,
        configure_boss_slot::Bosstiary,
        create_buddy::CreateBuddy,
        create_market_offer::{CreateMarketOffer, MarketOfferKind},
        create_private_channel::CreatePrivateChannel,
        cyclopedia_house_auction::{CyclopediaHouseAuction, CyclopediaHouseAuctionAction},
        delete_buddy::DeleteBuddy,
        disband_party::DisbandParty,
        enter_game::EnterGame,
        equip_item::EquipItem,
        exiva_restrictions::ExivaRestrictions,
        extended_opcode::ExtendedOpcode,
        face::Face,
        fight_modes::{ChaseMode, FightMode, FightModes, SecureMode},
        forge_action::{ForgeAction, ForgeActionKind},
        forge_history::ForgeHistory,
        friend_system_action::FriendSystemAction,
        get_reward_daily::{DailyRewardItem, GetRewardDaily},
        imbuements::Imbuements,
        inspect_item_details::InspectItemDetails,
        inspect_object::{InspectObject, InspectObjectKind},
        inspect_offer::InspectOffer,
        inspect_trade::InspectTrade,
        invite_private_channel::InvitePrivateChannel,
        invite_to_party::InviteToParty,
        invite_to_private_channel::InviteToPrivateChannel,
        join_channel::JoinChannel,
        join_party::JoinParty,
        keep_alive::KeepAlive,
        leader_finder_action::{
            LeaderFinderAction, LeaderFinderActionKind, TeamFinderActivity, TeamFinderListing,
        },
        leave_channel::LeaveChannel,
        leave_market::LeaveMarket,
        leave_npc_channel::LeaveNpcChannel,
        leave_npc_shop::LeaveNpcShop,
        leave_party::LeaveParty,
        login::{
            DecryptedLogin, Login, LoginCredentials, LoginDecodeError, LoginDecoder, LoginHeader,
        },
        logout::Logout,
        look_at::LookAt,
        look_in_battle_list::LookInBattleList,
        look_in_npc_shop::LookInNpcShop,
        loot_container::{LootContainer, LootContainerAction},
        map_aware_range::MapAwareRange,
        member_finder_action::{MemberFinderAction, MemberFinderActionKind},
        modal_window_answer::ModalWindowAnswer,
        mount::Mount,
        move_up_container::MoveUpContainer,
        offer_trade::OfferTrade,
        open_bestiary::OpenBestiary,
        open_bestiary_overview::OpenBestiaryOverview,
        open_bless_dialog::OpenBlessDialog,
        open_bosstiary::OpenBosstiary,
        open_outfit_dialog::OpenOutfitDialog,
        open_parent_container::OpenParentContainer,
        open_prey_dialog::OpenPreyDialog,
        open_quest_line::OpenQuestLine,
        open_quest_log::OpenQuestLog,
        open_reward_history::OpenRewardHistory,
        open_reward_wall::OpenRewardWall,
        open_rule_violation::OpenRuleViolation,
        open_store::OpenStore,
        open_tracked_quest_log::OpenTrackedQuestLog,
        open_transaction_history::OpenTransactionHistory,
        open_wheel::OpenWheel,
        party_analyzer_action::{PartyAnalyzerAction, PartyAnalyzerActionKind},
        pass_party_leadership::PassPartyLeadership,
        ping_latency::PingLatency,
        prey_action::{PreyAction, PreyActionKind},
        purchase_npc_shop::PurchaseNpcShop,
        query_boss_slot_info::QueryBossSlotInfo,
        query_depot_search_item::QueryDepotSearchItem,
        query_highscores::{HighscoreQueryKind, QueryHighscores},
        quick_loot::{QuickLoot, QuickLootAction},
        quick_loot_filter::QuickLootFilter,
        remove_from_private_channel::RemoveFromPrivateChannel,
        retrieve_depot_search::RetrieveDepotSearch,
        revoke_party_invite::RevokePartyInvite,
        rotate_item::RotateItem,
        rule_violation_report::RuleViolationReport,
        save_wheel::SaveWheel,
        say::{Say, SpeakClass},
        search_bestiary::{BestiarySearchKind, SearchBestiary},
        seek_in_container::SeekInContainer,
        sell_npc_shop::SellNpcShop,
        server_name::ServerName,
        set_monster_podium::SetMonsterPodium,
        set_typing_state::SetTypingState,
        stash_action::{StashAction, StashActionKind},
        step::Step,
        steps::Steps,
        store::{BrowseStoreOffers, Store},
        submit_house_window::SubmitHouseWindow,
        submit_text_window::SubmitTextWindow,
        target::Target,
        task_hunting_action::TaskHuntingAction,
        teleport::Teleport,
        throw_item::ThrowItem,
        tile::Tile,
        trail::Trail,
        transaction_history::{Format, TransactionHistory},
        transfer_coins::TransferCoins,
        update_buddy::UpdateBuddy,
        update_monster_tracker::UpdateMonsterTracker,
        update_outfit::{
            OutfitAppearance, OutfitMountAppearance, OutfitPreviewDetails, OutfitWindowDetails,
            PodiumOutfitDetails, PodiumTarget, UpdateOutfit, UpdateOutfitDetails,
        },
        use_item::UseItem,
        use_item_with_creature::UseItemWithCreature,
        use_item_with_target::UseItemWithTarget,
        wheel_gem::WheelGem,
        wrap::Wrap,
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
/// over a network or read from storage. The [`PacketKind`] is passed at decode
/// time to allow the system to dispatch the correct decoding logic.
///
/// # Methods
/// - [`Self::decode`]: Decodes the packet instance from a raw byte slice.
///
/// # Example
/// ```
/// use suon_protocol::packets::client::{Decodable, DecodableError, PacketKind};
///
/// struct Login {
///     username: String,
/// }
///
/// impl Decodable for Login {
///     fn decode(_: PacketKind, bytes: &mut &[u8]) -> Result<Self, DecodableError> {
///         use suon_protocol::packets::decoder::Decoder;
///
///         let username = (&mut *bytes).get_string()?;
///         Ok(Login { username })
///     }
/// }
///
/// let mut buffer: &[u8] = &[5, 0, b'A', b'l', b'i', b'c', b'e'];
/// let packet = Login::decode(PacketKind::Login, &mut buffer).unwrap();
///
/// assert_eq!(packet.username, "Alice");
/// ```
///
/// This trait is typically paired with the server-side
/// [`crate::packets::server::Encodable`] trait to allow symmetric serialization
/// and deserialization of packet types.
pub trait Decodable: Sized {
    /// Decodes the packet instance from a raw byte slice.
    ///
    /// Implementers should read the buffer according to the expected packet structure.
    /// Returns an error if the buffer is incomplete or contains invalid data.
    fn decode(kind: PacketKind, bytes: &mut &[u8]) -> Result<Self, DecodableError>;
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
    /// Sent after the login handshake to enter the game world.
    EnterGame = 15,
    /// Sent when a client logs out.
    Logout = 20,
    /// Uses the stash system.
    StashAction = 40,
    /// Retrieves an item from the depot-search results.
    RetrieveDepotSearch = 41,
    /// Updates a cyclopedia monster-tracker entry.
    UpdateMonsterTracker = 42,
    /// Updates the party analyzer.
    PartyAnalyzerAction = 43,
    /// Manages the leader-finder state.
    LeaderFinderAction = 44,
    /// Manages the member-finder state.
    MemberFinderAction = 45,
    /// Sends an extended opcode payload.
    ExtendedOpcode = 50,
    /// Updates the map-aware range dimensions requested by the client.
    MapAwareRange = 51,
    /// Updates the typing-indicator state.
    SetTypingState = 56,

    /// Updates the inventory imbuement tracker state.
    Imbuements = 96,
    /// Opens the wheel window for a given owner.
    OpenWheel = 97,
    /// Saves the wheel state payload.
    SaveWheel = 98,

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
    /// Requests a teleport position.
    TeleportLegacy = 115,

    /// Sent to measure latency between client and server.
    PingLatency = 29,
    /// Keeps the connection alive.
    KeepAlive = 30,

    /// Equips an item directly from the current client context.
    EquipItem = 119,
    /// Throws or moves an item from one tile to another.
    ThrowItem = 120,
    /// Looks at an item listed by the NPC shop flow.
    LookInNpcShop = 121,
    /// Purchases an item from an NPC shop.
    PurchaseNpcShop = 122,
    /// Sells an item to an NPC shop.
    SellNpcShop = 123,
    /// Leaves the NPC shop flow.
    LeaveNpcShop = 124,

    /// Offers an item for trade to another player.
    OfferTrade = 125,
    /// Inspects one item entry from the trade payload.
    InspectTrade = 126,
    /// Accepts the current trade.
    AcceptTradeOffer = 127,
    /// Closes the current trade.
    CloseTrade = 128,
    /// Performs a friend-system action.
    FriendSystemAction = 129,

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
    /// Performs a quick-loot action.
    QuickLoot = 143,
    /// Updates a managed loot container.
    LootContainer = 144,
    /// Updates the quick-loot filter list.
    QuickLootFilter = 145,
    /// Queries one item entry from depot search.
    QueryDepotSearchItem = 148,
    /// Opens the parent container from a depot-search entry.
    OpenParentContainer = 149,

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
    /// Opens a rule-violation conversation.
    OpenRuleViolation = 155,
    /// Closes a rule-violation conversation.
    CloseRuleViolation = 156,
    /// Cancels an in-progress rule-violation conversation.
    CancelRuleViolation = 157,
    /// Leaves the NPC channel.
    LeaveNpcChannel = 158,
    /// Configures a monster podium entry.
    SetMonsterPodium = 159,
    /// Changes fight modes.
    FightModes = 160,
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
    /// Updates buddy groups.
    BuddyGroup = 223,
    /// Opens the bestiary main data view.
    OpenBestiary = 225,
    /// Opens the bestiary overview.
    OpenBestiaryOverview = 226,
    /// Searches the bestiary.
    SearchBestiary = 227,
    /// Buys or assigns a charm rune.
    BuyCharmRune = 228,
    /// Browses cyclopedia character information.
    BrowseCharacter = 229,
    /// Reports a bug.
    BugReport = 230,
    /// Sends a wheel gem action payload.
    WheelGem = 231,
    /// Performs a prey action.
    PreyAction = 235,
    /// Requests the prey dialog contents.
    OpenPreyDialog = 237,
    /// Transfers transferable coins to another character.
    TransferCoins = 239,
    /// Opens the quest log.
    OpenQuestLog = 240,
    /// Opens a quest line.
    OpenQuestLine = 241,

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
    /// Disbands the current party.
    DisbandParty = 169,
    /// Creates a private channel.
    CreatePrivateChannel = 170,
    /// Invites a player to a private channel.
    InviteToPrivateChannel = 171,
    /// Removes a player from a private channel.
    RemoveFromPrivateChannel = 172,
    /// Performs a cyclopedia house-auction action.
    CyclopediaHouseAuction = 173,
    /// Opens the bosstiary overview data.
    OpenBosstiary = 174,
    /// Queries bosstiary slot information.
    QueryBossSlotInfo = 175,
    /// Configures a bosstiary boss slot.
    Bosstiary = 176,
    /// Queries highscores data.
    QueryHighscores = 177,
    /// Performs a task-hunting action.
    TaskHuntingAction = 186,

    /// Cancels both target and trail states.
    CancelTargetAndTrail = 190,
    /// Performs a forge action.
    ForgeAction = 191,
    /// Browses forge history.
    ForgeHistory = 192,
    /// Updates target-aim spell bindings.
    AimAtTarget = 200,
    /// Updates exiva restrictions in no-pvp worlds.
    ExivaRestrictions = 202,
    /// Browses a field.
    BrowseTile = 203,
    /// Seeks to an index inside a container.
    SeekInContainer = 204,
    /// Inspects an object from the map, trade, or cyclopedia.
    InspectObject = 205,
    /// Opens the blessings dialog.
    OpenBlessDialog = 207,
    /// Opens the tracked quest log.
    OpenTrackedQuestLog = 208,
    /// Requests the outfit selection data.
    OpenOutfitDialog = 210,
    /// Changes outfit or podium appearance.
    UpdateOutfit = 211,
    /// Enables or disables the current mount state.
    Mount = 212,
    /// Adds an imbuement to a slot.
    AddImbuement = 213,
    /// Clears an imbuement slot.
    RemoveImbuement = 214,
    /// Closes the imbuing window.
    CloseImbuingWindow = 215,
    /// Opens the reward wall.
    OpenRewardWall = 216,
    /// Opens reward history.
    OpenRewardHistory = 217,
    /// Claims the selected daily reward payload.
    GetRewardDaily = 218,
    /// Reports a rule violation.
    RuleViolationReport = 242,
    /// Inspects detailed item info.
    InspectItemDetails = 243,
    /// Requests an offer description payload.
    InspectOffer = 232,

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
    /// Collects items from the reward chest.
    CollectRewardChest = 255,
    /// Opens the in-game store.
    OpenStore = 250,
    /// Browses store offers using a typed action payload.
    BrowseStoreOffers = 251,
    /// Attempts to buy or redeem a store offer.
    BuyStoreOffer = 252,
    /// Opens the store transaction-history view.
    OpenTransactionHistory = 253,
    /// Browses a page from the transaction-history view.
    BrowseTransactionHistory = 254,
    /// Requests a teleport position.
    Teleport = 201,
}

impl TryFrom<u8> for PacketKind {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::ServerName),

            10 => Ok(Self::Login),
            15 => Ok(Self::EnterGame),
            20 => Ok(Self::Logout),
            40 => Ok(Self::StashAction),
            41 => Ok(Self::RetrieveDepotSearch),
            42 => Ok(Self::UpdateMonsterTracker),
            43 => Ok(Self::PartyAnalyzerAction),
            44 => Ok(Self::LeaderFinderAction),
            45 => Ok(Self::MemberFinderAction),
            50 => Ok(Self::ExtendedOpcode),
            51 => Ok(Self::MapAwareRange),
            56 => Ok(Self::SetTypingState),

            29 => Ok(Self::PingLatency),
            30 => Ok(Self::KeepAlive),
            96 => Ok(Self::Imbuements),
            97 => Ok(Self::OpenWheel),
            98 => Ok(Self::SaveWheel),
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
            115 => Ok(Self::TeleportLegacy),

            125 => Ok(Self::OfferTrade),
            126 => Ok(Self::InspectTrade),
            127 => Ok(Self::AcceptTradeOffer),
            128 => Ok(Self::CloseTrade),
            129 => Ok(Self::FriendSystemAction),
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
            143 => Ok(Self::QuickLoot),
            144 => Ok(Self::LootContainer),
            145 => Ok(Self::QuickLootFilter),
            148 => Ok(Self::QueryDepotSearchItem),
            149 => Ok(Self::OpenParentContainer),
            150 => Ok(Self::Say),
            151 => Ok(Self::Channels),
            152 => Ok(Self::JoinChannel),
            153 => Ok(Self::LeaveChannel),
            154 => Ok(Self::InvitePrivateChannel),
            155 => Ok(Self::OpenRuleViolation),
            156 => Ok(Self::CloseRuleViolation),
            157 => Ok(Self::CancelRuleViolation),
            158 => Ok(Self::LeaveNpcChannel),
            159 => Ok(Self::SetMonsterPodium),
            160 => Ok(Self::FightModes),
            161 => Ok(Self::Target),
            162 => Ok(Self::Trail),

            163 => Ok(Self::InviteToParty),
            164 => Ok(Self::JoinParty),
            165 => Ok(Self::RevokePartyInvite),
            166 => Ok(Self::PassPartyLeadership),
            167 => Ok(Self::LeaveParty),
            168 => Ok(Self::ChangeSharedPartyExperience),
            169 => Ok(Self::DisbandParty),
            170 => Ok(Self::CreatePrivateChannel),
            171 => Ok(Self::InviteToPrivateChannel),
            172 => Ok(Self::RemoveFromPrivateChannel),
            173 => Ok(Self::CyclopediaHouseAuction),
            174 => Ok(Self::OpenBosstiary),
            175 => Ok(Self::QueryBossSlotInfo),
            176 => Ok(Self::Bosstiary),
            177 => Ok(Self::QueryHighscores),
            186 => Ok(Self::TaskHuntingAction),
            190 => Ok(Self::CancelTargetAndTrail),
            191 => Ok(Self::ForgeAction),
            192 => Ok(Self::ForgeHistory),
            200 => Ok(Self::AimAtTarget),
            202 => Ok(Self::ExivaRestrictions),
            203 => Ok(Self::BrowseTile),
            204 => Ok(Self::SeekInContainer),
            205 => Ok(Self::InspectObject),
            207 => Ok(Self::OpenBlessDialog),
            208 => Ok(Self::OpenTrackedQuestLog),
            210 => Ok(Self::OpenOutfitDialog),
            211 => Ok(Self::UpdateOutfit),
            212 => Ok(Self::Mount),
            213 => Ok(Self::AddImbuement),
            214 => Ok(Self::RemoveImbuement),
            215 => Ok(Self::CloseImbuingWindow),
            216 => Ok(Self::OpenRewardWall),
            217 => Ok(Self::OpenRewardHistory),
            218 => Ok(Self::GetRewardDaily),
            201 => Ok(Self::Teleport),

            220 => Ok(Self::CreateBuddy),
            221 => Ok(Self::DeleteBuddy),
            222 => Ok(Self::UpdateBuddy),
            223 => Ok(Self::BuddyGroup),
            225 => Ok(Self::OpenBestiary),
            226 => Ok(Self::OpenBestiaryOverview),
            227 => Ok(Self::SearchBestiary),
            228 => Ok(Self::BuyCharmRune),
            229 => Ok(Self::BrowseCharacter),
            230 => Ok(Self::BugReport),
            231 => Ok(Self::WheelGem),
            232 => Ok(Self::InspectOffer),
            235 => Ok(Self::PreyAction),
            237 => Ok(Self::OpenPreyDialog),
            239 => Ok(Self::TransferCoins),
            240 => Ok(Self::OpenQuestLog),
            241 => Ok(Self::OpenQuestLine),
            242 => Ok(Self::RuleViolationReport),
            243 => Ok(Self::InspectItemDetails),

            244 => Ok(Self::LeaveMarket),
            245 => Ok(Self::BrowseMarket),
            246 => Ok(Self::CreateMarketOffer),
            247 => Ok(Self::CancelMarketOffer),
            248 => Ok(Self::AcceptMarketOffer),

            249 => Ok(Self::ModalWindowAnswer),
            250 => Ok(Self::OpenStore),
            251 => Ok(Self::BrowseStoreOffers),
            252 => Ok(Self::BuyStoreOffer),
            253 => Ok(Self::OpenTransactionHistory),
            254 => Ok(Self::BrowseTransactionHistory),
            255 => Ok(Self::CollectRewardChest),

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
        fn decode(_: PacketKind, bytes: &mut &[u8]) -> Result<Self, DecodableError> {
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

        let error = Packet::decode(PacketKind::PingLatency, &mut buffer)
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

        let packet_result = Packet::decode(PacketKind::PingLatency, &mut buffer);
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
