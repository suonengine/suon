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

    #[test]
    fn test_init_and_insert_table() {
        // Define a simple struct that implements the Table trait
        #[derive(Default)]
        struct MyTable;
        impl Table for MyTable {}

        // Create a new Bevy app
        let mut app = App::new();

        // Initialize the database table resource
        app.init_database_table::<MyTable>();

        // Verify that the Tables<MyTable> resource exists
        assert!(
            app.world().get_resource::<Tables<MyTable>>().is_some(),
            "Tables<MyTable> should be created after init_database_table"
        );
    }

    #[test]
    fn test_access_table_immutable() {
        // Define a table struct with a boolean field
        #[derive(Default)]
        struct MyTable {
            pub value: bool,
        }
        impl Table for MyTable {}

        // Set up app with minimal plugins
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Insert a table instance into the database
        app.insert_database_table(MyTable { value: false });

        // Check the initial value before modification
        app.add_systems(PreUpdate, |table: Database<MyTable>| {
            assert!(!table.value, "Initial value should be false");
        })
        // Modify the table's value
        .add_systems(Update, |mut table: DatabaseMut<MyTable>| {
            table.value = true;
        })
        // Verify the value after modification
        .add_systems(PostUpdate, |table: Database<MyTable>| {
            assert!(table.value, "Value should be true after update");
        });

        // Run the systems
        app.update();

        // Confirm the resource still exists
        let resource = app.world().get_resource::<Tables<MyTable>>();
        assert!(resource.is_some(), "Tables<MyTable> should still exist");

        // Confirm the value has been updated
        let table = resource.unwrap();
        assert!(table.value, "Table value should be true after update");
    }

    #[test]
    fn test_access_table_mutably() {
        // Define a table struct with a boolean field
        #[derive(Default)]
        struct MyTable {
            pub value: bool,
        }
        impl Table for MyTable {}

        // Set up app with minimal plugins
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Insert a table instance into the database
        app.insert_database_table(MyTable { value: false });

        // Mutably access the resource and modify its value
        {
            let resource = app.world_mut().get_resource_mut::<Tables<MyTable>>();
            assert!(
                resource.is_some(),
                "Tables<MyTable> resource should exist for mutation"
            );
            let mut table = resource.unwrap();
            // Check initial value
            assert!(!table.value, "Initial value should be false");
            // Change the value
            table.value = true;
        }

        // Confirm the change is reflected in the resource
        {
            let resource = app.world().get_resource::<Tables<MyTable>>();
            assert!(
                resource.is_some(),
                "Tables<MyTable> should exist after mutation"
            );
            let table = resource.unwrap();
            // Confirm updated value
            assert!(table.value, "Table value should be true after mutation");
        }
    }
}
