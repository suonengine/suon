use std::time::SystemTime;

use bevy::prelude::*;
use log::{debug, warn};
use suon_database::prelude::*;
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::CreateMarketOfferPacket;

use crate::{
    browse::MarketActorRef,
    offer::{MarketOffer, MarketOfferId, MarketOffersTable, MarketRateLimiter, MarketTradeSide},
    persistence::{MarketDirty, MarketSettings},
};

/// Intent requesting creation of a market offer.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCreateIntent {
    #[event_target]
    /// Entity that requested the offer creation.
    pub entity: Entity,
    /// Item identifier for the new offer.
    pub item_id: u16,
    /// Offered amount for the new entry.
    pub amount: u16,
    /// Price associated with the new offer.
    pub price: u64,
    /// Trade side for the new offer.
    pub side: MarketTradeSide,
    /// Whether the new offer should hide its owner.
    pub is_anonymous: bool,
}

/// Reason why a market-offer create request was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketOfferCreateError {
    /// The client is not bound to a market actor.
    MissingActor,
    /// The actor is blocked from creating market offers.
    ActorBlocked,
    /// The item is blocked from market offers.
    ItemBlocked,
    /// The active-offer limit has been reached.
    ActiveOfferLimitReached,
    /// The rate limit has been reached.
    RateLimitReached,
}

/// Event emitted when market-offer creation is rejected.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCreateRejected {
    #[event_target]
    /// Entity whose create-offer request was rejected.
    pub(crate) entity: Entity,
    /// Requested item identifier from the rejected create intent.
    pub(crate) item_id: u16,
    /// Rejection reason produced by market validation.
    pub(crate) error: MarketOfferCreateError,
}

impl MarketOfferCreateRejected {
    /// Returns the target entity.
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Returns the requested item identifier.
    pub fn item_id(&self) -> u16 {
        self.item_id
    }

    /// Returns the rejection reason.
    pub fn error(&self) -> MarketOfferCreateError {
        self.error
    }
}

/// Triggered after a market offer is created in memory.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCreated {
    #[event_target]
    /// Entity that completed the create-offer action.
    pub(crate) entity: Entity,
    /// Created offer snapshot after the successful operation.
    pub(crate) offer: MarketOffer,
}

impl MarketOfferCreated {
    /// Returns the target entity.
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Returns the created offer snapshot.
    pub fn offer(&self) -> &MarketOffer {
        &self.offer
    }
}

/// Translates create-offer packets into typed create intents.
pub(super) fn on_create_market_offer_packet(
    event: On<Packet<CreateMarketOfferPacket>>,
    mut commands: Commands,
) {
    let entity = event.entity();
    let packet = event.packet();

    commands.trigger(MarketOfferCreateIntent {
        entity,
        item_id: packet.item_id,
        amount: packet.amount,
        price: packet.price,
        side: MarketTradeSide::from(packet.offer_kind),
        is_anonymous: packet.is_anonymous,
    });
}

#[allow(clippy::too_many_arguments)]
/// Applies create-offer intents to in-memory market state.
pub(super) fn on_create_market_offer_intent(
    event: On<MarketOfferCreateIntent>,
    mut commands: Commands,
    settings: Res<MarketSettings>,
    actor_refs: Query<&MarketActorRef>,
    mut market_offers: DatabaseMut<MarketOffersTable>,
    mut rate_limiter: ResMut<MarketRateLimiter>,
    mut offer_sequence: ResMut<crate::offer::MarketOfferSequence>,
    mut dirty: ResMut<MarketDirty>,
) {
    let Some(actor_id) = actor_refs
        .get(event.entity)
        .ok()
        .map(MarketActorRef::actor_id)
    else {
        commands.trigger(MarketOfferCreateRejected {
            entity: event.entity,
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
                actor_id, event.entity, reason
            );
            Some(MarketOfferCreateError::RateLimitReached)
        }
    };

    if let Some(error) = validation_error {
        commands.trigger(MarketOfferCreateRejected {
            entity: event.entity,
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
    debug!(
        "Created market offer {:?} for actor {} and item {}",
        offer.id(),
        offer.actor_id(),
        offer.item_id()
    );
    dirty.mark();
    commands.trigger(MarketOfferCreated {
        entity: event.entity,
        offer,
    });
}
