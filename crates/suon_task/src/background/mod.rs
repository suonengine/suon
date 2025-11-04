use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

pub mod entity;
pub mod world;

/// Trait representing a background task that can be executed asynchronously.
/// It must be Send and 'static to be safely run in a background thread.
pub trait BackgroundTask: Send + Sync + 'static {
    /// The type of output produced by the task.
    type Output: Send + Sync + 'static;

    /// Executes the task asynchronously, returning a future with the result.
    fn run(self) -> impl Future<Output = Self::Output> + Send + 'static;
}

/// Extension trait for Bevy App to easily add background task handling systems.
pub trait AppWithBackgroundTasks {
    /// Adds systems to manage the completion of background tasks for the specified schedule label.
    ///
    /// # Type Parameters
    /// - `S`: The schedule label to insert the systems into.
    /// - `T`: The type of background task being managed.
    fn add_background_task_systems<S, T>(&mut self) -> &mut Self
    where
        T: BackgroundTask,
        S: ScheduleLabel + Default;
}

impl AppWithBackgroundTasks for App {
    /// Adds background task completion checking systems to the specified schedule label.
    fn add_background_task_systems<S, T>(&mut self) -> &mut Self
    where
        T: BackgroundTask,
        S: ScheduleLabel + Default,
    {
        self.add_systems(
            S::default(),
            (
                world::check_completed_world_tasks::<T>,
                entity::check_completed_entity_tasks::<T>,
            ),
        );
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::background::{
        entity::{EntityIn, EntityTaskCommands, EntityTaskTracker},
        world::{TaskCommands, WorldTaskTracker},
    };

    // Dummy background task for testing
    struct DummyTask(pub i32);

    impl BackgroundTask for DummyTask {
        type Output = i32;

        async fn run(self) -> Self::Output {
            self.0
        }
    }

    #[test]
    fn test_add_background_task_systems_integration() {
        // Initialize Bevy app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add background task systems to the app
        app.add_background_task_systems::<Update, DummyTask>();

        // Spawn an entity and attach a background task that outputs i32::MAX
        // The result is received through the EntityIn wrapper
        app.world_mut()
            .spawn_empty()
            .spawn_background_task_with_system(
                DummyTask(i32::MAX),
                |EntityIn((.., result)): EntityIn<i32>| {
                    assert!(result == i32::MAX);
                },
            );

        // Verify that the EntityTaskTracker component was added for this entity
        assert!(
            app.world_mut()
                .query::<&EntityTaskTracker<DummyTask>>()
                .iter(app.world())
                .next()
                .is_some(),
            "EntityTaskTracker should exist after spawning an entity background task"
        );

        // Spawn a background task directly in the world scope (not entity-bound)
        // The result is received through the In wrapper
        app.world_mut().spawn_background_task_with_system(
            DummyTask(i32::MAX),
            |In(result): In<i32>| {
                assert!(result == i32::MAX);
            },
        );

        // Verify that the WorldTaskTracker resource was added
        assert!(
            app.world_mut()
                .query::<&WorldTaskTracker<DummyTask>>()
                .iter(app.world())
                .next()
                .is_some(),
            "WorldTaskTracker should exist after spawning a world background task"
        );
    }

    #[test]
    fn test_check_completed_world_tasks_system() {
        // Initialize app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Add background task systems
        app.add_background_task_systems::<Update, DummyTask>();

        // Spawn an entity with a background task
        app.world_mut()
            .spawn_empty()
            .spawn_background_task_with_system(
                DummyTask(i32::MAX),
                |EntityIn((.., result)): EntityIn<i32>| {
                    assert!(result == i32::MAX);
                },
            );

        // Loop until the background task completes
        loop {
            app.update();

            // Check if any entities still have the task tracker component
            let remaining = app
                .world_mut()
                .query::<&WorldTaskTracker<DummyTask>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break; // All tasks completed and trackers removed
            }
        }

        // Check that the background task tracker component has been cleaned up
        assert!(
            app.world_mut()
                .query::<&WorldTaskTracker<DummyTask>>()
                .iter(app.world())
                .next()
                .is_none(),
            "WorldTaskTracker component should be removed after task completion"
        );
    }

    #[test]
    fn test_check_completed_entity_tasks_system() {
        // Initialize app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_background_task_systems::<Update, DummyTask>();

        // Spawn an entity with a background task
        let entity = app
            .world_mut()
            .spawn_empty()
            .spawn_background_task_with_system(
                DummyTask(i32::MAX),
                |EntityIn((.., result)): EntityIn<i32>| {
                    assert!(result == i32::MAX);
                },
            )
            .id();

        // Loop until the background task completes
        loop {
            app.update();

            // Check if any entities still have the task tracker component
            let remaining = app
                .world_mut()
                .query::<&EntityTaskTracker<DummyTask>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break; // All tasks completed and trackers removed
            }
        }

        // Check that the entity no longer has the task tracker component
        assert!(
            !app.world()
                .entity(entity)
                .contains::<EntityTaskTracker<DummyTask>>(),
            "EntityTaskTracker should be removed after task completion"
        );
    }
}
