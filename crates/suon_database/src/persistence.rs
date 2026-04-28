//! Persistence traits and background save scheduling for typed tables.
//!
//! This module replaces the old `SnapshotTable` + `TableMapper` split with two
//! focused traits implemented directly on the table type:
//!
//! - [`DbTable`] for snapshot persistence (load/save all rows at once).
//! - [`DbAppend`] for append-only journals (insert-only history).
//!
//! Tables are registered with the app via [`AppDbPersistenceExt`]:
//!
//! ```no_run
//! use bevy::prelude::*;
//! use suon_database::prelude::*;
//!
//! let mut app = App::new();
//! app.add_plugins(DbPlugin);
//! // app.init_db_persistent::<MyTable>();
//! // app.init_db_journal::<MyJournal>();
//! ```

use std::{any::type_name, marker::PhantomData, time::Duration};

use anyhow::Result;
use bevy::{
    app::AppExit,
    prelude::*,
    tasks::{IoTaskPool, Task, futures_lite::future},
};

use crate::{
    connection::DbConnection,
    settings::DbSettings,
    table::{AppDbExt, Table, Tables},
};

/// Snapshot-style persistent table that loads and saves all rows at once.
///
/// Implementors plug their existing in-memory state into [`replace_rows`] and
/// [`rows`], then describe how rows reach the database in
/// [`initialize_schema`], [`load`], and [`save`].
///
/// [`replace_rows`]: DbTable::replace_rows
/// [`rows`]: DbTable::rows
/// [`initialize_schema`]: DbTable::initialize_schema
/// [`load`]: DbTable::load
/// [`save`]: DbTable::save
pub trait DbTable: Table + Default {
    /// Row type exchanged between the table and the persistence backend.
    type Row: Clone + Send + Sync + 'static;

    /// Replaces the table contents with rows loaded from the backend.
    fn replace_rows(&mut self, rows: Vec<Self::Row>);

    /// Extracts a snapshot of the current rows for persistence.
    fn rows(&self) -> Vec<Self::Row>;

    /// Creates or updates the schema this table requires.
    fn initialize_schema(_: &DbConnection) -> Result<()> {
        Ok(())
    }

    /// Loads all rows from the backend.
    fn load(connection: &DbConnection) -> Result<Vec<Self::Row>>;

    /// Replaces all rows in the backend with `rows`.
    fn save(connection: &DbConnection, rows: &[Self::Row]) -> Result<()>;
}

/// Append-only journal where rows are inserted but never bulk-replaced.
///
/// Use this for audit logs and history tables where the in-memory state is
/// not authoritative — rows live only in the database.
pub trait DbAppend: Send + Sync + 'static {
    /// Row type appended into the backend.
    type Row: Send + Sync + 'static;

    /// Creates or updates the schema this journal requires.
    fn initialize_schema(_: &DbConnection) -> Result<()> {
        Ok(())
    }

    /// Appends a single row to the backend.
    fn append(connection: &DbConnection, row: &Self::Row) -> Result<()>;
}

/// Per-table persistence settings: flush cadence, shutdown behaviour, and
/// optional connection override.
#[derive(Resource, Debug, Clone)]
pub struct DbTableSettings<T: DbTable> {
    flush_interval: Duration,
    save_on_shutdown: bool,
    connection_override: Option<DbSettings>,
    _table: PhantomData<fn() -> T>,
}

impl<T: DbTable> DbTableSettings<T> {
    /// Creates settings for a persistent table.
    pub fn new(
        flush_interval: Duration,
        save_on_shutdown: bool,
        connection_override: Option<DbSettings>,
    ) -> Self {
        Self {
            flush_interval,
            save_on_shutdown,
            connection_override,
            _table: PhantomData,
        }
    }

    /// Returns the flush interval used to throttle background saves.
    pub fn flush_interval(&self) -> Duration {
        self.flush_interval
    }

    /// Returns whether shutdown should wait for a final save.
    pub fn save_on_shutdown(&self) -> bool {
        self.save_on_shutdown
    }

    /// Returns the per-table connection override, if set.
    pub fn connection_override(&self) -> Option<&DbSettings> {
        self.connection_override.as_ref()
    }
}

impl<T: DbTable> Default for DbTableSettings<T> {
    fn default() -> Self {
        Self::new(Duration::from_secs(60), true, None)
    }
}

/// Cached connection for tables that opt into a custom database override.
#[derive(Resource)]
struct DbTableConnection<T: DbTable> {
    connection: DbConnection,
    _table: PhantomData<fn() -> T>,
}

#[derive(Resource)]
struct DbTableTask<T: DbTable> {
    timer: Timer,
    pending: Option<PendingSave<T>>,
}

impl<T: DbTable> Default for DbTableTask<T> {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(60.0, TimerMode::Repeating),
            pending: None,
        }
    }
}

struct PendingSave<T: DbTable> {
    epoch: u64,
    task: Task<Result<usize>>,
    _table: PhantomData<fn() -> T>,
}

impl<T: DbTable> PendingSave<T> {
    fn new(epoch: u64, task: Task<Result<usize>>) -> Self {
        Self {
            epoch,
            task,
            _table: PhantomData,
        }
    }

    fn poll(&mut self) -> Option<Result<usize>> {
        future::block_on(future::poll_once(&mut self.task))
    }

    fn drain(self) -> Result<usize> {
        future::block_on(self.task)
    }
}

/// Extension trait registering persistent tables and journals on a Bevy [`App`].
pub trait AppDbPersistenceExt: AppDbExt {
    /// Registers a persistent snapshot table backed by [`DbTable`].
    ///
    /// Schedules the load on [`Startup`] and a periodic flush plus
    /// shutdown-drain on [`Update`]. Defaults to [`DbTableSettings::default`].
    fn init_db_persistent<T: DbTable>(&mut self) -> &mut Self;

    /// Registers a persistent snapshot table with custom [`DbTableSettings`].
    fn insert_db_table_settings<T: DbTable>(&mut self, settings: DbTableSettings<T>) -> &mut Self;

    /// Registers an append-only journal backed by [`DbAppend`].
    ///
    /// Schedules a one-shot schema initialization on [`Startup`].
    fn init_db_journal<T: DbAppend>(&mut self) -> &mut Self;
}

impl AppDbPersistenceExt for App {
    fn init_db_persistent<T: DbTable>(&mut self) -> &mut Self {
        register_persistent::<T>(self);
        self
    }

    fn insert_db_table_settings<T: DbTable>(&mut self, settings: DbTableSettings<T>) -> &mut Self {
        register_persistent::<T>(self);
        self.insert_resource(settings);
        self
    }

    fn init_db_journal<T: DbAppend>(&mut self) -> &mut Self {
        self.add_systems(Startup, initialize_journal::<T>);
        self
    }
}

fn register_persistent<T: DbTable>(app: &mut App) {
    if !app.world().contains_resource::<DbTableSettings<T>>() {
        app.init_resource::<DbTableSettings<T>>();
    }

    app.init_db_table::<T>()
        .init_resource::<DbTableTask<T>>()
        .add_systems(Startup, (open_table_override::<T>, load_table::<T>).chain())
        .add_systems(
            Update,
            (
                configure_timer::<T>,
                queue_save::<T>,
                poll_save::<T>,
                drain_on_exit::<T>,
            ),
        );
}

fn open_table_override<T: DbTable>(
    mut commands: Commands,
    settings: Res<DbTableSettings<T>>,
    existing: Option<Res<DbTableConnection<T>>>,
) {
    if existing.is_some() {
        return;
    }

    let Some(override_settings) = settings.connection_override() else {
        return;
    };

    match DbConnection::open(override_settings) {
        Ok(connection) => {
            commands.insert_resource(DbTableConnection::<T> {
                connection,
                _table: PhantomData,
            });
        }
        Err(error) => warn!(
            "Failed to open table-specific database connection for {}: {error:#}",
            type_name::<T>()
        ),
    }
}

fn load_table<T: DbTable>(
    settings: Res<DbSettings>,
    table_settings: Res<DbTableSettings<T>>,
    shared_connection: Option<Res<DbConnection>>,
    table_connection: Option<Res<DbTableConnection<T>>>,
    mut tables: ResMut<Tables<T>>,
) {
    let connection = active_connection::<T>(
        shared_connection.as_deref(),
        table_connection.as_deref(),
    );

    let Some(connection) = connection else {
        warn!(
            "Skipping load for {}: no DbConnection available (override set: {})",
            type_name::<T>(),
            table_settings.connection_override().is_some()
        );
        return;
    };

    let result = (|| {
        if settings.auto_initialize_schema() || table_settings.connection_override().is_some() {
            T::initialize_schema(connection)?;
        }

        let rows = T::load(connection)?;
        let count = rows.len();

        let mut loaded = T::default();
        loaded.replace_rows(rows);
        tables.replace_loaded(loaded);

        Ok::<_, anyhow::Error>(count)
    })();

    match result {
        Ok(count) => debug!(
            "Loaded {count} rows for persistent table {}",
            type_name::<T>()
        ),
        Err(error) => warn!(
            "Failed to load persistent table {}: {error:#}",
            type_name::<T>()
        ),
    }
}

fn configure_timer<T: DbTable>(
    settings: Res<DbTableSettings<T>>,
    mut task: ResMut<DbTableTask<T>>,
) {
    if !settings.is_changed() {
        return;
    }

    task.timer = Timer::from_seconds(
        settings.flush_interval().as_secs_f32().max(0.001),
        TimerMode::Repeating,
    );
}

fn queue_save<T: DbTable>(
    time: Res<Time>,
    shared_connection: Option<Res<DbConnection>>,
    table_connection: Option<Res<DbTableConnection<T>>>,
    tables: Res<Tables<T>>,
    mut task: ResMut<DbTableTask<T>>,
) {
    if task.pending.is_some() || !tables.is_dirty() {
        return;
    }

    if !task.timer.tick(time.delta()).just_finished() {
        return;
    }

    let Some(connection) = active_connection::<T>(
        shared_connection.as_deref(),
        table_connection.as_deref(),
    ) else {
        warn!(
            "Persistent table {} is dirty but no DbConnection is available",
            type_name::<T>()
        );
        return;
    };

    let connection = connection.clone();
    let rows = tables.rows();
    let epoch = tables.dirty_epoch();
    let task_handle = IoTaskPool::get().spawn(async move {
        let count = rows.len();
        T::save(&connection, &rows).map(|_| count)
    });

    task.pending = Some(PendingSave::<T>::new(epoch, task_handle));
}

fn poll_save<T: DbTable>(
    mut tables: ResMut<Tables<T>>,
    mut task: ResMut<DbTableTask<T>>,
) {
    let Some(pending) = task.pending.as_mut() else {
        return;
    };

    let Some(result) = pending.poll() else {
        return;
    };

    let epoch = pending.epoch;
    task.pending = None;

    match result {
        Ok(count) => {
            tables.mark_persisted(epoch);
            debug!(
                "Persisted {count} rows for table {}",
                type_name::<T>()
            );
        }
        Err(error) => warn!(
            "Failed to persist table {}: {error:#}",
            type_name::<T>()
        ),
    }
}

fn drain_on_exit<T: DbTable>(
    mut exits: MessageReader<AppExit>,
    settings: Res<DbTableSettings<T>>,
    shared_connection: Option<Res<DbConnection>>,
    table_connection: Option<Res<DbTableConnection<T>>>,
    mut tables: ResMut<Tables<T>>,
    mut task: ResMut<DbTableTask<T>>,
) {
    if exits.read().last().is_none() || !settings.save_on_shutdown() {
        return;
    }

    if let Some(pending) = task.pending.take() {
        let epoch = pending.epoch;
        match pending.drain() {
            Ok(count) => {
                tables.mark_persisted(epoch);
                debug!(
                    "Drained pending save of {count} rows for table {}",
                    type_name::<T>()
                );
            }
            Err(error) => warn!(
                "Pending save failed while draining table {}: {error:#}",
                type_name::<T>()
            ),
        }
    }

    if !tables.is_dirty() {
        return;
    }

    let Some(connection) = active_connection::<T>(
        shared_connection.as_deref(),
        table_connection.as_deref(),
    ) else {
        warn!(
            "App exit requested with dirty {}, but no DbConnection is available",
            type_name::<T>()
        );
        return;
    };

    let epoch = tables.dirty_epoch();
    let rows = tables.rows();
    match T::save(connection, &rows) {
        Ok(_) => {
            tables.mark_persisted(epoch);
            debug!(
                "Persisted final {} rows for table {}",
                rows.len(),
                type_name::<T>()
            );
        }
        Err(error) => warn!(
            "Failed to persist final table {} snapshot: {error:#}",
            type_name::<T>()
        ),
    }
}

fn initialize_journal<T: DbAppend>(connection: Option<Res<DbConnection>>) {
    let Some(connection) = connection else {
        warn!(
            "Skipping schema initialization for journal {}: DbConnection not available",
            type_name::<T>()
        );
        return;
    };

    if let Err(error) = T::initialize_schema(&connection) {
        warn!(
            "Failed to initialize schema for journal {}: {error:#}",
            type_name::<T>()
        );
    }
}

impl<T: DbTable> std::ops::Deref for DbTableConnection<T> {
    type Target = DbConnection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

fn active_connection<'a, T: DbTable>(
    shared: Option<&'a DbConnection>,
    table: Option<&'a DbTableConnection<T>>,
) -> Option<&'a DbConnection> {
    if let Some(table) = table {
        return Some(&table.connection);
    }
    shared
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        record::DbRecord,
        settings::DbSettingsBuilder,
        table::{DbMut, Table},
    };
    use anyhow::Context;
    use diesel::{ExpressionMethods, RunQueryDsl, sql_query, sql_types::Text};

    diesel::table! {
        notes (id) {
            id -> Integer,
            text -> Text,
        }
    }

    #[derive(Debug, Clone, diesel::Queryable, diesel::Selectable, diesel::Insertable)]
    #[diesel(table_name = notes)]
    struct NoteRecord {
        id: i32,
        text: String,
    }

    impl diesel::associations::HasTable for NoteRecord {
        type Table = notes::table;
        fn table() -> Self::Table {
            notes::table
        }
    }

    impl DbRecord for NoteRecord {
        type Query = diesel::helper_types::Select<
            notes::table,
            diesel::helper_types::AsSelect<Self, <crate::connection::DbDriver as diesel::Connection>::Backend>,
        >;
        type Columns = ();

        fn query() -> Self::Query {
            use diesel::{QueryDsl, SelectableHelper};
            notes::table.select(Self::as_select())
        }

        fn columns() -> Self::Columns {}
    }

    fn ensure_notes_schema(connection: &DbConnection) -> Result<()> {
        connection.execute(|driver| {
            sql_query("CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY, text TEXT NOT NULL)")
                .execute(driver)
                .map(|_| ())
                .map_err(anyhow::Error::from)
        })
    }

    #[derive(Default)]
    struct NotesTable {
        rows: Vec<(i32, String)>,
    }

    impl Table for NotesTable {}

    impl DbTable for NotesTable {
        type Row = (i32, String);

        fn replace_rows(&mut self, rows: Vec<Self::Row>) {
            self.rows = rows;
        }

        fn rows(&self) -> Vec<Self::Row> {
            self.rows.clone()
        }

        fn initialize_schema(connection: &DbConnection) -> Result<()> {
            ensure_notes_schema(connection)
        }

        fn load(connection: &DbConnection) -> Result<Vec<Self::Row>> {
            connection
                .execute(|driver| {
                    use diesel::QueryDsl;
                    notes::table
                        .order(notes::id.asc())
                        .load::<NoteRecord>(driver)
                        .map_err(anyhow::Error::from)
                })
                .context("loading notes failed")?
                .into_iter()
                .map(|note| Ok((note.id, note.text)))
                .collect()
        }

        fn save(connection: &DbConnection, rows: &[Self::Row]) -> Result<()> {
            let records: Vec<NoteRecord> = rows
                .iter()
                .map(|(id, text)| NoteRecord {
                    id: *id,
                    text: text.clone(),
                })
                .collect();

            connection.transaction(|driver| {
                diesel::delete(notes::table)
                    .execute(driver)
                    .context("clearing notes failed")?;
                for record in &records {
                    diesel::insert_into(notes::table)
                        .values(record)
                        .execute(driver)
                        .context("inserting note failed")?;
                }
                Ok(())
            })
        }
    }

    diesel::table! {
        notes_log (id) {
            id -> Integer,
            text -> Text,
        }
    }

    #[derive(Debug, Clone, diesel::Insertable)]
    #[diesel(table_name = notes_log)]
    struct NewLogRecord<'a> {
        text: &'a str,
    }

    impl diesel::associations::HasTable for NewLogRecord<'_> {
        type Table = notes_log::table;
        fn table() -> Self::Table {
            notes_log::table
        }
    }

    struct NotesJournal;

    impl DbAppend for NotesJournal {
        type Row = String;

        fn initialize_schema(connection: &DbConnection) -> Result<()> {
            connection.execute(|driver| {
                sql_query(
                    "CREATE TABLE IF NOT EXISTS notes_log (id INTEGER PRIMARY KEY, text TEXT NOT \
                     NULL)",
                )
                .execute(driver)
                .map(|_| ())
                .map_err(anyhow::Error::from)
            })
        }

        fn append(connection: &DbConnection, row: &Self::Row) -> Result<()> {
            connection.execute(|driver| {
                diesel::insert_into(notes_log::table)
                    .values(NewLogRecord { text: row.as_str() })
                    .execute(driver)
                    .map(|_| ())
                    .map_err(anyhow::Error::from)
            })
        }
    }

    fn in_memory_connection() -> (DbConnection, DbSettings) {
        let settings = DbSettingsBuilder {
            database_url: "sqlite::memory:".to_string(),
            ..DbSettingsBuilder::default()
        }
        .build()
        .expect("builder should create in-memory sqlite settings");

        let connection =
            DbConnection::open(&settings).expect("opening sqlite memory should succeed");
        (connection, settings)
    }

    #[derive(diesel::QueryableByName)]
    struct LogText {
        #[diesel(sql_type = Text)]
        text: String,
    }

    #[test]
    fn db_table_should_persist_full_snapshot_through_save_and_load() {
        let (connection, _settings) = in_memory_connection();
        ensure_notes_schema(&connection).expect("schema setup should succeed");

        NotesTable::save(&connection, &[(1, "first".to_string()), (2, "second".to_string())])
            .expect("save should succeed");

        let loaded = NotesTable::load(&connection).expect("load should succeed");
        assert_eq!(
            loaded,
            vec![(1, "first".to_string()), (2, "second".to_string())]
        );
    }

    #[test]
    fn db_append_should_insert_rows_through_append() {
        let (connection, _settings) = in_memory_connection();
        NotesJournal::initialize_schema(&connection).expect("schema setup should succeed");

        NotesJournal::append(&connection, &"hello".to_string()).expect("append should succeed");
        NotesJournal::append(&connection, &"world".to_string()).expect("append should succeed");

        let rows: Vec<LogText> = connection
            .execute(|driver| {
                sql_query("SELECT text FROM notes_log ORDER BY id")
                    .load::<LogText>(driver)
                    .map_err(anyhow::Error::from)
            })
            .expect("loading log rows should succeed");

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].text, "hello");
        assert_eq!(rows[1].text, "world");
    }

    #[test]
    fn init_db_persistent_should_load_table_from_storage_during_startup() {
        let (connection, settings) = in_memory_connection();
        NotesTable::initialize_schema(&connection).expect("schema setup should succeed");
        NotesTable::save(&connection, &[(7, "preloaded".to_string())])
            .expect("save should succeed");

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(settings);
        app.insert_resource(connection);
        app.init_db_persistent::<NotesTable>();
        app.update();

        let table = app.world().resource::<Tables<NotesTable>>();
        assert_eq!(table.rows.len(), 1);
        assert_eq!(table.rows[0], (7, "preloaded".to_string()));
        assert!(
            !table.is_dirty(),
            "loading rows from storage should not flag the table as dirty"
        );
    }

    #[test]
    fn dbmut_should_mark_persistent_table_dirty() {
        let (connection, settings) = in_memory_connection();
        NotesTable::initialize_schema(&connection).expect("schema setup should succeed");

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(settings);
        app.insert_resource(connection);
        app.init_db_persistent::<NotesTable>();

        app.add_systems(Update, |mut table: DbMut<NotesTable>| {
            table.rows.push((1, "new".to_string()));
        });

        app.update();

        let table = app.world().resource::<Tables<NotesTable>>();
        assert!(
            table.is_dirty(),
            "DbMut mutations should auto-bump the dirty epoch"
        );
    }
}
