use bevy::prelude::*;

use crate::offer::{MarketOffer, MarketOfferId, MarketTradeSide};

/// Intent requesting creation of a market offer.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCreateIntent {
    #[event_target]
    /// Client entity that requested the offer creation.
    pub client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub actor_id: Option<u32>,
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

/// Intent requesting cancellation of an existing market offer.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCancelIntent {
    #[event_target]
    /// Client entity that requested the cancellation.
    pub client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub actor_id: Option<u32>,
    /// Offer identifier that should be cancelled.
    pub offer_id: MarketOfferId,
}

/// Intent requesting acceptance of an existing market offer.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferAcceptIntent {
    #[event_target]
    /// Client entity that requested the acceptance.
    pub client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub actor_id: Option<u32>,
    /// Offer identifier that should be accepted.
    pub offer_id: MarketOfferId,
    /// Amount requested from the offer.
    pub accepted_amount: u16,
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
    /// Client entity whose create-offer request was rejected.
    pub(crate) client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub(crate) actor_id: Option<u32>,
    /// Requested item identifier from the rejected create intent.
    pub(crate) item_id: u16,
    /// Rejection reason produced by market validation.
    pub(crate) error: MarketOfferCreateError,
}

impl MarketOfferCreateRejected {
    /// Returns the target client entity.
    pub fn client(&self) -> Entity {
        self.client
    }

    /// Returns the bound actor identifier, when available.
    pub fn actor_id(&self) -> Option<u32> {
        self.actor_id
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

/// Event emitted when market-offer acceptance is rejected.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCancelRejected {
    #[event_target]
    /// Client entity whose cancel-offer request was rejected.
    pub(crate) client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub(crate) actor_id: Option<u32>,
    /// Offer identifier that could not be cancelled.
    pub(crate) offer_id: MarketOfferId,
}

impl MarketOfferCancelRejected {
    /// Returns the target client entity.
    pub fn client(&self) -> Entity {
        self.client
    }

    /// Returns the bound actor identifier, when available.
    pub fn actor_id(&self) -> Option<u32> {
        self.actor_id
    }

    /// Returns the target offer identifier.
    pub fn offer_id(&self) -> MarketOfferId {
        self.offer_id
    }
}

/// Event emitted when market-offer acceptance is rejected.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferAcceptRejected {
    #[event_target]
    /// Client entity whose accept-offer request was rejected.
    pub(crate) client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub(crate) actor_id: Option<u32>,
    /// Offer identifier that could not be accepted.
    pub(crate) offer_id: MarketOfferId,
}

impl MarketOfferAcceptRejected {
    /// Returns the target client entity.
    pub fn client(&self) -> Entity {
        self.client
    }

    /// Returns the bound actor identifier, when available.
    pub fn actor_id(&self) -> Option<u32> {
        self.actor_id
    }

    /// Returns the target offer identifier.
    pub fn offer_id(&self) -> MarketOfferId {
        self.offer_id
    }
}

/// Triggered after a market offer is created in memory.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCreated {
    #[event_target]
    /// Client entity that completed the create-offer action.
    pub(crate) client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub(crate) actor_id: Option<u32>,
    /// Created offer snapshot after the successful operation.
    pub(crate) offer: MarketOffer,
}

impl MarketOfferCreated {
    /// Returns the target client entity.
    pub fn client(&self) -> Entity {
        self.client
    }

    /// Returns the bound actor identifier, when available.
    pub fn actor_id(&self) -> Option<u32> {
        self.actor_id
    }

    /// Returns the created offer snapshot.
    pub fn offer(&self) -> &MarketOffer {
        &self.offer
    }
}

/// Triggered after a market offer is removed from memory.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCancelled {
    #[event_target]
    /// Client entity that completed the cancel-offer action.
    pub(crate) client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub(crate) actor_id: Option<u32>,
    /// Offer identifier that was targeted by the cancellation.
    pub(crate) offer_id: MarketOfferId,
    /// Removed offer snapshot, when the target offer existed.
    pub(crate) offer: Option<MarketOffer>,
}

impl MarketOfferCancelled {
    /// Returns the target client entity.
    pub fn client(&self) -> Entity {
        self.client
    }

    /// Returns the bound actor identifier, when available.
    pub fn actor_id(&self) -> Option<u32> {
        self.actor_id
    }

    /// Returns the cancelled offer identifier.
    pub fn offer_id(&self) -> MarketOfferId {
        self.offer_id
    }

    /// Returns the removed offer snapshot, when one existed.
    pub fn offer(&self) -> Option<&MarketOffer> {
        self.offer.as_ref()
    }
}

/// Triggered after a market offer accept changes in-memory market state.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferAccepted {
    #[event_target]
    /// Client entity that completed the accept-offer action.
    pub(crate) client: Entity,
    /// Persistent actor identifier bound to the client, when available.
    pub(crate) actor_id: Option<u32>,
    /// Offer identifier that was targeted by the accept operation.
    pub(crate) offer_id: MarketOfferId,
    /// Amount accepted from the target offer.
    pub(crate) accepted_amount: u16,
    /// Full offer snapshot before the accept operation.
    pub(crate) previous_offer: Option<MarketOffer>,
    /// Updated offer snapshot after the accept operation, when the offer remains open.
    pub(crate) updated_offer: Option<MarketOffer>,
}

impl MarketOfferAccepted {
    /// Returns the target client entity.
    pub fn client(&self) -> Entity {
        self.client
    }

    /// Returns the bound actor identifier, when available.
    pub fn actor_id(&self) -> Option<u32> {
        self.actor_id
    }

    /// Returns the accepted offer identifier.
    pub fn offer_id(&self) -> MarketOfferId {
        self.offer_id
    }

    /// Returns the amount accepted from the offer.
    pub fn accepted_amount(&self) -> u16 {
        self.accepted_amount
    }

    /// Returns the full offer snapshot before the accept operation.
    pub fn previous_offer(&self) -> Option<&MarketOffer> {
        self.previous_offer.as_ref()
    }

    /// Returns the updated offer snapshot after the accept operation, when the offer remains open.
    pub fn updated_offer(&self) -> Option<&MarketOffer> {
        self.updated_offer.as_ref()
    }
}
