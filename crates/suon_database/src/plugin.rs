//! App-level database plugin.
//!
//! [`DbPlugin`] is the only plugin a downstream app needs to add to wire the
//! database into Bevy. It loads [`DbSettings`] from disk during `PreStartup`
//! (or keeps an existing resource untouched), opens the shared
//! [`DbConnection`], and inserts both as resources so the rest of the app can
//! pull them via `Res`.
//!
//! Tables register themselves separately through
//! [`AppDbExt::init_db_table`](crate::table::AppDbExt::init_db_table) or
//! [`AppDbPersistenceExt`](crate::persistence::AppDbPersistenceExt) after the
//! plugin is added.

use bevy::prelude::*;

use crate::{connection::DbConnection, settings::DbSettings};

/// Plugin that loads database settings and opens the shared connection.
pub struct DbPlugin;

impl Plugin for DbPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, (load_settings, open_connection).chain());
    }
}

fn load_settings(mut commands: Commands, existing: Option<Res<DbSettings>>) {
    if let Some(settings) = existing {
        info!("Database settings already provided: {}", settings.summary());
        return;
    }

    info!("Loading database settings from disk.");
    let settings = DbSettings::load_or_default().expect("Failed to load database settings.");
    info!("Database settings loaded: {}", settings.summary());
    commands.insert_resource(settings);
}

fn open_connection(
    mut commands: Commands,
    settings: Res<DbSettings>,
    existing: Option<Res<DbConnection>>,
) {
    if existing.is_some() {
        return;
    }

    match DbConnection::open(&settings) {
        Ok(connection) => commands.insert_resource(connection),
        Err(error) => warn!(
            "Failed to open shared database connection ({}): {error:#}",
            settings.summary()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        settings::DbSettingsBuilder,
        table::{AppDbExt, Db, DbMut, Table, Tables},
    };
    use std::{
        env, fs,
        path::PathBuf,
        process,
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct CurrentDirGuard {
        previous: PathBuf,
    }

    impl CurrentDirGuard {
        fn enter(path: &PathBuf) -> Self {
            let previous = env::current_dir().expect("the test should read the current directory");
            env::set_current_dir(path).expect("the test should switch into the temp directory");
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            env::set_current_dir(&self.previous)
                .expect("the test should restore the previous current directory");
        }
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the unix epoch")
            .as_nanos();

        env::temp_dir().join(format!("suon-database-plugin-{}-{nanos}", process::id()))
    }

    #[derive(Default)]
    struct PreludeTable {
        value: usize,
    }

    impl Table for PreludeTable {}

    #[test]
    fn should_insert_database_settings_and_create_the_default_file() {
        let _lock = cwd_lock()
            .lock()
            .expect("the database plugin test should acquire the cwd lock");

        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("the temp test directory should be created");
        let _cwd_guard = CurrentDirGuard::enter(&temp_dir);

        let mut app = App::new();
        app.add_plugins(DbPlugin);
        app.update();

        assert!(
            temp_dir.join(DbSettings::PATH).exists(),
            "Should create the default settings file when missing"
        );
        assert!(
            app.world().contains_resource::<DbSettings>(),
            "Should insert DbSettings into the world"
        );
    }

    #[test]
    fn should_preserve_existing_database_settings() {
        let settings = DbSettingsBuilder {
            database_url: "sqlite::memory:".to_string(),
            ..DbSettingsBuilder::default()
        }
        .build()
        .expect("memory settings should validate");

        let mut app = App::new();
        app.insert_resource(settings.clone());
        app.add_plugins(DbPlugin);
        app.update();

        assert_eq!(app.world().resource::<DbSettings>(), &settings);
    }

    #[test]
    fn should_open_shared_connection_during_pre_startup() {
        let settings = DbSettingsBuilder {
            database_url: "sqlite::memory:".to_string(),
            ..DbSettingsBuilder::default()
        }
        .build()
        .expect("memory settings should validate");

        let mut app = App::new();
        app.insert_resource(settings);
        app.add_plugins(DbPlugin);
        app.update();

        assert!(
            app.world().contains_resource::<DbConnection>(),
            "DbPlugin should open the shared DbConnection during PreStartup"
        );
    }

    #[test]
    fn should_expose_typed_tables_through_db_and_db_mut() {
        let mut app = App::new();
        app.init_db_table::<PreludeTable>();

        app.add_systems(Update, |mut table: DbMut<PreludeTable>| {
            table.value = 11;
        })
        .add_systems(PostUpdate, |table: Db<PreludeTable>| {
            assert_eq!(table.value, 11);
        });

        app.update();

        assert_eq!(
            app.world().resource::<Tables<PreludeTable>>().value,
            11,
            "DbMut mutations should be observable through Tables<T>"
        );
    }
}
