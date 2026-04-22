use std::time::SystemTime;

use crate::{
    history::MarketHistoryAction,
    offer::{MarketOfferId, MarketTradeSide},
};

/// Immutable market history entry stored as an audit log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketHistoryEntry {
    recorded_at: SystemTime,
    action: MarketHistoryAction,
    actor_id: Option<u32>,
    offer_actor_id: Option<u32>,
    item_id: Option<u16>,
    offer_id: Option<MarketOfferId>,
    amount: u16,
    remaining_amount: Option<u16>,
    price: Option<u64>,
    side: Option<MarketTradeSide>,
}

impl MarketHistoryEntry {
    /// Creates a new immutable market-history entry.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        recorded_at: SystemTime,
        action: MarketHistoryAction,
        actor_id: Option<u32>,
        offer_actor_id: Option<u32>,
        item_id: Option<u16>,
        offer_id: Option<MarketOfferId>,
        amount: u16,
        remaining_amount: Option<u16>,
        price: Option<u64>,
        side: Option<MarketTradeSide>,
    ) -> Self {
        Self {
            recorded_at,
            action,
            actor_id,
            offer_actor_id,
            item_id,
            offer_id,
            amount,
            remaining_amount,
            price,
            side,
        }
    }

    /// Returns when the history entry was recorded.
    pub fn recorded_at(&self) -> SystemTime {
        self.recorded_at
    }

    /// Returns the recorded market action.
    pub fn action(&self) -> MarketHistoryAction {
        self.action
    }

    /// Returns the actor that initiated the action, when known.
    pub fn actor_id(&self) -> Option<u32> {
        self.actor_id
    }

    /// Returns the actor that owned the offer, when known.
    pub fn offer_actor_id(&self) -> Option<u32> {
        self.offer_actor_id
    }

    /// Returns the related item identifier, when known.
    pub fn item_id(&self) -> Option<u16> {
        self.item_id
    }

    /// Returns the related offer identifier, when known.
    pub fn offer_id(&self) -> Option<MarketOfferId> {
        self.offer_id
    }

    /// Returns the amount recorded in the history entry.
    pub fn amount(&self) -> u16 {
        self.amount
    }

    /// Returns the remaining amount after the action, when relevant.
    pub fn remaining_amount(&self) -> Option<u16> {
        self.remaining_amount
    }

    /// Returns the recorded price, when relevant.
    pub fn price(&self) -> Option<u64> {
        self.price
    }

    /// Returns the recorded trade side, when relevant.
    pub fn side(&self) -> Option<MarketTradeSide> {
        self.side
    }
}
