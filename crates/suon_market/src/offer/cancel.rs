use bevy::prelude::*;
use suon_database::prelude::*;
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::CancelMarketOfferPacket;

use crate::{
    offer::{MarketOffer, MarketOfferId, MarketOffersTable, cancel_offer},
    persistence::MarketDirty,
};

/// Intent requesting cancellation of an existing market offer.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCancelIntent {
    #[event_target]
    /// Entity that requested the cancellation.
    pub entity: Entity,
    /// Offer identifier that should be cancelled.
    pub offer_id: MarketOfferId,
}

/// Event emitted when market-offer acceptance is rejected.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferCancelRejected {
    #[event_target]
    /// Entity whose cancel-offer request was rejected.
    pub(crate) entity: Entity,
    /// Offer identifier that could not be cancelled.
    pub(crate) offer_id: MarketOfferId,
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

/// Translates cancel-offer packets into typed cancel intents.
pub(super) fn on_cancel_market_offer_packet(
    event: On<Packet<CancelMarketOfferPacket>>,
    mut commands: Commands,
) {
    let entity = event.entity();
    let packet = event.packet();

    commands.trigger(MarketOfferCancelIntent {
        entity,
        offer_id: MarketOfferId::new(packet.timestamp, packet.offer_counter),
    });
}

#[allow(clippy::too_many_arguments)]
/// Applies cancel-offer intents to in-memory market state.
pub(super) fn on_cancel_market_offer_intent(
    event: On<MarketOfferCancelIntent>,
    mut commands: Commands,
    mut offers: DatabaseMut<MarketOffersTable>,
    mut dirty: ResMut<MarketDirty>,
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
        });
        return;
    }

    cancel_offer(&outcome, &mut offers);

    dirty.mark();
    commands.trigger(outcome);
}
