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
            MarketBrowse, MarketBrowsePlugin, MarketBrowseScope, MarketPlayerRef,
            MarketRequestKind, MarketRequestKindError, MarketSession,
            TryMarketRequestKindFromPacketError,
        },
        history::{
            MarketHistoryAction, MarketHistoryEntry, MarketHistoryPlugin, MarketHistoryTable,
            ParseMarketHistoryActionError,
        },
        offer::{
            MarketItem, MarketItemsTable, MarketOffer, MarketOfferAccepted, MarketOfferCancelled,
            MarketOfferCreated, MarketOfferId, MarketOfferPlugin, MarketOffersTable,
            MarketPlayersTable, MarketTradeSide, ParseMarketTradeSideError, PlayerName,
        },
        persistence::{
            MarketOrm, MarketOrmResource, MarketPersistencePlugin, MarketPersistenceSettings,
            MarketPolicySettings, MarketSettings, SaveMarketData, ShutdownMarketData,
        },
    };
}

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
        MarketHistoryAction, MarketHistoryEntry, MarketHistoryTable, MarketItem, MarketItemsTable,
        MarketOffer, MarketOfferCancelled, MarketOfferId, MarketOffersTable, MarketOrm,
        MarketOrmResource, MarketPlayersTable, MarketTradeSide, PlayerName, SaveMarketData,
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
        players: Mutex<Vec<PlayerName>>,
        items: Mutex<Vec<MarketItem>>,
        offers: Mutex<Vec<MarketOffer>>,
        history: Mutex<Vec<MarketHistoryEntry>>,
    }

    impl MarketOrm for RecordingOrm {
        fn load_players(&self) -> anyhow::Result<Vec<PlayerName>> {
            Ok(self
                .players
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
                .contains_resource::<Tables<MarketPlayersTable>>()
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
            players: Mutex::new(vec![PlayerName {
                id: 7,
                name: "Ramon".into(),
            }]),
            items: Mutex::new(vec![MarketItem {
                id: 2160,
                name: "Crystal Coin".into(),
            }]),
            offers: Mutex::new(vec![MarketOffer {
                id: MarketOfferId {
                    timestamp: UNIX_EPOCH,
                    counter: 9,
                },
                item_id: 2160,
                player_id: 7,
                amount: 3,
                price: 100_000,
                side: MarketTradeSide::Sell,
                is_anonymous: false,
            }]),
            history: Mutex::new(vec![MarketHistoryEntry {
                id: 3,
                recorded_at: UNIX_EPOCH,
                action: MarketHistoryAction::Create,
                actor_player_id: Some(7),
                offer_player_id: Some(7),
                item_id: Some(2160),
                offer_id: Some(MarketOfferId {
                    timestamp: UNIX_EPOCH,
                    counter: 9,
                }),
                amount: 3,
                remaining_amount: None,
                price: Some(100_000),
                side: Some(MarketTradeSide::Sell),
            }]),
            ..Default::default()
        });
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(MarketOrmResource::new(orm));
        app.add_plugins(MarketPlugins);
        app.update();

        let players = app.world().resource::<Tables<MarketPlayersTable>>();
        let items = app.world().resource::<Tables<MarketItemsTable>>();
        let offers = app.world().resource::<Tables<MarketOffersTable>>();
        let history = app.world().resource::<Tables<MarketHistoryTable>>();

        assert_eq!(players.name(7), Some("Ramon"));
        assert_eq!(items.name(2160), Some("Crystal Coin"));
        assert!(
            offers
                .get(&MarketOfferId {
                    timestamp: UNIX_EPOCH,
                    counter: 9
                })
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
        let offer = MarketOffer {
            id: MarketOfferId {
                timestamp: UNIX_EPOCH,
                counter: 1,
            },
            item_id: 2160,
            player_id: 77,
            amount: 5,
            price: 20_000,
            side: MarketTradeSide::Sell,
            is_anonymous: false,
        };

        {
            let mut offers = app.world_mut().resource_mut::<Tables<MarketOffersTable>>();
            offers.create_offer(offer.clone());
        }

        let offers = app.world().resource::<Tables<MarketOffersTable>>();
        assert!(offers.get(&offer.id).is_some());
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
        app.insert_database_table(MarketOffersTable::from_iter([MarketOffer {
            id: MarketOfferId {
                timestamp: UNIX_EPOCH,
                counter: 1,
            },
            item_id: 2160,
            player_id: 77,
            amount: 5,
            price: 20_000,
            side: MarketTradeSide::Sell,
            is_anonymous: false,
        }]));
        let client = app.world_mut().spawn_empty().id();
        let event = MarketOfferCancelled {
            client,
            player_id: Some(77),
            player_name: None,
            offer_id: MarketOfferId {
                timestamp: UNIX_EPOCH,
                counter: 1,
            },
            offer: Some(MarketOffer {
                id: MarketOfferId {
                    timestamp: UNIX_EPOCH,
                    counter: 1,
                },
                item_id: 2160,
                player_id: 77,
                amount: 5,
                price: 20_000,
                side: MarketTradeSide::Sell,
                is_anonymous: false,
            }),
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
        app.insert_database_table(MarketOffersTable::from_iter([MarketOffer {
            id: MarketOfferId {
                timestamp: UNIX_EPOCH + Duration::from_secs(5),
                counter: 2,
            },
            item_id: 2160,
            player_id: 77,
            amount: 5,
            price: 20_000,
            side: MarketTradeSide::Sell,
            is_anonymous: false,
        }]));

        let offer_id = MarketOfferId {
            timestamp: UNIX_EPOCH + Duration::from_secs(5),
            counter: 2,
        };

        {
            let mut offers = app.world_mut().resource_mut::<Tables<MarketOffersTable>>();
            let updated = crate::offer::accept_offer(
                offer_id,
                2,
                Some(MarketOffer {
                    id: offer_id,
                    item_id: 2160,
                    player_id: 77,
                    amount: 5,
                    price: 20_000,
                    side: MarketTradeSide::Sell,
                    is_anonymous: false,
                }),
                &mut offers,
            );
            assert!(updated.is_some());
        }

        let offers = app.world().resource::<Tables<MarketOffersTable>>();
        let remaining = offers
            .get(&offer_id)
            .expect("offer should remain after partial accept");
        assert_eq!(remaining.amount, 3);
    }

    #[test]
    fn should_persist_market_snapshot_on_save_request() {
        let orm = Arc::new(RecordingOrm::default());
        let mut app = App::new();

        app.add_plugins(MinimalPlugins);
        app.insert_resource(MarketOrmResource::new(orm.clone()));
        app.add_plugins(MarketPlugins);
        app.insert_database_table(MarketOffersTable::from_iter([MarketOffer {
            id: MarketOfferId {
                timestamp: UNIX_EPOCH,
                counter: 3,
            },
            item_id: 2160,
            player_id: 77,
            amount: 2,
            price: 11_000,
            side: MarketTradeSide::Sell,
            is_anonymous: false,
        }]));
        app.insert_database_table(MarketHistoryTable::default());
        app.world_mut()
            .resource_mut::<Tables<MarketHistoryTable>>()
            .append(MarketHistoryEntry {
                id: 1,
                recorded_at: UNIX_EPOCH,
                action: MarketHistoryAction::Create,
                actor_player_id: Some(77),
                offer_player_id: Some(77),
                item_id: Some(2160),
                offer_id: Some(MarketOfferId {
                    timestamp: UNIX_EPOCH,
                    counter: 3,
                }),
                amount: 2,
                remaining_amount: None,
                price: Some(11_000),
                side: Some(MarketTradeSide::Sell),
            });
        app.world_mut().resource_mut::<persistence::MarketDirty>().0 = true;
        app.world_mut().trigger(SaveMarketData);
        app.update();

        let actions = orm
            .actions
            .lock()
            .expect("recording mutex should stay available");
        assert!(
            actions
                .iter()
                .any(|action| matches!(action, RecordedAction::Create(offer) if offer.amount == 2))
        );
        assert!(
            actions
                .iter()
                .any(|action| matches!(action, RecordedAction::History(_)))
        );
    }
}
