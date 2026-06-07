use crate::App;

/// Groups related resources and systems into a reusable bundle.
///
/// A plugin is registered via [`App::add_plugin`] and receives a mutable
/// reference to the [`App`] so it can register resources, startup systems,
/// and shutdown systems.
///
/// # Example
///
/// ```rust,ignore
/// struct MyPlugin;
///
/// impl Plugin for MyPlugin {
///     fn build(&self, app: &mut App) {
///         app.add_resource(MyConfig::default());
///         app.add_startup_system(my_system);
///     }
/// }
///
/// App::new().add_plugin(MyPlugin).run();
/// ```
pub trait Plugin: Send + Sync + 'static {
    /// Called during [`App::add_plugin`] to register resources and systems.
    fn build(&self, app: &mut App);
}
