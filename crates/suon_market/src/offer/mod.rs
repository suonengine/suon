use bevy::prelude::*;
use log::{debug, warn};
use std::{
    collections::HashMap,
    num::Wrapping,
    time::{Duration, SystemTime},
};
use suon_database::prelude::*;
use suon_macros::Table;
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::{
    AcceptMarketOfferPacket, CancelMarketOfferPacket, CreateMarketOfferPacket, MarketOfferKind,
};

use crate::{
    browse::MarketPlayerRef,
    history::{MarketHistoryAction, MarketHistoryEntry, MarketHistorySequence, MarketHistoryTable},
    persistence::{MarketDirty, MarketSettings},
};

/// A player name snapshot loaded for market lookups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerName {
    pub id: u32,
    pub name: String,
}

/// A market item snapshot loaded for market lookups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketItem {
    pub id: u16,
    pub name: String,
}

/// Whether a market offer is buying or selling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketTradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseMarketTradeSideError {
    pub value: String,
}

impl std::fmt::Display for ParseMarketTradeSideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported market trade side '{}'", self.value)
    }
}

impl std::error::Error for ParseMarketTradeSideError {}

impl std::fmt::Display for MarketTradeSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        };

        f.write_str(value)
    }
}

impl std::str::FromStr for MarketTradeSide {
    type Err = ParseMarketTradeSideError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "buy" => Ok(Self::Buy),
            "sell" => Ok(Self::Sell),
            other => Err(ParseMarketTradeSideError {
                value: other.to_string(),
            }),
        }
    }
}

impl From<MarketOfferKind> for MarketTradeSide {
    fn from(value: MarketOfferKind) -> Self {
        match value {
            MarketOfferKind::Buy => Self::Buy,
            MarketOfferKind::Sell => Self::Sell,
        }
    }
}

/// Stable identifier used by the client protocol for existing offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarketOfferId {
    pub timestamp: SystemTime,
    pub counter: u16,
}

/// Cached market offer row loaded from an ORM source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketOffer {
    pub id: MarketOfferId,
    pub item_id: u16,
    pub player_id: u32,
    pub amount: u16,
    pub price: u64,
    pub side: MarketTradeSide,
    pub is_anonymous: bool,
}

/// Table containing player names keyed by player id.
#[derive(Debug, Default, Table)]
pub struct MarketPlayersTable {
    by_id: HashMap<u32, String>,
}

impl MarketPlayersTable {
    pub fn replace(&mut self, rows: impl IntoIterator<Item = PlayerName>) {
        self.by_id = rows.into_iter().map(|row| (row.id, row.name)).collect();
    }

    pub fn insert(&mut self, row: PlayerName) -> Option<String> {
        self.by_id.insert(row.id, row.name)
    }

    pub fn name(&self, player_id: u32) -> Option<&str> {
        self.by_id.get(&player_id).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn rows(&self) -> Vec<PlayerName> {
        self.by_id
            .iter()
            .map(|(id, name)| PlayerName {
                id: *id,
                name: name.clone(),
            })
            .collect()
    }
}

impl SnapshotTable for MarketPlayersTable {
    type Row = PlayerName;

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketPlayersTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketPlayersTable::rows(self)
    }
}

/// Table containing market item names keyed by item id.
#[derive(Debug, Default, Table)]
pub struct MarketItemsTable {
    by_id: HashMap<u16, String>,
}

impl MarketItemsTable {
    pub fn replace(&mut self, rows: impl IntoIterator<Item = MarketItem>) {
        self.by_id = rows.into_iter().map(|row| (row.id, row.name)).collect();
    }

    pub fn insert(&mut self, row: MarketItem) -> Option<String> {
        self.by_id.insert(row.id, row.name)
    }

    pub fn name(&self, item_id: u16) -> Option<&str> {
        self.by_id.get(&item_id).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn rows(&self) -> Vec<MarketItem> {
        self.by_id
            .iter()
            .map(|(id, name)| MarketItem {
                id: *id,
                name: name.clone(),
            })
            .collect()
    }
}

impl SnapshotTable for MarketItemsTable {
    type Row = MarketItem;

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketItemsTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketItemsTable::rows(self)
    }
}

/// Table containing currently known market offers.
#[derive(Debug, Default, Table)]
pub struct MarketOffersTable {
    by_id: HashMap<MarketOfferId, MarketOffer>,
}

impl MarketOffersTable {
    pub fn replace(&mut self, rows: impl IntoIterator<Item = MarketOffer>) {
        self.by_id = rows.into_iter().map(|row| (row.id, row)).collect();
    }

    pub fn insert(&mut self, row: MarketOffer) -> Option<MarketOffer> {
        self.by_id.insert(row.id, row)
    }

    pub fn create_offer(&mut self, offer: MarketOffer) -> Option<MarketOffer> {
        self.insert(offer)
    }

    pub fn get(&self, id: &MarketOfferId) -> Option<&MarketOffer> {
        self.by_id.get(id)
    }

    pub fn remove(&mut self, id: &MarketOfferId) -> Option<MarketOffer> {
        self.by_id.remove(id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &MarketOffer> {
        self.by_id.values()
    }

    pub fn rows(&self) -> Vec<MarketOffer> {
        self.by_id.values().cloned().collect()
    }
}

impl SnapshotTable for MarketOffersTable {
    type Row = MarketOffer;

    fn replace_rows(&mut self, rows: Vec<Self::Row>) {
        MarketOffersTable::replace(self, rows);
    }

    fn rows(&self) -> Vec<Self::Row> {
        MarketOffersTable::rows(self)
    }
}

impl FromIterator<MarketOffer> for MarketOffersTable {
    fn from_iter<T: IntoIterator<Item = MarketOffer>>(iter: T) -> Self {
        let mut table = Self::default();
        table.replace(iter);
        table
    }
}

/// Triggered after a market offer is created in memory.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCreated {
    #[event_target]
    pub client: Entity,
    pub player_id: Option<u32>,
    pub player_name: Option<String>,
    pub offer: MarketOffer,
    pub item_name: Option<String>,
}

/// Triggered after a market offer is removed from memory.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCancelled {
    #[event_target]
    pub client: Entity,
    pub player_id: Option<u32>,
    pub player_name: Option<String>,
    pub offer_id: MarketOfferId,
    pub offer: Option<MarketOffer>,
}

/// Triggered after a market offer accept changes in-memory market state.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferAccepted {
    #[event_target]
    pub client: Entity,
    pub player_id: Option<u32>,
    pub player_name: Option<String>,
    pub offer_id: MarketOfferId,
    pub accepted_amount: u16,
    pub previous_offer: Option<MarketOffer>,
    pub updated_offer: Option<MarketOffer>,
}

#[derive(Debug, Resource, Default)]
pub(crate) struct MarketRateLimiter {
    by_player: std::collections::HashMap<u32, std::collections::VecDeque<SystemTime>>,
}

impl MarketRateLimiter {
    pub(crate) fn record_offer_create(
        &mut self,
        player_id: u32,
        now: SystemTime,
        minute_limit: usize,
        hour_limit: usize,
    ) -> bool {
        let entries = self.by_player.entry(player_id).or_default();
        prune_entries(entries, now, Duration::from_secs(60 * 60));

        let within_hour = entries.len();
        let within_minute = entries
            .iter()
            .filter(|timestamp| {
                now.duration_since(**timestamp)
                    .map(|elapsed| elapsed <= Duration::from_secs(60))
                    .unwrap_or(false)
            })
            .count();

        if within_minute >= minute_limit || within_hour >= hour_limit {
            return false;
        }

        entries.push_back(now);
        true
    }
}

#[derive(Debug, Resource, Default)]
pub(crate) struct MarketOfferSequence {
    next_counter: Wrapping<u16>,
}

impl MarketOfferSequence {
    pub(crate) fn seed_from_offers(&mut self, offers: &MarketOffersTable) {
        self.next_counter = Wrapping(
            offers
                .iter()
                .map(|offer| offer.id.counter)
                .max()
                .unwrap_or(0),
        );
    }

    pub(crate) fn next(&mut self) -> u16 {
        self.next_counter += Wrapping(1);
        self.next_counter.0
    }
}

pub struct MarketOfferPlugin;

impl Plugin for MarketOfferPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarketOfferSequence>();
        app.init_resource::<MarketRateLimiter>();
        app.add_observer(on_create_market_offer_packet)
            .add_observer(on_cancel_market_offer_packet)
            .add_observer(on_accept_market_offer_packet);
    }
}

#[allow(clippy::too_many_arguments)]
fn on_create_market_offer_packet(
    event: On<Packet<CreateMarketOfferPacket>>,
    mut commands: Commands,
    settings: Res<MarketSettings>,
    player_refs: Query<&MarketPlayerRef>,
    players: Database<MarketPlayersTable>,
    items: Database<MarketItemsTable>,
    mut market_offers: DatabaseMut<MarketOffersTable>,
    mut history: DatabaseMut<MarketHistoryTable>,
    mut rate_limiter: ResMut<MarketRateLimiter>,
    mut offer_sequence: ResMut<MarketOfferSequence>,
    mut history_sequence: ResMut<MarketHistorySequence>,
    mut dirty: ResMut<MarketDirty>,
) {
    let client = event.entity();
    let packet = event.packet();
    let player_id = player_refs.get(client).ok().map(|entry| entry.player_id);
    let player_name = player_id.and_then(|id| players.name(id).map(str::to_owned));
    let Some(player_id) = player_id else {
        warn!(
            "Ignoring market offer creation for client {:?}: missing MarketPlayerRef/player id",
            client
        );
        return;
    };

    let active_offers = market_offers
        .iter()
        .filter(|offer| offer.player_id == player_id)
        .count();

    let now = SystemTime::now();
    if let Err(reason) = settings.policy.validate_offer_creation(
        player_id,
        packet.item_id,
        active_offers,
        &mut rate_limiter,
        now,
    ) {
        warn!(
            "Ignoring market offer creation for player {} on client {:?}: {}",
            player_id, client, reason
        );
        return;
    }

    let offer = MarketOffer {
        id: MarketOfferId {
            timestamp: SystemTime::now(),
            counter: offer_sequence.next(),
        },
        item_id: packet.item_id,
        player_id,
        amount: packet.amount,
        price: packet.price,
        side: MarketTradeSide::from(packet.offer_kind),
        is_anonymous: packet.is_anonymous,
    };

    market_offers.create_offer(offer.clone());
    history.append(MarketHistoryEntry {
        id: history_sequence.next(),
        recorded_at: now,
        action: MarketHistoryAction::Create,
        actor_player_id: Some(player_id),
        offer_player_id: Some(player_id),
        item_id: Some(offer.item_id),
        offer_id: Some(offer.id),
        amount: offer.amount,
        remaining_amount: None,
        price: Some(offer.price),
        side: Some(offer.side),
    });
    debug!(
        "Created market offer {:?} for player {} and item {}",
        offer.id, offer.player_id, offer.item_id
    );
    dirty.0 = true;
    commands.trigger(MarketOfferCreated {
        client,
        player_id: Some(player_id),
        player_name,
        offer,
        item_name: items.name(packet.item_id).map(str::to_owned),
    });
}

#[allow(clippy::too_many_arguments)]
fn on_cancel_market_offer_packet(
    event: On<Packet<CancelMarketOfferPacket>>,
    mut commands: Commands,
    player_refs: Query<&MarketPlayerRef>,
    players: Database<MarketPlayersTable>,
    mut offers: DatabaseMut<MarketOffersTable>,
    mut history: DatabaseMut<MarketHistoryTable>,
    mut history_sequence: ResMut<MarketHistorySequence>,
    mut dirty: ResMut<MarketDirty>,
) {
    let client = event.entity();
    let packet = event.packet();
    let offer_id = MarketOfferId {
        timestamp: packet.timestamp,
        counter: packet.offer_counter,
    };
    let player_id = player_refs.get(client).ok().map(|entry| entry.player_id);
    let player_name = player_id.and_then(|id| players.name(id).map(str::to_owned));
    let offer = offers.get(&offer_id).cloned();

    let outcome = MarketOfferCancelled {
        client,
        player_id,
        player_name,
        offer_id,
        offer,
    };

    cancel_offer(&outcome, &mut offers);
    if let Some(offer) = outcome.offer.as_ref() {
        history.append(MarketHistoryEntry {
            id: history_sequence.next(),
            recorded_at: SystemTime::now(),
            action: MarketHistoryAction::Cancel,
            actor_player_id: player_id,
            offer_player_id: Some(offer.player_id),
            item_id: Some(offer.item_id),
            offer_id: Some(offer.id),
            amount: offer.amount,
            remaining_amount: None,
            price: Some(offer.price),
            side: Some(offer.side),
        });
    }
    dirty.0 = true;
    commands.trigger(outcome);
}

#[allow(clippy::too_many_arguments)]
fn on_accept_market_offer_packet(
    event: On<Packet<AcceptMarketOfferPacket>>,
    mut commands: Commands,
    player_refs: Query<&MarketPlayerRef>,
    players: Database<MarketPlayersTable>,
    mut offers: DatabaseMut<MarketOffersTable>,
    mut history: DatabaseMut<MarketHistoryTable>,
    mut history_sequence: ResMut<MarketHistorySequence>,
    mut dirty: ResMut<MarketDirty>,
) {
    let client = event.entity();
    let packet = event.packet();
    let offer_id = MarketOfferId {
        timestamp: packet.timestamp,
        counter: packet.offer_counter,
    };
    let player_id = player_refs.get(client).ok().map(|entry| entry.player_id);
    let player_name = player_id.and_then(|id| players.name(id).map(str::to_owned));
    let previous_offer = offers.get(&offer_id).cloned();
    let updated_offer = accept_offer(offer_id, packet.amount, previous_offer.clone(), &mut offers);
    if let Some(previous_offer) = previous_offer.as_ref() {
        history.append(MarketHistoryEntry {
            id: history_sequence.next(),
            recorded_at: SystemTime::now(),
            action: MarketHistoryAction::Accept,
            actor_player_id: player_id,
            offer_player_id: Some(previous_offer.player_id),
            item_id: Some(previous_offer.item_id),
            offer_id: Some(previous_offer.id),
            amount: packet.amount,
            remaining_amount: updated_offer.as_ref().map(|offer| offer.amount),
            price: Some(previous_offer.price),
            side: Some(previous_offer.side),
        });
    }
    dirty.0 = true;

    commands.trigger(MarketOfferAccepted {
        client,
        player_id,
        player_name,
        offer_id,
        accepted_amount: packet.amount,
        previous_offer,
        updated_offer,
    });
}

pub(crate) fn cancel_offer(event: &MarketOfferCancelled, offers: &mut MarketOffersTable) {
    if event.offer.is_none() {
        debug!(
            "Skipping market offer cancellation for {:?}: offer {:?} was not cached",
            event.client, event.offer_id
        );
        return;
    }

    let _ = offers.remove(&event.offer_id);
}

pub(crate) fn accept_offer(
    offer_id: MarketOfferId,
    accepted_amount: u16,
    previous_offer: Option<MarketOffer>,
    offers: &mut MarketOffersTable,
) -> Option<MarketOffer> {
    let Some(existing_offer) = previous_offer else {
        debug!(
            "Skipping market offer accept for {:?}: offer {:?} was not cached",
            offer_id, accepted_amount
        );
        return None;
    };

    if accepted_amount == 0 {
        warn!(
            "Ignoring market offer accept for {:?}: accepted amount was zero",
            offer_id
        );
        return Some(existing_offer);
    }

    if accepted_amount >= existing_offer.amount {
        let _ = offers.remove(&offer_id);
        return None;
    }

    let updated_offer = MarketOffer {
        amount: existing_offer.amount - accepted_amount,
        ..existing_offer
    };

    offers.insert(updated_offer.clone());
    Some(updated_offer)
}

fn prune_entries(
    entries: &mut std::collections::VecDeque<SystemTime>,
    now: SystemTime,
    window: Duration,
) {
    while let Some(front) = entries.front().copied() {
        let expired = now
            .duration_since(front)
            .map(|elapsed| elapsed > window)
            .unwrap_or(false);

        if expired {
            entries.pop_front();
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{MarketPersistenceSettings, MarketPolicySettings, MarketSettings};
    use std::time::UNIX_EPOCH;

    #[test]
    fn should_convert_market_trade_side_to_and_from_string() {
        assert_eq!(MarketTradeSide::Buy.to_string(), "buy");
        assert_eq!("sell".parse::<MarketTradeSide>(), Ok(MarketTradeSide::Sell));
        assert!("weird".parse::<MarketTradeSide>().is_err());
    }

    #[test]
    fn should_replace_cached_market_offers() {
        let mut table = MarketOffersTable::default();

        table.replace([MarketOffer {
            id: MarketOfferId {
                timestamp: UNIX_EPOCH,
                counter: 1,
            },
            item_id: 2160,
            player_id: 7,
            amount: 1,
            price: 100,
            side: MarketTradeSide::Sell,
            is_anonymous: false,
        }]);

        assert!(
            table
                .get(&MarketOfferId {
                    timestamp: UNIX_EPOCH,
                    counter: 1
                })
                .is_some()
        );
    }

    #[test]
    fn should_reject_offer_creation_for_blocked_player() {
        let settings = MarketSettings {
            persistence: MarketPersistenceSettings::default(),
            policy: MarketPolicySettings {
                blocked_player_ids: vec![77],
                ..MarketPolicySettings::default()
            },
        };

        let result = settings.policy.validate_offer_creation(
            77,
            2160,
            0,
            &mut MarketRateLimiter::default(),
            UNIX_EPOCH,
        );

        assert_eq!(result, Err("player is blocked from market offers"));
    }

    #[test]
    fn should_reject_offer_creation_when_rate_limit_is_hit() {
        let settings = MarketSettings {
            persistence: MarketPersistenceSettings::default(),
            policy: MarketPolicySettings {
                max_create_per_minute: 1,
                max_create_per_hour: 10,
                ..MarketPolicySettings::default()
            },
        };
        let mut limiter = MarketRateLimiter::default();

        let first = settings
            .policy
            .validate_offer_creation(77, 2160, 0, &mut limiter, UNIX_EPOCH);
        let second = settings.policy.validate_offer_creation(
            77,
            2160,
            0,
            &mut limiter,
            UNIX_EPOCH + Duration::from_secs(1),
        );

        assert!(first.is_ok());
        assert_eq!(second, Err("market offer rate limit reached"));
    }
}
