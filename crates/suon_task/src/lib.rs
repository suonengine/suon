//! Background task helpers for Bevy apps.
//!
//! This crate provides small wrappers around Bevy task pools so world-scoped and
//! entity-scoped background work can be spawned and reconciled back into ECS
//! systems once the work finishes.
//!
//! # Examples
//! ```no_run
//! use bevy::prelude::*;
//! use suon_task::prelude::*;
//!
//! struct ExampleTask;
//!
//! impl BackgroundTask for ExampleTask {
//!     type Output = ();
//!
//!     async fn run(self) -> Self::Output {}
//! }
//!
//! let mut app = App::new();
//! app.add_plugins(MinimalPlugins);
//! app.add_background_task_systems::<Update, ExampleTask>();
//!
//! assert_eq!(std::mem::size_of::<ExampleTask>(), 0);
//! ```

mod background;

pub mod prelude {
    pub use crate::background::{
        AppWithBackgroundTasks, BackgroundTask,
        entity::{EntityIn, EntityTaskCommands},
        world::TaskCommands,
    };
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    struct DummyTask;

    impl crate::background::BackgroundTask for DummyTask {
        type Output = ();

        async fn run(self) -> Self::Output {}
    }

    #[test]
    fn should_expose_background_module() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        crate::background::AppWithBackgroundTasks::add_background_task_systems::<Update, DummyTask>(
            &mut app,
        );

        assert!(
            app.world()
                .components()
                .resource_id::<bevy::ecs::schedule::Schedules>()
                .is_some(),
            "The crate root should expose the background module so apps can register its systems"
        );
    }

    #[test]
    fn should_expose_background_api_through_prelude() {
        use crate::prelude::*;

        fn assert_app_trait<T: AppWithBackgroundTasks>() {}
        fn assert_entity_trait<T: EntityTaskCommands>() {}
        fn assert_world_trait<T: TaskCommands>() {}

        let _ = std::mem::size_of::<EntityIn<usize>>();
        assert_app_trait::<App>();
        assert_entity_trait::<EntityWorldMut<'_>>();
        assert_world_trait::<World>();

        struct PreludeTask;
        impl BackgroundTask for PreludeTask {
            type Output = ();

            async fn run(self) -> Self::Output {}
        }

        let _ = std::mem::size_of::<PreludeTask>();
    }
}
