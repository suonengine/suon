//! Snapshot loading and saving contracts for typed database tables.

use anyhow::Result;
use std::fmt::Debug;

use crate::{
    Table,
    connection::{DatabaseConnection, DatabaseData},
};

/// Snapshot-oriented trait for `Table` implementations that can be loaded from
/// and persisted back to a backing store.
pub trait SnapshotTable: Table {
    /// Row type exchanged between the table and the persistence backend.
    type Row: Clone + Debug + Send + Sync + 'static;

    /// Replaces the table contents from rows loaded from the backend.
    fn replace_rows(&mut self, rows: Vec<Self::Row>);

    /// Extracts rows from the table for persistence.
    fn rows(&self) -> Vec<Self::Row>;
}

/// Mapper that knows how to initialize schema and convert one table snapshot to
/// and from a specific backing store.
pub trait TableMapper<T: SnapshotTable, D: DatabaseData>: Send + Sync + 'static {
    /// Creates or updates the schema required by this mapper.
    fn initialize_schema(&self, _: &DatabaseConnection<D>) -> Result<()> {
        Ok(())
    }

    /// Loads rows for the target table from the backend.
    fn load_rows(&self, database: &DatabaseConnection<D>) -> Result<Vec<T::Row>>;

    /// Saves rows for the target table into the backend.
    fn save_rows(&self, database: &DatabaseConnection<D>, rows: &[T::Row]) -> Result<()>;
}

/// Convenience methods implemented for every [`SnapshotTable`].
pub trait SnapshotTableExt: SnapshotTable + Sized {
    /// Runs schema initialization through the mapper for the given backend.
    fn initialize_schema<D, M>(&self, database: &DatabaseConnection<D>, mapper: &M) -> Result<()>
    where
        D: DatabaseData,
        M: TableMapper<Self, D>,
    {
        mapper.initialize_schema(database)
    }

    /// Loads rows from the backend and replaces the table contents.
    fn load_from_database<D, M>(
        &mut self,
        database: &DatabaseConnection<D>,
        mapper: &M,
    ) -> Result<usize>
    where
        D: DatabaseData,
        M: TableMapper<Self, D>,
    {
        let rows = mapper.load_rows(database)?;
        let count = rows.len();
        self.replace_rows(rows);
        Ok(count)
    }

    /// Saves the current table contents into the backend.
    fn save_to_database<D, M>(&self, database: &DatabaseConnection<D>, mapper: &M) -> Result<usize>
    where
        D: DatabaseData,
        M: TableMapper<Self, D>,
    {
        let rows = self.rows();
        let count = rows.len();
        mapper.save_rows(database, &rows)?;
        Ok(count)
    }
}

impl<T> SnapshotTableExt for T where T: SnapshotTable {}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;
    use std::sync::Mutex;

    use crate::connection::{DatabaseConnection, DatabaseData};

    #[derive(Default)]
    struct DemoTable {
        rows: Vec<u32>,
    }

    impl Table for DemoTable {}

    impl SnapshotTable for DemoTable {
        type Row = u32;

        fn replace_rows(&mut self, rows: Vec<Self::Row>) {
            self.rows = rows;
        }

        fn rows(&self) -> Vec<Self::Row> {
            self.rows.clone()
        }
    }

    struct DemoData;

    impl DatabaseData for DemoData {}

    #[derive(Default)]
    struct DemoMapper {
        stored: Mutex<Vec<u32>>,
        initialize_calls: Mutex<usize>,
    }

    impl TableMapper<DemoTable, DemoData> for DemoMapper {
        fn initialize_schema(&self, _: &DatabaseConnection<DemoData>) -> Result<()> {
            *self
                .initialize_calls
                .lock()
                .expect("initialize mutex should stay available") += 1;
            Ok(())
        }

        fn load_rows(&self, _: &DatabaseConnection<DemoData>) -> Result<Vec<u32>> {
            Ok(self
                .stored
                .lock()
                .expect("demo mutex should stay available")
                .clone())
        }

        fn save_rows(&self, _: &DatabaseConnection<DemoData>, rows: &[u32]) -> Result<()> {
            *self
                .stored
                .lock()
                .expect("demo mutex should stay available") = rows.to_vec();
            Ok(())
        }
    }

    #[test]
    fn snapshot_helpers_should_replace_and_save_rows() {
        let database = DatabaseConnection::new(DemoData);
        let mapper = DemoMapper {
            stored: Mutex::new(vec![7, 11]),
            initialize_calls: Mutex::new(0),
        };

        let mut table = DemoTable::default();
        table
            .initialize_schema(&database, &mapper)
            .expect("schema initialization should succeed");

        let loaded = table
            .load_from_database(&database, &mapper)
            .expect("table should load");

        assert_eq!(
            loaded, 2,
            "load_from_database should report the number of rows loaded from the mapper"
        );

        assert_eq!(
            table.rows(),
            vec![7, 11],
            "load_from_database should replace the table rows with the mapper output"
        );

        table.replace_rows(vec![3, 5, 8]);
        let saved = table
            .save_to_database(&database, &mapper)
            .expect("table should save");

        assert_eq!(
            saved, 3,
            "save_to_database should report the number of rows written to the mapper"
        );

        assert_eq!(
            mapper
                .stored
                .lock()
                .expect("demo mutex should stay available")
                .as_slice(),
            &[3, 5, 8],
            "save_to_database should forward the current table rows to the mapper"
        );

        assert_eq!(
            *mapper
                .initialize_calls
                .lock()
                .expect("initialize mutex should stay available"),
            1,
            "initialize_schema should delegate schema setup to the mapper exactly once"
        );
    }

    #[test]
    fn should_use_default_initialize_schema_when_mapper_does_not_override_it() {
        struct MinimalMapper;

        impl TableMapper<DemoTable, DemoData> for MinimalMapper {
            fn load_rows(&self, _: &DatabaseConnection<DemoData>) -> Result<Vec<u32>> {
                Ok(Vec::new())
            }

            fn save_rows(&self, _: &DatabaseConnection<DemoData>, _: &[u32]) -> Result<()> {
                Ok(())
            }
        }

        let database = DatabaseConnection::new(DemoData);

        let table = DemoTable::default();
        table
            .initialize_schema(&database, &MinimalMapper)
            .expect("the default initialize_schema implementation should succeed");

        assert!(
            table.rows().is_empty(),
            "The default initialize_schema implementation should not mutate the target table"
        );
    }

    #[test]
    fn should_not_mutate_table_when_loading_rows_fails() {
        struct FailingLoadMapper;

        impl TableMapper<DemoTable, DemoData> for FailingLoadMapper {
            fn load_rows(&self, _: &DatabaseConnection<DemoData>) -> Result<Vec<u32>> {
                Err(anyhow!("load failed"))
            }

            fn save_rows(&self, _: &DatabaseConnection<DemoData>, _: &[u32]) -> Result<()> {
                Ok(())
            }
        }

        let database = DatabaseConnection::new(DemoData);
        let mut table = DemoTable { rows: vec![99] };

        let error = table
            .load_from_database(&database, &FailingLoadMapper)
            .expect_err("load_from_database should propagate mapper failures");

        assert!(
            error.to_string().contains("load failed"),
            "load_from_database should preserve the mapper failure context"
        );

        assert_eq!(
            table.rows(),
            vec![99],
            "load_from_database should leave the existing table state untouched on failure"
        );
    }

    #[test]
    fn should_propagate_mapper_errors_when_saving_rows() {
        struct FailingSaveMapper;

        impl TableMapper<DemoTable, DemoData> for FailingSaveMapper {
            fn load_rows(&self, _: &DatabaseConnection<DemoData>) -> Result<Vec<u32>> {
                Ok(Vec::new())
            }

            fn save_rows(&self, _: &DatabaseConnection<DemoData>, _: &[u32]) -> Result<()> {
                Err(anyhow!("save failed"))
            }
        }

        let database = DatabaseConnection::new(DemoData);
        let table = DemoTable {
            rows: vec![1, 2, 3],
        };

        let error = table
            .save_to_database(&database, &FailingSaveMapper)
            .expect_err("save_to_database should propagate mapper failures");

        assert!(
            error.to_string().contains("save failed"),
            "save_to_database should preserve the mapper failure context"
        );
    }
}
