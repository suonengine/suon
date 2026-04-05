//! Entity-bound background task helpers.

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

    fn run_until_entity_tasks_finish<T: BackgroundTask>(app: &mut App) {
        loop {
            app.update();

            let remaining = app
                .world_mut()
                .query::<&EntityTaskTracker<T>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break;
            }
        }
    }

    #[test]
    fn should_run_completion_system_for_entity_background_tasks() {
        use std::sync::{Arc, Mutex};

        struct DummyTask(pub i32);

        impl BackgroundTask for DummyTask {
            type Output = i32;

            async fn run(self) -> Self::Output {
                self.0
            }
        }

        let callback_result = Arc::new(Mutex::new(None));
        let callback_result_clone = callback_result.clone();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world_mut()
            .spawn_empty()
            .spawn_background_task_with_system(
                DummyTask(99),
                move |EntityIn(result): EntityIn<i32>| {
                    *callback_result_clone.lock().unwrap() = Some(result);
                },
            );

        app.add_systems(Update, check_completed_entity_tasks::<DummyTask>);

        run_until_entity_tasks_finish::<DummyTask>(&mut app);

        let result = callback_result.lock().unwrap();
        assert!(result.is_some(), "Callback did not produce a result");

        let (entity, value) = result.as_ref().unwrap();
        assert_eq!(
            *value, 99,
            "The callback result value does not match expected"
        );

        assert!(
            !app.world()
                .entity(*entity)
                .contains::<EntityTaskTracker<DummyTask>>(),
            "Entity should not have EntityTaskTracker after task completion"
        );
    }

    #[test]
    fn should_remove_entity_task_tracker_after_completion() {
        struct SimpleTask;

        impl BackgroundTask for SimpleTask {
            type Output = String;

            async fn run(self) -> Self::Output {
                "done".to_string()
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let entity = app
            .world_mut()
            .spawn_empty()
            .spawn_background_task(SimpleTask)
            .id();

        app.add_systems(Update, check_completed_entity_tasks::<SimpleTask>);

        run_until_entity_tasks_finish::<SimpleTask>(&mut app);

        assert!(
            !app.world()
                .entity(entity)
                .contains::<EntityTaskTracker<SimpleTask>>(),
            "EntityTaskTracker component should be removed after task completion"
        );
    }

    #[test]
    fn should_collect_results_from_multiple_entity_background_tasks() {
        use std::{
            sync::{Arc, Mutex},
            thread::sleep,
            time::Duration,
        };

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

        let results = Arc::new(Mutex::new(Vec::<(Entity, i32)>::new()));

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

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
                            results_clone.lock().unwrap().push(result);
                        },
                    )
                    .id();
                (entity, i)
            })
            .collect::<Vec<(Entity, i32)>>();

        app.add_systems(Update, check_completed_entity_tasks::<DelayedEntityTask>);

        run_until_entity_tasks_finish::<DelayedEntityTask>(&mut app);

        let results_vec = results.lock().unwrap().clone();

        assert_eq!(
            results_vec, entities,
            "Results from callbacks do not match expected values"
        );
    }

    #[test]
    fn should_remove_entity_task_tracker_without_callback() {
        struct ImmediateTask;

        impl BackgroundTask for ImmediateTask {
            type Output = ();

            async fn run(self) -> Self::Output {}
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, check_completed_entity_tasks::<ImmediateTask>);

        let entity = app
            .world_mut()
            .spawn_empty()
            .spawn_background_task(ImmediateTask)
            .id();

        run_until_entity_tasks_finish::<ImmediateTask>(&mut app);

        assert!(
            !app.world()
                .entity(entity)
                .contains::<EntityTaskTracker<ImmediateTask>>(),
            "Entity trackers should still be removed when no completion callback is registered"
        );
    }
}
