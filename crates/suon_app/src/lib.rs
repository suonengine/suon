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

    /// Registers a resource so it is available to systems via
    /// [`Resources::get`](suon_resource::Resources::get).
    pub fn add_resource<T: Resource>(&mut self, resource: T) -> &mut Self {
        self.resources.insert(resource);
        self
    }

    /// Registers a plugin, which may add resources and systems to the app.
    pub fn add_plugin(&mut self, plugin: impl Plugin) -> &mut Self {
        plugin.build(self);
        self
    }

    /// Registers a system that runs once at startup, before the task loop.
    pub fn add_startup_system(&mut self, system: impl IntoSystem) -> &mut Self {
        self.startup_systems.push(Box::new(system.into_system()));
        self
    }

    /// Registers a system that runs once during shutdown, after the task
    /// loop exits.
    pub fn add_shutdown_system(&mut self, system: impl IntoSystem) -> &mut Self {
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

        for system in std::mem::take(&mut self.startup_systems) {
            system.run(&mut self.resources);
        }

        let mut buffer = Vec::with_capacity(64);
        loop {
            self.channel.drain_into(&mut buffer);

            for task in buffer.drain(..) {
                task.run(&mut self.resources);
            }

            if **self.resources.get::<crate::shutdown::Exit>() {
                break;
            }
        }

        for system in std::mem::take(&mut self.shutdown_systems) {
            system.run(&mut self.resources);
        }
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

    #[derive(Resource, Deref, DerefMut)]
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
        fn run(self: Box<Self>, resources: &mut Resources) {
            let _ = resources.get::<Channel>();
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
            fn run(self: Box<Self>, resources: &mut Resources) {
                **resources.get_mut::<Num>() = 1;
                let channel = resources.get::<Channel>();
                channel.send(SecondStep);
            }
        }

        #[derive(Task)]
        struct SecondStep;
        impl TaskHandler for SecondStep {
            fn run(self: Box<Self>, resources: &mut Resources) {
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
}
