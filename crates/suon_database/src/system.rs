//! Startup systems for initializing database resources.

use bevy::prelude::*;
#[cfg(test)]
use std::{
    env, fs,
    path::PathBuf,
    process,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::settings::DatabaseSettings;

/// Loads database settings during startup and inserts them into the Bevy world.
pub(crate) fn initialize_settings(mut commands: Commands, settings: Option<Res<DatabaseSettings>>) {
    if let Some(settings) = settings {
        debug!("Found a pre-existing DatabaseSettings resource.");

        settings
            .validate()
            .expect("Failed to validate database settings resource.");

        return;
    }

    info!("Loading database settings from disk.");

    let settings = DatabaseSettings::load_or_default().expect("Failed to load database settings.");

    commands.insert_resource(settings);

    info!("Inserted DatabaseSettings into the world.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DatabasePlugin;

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

    #[test]
    fn should_insert_database_settings_and_create_the_default_file() {
        let _lock = cwd_lock()
            .lock()
            .expect("the database plugin test should acquire the cwd lock");

        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("the temp test directory should be created");
        let _cwd_guard = CurrentDirGuard::enter(&temp_dir);

        let mut app = App::new();
        app.add_plugins(DatabasePlugin);
        app.update();

        assert!(
            temp_dir.join(DatabaseSettings::PATH).exists(),
            "Should create the default database settings file when it is missing"
        );

        assert!(
            app.world().contains_resource::<DatabaseSettings>(),
            "Should insert DatabaseSettings into the world"
        );
    }

    #[test]
    fn should_preserve_existing_database_settings() {
        let settings = DatabaseSettings::default();

        let mut app = App::new();
        app.insert_resource(settings.clone());
        app.add_plugins(DatabasePlugin);
        app.update();

        assert_eq!(
            app.world().resource::<DatabaseSettings>(),
            &settings,
            "Should preserve an existing DatabaseSettings resource"
        );
    }
}
