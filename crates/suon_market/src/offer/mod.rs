mod accept;
mod cancel;
mod create;
mod logic;
mod model;
mod tables;

use bevy::prelude::*;

pub use self::{
    accept::{MarketOfferAcceptIntent, MarketOfferAcceptRejected, MarketOfferAccepted},
    cancel::{MarketOfferCancelIntent, MarketOfferCancelRejected, MarketOfferCancelled},
    create::{
        MarketOfferCreateError, MarketOfferCreateIntent, MarketOfferCreateRejected,
        MarketOfferCreated,
    },
    model::{
        MarketActorName, MarketOffer, MarketOfferId, MarketTradeSide, ParseMarketTradeSideError,
    },
    tables::{MarketActorsTable, MarketItemsTable, MarketOffersTable},
};

pub(crate) use self::logic::{MarketOfferSequence, MarketRateLimiter, accept_offer, cancel_offer};

pub(crate) struct MarketOfferPlugin;

impl Plugin for MarketOfferPlugin {
    fn build(&self, app: &mut App) {
        info!("Starting the market offer systems");

        app.init_resource::<MarketOfferSequence>();
        app.init_resource::<MarketRateLimiter>();
        app.add_observer(create::on_create_market_offer_packet)
            .add_observer(cancel::on_cancel_market_offer_packet)
            .add_observer(accept::on_accept_market_offer_packet)
            .add_observer(create::on_create_market_offer_intent)
            .add_observer(cancel::on_cancel_market_offer_intent)
            .add_observer(accept::on_accept_market_offer_intent);
    }
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

        let first =
            settings
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
