# Examples

Runnable examples for the Suon MMORPG framework.

## Running an Example

From the repository root:

```sh
cargo run --example <name>
```

## Example List

### `server`

```sh
cargo run --example server
```

Bootstraps a minimal Suon server using `SuonPlugin`. This is the simplest possible
entry point — all configuration is loaded from the `settings/` directory.

```rust,ignore
use suon::prelude::*;

fn main() {
    App::new()
        .add_plugins(SuonPlugin)
        .run();
}
```

`SuonPlugin` installs:

- Headless Bevy runtime (thread count from `settings/Settings.toml`)
- `ObservabilityPlugin` (diagnostics from `settings/ObservabilitySettings.toml`)
- `ChunkPlugins` — world chunk and occupancy management
- `MovementPlugins` — step and teleport intent processing
- `NetworkPlugins` — TCP session management
- `LuaPlugin` — Lua 5.4 scripting runtime

Missing settings files are created with defaults on first run.
