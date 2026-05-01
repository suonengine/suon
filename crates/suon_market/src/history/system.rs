use std::time::SystemTime;

use bevy::prelude::*;
use log::warn;
use suon_database::prelude::*;

use crate::{
    history::{MarketHistoryAction, MarketHistoryEntry},
    offer::{MarketOfferAccepted, MarketOfferCancelled, MarketOfferCreated},
    persistence::MarketHistoryJournal,
    session::MarketActorRef,
};

pub(super) fn on_market_offer_created(
    event: On<MarketOfferCreated>,
    connection: Option<Res<DbConnection>>,
) {
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

    append_history(entry, connection);
}

pub(super) fn on_market_offer_cancelled(
    event: On<MarketOfferCancelled>,
    actor_refs: Query<&MarketActorRef>,
    connection: Option<Res<DbConnection>>,
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

    append_history(entry, connection);
}

pub(super) fn on_market_offer_accepted(
    event: On<MarketOfferAccepted>,
    actor_refs: Query<&MarketActorRef>,
    connection: Option<Res<DbConnection>>,
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

    append_history(entry, connection);
}

fn append_history(entry: MarketHistoryEntry, connection: Option<Res<DbConnection>>) {
    let Some(connection) = connection else {
        warn!("Dropping market history entry because no DbConnection is available");
        return;
    };

    if let Err(error) = MarketHistoryJournal::append(&connection, &entry) {
        warn!("Failed to append market history entry: {error:#}");
    }
}
