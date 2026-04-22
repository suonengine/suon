# suon (umbrella crate)

The `src/` directory contains the root `suon` crate — the single published entry point
for the entire workspace.

## Files

| File | Purpose |
|---|---|
| `lib.rs` | Defines `SuonPlugin` and the workspace prelude |
| `settings.rs` | `Settings` resource and TOML file I/O |
| `system.rs` | `PreStartup`/`Startup` bootstrap systems for root settings |

## `SuonPlugin`

`SuonPlugin` is the one-line way to start a Suon server. It reads `Settings` from
`settings/Settings.toml`, loads them into the world during `PreStartup`, and installs:

| Plugin | Source |
|---|---|
| `MinimalPlugins` | Bevy headless runtime, thread pool sized by `Settings::threads` |
| `ScheduleRunnerPlugin` | Installed when `Settings::schedule_runner = true` |
| `ObservabilityPlugin` | Logging and diagnostics from `suon_observability` |
| `ChunkPlugins` | World chunk system from `suon_chunk` |
| `MovementPlugins` | Movement intent processing from `suon_movement` |
| `NetworkPlugins` | TCP networking from `suon_network` |
| `LuaPlugin` | Lua 5.4 scripting from `suon_lua` |

## `Settings`

```toml
# settings/Settings.toml
threads = 4
event_loop = 16.0       # target ticks per second
fixed_event_loop = 8.0  # fixed-timestep ticks per second
schedule_runner = false
```

`Settings::load_or_default()` reads the file; if it does not exist the directory and file
are created with defaults.

`SuonPlugin` uses those settings in two phases:
- `PreStartup`: load or preserve the `Settings` resource
- `Startup`: initialize `Time<Fixed>` from `Settings::fixed_event_loop`

## Prelude

`suon::prelude` re-exports the preludes of all workspace crates plus `bevy::prelude`,
so a single `use suon::prelude::*;` is enough to access every type in the framework:

```rust,ignore
use suon::prelude::*;

fn main() {
    App::new()
        .add_plugins(SuonPlugin)
        .run();
}
```
