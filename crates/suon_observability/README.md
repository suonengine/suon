# suon_observability

Server observability and diagnostics for the Suon MMORPG framework.

`suon_observability` provides:

- A single `ObservabilitySettings` resource that toggles all diagnostics
- Optional logging, Bevy metrics, frame-time, entity-count, and system-information plugins
- Settings loaded from (or written to) `settings/ObservabilitySettings.toml`
- `ObservabilityPlugin` that wires the selected diagnostics into the Bevy app

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_observability = { path = "../suon_observability" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_observability::prelude::*;

fn main() {
    let settings = ObservabilitySettings::load_or_default();

    App::new()
        .add_plugins((MinimalPlugins, ObservabilityPlugin))
        .insert_resource(settings)
        .run();
}
```

## How It Works

`ObservabilityPlugin` reads `ObservabilitySettings` during app construction and
conditionally installs Bevy diagnostic plugins based on the active flags. All settings
default to a conservative production configuration (logging on, heavy diagnostics off).

### Settings

| Field | Default | Effect |
|---|---|---|
| `log` | `true` | Install Bevy `LogPlugin` |
| `metrics` | `false` | Install Bevy metrics/diagnostics infrastructure |
| `log_metrics` | `false` | Install `LogDiagnosticsPlugin` (prints diagnostics to the log) |
| `frame_time` | `false` | Install `FrameTimeDiagnosticsPlugin` |
| `entity_count` | `false` | Install `EntityCountDiagnosticsPlugin` |
| `system_information` | `false` | Install `SystemInformationDiagnosticsPlugin` |

### Configuration File

`ObservabilitySettings::load_or_default()` reads from `settings/ObservabilitySettings.toml`.
If the file does not exist, it is created with defaults so the server starts without errors.

```toml
# settings/ObservabilitySettings.toml
log = true
metrics = false
log_metrics = false
frame_time = false
entity_count = false
system_information = false
```
