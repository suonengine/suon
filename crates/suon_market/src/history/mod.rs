mod action;
mod entry;

use std::time::SystemTime;

use bevy::prelude::*;
use log::warn;

use crate::{
    browse::MarketActorRef,
    offer::{MarketOfferAccepted, MarketOfferCancelled, MarketOfferCreated},
    persistence::MarketOrmResource,
};

pub use self::{
    action::{MarketHistoryAction, ParseMarketHistoryActionError},
    entry::MarketHistoryEntry,
};

pub(crate) struct MarketHistoryPlugin;

impl Plugin for MarketHistoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_market_offer_created)
            .add_observer(on_market_offer_cancelled)
            .add_observer(on_market_offer_accepted);
    }
}

fn on_market_offer_created(event: On<MarketOfferCreated>, orm: Option<Res<MarketOrmResource>>) {
    let offer = event.offer();
    let entry = MarketHistoryEntry::new(
        SystemTime::now(),
        MarketHistoryAction::Create,
        Some(offer.actor_id()),
        Some(offer.actor_id()),
        Some(offer.item_id()),
        Some(offer.id()),
        offer.amount(),
        None,
        Some(offer.price()),
        Some(offer.side()),
    );

    let Some(orm) = orm else {
        warn!("Dropping market history entry because no MarketOrmResource is available");
        return;
    };

    if let Err(error) = orm.insert_history(&entry) {
        warn!("Failed to append market history entry: {error:#}");
    }
}

fn on_market_offer_cancelled(
    event: On<MarketOfferCancelled>,
    actor_refs: Query<&MarketActorRef>,
    orm: Option<Res<MarketOrmResource>>,
) {
    let actor_id = actor_refs
        .get(event.entity())
        .ok()
        .map(MarketActorRef::actor_id);
    let Some(offer) = event.offer() else {
        return;
    };
    let entry = MarketHistoryEntry::new(
        SystemTime::now(),
        MarketHistoryAction::Cancel,
        actor_id,
        Some(offer.actor_id()),
        Some(offer.item_id()),
        Some(offer.id()),
        offer.amount(),
        None,
        Some(offer.price()),
        Some(offer.side()),
    );

    let Some(orm) = orm else {
        warn!("Dropping market history entry because no MarketOrmResource is available");
        return;
    };

    if let Err(error) = orm.insert_history(&entry) {
        warn!("Failed to append market history entry: {error:#}");
    }
}

fn on_market_offer_accepted(
    event: On<MarketOfferAccepted>,
    actor_refs: Query<&MarketActorRef>,
    orm: Option<Res<MarketOrmResource>>,
) {
    let actor_id = actor_refs
        .get(event.entity())
        .ok()
        .map(MarketActorRef::actor_id);
    let Some(previous_offer) = event.previous_offer() else {
        return;
    };
    let entry = MarketHistoryEntry::new(
        SystemTime::now(),
        MarketHistoryAction::Accept,
        actor_id,
        Some(previous_offer.actor_id()),
        Some(previous_offer.item_id()),
        Some(previous_offer.id()),
        event.accepted_amount(),
        event.updated_offer().map(|offer| offer.amount()),
        Some(previous_offer.price()),
        Some(previous_offer.side()),
    );

    let Some(orm) = orm else {
        warn!("Dropping market history entry because no MarketOrmResource is available");
        return;
    };

    if let Err(error) = orm.insert_history(&entry) {
        warn!("Failed to append market history entry: {error:#}");
    }
}
