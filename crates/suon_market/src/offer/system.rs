use std::time::SystemTime;

use bevy::prelude::*;
use log::debug;
use suon_database::prelude::*;

use crate::{
    offer::{
        MarketOffer, MarketOfferAcceptError, MarketOfferAcceptIntent, MarketOfferAcceptRejected,
        MarketOfferAccepted, MarketOfferCancelError, MarketOfferCancelIntent,
        MarketOfferCancelRejected, MarketOfferCancelled, MarketOfferCreateError,
        MarketOfferCreateIntent, MarketOfferCreateRejected, MarketOfferCreated, MarketOfferId,
        MarketOfferSequence, MarketOffersTable, MarketRateLimiter, accept_offer, cancel_offer,
    },
    persistence::MarketSettings,
    session::MarketActorRef,
};

#[allow(clippy::too_many_arguments)]
/// Applies create-offer intents to in-memory market state.
pub(super) fn on_create_market_offer_intent(
    event: On<MarketOfferCreateIntent>,
    mut commands: Commands,
    settings: Res<MarketSettings>,
    actor_refs: Query<&MarketActorRef>,
    mut market_offers: DbMut<MarketOffersTable>,
    mut rate_limiter: ResMut<MarketRateLimiter>,
    mut offer_sequence: ResMut<MarketOfferSequence>,
) {
    let Some(actor_id) = actor_refs
        .get(event.entity)
        .ok()
        .map(MarketActorRef::actor_id)
    else {
        reject_create(
            &mut commands,
            event.entity,
            event.item_id,
            MarketOfferCreateError::MissingActor,
        );
        return;
    };

    let active_offers = market_offers
        .iter()
        .filter(|offer| offer.actor_id() == actor_id)
        .count();

    let now = SystemTime::now();
    if let Err(error) = settings.policy().validate_offer_creation(
        actor_id,
        event.item_id,
        active_offers,
        &mut rate_limiter,
        now,
    ) {
        reject_create(&mut commands, event.entity, event.item_id, error);
        return;
    }

    let offer = MarketOffer::new(
        MarketOfferId::new(now, offer_sequence.next()),
        event.item_id,
        actor_id,
        event.amount,
        event.price,
        event.side,
        event.is_anonymous,
    );

    market_offers.create_offer(offer.clone());
    debug!(
        "Created market offer {:?} for actor {} and item {}",
        offer.id(),
        offer.actor_id(),
        offer.item_id()
    );
    commands.trigger(MarketOfferCreated {
        entity: event.entity,
        offer,
    });
}

#[allow(clippy::too_many_arguments)]
/// Applies cancel-offer intents to in-memory market state.
pub(super) fn on_cancel_market_offer_intent(
    event: On<MarketOfferCancelIntent>,
    mut commands: Commands,
    mut offers: DbMut<MarketOffersTable>,
) {
    let offer = offers.get(&event.offer_id).cloned();
    let outcome = MarketOfferCancelled {
        entity: event.entity,
        offer_id: event.offer_id,
        offer,
    };

    if outcome.offer().is_none() {
        commands.trigger(MarketOfferCancelRejected {
            entity: event.entity,
            offer_id: event.offer_id,
            error: MarketOfferCancelError::MissingOffer,
        });
        return;
    }

    cancel_offer(&outcome, &mut offers);

    commands.trigger(outcome);
}

#[allow(clippy::too_many_arguments)]
/// Applies accept-offer intents to in-memory market state.
pub(super) fn on_accept_market_offer_intent(
    event: On<MarketOfferAcceptIntent>,
    mut commands: Commands,
    mut offers: DbMut<MarketOffersTable>,
) {
    let previous_offer = offers.get(&event.offer_id).cloned();
    let updated_offer = accept_offer(
        event.offer_id,
        event.accepted_amount,
        previous_offer.clone(),
        &mut offers,
    );

    if previous_offer.is_none() {
        commands.trigger(MarketOfferAcceptRejected {
            entity: event.entity,
            offer_id: event.offer_id,
            error: MarketOfferAcceptError::MissingOffer,
        });
        return;
    }

    commands.trigger(MarketOfferAccepted {
        entity: event.entity,
        offer_id: event.offer_id,
        accepted_amount: event.accepted_amount,
        previous_offer,
        updated_offer,
    });
}

fn reject_create(
    commands: &mut Commands,
    entity: Entity,
    item_id: u16,
    error: MarketOfferCreateError,
) {
    commands.trigger(MarketOfferCreateRejected {
        entity,
        item_id,
        error,
    });
}
