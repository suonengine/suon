# suon_rng

Deterministic random number resources for the Suon MMORPG framework.

`suon_rng` provides:

- `Random` - ChaCha8-backed deterministic RNG for gameplay rules
- `FastRandom` - WyRand-backed deterministic RNG for hot paths
- `RngSettings` - TOML-backed server seed configuration
- `RNGPlugin` - Bevy plugin that loads settings and registers RNG resources

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_rng = { path = "../suon_rng" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_rng::prelude::*;

fn roll(mut random: ResMut<Random>) {
    let damage = random.range_u32(10, 20);
    let critical = random.chance(5, 100);

    info!("damage={damage}, critical={critical}");
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, RNGPlugin))
        .add_systems(Update, roll)
        .run();
}
```

On first run, `RNGPlugin` creates:

```text
settings/RngSettings.toml
```

The generated `seed` uses the current Unix timestamp in milliseconds. The same seed can be kept
for reproducible server behavior or replaced manually for a fresh world run.

## How It Works

Suon keeps two deterministic RNG resources:

| Type | Role |
|---|---|
| `Random` | Gameplay RNG backed by ChaCha8 |
| `FastRandom` | Lightweight RNG backed by WyRand |
| `RngSettings` | Server seed loaded from `settings/RngSettings.toml` |
| `RNGPlugin` | Loads settings, prints the seed, and inserts RNG resources |

`FastRandom` derives its seed from the configured server seed, so operators only need to track one
value when reproducing a server session.

## Seeded Runs

```rust,ignore
use bevy::prelude::*;
use suon_rng::prelude::*;

let mut app = App::new();
app.insert_resource(RngSettings::new(1_774_999_200_000));
app.add_plugins(RNGPlugin);
```

When no `RngSettings` resource is provided, the plugin loads or creates
`settings/RngSettings.toml` and logs the seed used during startup.
