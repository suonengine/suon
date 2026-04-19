use bevy::prelude::*;

use crate::server::{
    connection::{limiter::Limiter, throttle::Throttle},
    settings::Settings,
};

/// Loads the server settings and initializes related resources.
pub(crate) fn initialize_settings(mut commands: Commands) {
    let settings = Settings::load_or_default().expect("Failed to load network server settings.");

    commands.insert_resource(Throttle::new(settings));
    commands.insert_resource(Limiter::new(settings));
    commands.insert_resource(settings);

    info!("Server settings initialized successfully.");
}

#[cfg(test)]
mod tests {
    use super::*;
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
            env::set_current_dir(path).expect("the test should switch to the temp directory");
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

        env::temp_dir().join(format!(
            "suon-network-initialize-settings-{}-{nanos}",
            process::id()
        ))
    }

    #[test]
    fn should_insert_network_resources_and_create_a_default_settings_file() {
        let _lock = cwd_lock()
            .lock()
            .expect("the initialize_settings test should acquire the cwd lock");
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("the temp test directory should be created");
        let _cwd_guard = CurrentDirGuard::enter(&temp_dir);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, initialize_settings);

        app.update();

        assert!(
            temp_dir
                .join("settings/NetworkServerSettings.toml")
                .exists(),
            "initialize_settings should create the default settings file when it is missing"
        );

        assert!(
            app.world().contains_resource::<Settings>(),
            "initialize_settings should insert the loaded settings resource"
        );

        assert!(
            app.world().contains_resource::<Throttle>(),
            "initialize_settings should insert the throttle resource derived from settings"
        );

        assert!(
            app.world().contains_resource::<Limiter>(),
            "initialize_settings should insert the limiter resource derived from settings"
        );
    }
}
