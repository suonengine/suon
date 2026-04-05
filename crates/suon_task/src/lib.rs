//! Background task helpers for Bevy apps.
//!
//! This crate provides small wrappers around Bevy task pools so world-scoped and
//! entity-scoped background work can be spawned and reconciled back into ECS
//! systems once the work finishes.

pub mod background;

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
}
