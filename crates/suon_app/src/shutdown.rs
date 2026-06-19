use suon_channel::TaskHandler;
use suon_macros::{Deref, DerefMut, Resource, Task};
use suon_resource::Resources;

/// Signals the application to shut down gracefully.
///
/// When sent through the channel (via [`Channel::send`]), this task sets
/// the internal `Exit` flag to `true`, causing the event loop in
/// [`App::run`] to exit after the current batch of tasks completes.
///
/// [`App::run`]: crate::App::run
/// [`Channel::send`]: suon_channel::Channel::send
#[derive(Task)]
pub struct Shutdown;

impl TaskHandler for Shutdown {
    fn run(&mut self, resources: &mut Resources) {
        **resources.get_mut::<Exit>() = true;
    }
}

/// Internal flag that controls the lifetime of the event loop.
///
/// Initialised to `false` at the start of [`App::run`] and set to `true`
/// by the `Shutdown` task.  Once `true`, the task-dispatch loop exits
/// and shutdown systems execute.
///
/// [`App::run`]: crate::App::run
#[derive(Resource, Deref, DerefMut, Default)]
pub struct Exit(bool);

impl Exit {
    pub fn trigger(&mut self) {
        self.0 = true;
    }
}
