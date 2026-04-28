//! Diesel-backed database integration for Bevy apps.
//!
//! `suon_database` is the facilitator layer. It exposes one connection type
//! ([`DbConnection`]), one plugin ([`DbPlugin`]), and a small set of traits
//! (`Table`, `DbTable`, `DbAppend`) that wire user-defined structs into
//! typed Bevy resources backed by Diesel.
//!
//! # Quick start
//!
//! ```ignore
//! use bevy::prelude::*;
//! use suon_database::prelude::*;
//! use suon_macros::Table;
//!
//! #[derive(Table, Default)]
//! struct InventoryTable {
//!     items: Vec<u32>,
//! }
//!
//! fn read_inventory(inventory: Db<InventoryTable>) {
//!     for item in &inventory.items {
//!         println!("item {item}");
//!     }
//! }
//!
//! fn add_item(mut inventory: DbMut<InventoryTable>) {
//!     inventory.items.push(42); // DbMut auto-bumps the dirty epoch
//! }
//!
//! App::new()
//!     .add_plugins(MinimalPlugins)
//!     .add_plugins(DbPlugin)
//!     .init_db_table::<InventoryTable>()
//!     .add_systems(Startup, add_item)
//!     .add_systems(Update, read_inventory)
//!     .run();
//! ```
//!
//! [`DbConnection`]: crate::connection::DbConnection
//! [`DbPlugin`]: crate::plugin::DbPlugin

pub mod connection;
pub mod convert;
pub mod persistence;
pub mod plugin;
pub mod record;
pub mod settings;
pub mod table;

/// Common imports for apps that depend on `suon_database`.
pub mod prelude {
    pub use super::{
        connection::{DbBackend, DbConnection, DbDriver},
        convert::{FieldTryIntoExt, SystemTimeDbExt},
        persistence::{AppDbPersistenceExt, DbAppend, DbTable, DbTableSettings},
        plugin::DbPlugin,
        record::{DbRecord, PendingInsert, PendingStatement},
        settings::{DbSettings, DbSettingsBuilder},
        table::{AppDbExt, Db, DbMut, Table, Tables},
    };
    pub use diesel::{
        RunQueryDsl, sql_query,
        sql_types::{BigInt, Bool, Double, Float, Integer, Nullable, SmallInt, Text, Timestamp},
    };
}
