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

    #[test]
    fn test_spawn_background_task_with_system_callback() {
        use std::sync::{Arc, Mutex};

        /// Dummy task that immediately returns a specified value
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

        // Create the Bevy app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn a background task and register a system callback to capture its result
        app.world_mut().spawn_background_task_with_system(
            ImmediateTask(42),
            move |In(result): In<i32>| {
                // Store the callback result for later verification
                *callback_result_clone.lock().unwrap() = Some(result);
            },
        );

        // Add a system that checks for completed world tasks each frame
        app.add_systems(Update, check_completed_world_tasks::<ImmediateTask>);

        // Loop until the background task completes
        loop {
            app.update();

            // Check if any entities still have the task tracker component
            let remaining = app
                .world_mut()
                .query::<&WorldTaskTracker<ImmediateTask>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break; // All tasks completed and trackers removed
            }
        }

        // Assert that the system callback received the expected result
        assert_eq!(
            *callback_result.lock().unwrap(),
            Some(42),
            "The callback did not capture the expected result"
        );
    }

    #[test]
    fn test_background_task_sets_completion_flag() {
        use std::sync::{Arc, Mutex};

        /// Dummy task that updates a completion flag when done
        struct FlagTask {
            pub result_value: i32,
            pub completed: Arc<Mutex<bool>>,
        }

        impl BackgroundTask for FlagTask {
            type Output = i32;

            async fn run(self) -> Self::Output {
                let completed = self.completed.clone();
                // Yield to simulate some work
                future::yield_now().await;
                // Mark the task as completed
                *completed.lock().unwrap() = true;
                self.result_value
            }
        }

        let completion_flag = Arc::new(Mutex::new(false));

        // Create the app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn a background task that sets the completion flag
        app.world_mut().spawn_background_task(FlagTask {
            result_value: 42,
            completed: completion_flag.clone(),
        });

        // Add a system to check for task completion each frame
        app.add_systems(Update, check_completed_world_tasks::<FlagTask>);

        // Loop until the background task completes
        loop {
            app.update();

            // Check if any entities still have the task tracker component
            let remaining = app
                .world_mut()
                .query::<&WorldTaskTracker<FlagTask>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break; // All tasks completed and trackers removed
            }
        }

        // Assert that the task has completed and the flag is set
        assert!(
            *completion_flag.lock().unwrap(),
            "The task did not mark as completed"
        );
    }

    #[test]
    fn test_slow_task_respects_sla() {
        use std::{thread::sleep, time::Duration};

        /// A slow task that takes approximately 10 milliseconds to complete
        struct SlowTask;

        impl BackgroundTask for SlowTask {
            type Output = ();

            async fn run(self) -> Self::Output {
                sleep(Duration::from_millis(10));
            }
        }

        // Create the app and set the SLA timeout resource to 20ms
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn the slow background task
        app.world_mut().spawn_background_task(SlowTask);

        // Add system to check for task completion
        app.add_systems(Update, check_completed_world_tasks::<SlowTask>);

        // Loop until the background task completes
        loop {
            app.update();

            // Check if any entities still have the task tracker component
            let remaining = app
                .world_mut()
                .query::<&WorldTaskTracker<SlowTask>>()
                .iter(app.world())
                .count();

            if remaining == 0 {
                break; // All tasks completed and trackers removed
            }
        }
    }

    #[test]
    fn test_multiple_background_tasks_with_callbacks() {
        use std::{
            sync::{Arc, Mutex},
            thread::sleep,
            time::Duration,
        };

        // Define a task that delays for a specified duration before returning its value
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

        // Shared vector to store results from callbacks
        let results = Arc::new(Mutex::new(Vec::<i32>::new()));
        let results_clone = results.clone();

        // Create the app
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Spawn multiple delayed tasks with different values and delays
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
                    // Push each completed result into the shared vector
                    results_inner.lock().unwrap().push(result);
                },
            );
        }

        // Add system to monitor and process completed tasks
        app.add_systems(Update, check_completed_world_tasks::<DelayedTask>);

        // Run frames until all tasks are finished
        loop {
            app.update();

            // Count remaining tasks
            let remaining_tasks = app
                .world_mut()
                .query::<&WorldTaskTracker<DelayedTask>>()
                .iter(app.world())
                .count();

            if remaining_tasks == 0 {
                break; // All tasks completed
            }
        }

        // Verify all results are received
        let mut results_vec = results.lock().unwrap().clone();
        results_vec.sort();
        assert_eq!(
            results_vec,
            vec![1, 2, 3],
            "Results do not match expected values"
        );
    }
}
