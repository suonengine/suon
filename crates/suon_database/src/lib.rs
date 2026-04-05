//! Lightweight database-table resources for Bevy apps.
//!
//! This crate wraps typed resources in a small table abstraction so systems can
//! read and mutate game data through focused `SystemParam` types instead of
//! reaching for raw resources directly.
//!
//! # Examples
//! ```
//! use bevy::prelude::*;
//! use suon_database::{AppTablesExt, Database, DatabaseMut, Table};
//!
//! #[derive(Default)]
//! struct HealthTable {
//!     hp: u32,
//! }
//!
//! impl Table for HealthTable {}
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! app.init_database_table::<HealthTable>();
//!
//! app.add_systems(Update, |mut table: DatabaseMut<HealthTable>| {
//!     table.hp = 42;
//! });
//! app.add_systems(PostUpdate, |table: Database<HealthTable>| {
//!     assert_eq!(table.hp, 42);
//! });
//!
//! app.update();
//! ```

use bevy::{ecs::system::SystemParam, prelude::*};

pub mod prelude {
    pub use super::{AppTablesExt, Database, DatabaseMut, Table, Tables};
}

/// Trait that marks a structure as a database table.
/// Types implementing `Table` can be stored in resource tables.
///
/// # Examples
/// ```
/// use suon_database::Table;
///
/// struct MyTable;
///
/// impl Table for MyTable {}
/// ```
pub trait Table: Send + Sync + 'static {}

/// Resource that holds a specific table of type `T`.
/// Provides shared access to the table.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use suon_database::{AppTablesExt, Table, Tables};
///
/// #[derive(Default)]
/// struct MyTable {
///     value: bool,
/// }
///
/// impl Table for MyTable {}
///
/// let mut app = App::new();
/// app.init_database_table::<MyTable>();
///
/// assert!(app.world().get_resource::<Tables<MyTable>>().is_some());
/// ```
#[derive(Resource, Deref, DerefMut, Default)]
pub struct Tables<T: Table> {
    /// The actual table data.
    table: T,
}

/// System parameter for immutable access to a table of type `E`.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use suon_database::{AppTablesExt, Database, Table};
///
/// #[derive(Default)]
/// struct MyTable {
///     value: u32,
/// }
///
/// impl Table for MyTable {}
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.insert_database_table(MyTable { value: 7 });
/// app.add_systems(Update, |table: Database<MyTable>| {
///     assert_eq!(table.value, 7);
/// });
///
/// app.update();
/// ```
#[derive(SystemParam, Deref, DerefMut)]
pub struct Database<'w, E: Table> {
    /// Reference to the resource containing the table.
    #[system_param(validation_message = "Table not initialized")]
    tables: Res<'w, Tables<E>>,
}

/// System parameter for mutable access to a table of type `E`.
///
/// # Examples
/// ```
/// use bevy::prelude::*;
/// use suon_database::{AppTablesExt, DatabaseMut, Table, Tables};
///
/// #[derive(Default)]
/// struct MyTable {
///     value: u32,
/// }
///
/// impl Table for MyTable {}
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.init_database_table::<MyTable>();
/// app.add_systems(Update, |mut table: DatabaseMut<MyTable>| {
///     table.value = 9;
/// });
///
/// app.update();
///
/// assert_eq!(
///     app.world().get_resource::<Tables<MyTable>>().unwrap().value,
///     9
/// );
/// ```
#[derive(SystemParam, Deref, DerefMut)]
pub struct DatabaseMut<'w, E: Table> {
    /// Mutable reference to the resource containing the table.
    #[system_param(validation_message = "Table not initialized")]
    tables: ResMut<'w, Tables<E>>,
}

/// Extension trait providing convenience methods for managing database tables within Bevy's `App`.
pub trait AppTablesExt {
    /// Initializes a resource for the specified table type `T` with its default value.
    /// If the resource already exists, this does nothing.
    ///
    /// # Examples
    /// ```
    /// use bevy::prelude::*;
    /// use suon_database::{AppTablesExt, Table, Tables};
    ///
    /// #[derive(Default)]
    /// struct MyTable {
    ///     value: bool,
    /// }
    ///
    /// impl Table for MyTable {}
    ///
    /// let mut app = App::new();
    /// app.init_database_table::<MyTable>();
    ///
    /// assert!(app.world().get_resource::<Tables<MyTable>>().is_some());
    /// ```
    fn init_database_table<T: Table + Default>(&mut self) -> &mut Self;

    /// Inserts a specific instance of a table `table` into the app's resources.
    /// Overwrites any existing resource of the same type.
    ///
    /// # Examples
    /// ```
    /// use bevy::prelude::*;
    /// use suon_database::{AppTablesExt, Table, Tables};
    ///
    /// #[derive(Default)]
    /// struct MyTable {
    ///     value: bool,
    /// }
    ///
    /// impl Table for MyTable {}
    ///
    /// let mut app = App::new();
    /// app.insert_database_table(MyTable { value: true });
    ///
    /// assert!(app.world().get_resource::<Tables<MyTable>>().unwrap().value);
    /// ```
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
            "Tables<MyTable> should be created after init_database_table"
        );
    }

    #[test]
    fn should_access_table_through_database_system_params() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.insert_database_table(MyTable { value: false });

        app.add_systems(PreUpdate, |table: Database<MyTable>| {
            assert!(!table.value, "Initial value should be false");
        })
        .add_systems(Update, |mut table: DatabaseMut<MyTable>| {
            table.value = true;
        })
        .add_systems(PostUpdate, |table: Database<MyTable>| {
            assert!(table.value, "Value should be true after update");
        });

        app.update();

        let resource = app.world().get_resource::<Tables<MyTable>>();
        assert!(resource.is_some(), "Tables<MyTable> should still exist");

        let table = resource.unwrap();
        assert!(table.value, "Table value should be true after update");
    }

    #[test]
    fn should_mutate_table_resource_directly() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.insert_database_table(MyTable { value: false });

        {
            let resource = app.world_mut().get_resource_mut::<Tables<MyTable>>();
            assert!(
                resource.is_some(),
                "Tables<MyTable> resource should exist for mutation"
            );
            let mut table = resource.unwrap();
            assert!(!table.value, "Initial value should be false");
            table.value = true;
        }

        {
            let resource = app.world().get_resource::<Tables<MyTable>>();
            assert!(
                resource.is_some(),
                "Tables<MyTable> should exist after mutation"
            );
            let table = resource.unwrap();
            assert!(table.value, "Table value should be true after mutation");
        }
    }

    #[test]
    fn should_keep_existing_table_when_initializing_again() {
        let mut app = App::new();

        app.insert_database_table(MyTable { value: true });
        app.init_database_table::<MyTable>();

        let table = app
            .world()
            .get_resource::<Tables<MyTable>>()
            .expect("Tables<MyTable> should exist after initialization");

        assert!(
            table.value,
            "init_database_table should preserve an already inserted table resource"
        );
    }

    #[test]
    fn should_overwrite_previous_table_when_inserting_again() {
        let mut app = App::new();

        app.insert_database_table(MyTable { value: false });
        app.insert_database_table(MyTable { value: true });

        let table = app
            .world()
            .get_resource::<Tables<MyTable>>()
            .expect("Tables<MyTable> should exist after insertion");

        assert!(
            table.value,
            "insert_database_table should overwrite the previous table resource"
        );
    }

    #[test]
    fn should_access_database_mut_through_prelude() {
        use crate::prelude::{AppTablesExt, DatabaseMut, Table, Tables};

        #[derive(Default)]
        struct PreludeTable {
            value: usize,
        }

        impl Table for PreludeTable {}

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_database_table::<PreludeTable>();

        app.add_systems(Update, |mut table: DatabaseMut<PreludeTable>| {
            table.value = 11;
        });

        app.update();

        assert_eq!(
            app.world()
                .get_resource::<Tables<PreludeTable>>()
                .expect("Tables<PreludeTable> should exist after initialization")
                .value,
            11,
            "The prelude should expose DatabaseMut for mutable table access"
        );
    }

    #[test]
    fn should_chain_table_extension_methods() {
        let mut app = App::new();

        let returned = app
            .init_database_table::<MyTable>()
            .insert_database_table(MyTable { value: true });

        assert!(
            std::ptr::eq(returned, &app),
            "AppTablesExt methods should return the same App reference for chaining"
        );
    }
}
