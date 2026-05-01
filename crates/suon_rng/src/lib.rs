//! RNG resources for Suon systems.

mod distribution;
mod fast_random;
mod random;
mod seed;
mod settings;

use bevy::prelude::*;
use log::info;

pub use fast_random::{FastRandom, FastRandomAlgorithm};
pub use random::{Random, RandomAlgorithm};
pub use settings::RngSettings;

/// Bevy plugin that exposes RNG helpers as ECS resources.
pub struct RNGPlugin;

impl Plugin for RNGPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, initialize_rng);
    }
}

fn initialize_rng(
    mut commands: Commands,
    settings: Option<Res<RngSettings>>,
    random: Option<Res<Random>>,
    fast_random: Option<Res<FastRandom>>,
) {
    let settings = settings.map(|settings| *settings).unwrap_or_else(|| {
        let settings = RngSettings::load_or_default().expect("Failed to load RNG settings.");
        commands.insert_resource(settings);
        settings
    });

    if random.is_none() {
        commands.insert_resource(Random::seed_from_u64(settings.seed()));
    }

    if fast_random.is_none() {
        commands.insert_resource(FastRandom::seed_from_u64(settings.fast_seed()));
    }

    info!("Initialized with {}", settings.summary());
}

pub mod prelude {
    pub use crate::{
        FastRandom, FastRandomAlgorithm, RNGPlugin, Random, RandomAlgorithm, RngSettings,
    };
    pub use rand_core::{Rng, SeedableRng};
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::{
        env,
        path::PathBuf,
        process,
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    pub(crate) fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    pub(crate) struct CurrentDirGuard {
        previous: PathBuf,
    }

    impl CurrentDirGuard {
        pub(crate) fn enter(path: &PathBuf) -> Self {
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

    pub(crate) fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after the unix epoch")
            .as_nanos();

        env::temp_dir().join(format!("{prefix}-{}-{nanos}", process::id()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{CurrentDirGuard, cwd_lock, unique_temp_dir};
    use std::fs;

    #[test]
    fn should_register_rng_resources_with_plugin_and_create_settings() {
        let _lock = cwd_lock()
            .lock()
            .expect("the RNG plugin test should acquire the cwd lock");

        let temp_dir = unique_temp_dir("suon-rng-plugin");
        fs::create_dir_all(&temp_dir).expect("the temp test directory should be created");
        let _cwd_guard = CurrentDirGuard::enter(&temp_dir);

        let mut app = App::new();
        app.add_plugins(RNGPlugin);
        app.update();

        assert!(temp_dir.join(RngSettings::PATH).exists());
        assert!(app.world().get_resource::<RngSettings>().is_some());
        assert!(app.world().get_resource::<Random>().is_some());
        assert!(app.world().get_resource::<FastRandom>().is_some());
    }

    #[test]
    fn should_preserve_existing_rng_settings() {
        let settings = RngSettings::new(42);

        let mut app = App::new();
        app.insert_resource(settings);
        app.add_plugins(RNGPlugin);
        app.update();

        assert_eq!(app.world().resource::<RngSettings>(), &settings);
    }

    #[test]
    fn should_seed_resources_from_existing_rng_settings() {
        let settings = RngSettings::new(42);

        let mut app = App::new();
        app.insert_resource(settings);
        app.add_plugins(RNGPlugin);
        app.update();

        let mut expected_random = Random::seed_from_u64(settings.seed());
        let mut actual_random = app.world_mut().resource_mut::<Random>();
        assert_eq!(actual_random.next_u64(), expected_random.next_u64());

        let mut expected_fast_random = FastRandom::seed_from_u64(settings.fast_seed());
        let mut actual_fast_random = app.world_mut().resource_mut::<FastRandom>();
        assert_eq!(
            actual_fast_random.next_u64(),
            expected_fast_random.next_u64()
        );
    }

    #[test]
    fn should_preserve_existing_rng_resources() {
        let settings = RngSettings::new(42);

        let mut random = Random::seed_from_u64(7);
        let mut fast_random = FastRandom::seed_from_u64(9);
        random.next_u64();
        fast_random.next_u64();

        let mut expected_random = random.clone();
        let mut expected_fast_random = fast_random.clone();

        let mut app = App::new();
        app.insert_resource(settings);
        app.insert_resource(random);
        app.insert_resource(fast_random);
        app.add_plugins(RNGPlugin);
        app.update();

        assert_eq!(
            app.world_mut().resource_mut::<Random>().next_u64(),
            expected_random.next_u64()
        );

        assert_eq!(
            app.world_mut().resource_mut::<FastRandom>().next_u64(),
            expected_fast_random.next_u64()
        );
    }
}
