//! World-scoped background task helpers.

use bevy::{
    ecs::system::SystemId,
    prelude::*,
    tasks::{IoTaskPool, Task, futures_lite::future},
};

use crate::background::BackgroundTask;

/// Component to track a world background task associated with an entity.
#[derive(Component)]
pub(crate) struct WorldTaskTracker<T: BackgroundTask> {
    task: Task<T::Output>,
    system_id: Option<SystemId<In<T::Output>>>,
}

/// Trait for spawning background tasks within command context.
pub trait TaskCommands {
    /// Spawns a background task without an associated system.
    fn spawn_background_task<T>(&mut self, task: T)
    where
        T: BackgroundTask;

    /// Spawns a background task and registers a system to run upon completion.
    fn spawn_background_task_with_system<T, S, Marker>(&mut self, task: T, system: S)
    where
        T: BackgroundTask,
        S: IntoSystem<In<T::Output>, (), Marker> + Send + Sync + 'static,
        Marker: Send + Sync + 'static;
}

impl TaskCommands for World {
    fn spawn_background_task<T>(&mut self, task: T)
    where
        T: BackgroundTask,
    {
        let task_handle = IoTaskPool::get().spawn(async move { task.run().await });

        self.spawn(WorldTaskTracker::<T> {
            task: task_handle,
            system_id: None,
        });
    }

    fn spawn_background_task_with_system<T, S, Marker>(&mut self, task: T, system: S)
    where
        T: BackgroundTask,
        S: IntoSystem<In<T::Output>, (), Marker> + Send + Sync + 'static,
        Marker: Send + Sync + 'static,
    {
        let task_handle = IoTaskPool::get().spawn(async move { task.run().await });
        let system: S::System = IntoSystem::into_system(system);
        let system_id = self.register_system(system);

        self.spawn(WorldTaskTracker::<T> {
            task: task_handle,
            system_id: Some(system_id),
        });
    }
}

impl<'w, 's> TaskCommands for Commands<'w, 's> {
    fn spawn_background_task<T>(&mut self, task: T)
    where
        T: BackgroundTask,
    {
        let task_handle = IoTaskPool::get().spawn(async move { task.run().await });

        self.spawn(WorldTaskTracker::<T> {
            task: task_handle,
            system_id: None,
        });
    }

    fn spawn_background_task_with_system<T, S, Marker>(&mut self, task: T, system: S)
    where
        T: BackgroundTask,
        S: IntoSystem<In<T::Output>, (), Marker> + Send + Sync + 'static,
        Marker: Send + Sync + 'static,
    {
        let task_handle = IoTaskPool::get().spawn(async move { task.run().await });
        let system: S::System = IntoSystem::into_system(system);
        let system_id = self.register_system(system);

        self.spawn(WorldTaskTracker::<T> {
            task: task_handle,
            system_id: Some(system_id),
        });
    }
}

/// System to check for completed background tasks and run associated systems.
pub(crate) fn check_completed_world_tasks<T: BackgroundTask>(
    mut commands: Commands,
    mut query: Query<(Entity, &mut WorldTaskTracker<T>)>,
) {
    for (entity, mut tracker) in query.iter_mut() {
        // Poll the task asynchronously until it completes
        let Some(result) = future::block_on(future::poll_once(&mut tracker.task)) else {
            continue;
        };

        // Run the associated system if registered
        if let Some(system_id) = tracker.system_id {
            commands.run_system_with(system_id, result);
        }

        // Despawn the entity after task completion
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run_until_world_tasks_finish<T: BackgroundTask>(app: &mut App) {
        loop {
            app.update();

            let remaining = app
                .world_mut()
                .query::<&WorldTaskTracker<T>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break;
            }
        }
    }

    #[test]
    fn should_run_completion_system_for_world_background_tasks() {
        use std::sync::{Arc, Mutex};

        struct ImmediateTask(pub i32);

        impl BackgroundTask for ImmediateTask {
            type Output = i32;

            async fn run(self) -> Self::Output {
                self.0
            }
        }

        // Shared variable to capture the result from the system callback
        let callback_result = Arc::new(Mutex::new(None));
        let callback_result_clone = callback_result.clone();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world_mut().spawn_background_task_with_system(
            ImmediateTask(42),
            move |In(result): In<i32>| {
                *callback_result_clone.lock().unwrap() = Some(result);
            },
        );

        app.add_systems(Update, check_completed_world_tasks::<ImmediateTask>);

        run_until_world_tasks_finish::<ImmediateTask>(&mut app);

        assert_eq!(
            *callback_result.lock().unwrap(),
            Some(42),
            "The callback did not capture the expected result"
        );
    }

    #[test]
    fn should_complete_world_background_tasks_without_callbacks() {
        use std::sync::{Arc, Mutex};

        struct FlagTask {
            pub result_value: i32,
            pub completed: Arc<Mutex<bool>>,
        }

        impl BackgroundTask for FlagTask {
            type Output = i32;

            async fn run(self) -> Self::Output {
                let completed = self.completed.clone();
                future::yield_now().await;
                *completed.lock().unwrap() = true;
                self.result_value
            }
        }

        let completion_flag = Arc::new(Mutex::new(false));

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world_mut().spawn_background_task(FlagTask {
            result_value: 42,
            completed: completion_flag.clone(),
        });

        app.add_systems(Update, check_completed_world_tasks::<FlagTask>);

        run_until_world_tasks_finish::<FlagTask>(&mut app);

        assert!(
            *completion_flag.lock().unwrap(),
            "The task did not mark as completed"
        );
    }

    #[test]
    fn should_finish_slow_world_background_tasks() {
        use std::{thread::sleep, time::Duration};

        struct SlowTask;

        impl BackgroundTask for SlowTask {
            type Output = ();

            async fn run(self) -> Self::Output {
                sleep(Duration::from_millis(10));
            }
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.world_mut().spawn_background_task(SlowTask);

        app.add_systems(Update, check_completed_world_tasks::<SlowTask>);

        run_until_world_tasks_finish::<SlowTask>(&mut app);
    }

    #[test]
    fn should_collect_results_from_multiple_world_background_tasks() {
        use std::{
            sync::{Arc, Mutex},
            thread::sleep,
            time::Duration,
        };

        struct DelayedTask(pub i32, pub Duration);

        impl BackgroundTask for DelayedTask {
            type Output = i32;

            async fn run(self) -> Self::Output {
                let delay = self.1;
                let value = self.0;
                sleep(delay);
                value
            }
        }

        let results = Arc::new(Mutex::new(Vec::<i32>::new()));
        let results_clone = results.clone();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let task_params = vec![
            (1, Duration::from_millis(10)),
            (2, Duration::from_millis(20)),
            (3, Duration::from_millis(30)),
        ];

        for (value, delay) in &task_params {
            let results_inner = results_clone.clone();
            app.world_mut().spawn_background_task_with_system(
                DelayedTask(*value, *delay),
                move |In(result): In<i32>| {
                    results_inner.lock().unwrap().push(result);
                },
            );
        }

        app.add_systems(Update, check_completed_world_tasks::<DelayedTask>);

        run_until_world_tasks_finish::<DelayedTask>(&mut app);

        let mut results_vec = results.lock().unwrap().clone();
        results_vec.sort();
        assert_eq!(
            results_vec,
            vec![1, 2, 3],
            "Results do not match expected values"
        );
    }

    #[test]
    fn should_despawn_world_task_tracker_without_callback() {
        struct ImmediateTask;

        impl BackgroundTask for ImmediateTask {
            type Output = ();

            async fn run(self) -> Self::Output {}
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, check_completed_world_tasks::<ImmediateTask>);

        app.world_mut().spawn_background_task(ImmediateTask);

        run_until_world_tasks_finish::<ImmediateTask>(&mut app);

        assert!(
            app.world_mut()
                .query::<&WorldTaskTracker<ImmediateTask>>()
                .iter(app.world())
                .next()
                .is_none(),
            "World task tracker entities should be despawned even without callbacks"
        );
    }
}
