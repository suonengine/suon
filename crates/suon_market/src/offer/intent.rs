use bevy::prelude::*;

use crate::offer::{MarketOfferId, MarketTradeSide};

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

/// Intent requesting cancellation of an existing market offer.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCancelIntent {
    #[event_target]
    /// Entity that requested the cancellation.
    pub entity: Entity,

    /// Offer identifier that should be cancelled.
    pub offer_id: MarketOfferId,
}

/// Intent requesting acceptance of an existing market offer.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferAcceptIntent {
    #[event_target]
    /// Entity that requested the acceptance.
    pub entity: Entity,

    /// Offer identifier that should be accepted.
    pub offer_id: MarketOfferId,

    /// Amount requested from the offer.
    pub accepted_amount: u16,
}
