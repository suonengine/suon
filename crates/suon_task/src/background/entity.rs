use bevy::{
    ecs::system::SystemId,
    prelude::*,
    tasks::{IoTaskPool, Task, futures_lite::future},
};

use crate::background::BackgroundTask;

/// Wrapper struct for passing entity and task result as system input.
#[derive(Debug)]
pub struct EntityIn<T>(pub (Entity, T));

impl<T: 'static> SystemInput for EntityIn<T> {
    type Param<'i> = EntityIn<T>;
    type Inner<'i> = EntityIn<T>;

    /// Wraps the inner value into the input struct.
    fn wrap(this: Self::Inner<'_>) -> Self::Param<'_> {
        this
    }
}

/// Component to track a background task associated with an entity.
#[derive(Component)]
pub(crate) struct EntityTaskTracker<T: BackgroundTask> {
    task: Task<T::Output>,
    system_id: Option<SystemId<EntityIn<T::Output>>>,
}

/// Trait providing methods to spawn background tasks on entities.
pub trait EntityTaskCommands {
    /// Spawns a background task and attaches a tracker component.
    fn spawn_background_task<T>(&mut self, command: T) -> &mut Self
    where
        T: BackgroundTask;

    /// Spawns a background task and registers a system to run upon completion.
    fn spawn_background_task_with_system<T, S, Marker>(
        &mut self,
        command: T,
        system: S,
    ) -> &mut Self
    where
        T: BackgroundTask,
        S: IntoSystem<EntityIn<T::Output>, (), Marker> + Send + Sync + 'static,
        Marker: Send + Sync + 'static;
}

impl<'a> EntityTaskCommands for EntityWorldMut<'a> {
    fn spawn_background_task<T>(&mut self, command: T) -> &mut Self
    where
        T: BackgroundTask,
    {
        // Spawn the async background task
        let task = IoTaskPool::get().spawn(async move { command.run().await });

        // Insert the task tracker component
        self.insert(EntityTaskTracker::<T> {
            task,
            system_id: None,
        });

        self
    }

    fn spawn_background_task_with_system<T, S, Marker>(
        &mut self,
        command: T,
        system: S,
    ) -> &mut Self
    where
        T: BackgroundTask,
        S: IntoSystem<EntityIn<T::Output>, (), Marker> + Send + Sync + 'static,
        Marker: Send + Sync + 'static,
    {
        // Spawn the async background task
        let task = IoTaskPool::get().spawn(async move { command.run().await });
        // Convert the system into a Bevy system
        let system: S::System = IntoSystem::into_system(system);
        // Register the system to be run after task completion
        let system_id = self.world_scope(|world: &mut World| world.register_system(system));

        // Insert the task tracker component with linked system
        self.insert(EntityTaskTracker::<T> {
            task,
            system_id: Some(system_id),
        });

        self
    }
}

impl<'a> EntityTaskCommands for EntityCommands<'a> {
    fn spawn_background_task<T>(&mut self, command: T) -> &mut Self
    where
        T: BackgroundTask,
    {
        // Spawn the async background task
        let task = IoTaskPool::get().spawn(command.run());

        // Insert the task tracker component
        self.insert(EntityTaskTracker::<T> {
            task,
            system_id: None,
        });

        self
    }

    fn spawn_background_task_with_system<T, S, Marker>(
        &mut self,
        command: T,
        system: S,
    ) -> &mut Self
    where
        T: BackgroundTask,
        S: IntoSystem<EntityIn<T::Output>, (), Marker> + Send + Sync + 'static,
        Marker: Send + Sync + 'static,
    {
        // Spawn the async background task
        let task = IoTaskPool::get().spawn(async move { command.run().await });
        // Convert the system into a Bevy system
        let system: S::System = IntoSystem::into_system(system);
        // Register the system to be run after task completion
        let system_id = self.commands().register_system(system);

        // Insert the task tracker component with linked system
        self.insert(EntityTaskTracker::<T> {
            task,
            system_id: Some(system_id),
        });

        self
    }
}

/// System to check for completed entity background tasks and execute associated systems.
pub(crate) fn check_completed_entity_tasks<C: BackgroundTask>(
    mut commands: Commands,
    mut query: Query<(Entity, &mut EntityTaskTracker<C>)>,
) {
    for (entity, mut tracker) in query.iter_mut() {
        // Poll the task asynchronously until it completes
        let Some(result) = future::block_on(future::poll_once(&mut tracker.task)) else {
            continue;
        };

        // Run the associated system with the entity and task result
        if let Some(system_id) = tracker.system_id {
            commands.run_system_with(system_id, EntityIn((entity, result)));
        }

        // Remove the tracker component after task completion
        commands.entity(entity).remove::<EntityTaskTracker<C>>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spawn_entity_background_task_with_system_callback() {
        use std::sync::{Arc, Mutex};

        // Define a dummy background task that simply returns its input value
        struct DummyTask(pub i32);

        impl BackgroundTask for DummyTask {
            type Output = i32;

            async fn run(self) -> Self::Output {
                self.0
            }
        }

        // Shared variable to store the callback's result for verification
        let callback_result = Arc::new(Mutex::new(None));
        let callback_result_clone = callback_result.clone();

        // Initialize Bevy app and add minimal plugins
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn an entity and attach a background task with a system callback
        app.world_mut()
            .spawn_empty()
            .spawn_background_task_with_system(
                DummyTask(99),
                move |EntityIn(result): EntityIn<i32>| {
                    // Callback captures the result and stores it for later validation
                    *callback_result_clone.lock().unwrap() = Some(result);
                },
            );

        // Add a system to process completed entity tasks
        app.add_systems(Update, check_completed_entity_tasks::<DummyTask>);

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

        // Validate that the callback was invoked and received the expected value
        let result = callback_result.lock().unwrap();
        assert!(result.is_some(), "Callback did not produce a result");

        let (entity, value) = result.as_ref().unwrap();
        assert_eq!(
            *value, 99,
            "The callback result value does not match expected"
        );

        // Confirm that the entity no longer has the task tracker component after completion
        assert!(
            !app.world()
                .entity(*entity)
                .contains::<EntityTaskTracker<DummyTask>>(),
            "Entity should not have EntityTaskTracker after task completion"
        );
    }

    #[test]
    fn test_entity_background_task_sets_and_removes_tracker() {
        // Define a simple background task that returns a fixed string
        struct SimpleTask;

        impl BackgroundTask for SimpleTask {
            type Output = String;

            async fn run(self) -> Self::Output {
                "done".to_string()
            }
        }

        // Initialize Bevy app with minimal plugins
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn an entity and attach a background task
        let entity = app
            .world_mut()
            .spawn_empty()
            .spawn_background_task(SimpleTask)
            .id();

        // Add a system to monitor task completion and cleanup
        app.add_systems(Update, check_completed_entity_tasks::<SimpleTask>);

        // Loop until the background task completes
        loop {
            app.update();

            // Check if any entities still have the task tracker component
            let remaining = app
                .world_mut()
                .query::<&EntityTaskTracker<SimpleTask>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break; // All tasks completed and trackers removed
            }
        }

        // Assert that the task tracker component has been cleaned up from the entity
        assert!(
            !app.world()
                .entity(entity)
                .contains::<EntityTaskTracker<SimpleTask>>(),
            "EntityTaskTracker component should be removed after task completion"
        );
    }

    #[test]
    fn test_multiple_entity_background_tasks_with_callbacks() {
        use std::{
            sync::{Arc, Mutex},
            thread::sleep,
            time::Duration,
        };

        // Define a background task that introduces a delay before returning a value
        struct DelayedEntityTask(pub i32, pub Duration);

        impl BackgroundTask for DelayedEntityTask {
            type Output = i32;

            async fn run(self) -> Self::Output {
                let delay = self.1;
                let value = self.0;
                sleep(delay);
                value
            }
        }

        // Shared vector to collect results from callbacks
        let results = Arc::new(Mutex::new(Vec::<(Entity, i32)>::new()));

        // Initialize Bevy app with minimal setup
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn multiple entities, each with a background delayed task and callback
        let entities = (0..3)
            .map(|i| {
                let results_clone = results.clone();
                let entity = app
                    .world_mut()
                    .spawn_empty()
                    .spawn_background_task_with_system(
                        DelayedEntityTask(
                            i,
                            Duration::from_millis((100 * (i + 1)).try_into().unwrap()),
                        ),
                        move |EntityIn(result): EntityIn<i32>| {
                            // Push each result along with its entity into the shared vector
                            results_clone.lock().unwrap().push(result);
                        },
                    )
                    .id();
                (entity, i)
            })
            .collect::<Vec<(Entity, i32)>>();

        // Add system to monitor and process completed tasks
        app.add_systems(Update, check_completed_entity_tasks::<DelayedEntityTask>);

        // Run until all tasks have completed
        loop {
            app.update();

            // Count how many entities still have pending task trackers
            let remaining = app
                .world_mut()
                .query::<&EntityTaskTracker<DelayedEntityTask>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break; // All tasks completed
            }
        }

        // Collect and verify all callback results
        let results_vec = results.lock().unwrap().clone();

        assert_eq!(
            results_vec, entities,
            "Results from callbacks do not match expected values"
        );
    }
}
