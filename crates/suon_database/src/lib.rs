//! Database tables and persistence helpers for Bevy apps.
//!
//! This crate combines two layers:
//! - typed ECS table resources for gameplay systems
//! - task-backed persistence helpers for loading and saving those tables
//!
//! # Modules
//!
//! - [`prelude::DatabaseConnection`], [`prelude::DatabaseData`], and
//!   [`prelude::DatabasePool`] for
//!   backend-neutral connections and the default SQL pool
//! - database startup systems for loading settings into the Bevy world
//! - [`prelude::SnapshotTable`], [`prelude::SnapshotTableExt`], and
//!   [`prelude::TableMapper`] for table snapshot loading and saving contracts
//! - [`prelude::DatabaseSettings`] and [`prelude::DatabaseSettingsBuilder`] for
//!   generic connection settings
//! - [`prelude::FieldTryIntoExt`],
//!   [`prelude::SystemTimeDatabaseConvertExt`], and
//!   integer conversion helpers for loss-checked integer and time
//!   conversions used by mappers
//!
//! # Examples
//! ```no_run
//! use bevy::prelude::*;
//! use suon_database::{AppTablesExt, DatabaseMut, Table};
//!
//! #[derive(Default)]
//! struct HealthTable {
//!     hp: u32,
//! }
//!
//! impl Table for HealthTable {}
//!
//! let mut app = App::new();
//! app.init_database_table::<HealthTable>();
//!
//! app.add_systems(Update, |mut table: DatabaseMut<HealthTable>| {
//!     table.hp = 42;
//! });
//! ```

mod connection;
mod convert;
mod settings;
mod snapshot;
mod system;

use crate::system::*;
use bevy::{ecs::system::SystemParam, prelude::*};

/// Common imports for apps and infrastructure crates that use `suon_database`.
pub mod prelude {
    pub use super::{
        AppTablesExt, Database, DatabaseMut, DatabasePlugin, Table, Tables,
        connection::{DatabaseBackend, DatabaseConnection, DatabaseData, DatabasePool, PoolData},
        convert::{FieldTryIntoExt, SystemTimeDatabaseConvertExt},
        settings::{DatabaseSettings, DatabaseSettingsBuilder},
        snapshot::{SnapshotTable, SnapshotTableExt, TableMapper},
    };
}

/// Plugin that loads database settings into the Bevy world during startup.
pub struct DatabasePlugin;

impl Plugin for DatabasePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_settings);
    }
}

/// Trait that marks a structure as a database table.
/// Types implementing `Table` can be stored in resource tables.
pub trait Table: Send + Sync + 'static {}

/// Resource that holds a specific table of type `T`.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct Tables<T: Table> {
    /// Inner table value stored as a Bevy resource.
    table: T,
}

/// System parameter for immutable access to a table of type `E`.
#[derive(SystemParam, Deref, DerefMut)]
pub struct Database<'w, E: Table> {
    /// Shared access to the resource that stores the typed table.
    #[system_param(validation_message = "Table not initialized")]
    tables: Res<'w, Tables<E>>,
}

/// System parameter for mutable access to a table of type `E`.
#[derive(SystemParam, Deref, DerefMut)]
pub struct DatabaseMut<'w, E: Table> {
    /// Mutable access to the resource that stores the typed table.
    #[system_param(validation_message = "Table not initialized")]
    tables: ResMut<'w, Tables<E>>,
}

/// Extension trait providing convenience methods for managing database tables within Bevy's `App`.
pub trait AppTablesExt {
    /// Initializes a typed table resource with its default value when missing.
    fn init_database_table<T: Table + Default>(&mut self) -> &mut Self;

    /// Inserts or replaces a typed table resource with a concrete value.
    fn insert_database_table<T: Table>(&mut self, table: T) -> &mut Self;
}

impl AppTablesExt for App {
    fn init_database_table<T: Table + Default>(&mut self) -> &mut Self {
        self.init_resource::<Tables<T>>();
        self
    }

    fn insert_database_table<T: Table>(&mut self, table: T) -> &mut Self {
        self.insert_resource(Tables { table });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::{DatabaseConnection, DatabaseData};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct MyTable {
        pub value: bool,
    }

    impl Table for MyTable {}

    #[test]
    fn should_initialize_database_table_resource() {
        let mut app = App::new();
        app.init_database_table::<MyTable>();

        assert!(
            app.world().get_resource::<Tables<MyTable>>().is_some(),
            "init_database_table should create a Tables<MyTable> resource"
        );
    }

    #[test]
    fn should_access_table_through_database_system_params() {
        let mut app = App::new();
        app.insert_database_table(MyTable { value: false });

        app.add_systems(PreUpdate, |table: Database<MyTable>| {
            assert!(
                !table.value,
                "Database<T> should expose the initial table value before mutation"
            );
        })
        .add_systems(Update, |mut table: DatabaseMut<MyTable>| {
            table.value = true;
        })
        .add_systems(PostUpdate, |table: Database<MyTable>| {
            assert!(
                table.value,
                "Database<T> should observe the value written through DatabaseMut<T>"
            );
        });

        app.update();

        let resource = app.world().get_resource::<Tables<MyTable>>();
        assert!(
            resource.is_some(),
            "Tables<MyTable> should remain registered after systems run"
        );

        assert!(
            resource.unwrap().value,
            "The stored table value should be updated to true after the update system"
        );
    }

    #[test]
    fn should_keep_existing_table_when_initializing_again() {
        let mut app = App::new();
        app.insert_database_table(MyTable { value: true });
        app.init_database_table::<MyTable>();

        let table = app.world().get_resource::<Tables<MyTable>>().unwrap();
        assert!(
            table.value,
            "init_database_table should preserve an already inserted table resource"
        );
    }

    #[test]
    fn should_overwrite_existing_table_when_inserting_again() {
        let mut app = App::new();
        app.insert_database_table(MyTable { value: false });
        app.insert_database_table(MyTable { value: true });

        let table = app
            .world()
            .get_resource::<Tables<MyTable>>()
            .expect("Tables<MyTable> should exist after repeated insertion");

        assert!(
            table.value,
            "insert_database_table should overwrite the previously inserted table value"
        );
    }

    #[test]
    fn should_allow_table_extension_methods_to_chain() {
        let mut app = App::new();
        let returned = app
            .init_database_table::<MyTable>()
            .insert_database_table(MyTable { value: true });

        assert!(
            std::ptr::eq(returned, &app),
            "AppTablesExt methods should return the same App reference for fluent chaining"
        );
    }

    #[test]
    fn should_access_database_mut_through_prelude() {
        use crate::prelude::*;

        #[derive(Default)]
        struct PreludeTable {
            value: usize,
        }

        impl Table for PreludeTable {}

        let mut app = App::new();
        app.init_database_table::<PreludeTable>();

        app.add_systems(Update, |mut table: DatabaseMut<PreludeTable>| {
            table.value = 11;
        });

        app.update();

        assert_eq!(
            app.world()
                .get_resource::<Tables<PreludeTable>>()
                .unwrap()
                .value,
            11,
            "The prelude should expose DatabaseMut<T> so systems can mutate typed tables"
        );
    }

    #[test]
    fn should_expose_database_api_through_prelude() {
        use crate::prelude::*;

        #[derive(Default)]
        struct PreludeTable {
            _value: usize,
        }

        impl Table for PreludeTable {}

        struct PreludeData;

        impl DatabaseData for PreludeData {}

        let connection = DatabaseConnection::new(PreludeData);

        let mut app = App::new();
        app.init_database_table::<PreludeTable>();

        let _ = std::mem::size_of::<Database<'static, PreludeTable>>();
        let _ = std::mem::size_of::<DatabaseMut<'static, PreludeTable>>();
        let _ = std::mem::size_of::<DatabasePlugin>();
        let _ = std::mem::size_of::<Tables<PreludeTable>>();
        let _ = connection.data();

        assert!(
            app.world().contains_resource::<Tables<PreludeTable>>(),
            "The prelude should expose AppTablesExt and typed table accessors"
        );
    }

    #[test]
    fn should_allow_direct_resource_access_to_typed_tables() {
        let mut app = App::new();
        app.insert_database_table(MyTable { value: true });

        let table = app
            .world_mut()
            .get_resource_mut::<Tables<MyTable>>()
            .expect("Tables<MyTable> should be available for direct mutable access");

        assert!(
            table.value,
            "Tables<T> should dereference to the inserted table value for direct resource access"
        );
    }

    #[test]
    fn should_default_tables_resource_using_table_default() {
        let table = Tables::<MyTable>::default();

        assert!(
            !table.value,
            "Tables<T>::default should build the inner table from T::default"
        );
    }

    #[test]
    fn should_allow_tables_resource_to_mutate_through_deref_mut() {
        let mut table = Tables {
            table: MyTable { value: false },
        };

        table.value = true;

        assert!(
            table.table.value,
            "Tables<T> should derive DerefMut so callers can mutate the inner table directly"
        );
    }

    #[test]
    fn should_support_task_backed_connections_for_app_level_tests() {
        #[derive(Clone)]
        struct DemoData {
            flag: Arc<Mutex<bool>>,
        }

        impl DatabaseData for DemoData {}

        let flag = Arc::new(Mutex::new(false));
        let connection = DatabaseConnection::new(DemoData { flag: flag.clone() });

        connection.block_on(async {
            *flag.lock().expect("flag mutex should stay available") = true;
        });

        assert!(
            *connection
                .data()
                .flag
                .lock()
                .expect("flag mutex should stay available"),
            "Connection::block_on should execute futures through Bevy task utilities"
        );
    }
}
