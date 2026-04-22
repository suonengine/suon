use bevy::prelude::*;
use suon_database::prelude::*;
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::AcceptMarketOfferPacket;

use crate::{
    offer::{MarketOffer, MarketOfferId, MarketOffersTable, accept_offer},
    persistence::MarketDirty,
};

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

/// Event emitted when market-offer acceptance is rejected.
#[derive(Debug, Clone, EntityEvent, PartialEq, Eq)]
pub struct MarketOfferAcceptRejected {
    #[event_target]
    /// Entity whose accept-offer request was rejected.
    pub(crate) entity: Entity,
    /// Offer identifier that could not be accepted.
    pub(crate) offer_id: MarketOfferId,
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

/// Translates accept-offer packets into typed accept intents.
pub(super) fn on_accept_market_offer_packet(
    event: On<Packet<AcceptMarketOfferPacket>>,
    mut commands: Commands,
) {
    let entity = event.entity();
    let packet = event.packet();

    commands.trigger(MarketOfferAcceptIntent {
        entity,
        offer_id: MarketOfferId::new(packet.timestamp, packet.offer_counter),
        accepted_amount: packet.amount,
    });
}

#[allow(clippy::too_many_arguments)]
/// Applies accept-offer intents to in-memory market state.
pub(super) fn on_accept_market_offer_intent(
    event: On<MarketOfferAcceptIntent>,
    mut commands: Commands,
    mut offers: DatabaseMut<MarketOffersTable>,
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
            entity: event.entity,
            offer_id: event.offer_id,
        });
        return;
    }

    dirty.mark();
    commands.trigger(MarketOfferAccepted {
        entity: event.entity,
        offer_id: event.offer_id,
        accepted_amount: event.accepted_amount,
        previous_offer,
        updated_offer,
    });
}
