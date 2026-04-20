use bevy::prelude::*;
use log::warn;
use suon_database::prelude::*;
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::{BrowseMarketPacket, LeaveMarketPacket};

use crate::offer::{MarketItemsTable, MarketPlayersTable};

/// Links a client entity to a persistent player id for market lookups.
#[derive(Debug, Clone, Copy, Component, PartialEq, Eq)]
pub struct MarketPlayerRef {
    pub player_id: u32,
}

/// Tracks the current market UI state for a client entity.
#[derive(Debug, Clone, Component, Default, PartialEq, Eq)]
pub struct MarketSession {
    pub is_open: bool,
    pub last_browse: Option<MarketBrowseScope>,
}

/// Higher-level interpretation of the client's market browse action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarketBrowseScope {
    OwnOffers,
    OwnHistory,
    Item {
        item_id: u16,
        item_name: Option<String>,
    },
}

impl MarketBrowseScope {
    pub fn request_kind(&self) -> MarketRequestKind {
        match self {
            Self::OwnOffers => MarketRequestKind::OwnOffers,
            Self::OwnHistory => MarketRequestKind::OwnHistory,
            Self::Item { .. } => MarketRequestKind::Item,
        }
    }

    pub fn from_packet(
        packet: &BrowseMarketPacket,
        item_name: impl FnOnce(u16) -> Option<String>,
    ) -> Option<Self> {
        match MarketRequestKind::try_from(packet.request_kind).ok()? {
            MarketRequestKind::OwnOffers => Some(Self::OwnOffers),
            MarketRequestKind::OwnHistory => Some(Self::OwnHistory),
            MarketRequestKind::Item => {
                let item_id = packet.sprite_id?;
                Some(Self::Item {
                    item_id,
                    item_name: item_name(item_id),
                })
            }
        }
    }
}

/// Triggered when the client browses market data.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketBrowse {
    #[event_target]
    pub client: Entity,
    pub player_id: Option<u32>,
    pub player_name: Option<String>,
    pub scope: MarketBrowseScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketRequestKind {
    OwnOffers,
    OwnHistory,
    Item,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarketRequestKindError {
    pub value: u8,
}

impl std::fmt::Display for MarketRequestKindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported market request kind {}", self.value)
    }
}

impl std::error::Error for MarketRequestKindError {}

impl TryFrom<u8> for MarketRequestKind {
    type Error = MarketRequestKindError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::OwnOffers),
            1 => Ok(Self::OwnHistory),
            2..=u8::MAX => Ok(Self::Item),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TryMarketRequestKindFromPacketError {
    MissingItemId { request_kind: u8 },
}

impl std::fmt::Display for TryMarketRequestKindFromPacketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingItemId { request_kind } => {
                write!(
                    f,
                    "market browse request kind {request_kind} requires an item id"
                )
            }
        }
    }
}

impl std::error::Error for TryMarketRequestKindFromPacketError {}

pub struct MarketBrowsePlugin;

impl Plugin for MarketBrowsePlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_leave_market_packet)
            .add_observer(on_browse_market_packet);
    }
}

fn on_leave_market_packet(
    event: On<Packet<LeaveMarketPacket>>,
    mut commands: Commands,
    mut sessions: Query<&mut MarketSession>,
) {
    set_market_session(event.entity(), false, None, &mut sessions, &mut commands);
}

fn on_browse_market_packet(
    event: On<Packet<BrowseMarketPacket>>,
    mut commands: Commands,
    mut sessions: Query<&mut MarketSession>,
    player_refs: Query<&MarketPlayerRef>,
    players: Database<MarketPlayersTable>,
    items: Database<MarketItemsTable>,
) {
    let client = event.entity();
    let packet = event.packet();

    let Some(scope) =
        MarketBrowseScope::from_packet(packet, |item_id| items.name(item_id).map(str::to_owned))
    else {
        warn!(
            "Ignoring malformed market browse packet from entity {:?}: request_kind={}, \
             sprite_id={:?}",
            client, packet.request_kind, packet.sprite_id
        );
        return;
    };

    set_market_session(
        client,
        true,
        Some(scope.clone()),
        &mut sessions,
        &mut commands,
    );

    let player_id = player_refs.get(client).ok().map(|entry| entry.player_id);
    let player_name = player_id.and_then(|id| players.name(id).map(str::to_owned));

    commands.trigger(MarketBrowse {
        client,
        player_id,
        player_name,
        scope,
    });
}

fn set_market_session(
    client: Entity,
    is_open: bool,
    last_browse: Option<MarketBrowseScope>,
    sessions: &mut Query<&mut MarketSession>,
    commands: &mut Commands,
) {
    if let Ok(mut session) = sessions.get_mut(client) {
        session.is_open = is_open;
        session.last_browse = last_browse;
    } else {
        commands.entity(client).insert(MarketSession {
            is_open,
            last_browse,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_translate_browse_scope_to_own_offers() {
        let packet = BrowseMarketPacket {
            request_kind: 0,
            sprite_id: None,
        };

        assert_eq!(
            MarketBrowseScope::from_packet(&packet, |_| None),
            Some(MarketBrowseScope::OwnOffers)
        );
    }

    #[test]
    fn should_translate_browse_scope_to_named_item() {
        let mut items = MarketItemsTable::default();
        items.insert(crate::offer::MarketItem {
            id: 2160,
            name: "Crystal Coin".into(),
        });

        let packet = BrowseMarketPacket {
            request_kind: 3,
            sprite_id: Some(2160),
        };

        assert_eq!(
            MarketBrowseScope::from_packet(&packet, |item_id| {
                items.name(item_id).map(str::to_owned)
            }),
            Some(MarketBrowseScope::Item {
                item_id: 2160,
                item_name: Some("Crystal Coin".into())
            })
        );
    }

    #[test]
    fn should_reject_item_browse_without_item_id() {
        let packet = BrowseMarketPacket {
            request_kind: 5,
            sprite_id: None,
        };

        assert_eq!(MarketBrowseScope::from_packet(&packet, |_| None), None);
    }
}
