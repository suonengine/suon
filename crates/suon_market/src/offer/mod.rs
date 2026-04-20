mod events;
mod logic;
mod model;
mod tables;

use std::time::SystemTime;

use bevy::prelude::*;
use log::{debug, warn};
use suon_database::prelude::*;
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::{
    AcceptMarketOfferPacket, CancelMarketOfferPacket, CreateMarketOfferPacket,
};

use crate::{
    browse::MarketActorRef,
    history::{MarketHistoryAction, MarketHistoryEntry, MarketHistorySequence, MarketHistoryTable},
    persistence::{MarketDirty, MarketSettings},
};

pub use self::{
    events::{
        MarketOfferAcceptIntent, MarketOfferAcceptRejected, MarketOfferAccepted,
        MarketOfferCancelIntent, MarketOfferCancelRejected, MarketOfferCancelled,
        MarketOfferCreateError, MarketOfferCreateIntent, MarketOfferCreateRejected,
        MarketOfferCreated,
    },
    model::{
        MarketActorName, MarketItem, MarketOffer, MarketOfferId, MarketTradeSide,
        ParseMarketTradeSideError,
    },
    tables::{MarketActorsTable, MarketItemsTable, MarketOffersTable},
};

pub(crate) use self::logic::{accept_offer, cancel_offer, MarketOfferSequence, MarketRateLimiter};

pub(crate) struct MarketOfferPlugin;

impl Plugin for MarketOfferPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MarketOfferSequence>();
        app.init_resource::<MarketRateLimiter>();
        app.add_observer(on_create_market_offer_packet)
            .add_observer(on_cancel_market_offer_packet)
            .add_observer(on_accept_market_offer_packet)
            .add_observer(on_create_market_offer_intent)
            .add_observer(on_cancel_market_offer_intent)
            .add_observer(on_accept_market_offer_intent);
    }
}

fn on_create_market_offer_packet(
    event: On<Packet<CreateMarketOfferPacket>>,
    mut commands: Commands,
    actor_refs: Query<&MarketActorRef>,
) {
    let client = event.entity();
    let packet = event.packet();
    let actor_id = actor_refs.get(client).ok().map(MarketActorRef::actor_id);

    commands.trigger(MarketOfferCreateIntent {
        client,
        actor_id,
        item_id: packet.item_id,
        amount: packet.amount,
        price: packet.price,
        side: MarketTradeSide::from(packet.offer_kind),
        is_anonymous: packet.is_anonymous,
    });
}

fn on_cancel_market_offer_packet(
    event: On<Packet<CancelMarketOfferPacket>>,
    mut commands: Commands,
    actor_refs: Query<&MarketActorRef>,
) {
    let client = event.entity();
    let packet = event.packet();
    let actor_id = actor_refs.get(client).ok().map(MarketActorRef::actor_id);

    commands.trigger(MarketOfferCancelIntent {
        client,
        actor_id,
        offer_id: MarketOfferId::new(packet.timestamp, packet.offer_counter),
    });
}

fn on_accept_market_offer_packet(
    event: On<Packet<AcceptMarketOfferPacket>>,
    mut commands: Commands,
    actor_refs: Query<&MarketActorRef>,
) {
    let client = event.entity();
    let packet = event.packet();
    let actor_id = actor_refs.get(client).ok().map(MarketActorRef::actor_id);

    commands.trigger(MarketOfferAcceptIntent {
        client,
        actor_id,
        offer_id: MarketOfferId::new(packet.timestamp, packet.offer_counter),
        accepted_amount: packet.amount,
    });
}

#[allow(clippy::too_many_arguments)]
fn on_create_market_offer_intent(
    event: On<MarketOfferCreateIntent>,
    mut commands: Commands,
    settings: Res<MarketSettings>,
    mut market_offers: DatabaseMut<MarketOffersTable>,
    mut history: DatabaseMut<MarketHistoryTable>,
    mut rate_limiter: ResMut<MarketRateLimiter>,
    mut offer_sequence: ResMut<MarketOfferSequence>,
    mut history_sequence: ResMut<MarketHistorySequence>,
    mut dirty: ResMut<MarketDirty>,
) {
    let Some(actor_id) = event.actor_id else {
        commands.trigger(MarketOfferCreateRejected {
            client: event.client,
            actor_id: None,
            item_id: event.item_id,
            error: MarketOfferCreateError::MissingActor,
        });
        return;
    };

    let active_offers = market_offers
        .iter()
        .filter(|offer| offer.actor_id() == actor_id)
        .count();

    let now = SystemTime::now();
    let validation_error = match settings.policy().validate_offer_creation(
        actor_id,
        event.item_id,
        active_offers,
        &mut rate_limiter,
        now,
    ) {
        Ok(()) => None,
        Err("actor is blocked from market offers") => Some(MarketOfferCreateError::ActorBlocked),
        Err("item is blocked from market offers") => Some(MarketOfferCreateError::ItemBlocked),
        Err("active market offer limit reached") => {
            Some(MarketOfferCreateError::ActiveOfferLimitReached)
        }
        Err("market offer rate limit reached") => Some(MarketOfferCreateError::RateLimitReached),
        Err(reason) => {
            warn!(
                "Rejecting market offer creation for actor {} on client {:?}: {}",
                actor_id,
                event.client,
                reason
            );
            Some(MarketOfferCreateError::RateLimitReached)
        }
    };

    if let Some(error) = validation_error {
        commands.trigger(MarketOfferCreateRejected {
            client: event.client,
            actor_id: Some(actor_id),
            item_id: event.item_id,
            error,
        });
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
    history.append(MarketHistoryEntry::new(
        history_sequence.next(),
        now,
        MarketHistoryAction::Create,
        Some(actor_id),
        Some(actor_id),
        Some(offer.item_id()),
        Some(offer.id()),
        offer.amount(),
        None,
        Some(offer.price()),
        Some(offer.side()),
    ));
    debug!(
        "Created market offer {:?} for actor {} and item {}",
        offer.id(),
        offer.actor_id(),
        offer.item_id()
    );
    dirty.mark();
    commands.trigger(MarketOfferCreated {
        client: event.client,
        actor_id: Some(actor_id),
        offer,
    });
}

#[allow(clippy::too_many_arguments)]
fn on_cancel_market_offer_intent(
    event: On<MarketOfferCancelIntent>,
    mut commands: Commands,
    mut offers: DatabaseMut<MarketOffersTable>,
    mut history: DatabaseMut<MarketHistoryTable>,
    mut history_sequence: ResMut<MarketHistorySequence>,
    mut dirty: ResMut<MarketDirty>,
) {
    let offer = offers.get(&event.offer_id).cloned();
    let outcome = MarketOfferCancelled {
        client: event.client,
        actor_id: event.actor_id,
        offer_id: event.offer_id,
        offer,
    };

    if outcome.offer().is_none() {
        commands.trigger(MarketOfferCancelRejected {
            client: event.client,
            actor_id: event.actor_id,
            offer_id: event.offer_id,
        });
        return;
    }

    cancel_offer(&outcome, &mut offers);

    if let Some(offer) = outcome.offer() {
        history.append(MarketHistoryEntry::new(
            history_sequence.next(),
            SystemTime::now(),
            MarketHistoryAction::Cancel,
            event.actor_id,
            Some(offer.actor_id()),
            Some(offer.item_id()),
            Some(offer.id()),
            offer.amount(),
            None,
            Some(offer.price()),
            Some(offer.side()),
        ));
    }

    dirty.mark();
    commands.trigger(outcome);
}

#[allow(clippy::too_many_arguments)]
fn on_accept_market_offer_intent(
    event: On<MarketOfferAcceptIntent>,
    mut commands: Commands,
    mut offers: DatabaseMut<MarketOffersTable>,
    mut history: DatabaseMut<MarketHistoryTable>,
    mut history_sequence: ResMut<MarketHistorySequence>,
    mut dirty: ResMut<MarketDirty>,
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
            client: event.client,
            actor_id: event.actor_id,
            offer_id: event.offer_id,
        });
        return;
    }

    if let Some(previous_offer) = previous_offer.as_ref() {
        history.append(MarketHistoryEntry::new(
            history_sequence.next(),
            SystemTime::now(),
            MarketHistoryAction::Accept,
            event.actor_id,
            Some(previous_offer.actor_id()),
            Some(previous_offer.item_id()),
            Some(previous_offer.id()),
            event.accepted_amount,
            updated_offer.as_ref().map(MarketOffer::amount),
            Some(previous_offer.price()),
            Some(previous_offer.side()),
        ));
    }

    dirty.mark();
    commands.trigger(MarketOfferAccepted {
        client: event.client,
        actor_id: event.actor_id,
        offer_id: event.offer_id,
        accepted_amount: event.accepted_amount,
        previous_offer,
        updated_offer,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::{MarketPersistenceSettings, MarketPolicySettings, MarketSettings};
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn should_convert_market_trade_side_to_and_from_string() {
        assert_eq!(MarketTradeSide::Buy.to_string(), "buy");
        assert_eq!("sell".parse::<MarketTradeSide>(), Ok(MarketTradeSide::Sell));
        assert!("weird".parse::<MarketTradeSide>().is_err());
    }

    #[test]
    fn should_replace_cached_market_offers() {
        let mut table = MarketOffersTable::default();

        table.replace([MarketOffer::new(
            MarketOfferId::new(UNIX_EPOCH, 1),
            2160,
            7,
            1,
            100,
            MarketTradeSide::Sell,
            false,
        )]);

        assert!(table.get(&MarketOfferId::new(UNIX_EPOCH, 1)).is_some());
    }

    #[test]
    fn should_reject_offer_creation_for_blocked_player() {
        let settings = MarketSettings::new(
            MarketPersistenceSettings::default(),
            MarketPolicySettings::new(100, 20, 200, Vec::new(), vec![77]),
        );

        let result = settings.policy().validate_offer_creation(
            77,
            2160,
            0,
            &mut MarketRateLimiter::default(),
            UNIX_EPOCH,
        );

        assert_eq!(result, Err("actor is blocked from market offers"));
    }

    #[test]
    fn should_reject_offer_creation_when_rate_limit_is_hit() {
        let settings = MarketSettings::new(
            MarketPersistenceSettings::default(),
            MarketPolicySettings::new(100, 1, 10, Vec::new(), Vec::new()),
        );
        let mut limiter = MarketRateLimiter::default();

        let first = settings
            .policy()
            .validate_offer_creation(77, 2160, 0, &mut limiter, UNIX_EPOCH);
        let second = settings.policy().validate_offer_creation(
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
