use bevy::prelude::*;
use suon_network::prelude::Packet;
use suon_protocol_client::prelude::MarketPacket;

use crate::{
    browse::BrowseMarketIntent,
    offer::{
        MarketOfferAcceptIntent, MarketOfferCancelIntent, MarketOfferCreateIntent, MarketOfferId,
        MarketTradeSide,
    },
    session::CloseMarketSessionIntent,
};

/// Translates inbound market packets into typed market intents.
pub(super) fn on_market_packet(event: On<Packet<MarketPacket>>, mut commands: Commands) {
    let entity = event.entity();

    match *event.packet() {
        MarketPacket::Leave => {
            commands.trigger(CloseMarketSessionIntent { entity });
        }
        MarketPacket::Browse { request_kind } => {
            commands.trigger(BrowseMarketIntent {
                entity,
                scope: request_kind,
            });
        }
        MarketPacket::CreateOffer {
            offer_kind,
            item_id,
            amount,
            price,
            is_anonymous,
            ..
        } => {
            commands.trigger(MarketOfferCreateIntent {
                entity,
                item_id,
                amount,
                price,
                side: MarketTradeSide::from(offer_kind),
                is_anonymous,
            });
        }
        MarketPacket::CancelOffer {
            timestamp,
            offer_counter,
        } => {
            commands.trigger(MarketOfferCancelIntent {
                entity,
                offer_id: MarketOfferId::new(timestamp, offer_counter),
            });
        }
        MarketPacket::AcceptOffer {
            timestamp,
            offer_counter,
            amount,
        } => {
            commands.trigger(MarketOfferAcceptIntent {
                entity,
                offer_id: MarketOfferId::new(timestamp, offer_counter),
                accepted_amount: amount,
            });
        }
    }
}
