use bevy::prelude::*;

use crate::settings::Settings;

pub(crate) fn bootstrap_settings(app: &App) -> Settings {
    app.world()
        .get_resource::<Settings>()
        .copied()
        .unwrap_or_else(|| Settings::load_or_default().expect("Failed to load Suon settings."))
}

/// Loads the root Suon settings during pre-startup.
pub(crate) fn initialize_settings(mut commands: Commands, settings: Option<Res<Settings>>) {
    if let Some(settings) = settings {
        info!(
            "Suon settings already provided by app: {}",
            settings.summary()
        );
        return;
    }

    let settings = Settings::load_or_default().expect("Failed to load Suon settings.");
    commands.insert_resource(settings);

    info!("Suon settings loaded: {}", settings.summary());
}

/// Initializes the fixed timestep resource from the loaded Suon settings during startup.
pub(crate) fn initialize_fixed_time(mut commands: Commands, settings: Res<Settings>) {
    commands.insert_resource(Time::<Fixed>::from_seconds(
        settings.fixed_event_loop_seconds(),
    ));

    info!(
        "Suon fixed timestep initialized: seconds={:.6}, hz={:.2}",
        settings.fixed_event_loop_seconds(),
        settings.fixed_event_loop_hz()
    );
}

/// Logs a startup summary with non-sensitive runtime details.
pub(crate) fn log_startup_summary(settings: Res<Settings>) {
    info!(
        "Suon startup complete: task_threads={}, schedule_runner={}, app_loop_hz={:.2}, \
         fixed_loop_hz={:.2}",
        settings.threads,
        settings.schedule_runner,
        settings.event_loop_hz(),
        settings.fixed_event_loop_hz()
    );
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

        env::temp_dir().join(format!("suon-root-settings-{}-{nanos}", process::id()))
    }

    #[test]
    fn should_load_suon_settings_during_prestartup_and_fixed_time_during_startup() {
        let _lock = cwd_lock()
            .lock()
            .expect("the test should acquire the cwd lock");
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("the temp test directory should be created");
        let _cwd_guard = CurrentDirGuard::enter(&temp_dir);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(PreStartup, initialize_settings);
        app.add_systems(Startup, initialize_fixed_time);

        assert!(
            !app.world().contains_resource::<Settings>(),
            "The bootstrap should defer inserting Settings until PreStartup"
        );

        app.update();

        assert!(
            temp_dir.join(Settings::PATH).exists(),
            "PreStartup should create the default settings file when it is missing"
        );

        assert!(
            app.world().contains_resource::<Settings>(),
            "PreStartup should insert Settings into the world"
        );

        assert!(
            app.world().contains_resource::<Time<Fixed>>(),
            "Startup should insert Time<Fixed> after Settings are available"
        );

        assert_eq!(
            app.world()
                .resource::<Time<Fixed>>()
                .timestep()
                .as_secs_f64(),
            Settings::default().fixed_event_loop_seconds(),
            "Startup should configure the fixed timestep from the loaded settings"
        );
    }

    #[test]
    fn should_preserve_existing_suon_settings_resource() {
        let mut app = App::new();
        let settings = Settings {
            threads: 3,
            event_loop_hz: 4.0,
            fixed_event_loop_hz: 2.0,
            schedule_runner: false,
        };

        app.insert_resource(settings);
        app.add_plugins(MinimalPlugins);
        app.add_systems(PreStartup, initialize_settings);
        app.add_systems(Startup, initialize_fixed_time);
        app.update();

        assert_eq!(
            app.world().resource::<Settings>(),
            &settings,
            "PreStartup should preserve the Settings resource provided by the app"
        );

        assert_eq!(
            app.world()
                .resource::<Time<Fixed>>()
                .timestep()
                .as_secs_f64(),
            settings.fixed_event_loop_seconds(),
            "Startup should derive Time<Fixed> from the preserved Settings resource"
        );
    }

    #[test]
    fn should_bootstrap_from_existing_settings_resource() {
        let mut app = App::new();
        let settings = Settings {
            threads: 5,
            event_loop_hz: 10.0,
            fixed_event_loop_hz: 5.0,
            schedule_runner: false,
        };

        app.insert_resource(settings);

        assert_eq!(
            bootstrap_settings(&app),
            settings,
            "bootstrap_settings should prefer a Settings resource already inserted into the app"
        );
    }
}
