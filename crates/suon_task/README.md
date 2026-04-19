# suon_task

Async task scheduling integration for the Suon MMORPG framework.

`suon_task` provides:

- A `BackgroundTask` trait for defining async work units
- `TaskCommands` and `EntityTaskCommands` for spawning tasks from Bevy systems
- `EntityIn<T>` for entity-scoped task inputs
- `AppWithBackgroundTasks` extension to register task result-reconciliation systems

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_task = { path = "../suon_task" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_task::prelude::*;

struct LoadProfile {
    account_id: u64,
}

impl BackgroundTask for LoadProfile {
    type Output = Option<PlayerData>;

    async fn run(self) -> Self::Output {
        // async database fetch — runs off the main thread
        fetch_profile(self.account_id).await
    }
}

fn spawn_load(mut commands: Commands, entity: Entity) {
    commands
        .entity(entity)
        .spawn_task(LoadProfile { account_id: 42 });
}

fn handle_result(mut tasks: TaskCommands<LoadProfile>) {
    for (entity, result) in tasks.drain() {
        if let Some(data) = result {
            println!("Loaded profile for {:?}: {:?}", entity, data);
        }
    }
}

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_background_task_systems::<Update, LoadProfile>()
        .add_systems(Update, (spawn_load, handle_result))
        .run();
}
```

## How It Works

`suon_task` is a thin wrapper around Bevy's `AsyncComputeTaskPool`. When you call
`spawn_task`, the async work runs on a background thread pool. Each frame, the registered
reconciliation system polls completed futures and returns their outputs through
`TaskCommands::drain()`, keeping all result handling on the main ECS thread.

### Entity-Scoped Tasks

`EntityTaskCommands` binds a task to a specific entity. If the entity is despawned before
the task completes, the result is silently discarded — no cleanup needed.

`EntityIn<T>` wraps a task input together with its owning entity for cases where the
background work needs to know which entity it belongs to.
