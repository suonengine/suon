use suon_resource::Resources;

/// Executable unit of logic that operates on the global resource container.
///
/// Systems are the building blocks of the application lifecycle. Each system
/// receives a mutable reference to [`Resources`] and can read or mutate any
/// registered resource.
///
/// # Implementing
///
/// The simplest way is to pass a closure to [`crate::App::add_startup_system`]
/// or [`crate::App::add_shutdown_system`] — any `FnOnce(&mut Resources)`
/// implements [`System`] automatically.
pub trait System: Send + 'static {
    /// Executes the system with the given resource container.
    fn run(self: Box<Self>, resources: &mut Resources);
}

/// Conversion trait into a boxable [`System`].
///
/// This trait mirrors [`System`] itself and exists so that [`App`] methods can
/// accept ergonomic system definitions (closures, function pointers, or custom
/// types) and erase them into `Box<dyn System>`.
///
/// [`App`]: crate::App
pub trait IntoSystem: Send + 'static {
    /// The concrete system type produced by this conversion.
    type System: System;

    /// Converts `self` into a [`System`] value.
    fn into_system(self) -> Self::System;
}

/// Blanket implementation: any `FnOnce(&mut Resources)` closure qualifies as a
/// [`System`].
impl<F: FnOnce(&mut Resources) + Send + 'static> System for F {
    fn run(self: Box<Self>, resources: &mut Resources) {
        (self)(resources);
    }
}

/// Blanket implementation: any `FnOnce(&mut Resources)` closure also satisfies
/// [`IntoSystem`].
impl<F: FnOnce(&mut Resources) + Send + 'static> IntoSystem for F {
    type System = F;

    fn into_system(self) -> Self::System {
        self
    }
}
