//! Market systems grouped by browsing, offers, history, and persistence.
//!
//! This crate sits above the protocol and networking layers. It keeps market
//! reference data in typed database tables, loads those tables during startup
//! through the `suon_database` persistence pipeline, and listens to typed
//! market packets through Bevy observers.
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
mod protocol;
mod session;

pub use persistence::MarketHistoryJournal;

pub mod prelude {
    pub use super::{
        MarketPlugins,
        browse::{BrowseMarket, BrowseMarketIntent, BrowseMarketRejected},
        history::{MarketHistoryAction, MarketHistoryEntry, ParseMarketHistoryActionError},
        offer::{
            MarketActorName, MarketActorsTable, MarketItemsTable, MarketOffer,
            MarketOfferAcceptIntent, MarketOfferAcceptRejected, MarketOfferAccepted,
            MarketOfferCancelIntent, MarketOfferCancelRejected, MarketOfferCancelled,
            MarketOfferCreateError, MarketOfferCreateIntent, MarketOfferCreateRejected,
            MarketOfferCreated, MarketOfferId, MarketOffersTable, MarketTradeSide,
            ParseMarketTradeSideError,
        },
        persistence::{
            MarketHistoryJournal, MarketPersistenceSettings, MarketPolicySettings, MarketSettings,
        },
        session::{CloseMarketSessionIntent, MarketActorRef, MarketSession},
    };
}

/// Plugin group that wires the full market domain into a Bevy app.
pub struct MarketPlugins;

impl PluginGroup for MarketPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(persistence::MarketPersistencePlugin)
            .add(history::MarketHistoryPlugin)
            .add(session::MarketSessionPlugin)
            .add(browse::MarketBrowsePlugin)
            .add(offer::MarketOfferPlugin)
            .add(protocol::MarketProtocolPlugin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::{
        MarketActorName, MarketActorRef, MarketActorsTable, MarketItemsTable, MarketOffer,
        MarketOfferCancelled, MarketOfferId, MarketOffersTable, MarketTradeSide,
    };
    use std::time::{Duration, UNIX_EPOCH};
    use suon_database::prelude::*;
    use suon_protocol_client::prelude::MarketBrowseKind;

    /// Builds a Bevy app pre-wired with the market plugin group on top of an
    /// in-memory SQLite connection.
    fn test_app() -> App {
        let settings = DbSettingsBuilder {
            database_url: "sqlite::memory:".to_string(),
            ..DbSettingsBuilder::default()
        }
        .build()
        .expect("memory settings should build");
        let connection =
            DbConnection::open(&settings).expect("opening sqlite memory should succeed");

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(settings);
        app.insert_resource(connection);
        app.add_plugins(MarketPlugins);
        app
    }

    #[test]
    fn should_build_market_plugin_group() {
        let mut app = test_app();
        app.update();

        assert_eq!(std::mem::size_of::<MarketPlugins>(), 0);
    }

    #[test]
    fn should_initialize_market_tables_when_plugins_are_added() {
        let app = test_app();

        assert!(app.world().contains_resource::<Tables<MarketActorsTable>>());
        assert!(app.world().contains_resource::<Tables<MarketItemsTable>>());
        assert!(app.world().contains_resource::<Tables<MarketOffersTable>>());
    }

    #[test]
    fn should_load_market_tables_from_database_during_startup() {
        let mut app = test_app();
        let connection = app.world().resource::<DbConnection>().clone();

        MarketActorsTable::initialize_schema(&connection).expect("schema setup should succeed");
        MarketItemsTable::initialize_schema(&connection).expect("schema setup should succeed");
        MarketOffersTable::initialize_schema(&connection).expect("schema setup should succeed");

        MarketActorsTable::save(&connection, &[MarketActorName::new(7, "Ramon")])
            .expect("actor save should succeed");
        MarketItemsTable::save(&connection, &[(2160, "Crystal Coin".to_string())])
            .expect("item save should succeed");
        MarketOffersTable::save(
            &connection,
            &[MarketOffer::new(
                MarketOfferId::new(UNIX_EPOCH, 9),
                2160,
                7,
                3,
                100_000,
                MarketTradeSide::Sell,
                false,
            )],
        )
        .expect("offer save should succeed");

        app.update();

        let actors = app.world().resource::<Tables<MarketActorsTable>>();
        let items = app.world().resource::<Tables<MarketItemsTable>>();
        let offers = app.world().resource::<Tables<MarketOffersTable>>();

        assert_eq!(actors.name(7), Some("Ramon"));
        assert_eq!(items.name(2160), Some("Crystal Coin"));
        assert!(offers.get(&MarketOfferId::new(UNIX_EPOCH, 9)).is_some());
    }

    #[test]
    fn should_create_market_offer_inside_market_crate() {
        let mut app = test_app();
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
    }

    #[test]
    fn should_cancel_market_offer_inside_market_crate() {
        let mut app = test_app();
        app.insert_dbtable(MarketOffersTable::from_iter([MarketOffer::new(
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
            entity: client,
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
    }

    #[test]
    fn should_accept_market_offer_and_reduce_remaining_amount() {
        let mut app = test_app();
        app.insert_dbtable(MarketOffersTable::from_iter([MarketOffer::new(
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
    fn should_mark_market_snapshot_dirty_on_table_mutation() {
        let mut app = test_app();
        app.update();

        {
            let mut offers = app.world_mut().resource_mut::<Tables<MarketOffersTable>>();
            offers.insert(MarketOffer::new(
                MarketOfferId::new(UNIX_EPOCH, 3),
                2160,
                77,
                2,
                11_000,
                MarketTradeSide::Sell,
                false,
            ));
        }

        let offers = app.world().resource::<Tables<MarketOffersTable>>();
        assert!(
            offers.is_dirty(),
            "mutating a persistent market table should mark it dirty for background saving"
        );
    }

    #[test]
    fn should_append_history_from_market_events() {
        let mut app = test_app();
        app.update();
        let connection = app.world().resource::<DbConnection>().clone();

        let client = app.world_mut().spawn(MarketActorRef::new(77)).id();
        app.world_mut().trigger(crate::offer::MarketOfferCreated {
            entity: client,
            offer: MarketOffer::new(
                MarketOfferId::new(UNIX_EPOCH, 4),
                2160,
                77,
                2,
                11_000,
                MarketTradeSide::Sell,
                false,
            ),
        });
        app.update();

        use diesel::{RunQueryDsl, sql_query, sql_types::BigInt};

        #[derive(diesel::QueryableByName)]
        struct CountRow {
            #[diesel(sql_type = BigInt)]
            count: i64,
        }

        let count = connection
            .execute(|driver| {
                sql_query("SELECT COUNT(*) AS count FROM market_history WHERE action = 'create'")
                    .get_result::<CountRow>(driver)
                    .map_err(anyhow::Error::from)
            })
            .expect("counting history rows should succeed");

        assert_eq!(
            count.count, 1,
            "creating an offer should append a history entry"
        );
    }

    #[test]
    fn should_store_market_browse_kind_in_session() {
        let session = session::MarketSession::new(Some(MarketBrowseKind::OwnOffers));
        assert_eq!(session.last_browse(), Some(MarketBrowseKind::OwnOffers));
    }
}
