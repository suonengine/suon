//! Lightweight database-table resources for Bevy apps.
//!
//! This crate wraps typed resources in a small table abstraction so systems can
//! read and mutate game data through focused `SystemParam` types instead of
//! reaching for raw resources directly.

use bevy::{ecs::system::SystemParam, prelude::*};

pub mod prelude {
    pub use super::{AppTablesExt, Database, Table, Tables};
}

/// Trait that marks a structure as a database table.
/// Types implementing `Table` can be stored in resource tables.
///
/// # Example
/// ```ignore
/// #[derive(Table)]
/// struct MyTable;
/// ```
pub trait Table: Send + Sync + 'static {}

/// Resource that holds a specific table of type `T`.
/// Provides shared access to the table.
#[derive(Resource, Deref, DerefMut, Default)]
pub struct Tables<T: Table> {
    /// The actual table data.
    table: T,
}

/// System parameter for immutable access to a table of type `E`.
#[derive(SystemParam, Deref, DerefMut)]
pub struct Database<'w, E: Table> {
    /// Reference to the resource containing the table.
    #[system_param(validation_message = "Table not initialized")]
    tables: Res<'w, Tables<E>>,
}

/// System parameter for mutable access to a table of type `E`.
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
    fn init_database_table<T: Table + Default>(&mut self) -> &mut Self;

    /// Inserts a specific instance of a table `table` into the app's resources.
    /// Overwrites any existing resource of the same type.
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
}
