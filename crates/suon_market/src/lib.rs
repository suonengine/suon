//! Market systems grouped by browsing, offers, history, and persistence.
//!
//! This crate sits above the protocol and networking layers. It keeps market
//! reference data in typed database tables, loads those tables during startup
//! through an ORM-style provider, and listens to typed market packets through
//! Bevy observers.
//!
//! # Examples
//! ```no_run
//! use bevy::prelude::*;
//! use suon_market::prelude::*;
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! app.add_plugins(MarketPlugins);
//! ```

use bevy::{app::PluginGroupBuilder, prelude::*};

mod browse;
mod history;
mod offer;
mod persistence;

pub mod prelude {
    pub use super::{
        MarketPlugins,
        browse::{
            MarketActorRef, MarketBrowse, MarketBrowseIntent, MarketBrowseRejected,
            MarketBrowseScope, MarketSession, TryMarketRequestKindFromPacketError,
        },
        history::{
            MarketHistoryAction, MarketHistoryEntry, MarketHistoryTable,
            ParseMarketHistoryActionError,
        },
        offer::{
            MarketActorName, MarketActorsTable, MarketItem, MarketItemsTable, MarketOffer,
            MarketOfferAcceptIntent, MarketOfferAcceptRejected, MarketOfferAccepted,
            MarketOfferCancelIntent, MarketOfferCancelRejected, MarketOfferCancelled,
            MarketOfferCreateError, MarketOfferCreateIntent, MarketOfferCreateRejected,
            MarketOfferCreated, MarketOfferId, MarketOffersTable, MarketTradeSide,
            ParseMarketTradeSideError,
        },
        persistence::{
            MarketOrm, MarketOrmResource, MarketPersistenceSettings, MarketPolicySettings,
            MarketSettings,
        },
    };
}

/// Plugin group that wires the full market domain into a Bevy app.
pub struct MarketPlugins;

impl PluginGroup for MarketPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(persistence::MarketPersistencePlugin)
            .add(history::MarketHistoryPlugin)
            .add(browse::MarketBrowsePlugin)
            .add(offer::MarketOfferPlugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{
        MarketActorName, MarketActorsTable, MarketHistoryAction, MarketHistoryEntry,
        MarketHistoryTable, MarketItem, MarketItemsTable, MarketOffer, MarketOfferCancelled,
        MarketOfferId, MarketOffersTable, MarketOrm, MarketOrmResource, MarketTradeSide,
    };
    use std::{
        sync::{Arc, Mutex},
        time::{Duration, UNIX_EPOCH},
    };
    use suon_database::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum RecordedAction {
        Create(MarketOffer),
        History(MarketHistoryEntry),
    }

    #[derive(Default)]
    struct RecordingOrm {
        actions: Mutex<Vec<RecordedAction>>,
        actors: Mutex<Vec<MarketActorName>>,
        items: Mutex<Vec<MarketItem>>,
        offers: Mutex<Vec<MarketOffer>>,
        history: Mutex<Vec<MarketHistoryEntry>>,
    }

    impl MarketOrm for RecordingOrm {
        fn load_actors(&self) -> anyhow::Result<Vec<MarketActorName>> {
            Ok(self
                .actors
                .lock()
                .expect("recording mutex should stay available")
                .clone())
        }

        fn load_items(&self) -> anyhow::Result<Vec<MarketItem>> {
            Ok(self
                .items
                .lock()
                .expect("recording mutex should stay available")
                .clone())
        }

        fn load_offers(&self) -> anyhow::Result<Vec<MarketOffer>> {
            Ok(self
                .offers
                .lock()
                .expect("recording mutex should stay available")
                .clone())
        }

        fn load_history(&self) -> anyhow::Result<Vec<MarketHistoryEntry>> {
            Ok(self
                .history
                .lock()
                .expect("recording mutex should stay available")
                .clone())
        }

        fn save_offers(&self, offers: &[MarketOffer]) -> anyhow::Result<()> {
            self.actions
                .lock()
                .expect("recording mutex should stay available")
                .extend(offers.iter().cloned().map(RecordedAction::Create));
            Ok(())
        }

        fn save_history(&self, history: &[MarketHistoryEntry]) -> anyhow::Result<()> {
            self.actions
                .lock()
                .expect("recording mutex should stay available")
                .extend(history.iter().cloned().map(RecordedAction::History));
            Ok(())
        }
    }

    #[test]
    fn should_build_market_plugin_group() {
        let orm = Arc::new(RecordingOrm::default());
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(MarketOrmResource::new(orm));
        app.add_plugins(MarketPlugins);
        app.update();

        assert_eq!(std::mem::size_of::<MarketPlugins>(), 0);
    }

    #[test]
    fn should_initialize_market_tables_when_plugins_are_added() {
        let orm = Arc::new(RecordingOrm::default());
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(MarketOrmResource::new(orm));
        app.add_plugins(MarketPlugins);

        assert!(
            app.world()
                .contains_resource::<Tables<MarketActorsTable>>()
        );
        assert!(app.world().contains_resource::<Tables<MarketItemsTable>>());
        assert!(app.world().contains_resource::<Tables<MarketOffersTable>>());
        assert!(
            app.world()
                .contains_resource::<Tables<MarketHistoryTable>>()
        );
    }

    #[test]
    fn should_load_market_tables_from_orm_during_startup() {
        let orm = Arc::new(RecordingOrm {
            actors: Mutex::new(vec![MarketActorName::new(7, "Ramon")]),
            items: Mutex::new(vec![MarketItem::new(2160, "Crystal Coin")]),
            offers: Mutex::new(vec![MarketOffer::new(
                MarketOfferId::new(UNIX_EPOCH, 9),
                2160,
                7,
                3,
                100_000,
                MarketTradeSide::Sell,
                false,
            )]),
            history: Mutex::new(vec![MarketHistoryEntry::new(
                3,
                UNIX_EPOCH,
                MarketHistoryAction::Create,
                Some(7),
                Some(7),
                Some(2160),
                Some(MarketOfferId::new(UNIX_EPOCH, 9)),
                3,
                None,
                Some(100_000),
                Some(MarketTradeSide::Sell),
            )]),
            ..Default::default()
        });
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(MarketOrmResource::new(orm));
        app.add_plugins(MarketPlugins);
        app.update();

        let actors = app.world().resource::<Tables<MarketActorsTable>>();
        let items = app.world().resource::<Tables<MarketItemsTable>>();
        let offers = app.world().resource::<Tables<MarketOffersTable>>();
        let history = app.world().resource::<Tables<MarketHistoryTable>>();

        assert_eq!(actors.name(7), Some("Ramon"));
        assert_eq!(items.name(2160), Some("Crystal Coin"));
        assert!(
            offers
                .get(&MarketOfferId::new(UNIX_EPOCH, 9))
                .is_some()
        );
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn should_create_market_offer_inside_market_crate() {
        let orm = Arc::new(RecordingOrm::default());
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(MarketOrmResource::new(orm.clone()));
        app.add_plugins(MarketPlugins);
        let offer = MarketOffer::new(
            MarketOfferId::new(UNIX_EPOCH, 1),
            2160,
            77,
            5,
            20_000,
            MarketTradeSide::Sell,
            false,
        );

        {
            let mut offers = app.world_mut().resource_mut::<Tables<MarketOffersTable>>();
            offers.create_offer(offer.clone());
        }

        let offers = app.world().resource::<Tables<MarketOffersTable>>();
        assert!(offers.get(&offer.id()).is_some());
        assert!(
            orm.actions
                .lock()
                .expect("recording mutex should stay available")
                .is_empty()
        );
    }

    #[test]
    fn should_cancel_market_offer_inside_market_crate() {
        let orm = Arc::new(RecordingOrm::default());
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(MarketOrmResource::new(orm.clone()));
        app.add_plugins(MarketPlugins);
        app.insert_database_table(MarketOffersTable::from_iter([MarketOffer::new(
            MarketOfferId::new(UNIX_EPOCH, 1),
            2160,
            77,
            5,
            20_000,
            MarketTradeSide::Sell,
            false,
        )]));
        let client = app.world_mut().spawn_empty().id();
        let event = MarketOfferCancelled {
            client,
            actor_id: Some(77),
            offer_id: MarketOfferId::new(UNIX_EPOCH, 1),
            offer: Some(MarketOffer::new(
                MarketOfferId::new(UNIX_EPOCH, 1),
                2160,
                77,
                5,
                20_000,
                MarketTradeSide::Sell,
                false,
            )),
        };

        {
            let mut offers = app.world_mut().resource_mut::<Tables<MarketOffersTable>>();
            crate::offer::cancel_offer(&event, &mut offers);
        }

        let offers = app.world().resource::<Tables<MarketOffersTable>>();
        assert_eq!(offers.len(), 0);
        assert!(
            orm.actions
                .lock()
                .expect("recording mutex should stay available")
                .is_empty()
        );
    }

    #[test]
    fn should_accept_market_offer_and_reduce_remaining_amount() {
        let orm = Arc::new(RecordingOrm::default());
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(MarketOrmResource::new(orm.clone()));
        app.add_plugins(MarketPlugins);
        app.insert_database_table(MarketOffersTable::from_iter([MarketOffer::new(
            MarketOfferId::new(UNIX_EPOCH + Duration::from_secs(5), 2),
            2160,
            77,
            5,
            20_000,
            MarketTradeSide::Sell,
            false,
        )]));

        let offer_id = MarketOfferId::new(UNIX_EPOCH + Duration::from_secs(5), 2);

        {
            let mut offers = app.world_mut().resource_mut::<Tables<MarketOffersTable>>();
            let updated = crate::offer::accept_offer(
                offer_id,
                2,
                Some(MarketOffer::new(
                    offer_id,
                    2160,
                    77,
                    5,
                    20_000,
                    MarketTradeSide::Sell,
                    false,
                )),
                &mut offers,
            );
            assert!(updated.is_some());
        }

        let offers = app.world().resource::<Tables<MarketOffersTable>>();
        let remaining = offers
            .get(&offer_id)
            .expect("offer should remain after partial accept");
        assert_eq!(remaining.amount(), 3);
    }

    #[test]
    fn should_persist_market_snapshot_on_app_exit() {
        let orm = Arc::new(RecordingOrm::default());
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(MarketOrmResource::new(orm.clone()));
        app.add_plugins(MarketPlugins);
        app.update();
        app.insert_database_table(MarketOffersTable::from_iter([MarketOffer::new(
            MarketOfferId::new(UNIX_EPOCH, 3),
            2160,
            77,
            2,
            11_000,
            MarketTradeSide::Sell,
            false,
        )]));
        app.insert_database_table(MarketHistoryTable::default());
        app.world_mut()
            .resource_mut::<Tables<MarketHistoryTable>>()
            .append(MarketHistoryEntry::new(
                1,
                UNIX_EPOCH,
                MarketHistoryAction::Create,
                Some(77),
                Some(77),
                Some(2160),
                Some(MarketOfferId::new(UNIX_EPOCH, 3)),
                2,
                None,
                Some(11_000),
                Some(MarketTradeSide::Sell),
            ));
        app.world_mut().resource_mut::<persistence::MarketDirty>().mark();
        app.world_mut().write_message(bevy::app::AppExit::Success);
        app.update();

        let actions = orm
            .actions
            .lock()
            .expect("recording mutex should stay available");
        assert!(
            actions
                .iter()
                .any(|action| matches!(action, RecordedAction::Create(offer) if offer.amount() == 2))
        );
        assert!(
            actions
                .iter()
                .any(|action| matches!(action, RecordedAction::History(_)))
        );
    }
}
