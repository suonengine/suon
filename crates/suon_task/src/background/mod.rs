//! Background task orchestration for world and entity scopes.

use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

pub mod entity;
pub mod world;

/// Trait representing a background task that can be executed asynchronously.
/// It must be Send and 'static to be safely run in a background thread.
pub trait BackgroundTask: Send + Sync + 'static {
    /// The type of output produced by the task.
    type Output: Send + Sync + 'static;

    /// Executes the task asynchronously, returning a future with the result.
    ///
    /// # Examples
    /// ```
    /// use suon_task::background::BackgroundTask;
    ///
    /// struct SumTask(u32, u32);
    ///
    /// impl BackgroundTask for SumTask {
    ///     type Output = u32;
    ///
    ///     async fn run(self) -> Self::Output {
    ///         self.0 + self.1
    ///     }
    /// }
    /// ```
    fn run(self) -> impl Future<Output = Self::Output> + Send + 'static;
}

/// Extension trait for Bevy App to easily add background task handling systems.
pub trait AppWithBackgroundTasks {
    /// Adds systems to manage the completion of background tasks for the specified schedule label.
    ///
    /// # Type Parameters
    /// - `S`: The schedule label to insert the systems into.
    /// - `T`: The type of background task being managed.
    ///
    /// # Examples
    /// ```
    /// use bevy::prelude::*;
    /// use suon_task::background::{AppWithBackgroundTasks, BackgroundTask};
    ///
    /// struct ExampleTask;
    ///
    /// impl BackgroundTask for ExampleTask {
    ///     type Output = ();
    ///
    ///     async fn run(self) -> Self::Output {}
    /// }
    ///
    /// let mut app = App::new();
    /// app.add_plugins(MinimalPlugins);
    /// app.add_background_task_systems::<Update, ExampleTask>();
    /// ```
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

    struct DummyTask(pub i32);

    impl BackgroundTask for DummyTask {
        type Output = i32;

        async fn run(self) -> Self::Output {
            self.0
        }
    }

    #[test]
    fn should_add_background_task_systems_to_the_app() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_background_task_systems::<Update, DummyTask>();

        app.world_mut()
            .spawn_empty()
            .spawn_background_task_with_system(
                DummyTask(i32::MAX),
                |EntityIn((.., result)): EntityIn<i32>| {
                    assert!(result == i32::MAX);
                },
            );

        assert!(
            app.world_mut()
                .query::<&EntityTaskTracker<DummyTask>>()
                .iter(app.world())
                .next()
                .is_some(),
            "EntityTaskTracker should exist after spawning an entity background task"
        );

        app.world_mut().spawn_background_task_with_system(
            DummyTask(i32::MAX),
            |In(result): In<i32>| {
                assert!(result == i32::MAX);
            },
        );

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
    fn should_remove_world_task_trackers_after_completion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_background_task_systems::<Update, DummyTask>();

        app.world_mut()
            .spawn_empty()
            .spawn_background_task_with_system(
                DummyTask(i32::MAX),
                |EntityIn((.., result)): EntityIn<i32>| {
                    assert!(result == i32::MAX);
                },
            );

        loop {
            app.update();

            let remaining = app
                .world_mut()
                .query::<&WorldTaskTracker<DummyTask>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break;
            }
        }

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
    fn should_remove_entity_task_trackers_after_completion() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_background_task_systems::<Update, DummyTask>();

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

        loop {
            app.update();

            let remaining = app
                .world_mut()
                .query::<&EntityTaskTracker<DummyTask>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break;
            }
        }

        assert!(
            !app.world()
                .entity(entity)
                .contains::<EntityTaskTracker<DummyTask>>(),
            "EntityTaskTracker should be removed after task completion"
        );
    }

    #[test]
    fn should_return_the_same_app_reference_for_background_task_system_registration() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let returned = app.add_background_task_systems::<Update, DummyTask>();

        assert!(
            std::ptr::eq(returned, &app),
            "add_background_task_systems should support fluent chaining by returning the same App"
        );
    }
}
