use bevy::prelude::*;

use crate::offer::{MarketOffer, MarketOfferId};

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

/// Reason why a market-offer cancel request was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketOfferCancelError {
    /// The target offer was not cached.
    MissingOffer,
}

/// Reason why a market-offer accept request was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketOfferAcceptError {
    /// The target offer was not cached.
    MissingOffer,
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

/// Event emitted when market-offer cancellation is rejected.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCancelRejected {
    #[event_target]
    /// Entity whose cancel-offer request was rejected.
    pub(crate) entity: Entity,

    /// Offer identifier that could not be cancelled.
    pub(crate) offer_id: MarketOfferId,

    /// Rejection reason produced by market validation.
    pub(crate) error: MarketOfferCancelError,
}

impl MarketOfferCancelRejected {
    /// Returns the target entity.
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Returns the target offer identifier.
    pub fn offer_id(&self) -> MarketOfferId {
        self.offer_id
    }

    /// Returns the rejection reason.
    pub fn error(&self) -> MarketOfferCancelError {
        self.error
    }
}

/// Event emitted when market-offer acceptance is rejected.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferAcceptRejected {
    #[event_target]
    /// Entity whose accept-offer request was rejected.
    pub(crate) entity: Entity,

    /// Offer identifier that could not be accepted.
    pub(crate) offer_id: MarketOfferId,

    /// Rejection reason produced by market validation.
    pub(crate) error: MarketOfferAcceptError,
}

impl MarketOfferAcceptRejected {
    /// Returns the target entity.
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Returns the target offer identifier.
    pub fn offer_id(&self) -> MarketOfferId {
        self.offer_id
    }

    /// Returns the rejection reason.
    pub fn error(&self) -> MarketOfferAcceptError {
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

/// Triggered after a market offer is removed from memory.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCancelled {
    #[event_target]
    /// Entity that completed the cancel-offer action.
    pub(crate) entity: Entity,

    /// Offer identifier that was targeted by the cancellation.
    pub(crate) offer_id: MarketOfferId,

    /// Removed offer snapshot, when the target offer existed.
    pub(crate) offer: Option<MarketOffer>,
}

impl MarketOfferCancelled {
    /// Returns the target entity.
    pub fn entity(&self) -> Entity {
        self.entity
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
    /// Entity that completed the accept-offer action.
    pub(crate) entity: Entity,

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
    /// Returns the target entity.
    pub fn entity(&self) -> Entity {
        self.entity
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
