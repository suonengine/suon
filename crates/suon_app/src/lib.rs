//! Application framework for the Suon MMORPG server.
//!
//! This crate provides the core runtime — [`App`] — which orchestrates the
//! lifecycle of the game server: startup systems, a task-dispatch loop, and
//! shutdown systems.
//!
//! # Lifecycle
//!
//! ```text
//! startup systems → task loop (drain + dispatch) → shutdown systems
//! ```
//!
//! Startup and shutdown systems are closures or types implementing
//! [`IntoSystem`].  The task loop processes
//! [`TaskHandler`]s sent through a shared [`Channel`].
//!
//! # Plugins
//!
//! Groups of related systems and resources can be bundled into a [`Plugin`]
//! and registered via [`App::add_plugin`].
//!
//! [`Channel`]: suon_channel::Channel
//! [`Plugin`]: plugin::Plugin
//! [`TaskHandler`]: suon_channel::TaskHandler

use suon_channel::Channel;
use suon_resource::{Resource, Resources};
use system::{IntoSystem, System};
use tracing::{debug, info};

use self::plugin::Plugin;

pub mod plugin;
pub mod shutdown;
pub mod system;

/// Top-level application runtime for the Suon game server.
///
/// Holds the global [`Resources`] container, a [`Channel`] for dispatching
/// tasks, and two lists of systems (startup and shutdown).
pub struct App {
    resources: Resources,
    channel: Channel,
    startup_systems: Vec<Box<dyn System>>,
    shutdown_systems: Vec<Box<dyn System>>,
}

impl App {
    /// Creates a new, empty `App` with no systems or resources registered.
    pub fn new() -> Self {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_level(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_file(false)
            .with_line_number(false)
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .try_init()
            .expect("tracing subscriber global default should be set once only");

        Self {
            resources: Resources::default(),
            channel: Channel::default(),
            startup_systems: Vec::new(),
            shutdown_systems: Vec::new(),
        }
    }

    /// Returns a clone of the internal channel, usable to send tasks into
    /// the event loop.
    pub fn channel(&self) -> Channel {
        self.channel.clone()
    }

    /// Inserts a `T::default()` resource if one is not already present.
    ///
    /// If `T` is already stored, the existing value is **overwritten**
    /// with the default.
    pub fn init_resource<T: Resource + Default>(&mut self) -> &mut Self {
        self.resources.init::<T>();
        self
    }

    /// Returns a shared reference to a resource.
    ///
    /// # Panics
    ///
    /// Panics if `T` has not been registered with [`add_resource`](Self::add_resource)
    /// or [`init_resource`](Self::init_resource).
    pub fn get_resource<T: Resource>(&self) -> &T {
        self.resources.get::<T>()
    }

    /// Returns a mutable reference to a resource.
    ///
    /// # Panics
    ///
    /// Panics if `T` has not been registered with [`add_resource`](Self::add_resource)
    /// or [`init_resource`](Self::init_resource).
    pub fn get_resource_mut<T: Resource>(&mut self) -> &mut T {
        self.resources.get_mut::<T>()
    }

    /// Returns a shared reference to a resource, or `None` if it has not
    /// been registered.
    pub fn try_get_resource<T: Resource>(&self) -> Option<&T> {
        self.resources.try_get::<T>()
    }

    /// Returns a mutable reference to a resource, or `None` if it has not
    /// been registered.
    pub fn try_get_resource_mut<T: Resource>(&mut self) -> Option<&mut T> {
        self.resources.try_get_mut::<T>()
    }

    /// Registers a resource so it is available to systems via
    /// [`Resources::get`](suon_resource::Resources::get).
    pub fn add_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.resources.insert(resource);
        self
    }

    /// Registers a plugin, which may add resources and systems to the app.
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        let name = std::any::type_name_of_val(&plugin);
        debug!(target: "App", "Adding plugin: {name}");
        plugin.build(self);
        self
    }

    /// Registers a system that runs once at startup, before the task loop.
    pub fn add_startup_system(&mut self, system: impl IntoSystem) -> &mut Self {
        debug!(target: "App", "Registered startup system: {}", std::any::type_name_of_val(&system));
        self.startup_systems.push(Box::new(system.into_system()));
        self
    }

    /// Registers a system that runs once during shutdown, after the task
    /// loop exits.
    pub fn add_shutdown_system(&mut self, system: impl IntoSystem) -> &mut Self {
        debug!(target: "App", "Registered shutdown system: {}", std::any::type_name_of_val(&system));
        self.shutdown_systems.push(Box::new(system.into_system()));
        self
    }

    /// Runs the application lifecycle:
    ///
    /// 1. Initialises the `Exit` resource and inserts the channel.
    /// 2. Runs all startup systems.
    /// 3. Enters the task-dispatch loop until an `Exit` signal is received.
    /// 4. Runs all shutdown systems.
    pub fn run(&mut self) {
        self.resources.init::<crate::shutdown::Exit>();
        self.resources.insert(self.channel.clone());

        let startup_count = self.startup_systems.len();
        info!(target: "App", "Running {startup_count} startup systems");
        for system in std::mem::take(&mut self.startup_systems) {
            system.run(&mut self.resources);
        }
        info!(target: "App", "Startup systems complete");

        info!(target: "App", "Entering task dispatch loop");
        let mut buffer = Vec::new();
        loop {
            self.channel.wait_and_drain(&mut buffer);

            for mut task in buffer.drain(..) {
                task.run(&mut self.resources);
            }

            if **self.resources.get::<crate::shutdown::Exit>() {
                info!(target: "App", "Shutdown signal received, exiting task loop");
                break;
            }
        }

        let shutdown_count = self.shutdown_systems.len();
        info!(target: "App", "Running {shutdown_count} shutdown systems");

        for system in std::mem::take(&mut self.shutdown_systems) {
            system.run(&mut self.resources);
        }
        info!(target: "App", "Shutdown complete");
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use crate::shutdown::Shutdown;
    use suon_channel::{Channel, TaskHandler};
    use suon_macros::{Deref, DerefMut, Resource, Task};
    use suon_resource::Resources;

    use super::*;

    #[derive(Resource, Default, Deref, DerefMut)]
    struct Num(i32);

    #[derive(Resource, Deref, DerefMut)]
    struct Label(String);

    #[test]
    fn new() {
        let app = App::new();
        assert!(app.startup_systems.is_empty());
        assert!(app.shutdown_systems.is_empty());
    }

    #[test]
    fn shutdown_breaks_loop() {
        App::new()
            .add_startup_system(|resources: &mut Resources| {
                let channel = resources.get::<Channel>();
                channel.send(Shutdown);
            })
            .run();
    }

    #[test]
    fn startup_runs_before_tasks() {
        let mut app = App::new();
        app.add_resource(Num(42));
        app.add_startup_system(|resources: &mut Resources| {
            assert_eq!(**resources.get::<Num>(), 42);
            let channel = resources.get::<Channel>();
            channel.send(Shutdown);
        });
        app.run();
    }

    #[test]
    fn shutdown_runs_after_loop() {
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        App::new()
            .add_startup_system(|resources: &mut Resources| {
                let channel = resources.get::<Channel>();
                channel.send(Shutdown);
            })
            .add_shutdown_system(move |_: &mut Resources| {
                flag_clone.store(true, Ordering::SeqCst);
            })
            .run();

        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn resources_accessible() {
        let mut app = App::new();
        app.add_resource(Num(42));
        app.add_resource(Label(String::from("hello")));
        assert_eq!(**app.resources.get::<Num>(), 42);
        assert_eq!(**app.resources.get::<Label>(), "hello");
    }

    #[test]
    fn channel_as_resource() {
        let mut app = App::new();
        app.add_startup_system(|resources: &mut Resources| {
            let channel = resources.get::<Channel>();
            channel.send(Shutdown);
            channel.send(ChannelCheck);
        });
        app.run();
    }

    #[derive(Task)]
    struct ChannelCheck;

    impl TaskHandler for ChannelCheck {
        fn run(&mut self, resources: &mut Resources) {
            resources.get::<Channel>();
        }
    }

    #[test]
    fn app_without_systems_starts_and_accepts_tasks() {
        let mut app = App::new();
        let channel = app.channel();
        channel.send(Shutdown);
        app.run();
    }

    struct AddNumPlugin;
    impl Plugin for AddNumPlugin {
        fn build(&self, app: &mut App) {
            app.add_resource(Num(10));
        }
    }

    struct StartupPlugin;
    impl Plugin for StartupPlugin {
        fn build(&self, app: &mut App) {
            app.add_resource(Num(0));
            app.add_startup_system(|resources: &mut Resources| {
                **resources.get_mut::<Num>() = 42;
                let channel = resources.get::<Channel>();
                channel.send(Shutdown);
            });
        }
    }

    #[test]
    fn plugin_adds_resource() {
        let mut app = App::new();
        app.add_plugin(AddNumPlugin);
        assert_eq!(**app.resources.get::<Num>(), 10);
    }

    #[test]
    fn plugin_adds_startup_system() {
        let mut app = App::new();
        app.add_plugin(StartupPlugin);
        app.run();
        assert_eq!(**app.resources.get::<Num>(), 42);
    }

    #[test]
    fn multiple_plugins() {
        struct PluginA;
        impl Plugin for PluginA {
            fn build(&self, app: &mut App) {
                app.add_resource(Num(1));
            }
        }

        struct PluginB;
        impl Plugin for PluginB {
            fn build(&self, app: &mut App) {
                app.add_resource(Label(String::from("b")));
            }
        }

        let mut app = App::new();
        app.add_plugin(PluginA);
        app.add_plugin(PluginB);
        assert_eq!(**app.resources.get::<Num>(), 1);
        assert_eq!(**app.resources.get::<Label>(), "b");
    }

    #[test]
    fn task_sends_task() {
        #[derive(Task)]
        struct FirstStep;
        impl TaskHandler for FirstStep {
            fn run(&mut self, resources: &mut Resources) {
                **resources.get_mut::<Num>() = 1;
                let channel = resources.get::<Channel>();
                channel.send(SecondStep);
            }
        }

        #[derive(Task)]
        struct SecondStep;
        impl TaskHandler for SecondStep {
            fn run(&mut self, resources: &mut Resources) {
                **resources.get_mut::<Num>() = 2;
                let channel = resources.get::<Channel>();
                channel.send(Shutdown);
            }
        }

        let mut app = App::new();
        app.add_resource(Num(0));
        app.add_startup_system(|resources: &mut Resources| {
            let channel = resources.get::<Channel>();
            channel.send(FirstStep);
        });
        app.run();
        assert_eq!(**app.resources.get::<Num>(), 2);
    }

    #[test]
    fn no_shutdown_systems_no_error() {
        App::new()
            .add_startup_system(|resources: &mut Resources| {
                let channel = resources.get::<Channel>();
                channel.send(Shutdown);
            })
            .run();
    }

    #[test]
    fn channel_before_run() {
        let mut app = App::new();
        let channel = app.channel();
        channel.send(Shutdown);
        app.run();
    }

    #[derive(Resource, Default, Debug, PartialEq)]
    struct Score(i32);

    #[test]
    fn init_resource_creates_default() {
        let mut app = App::new();
        app.init_resource::<Score>();
        assert_eq!(app.resources.get::<Score>().0, 0);
    }

    #[test]
    fn init_resource_overwrites_existing() {
        let mut app = App::new();
        app.add_resource(Score(42));
        app.init_resource::<Score>();
        assert_eq!(app.resources.get::<Score>().0, 0);
    }

    #[test]
    fn init_resource_chained() {
        let mut app = App::new();
        app.init_resource::<Score>().init_resource::<Num>();
        assert_eq!(app.resources.get::<Score>().0, 0);
        assert_eq!(app.resources.get::<Num>().0, 0);
    }

    #[test]
    fn get_resource_returns_registered() {
        let mut app = App::new();
        app.add_resource(Score(7));
        assert_eq!(app.get_resource::<Score>().0, 7);
    }

    #[test]
    #[should_panic(expected = "Resource `suon_app::tests::Score` not found")]
    fn get_resource_panics_when_missing() {
        let app = App::new();
        app.get_resource::<Score>();
    }

    #[test]
    fn get_resource_mut_allows_mutation() {
        let mut app = App::new();
        app.add_resource(Score(0));
        app.get_resource_mut::<Score>().0 = 42;
        assert_eq!(app.get_resource::<Score>().0, 42);
    }

    #[test]
    fn try_get_resource_returns_none_when_missing() {
        let app = App::new();
        assert!(app.try_get_resource::<Score>().is_none());
    }

    #[test]
    fn try_get_resource_returns_some_when_present() {
        let mut app = App::new();
        app.add_resource(Score(5));
        let result = app.try_get_resource::<Score>();
        assert_eq!(result, Some(&Score(5)));
    }

    #[test]
    fn try_get_resource_mut_returns_none_when_missing() {
        let mut app = App::new();
        assert!(app.try_get_resource_mut::<Score>().is_none());
    }

    #[test]
    fn try_get_resource_mut_allows_mutation() {
        let mut app = App::new();
        app.add_resource(Score(1));
        if let Some(score) = app.try_get_resource_mut::<Score>() {
            score.0 = 99;
        }
        assert_eq!(app.get_resource::<Score>().0, 99);
    }
}
