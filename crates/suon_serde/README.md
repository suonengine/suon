# suon_serde

Serialization helpers and derive utilities for the Suon MMORPG framework.

`suon_serde` provides:

- `as_millis` — serialize/deserialize `std::time::Duration` as integer milliseconds
- `as_secs` — serialize/deserialize `std::time::Duration` as integer seconds

## Installation

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
suon_serde = { path = "../suon_serde" }
```

## Quick Start

```rust,ignore
use std::time::Duration;
use serde::{Deserialize, Serialize};
use suon_serde::{as_millis, as_secs};

#[derive(Serialize, Deserialize)]
struct RateLimitConfig {
    #[serde(with = "as_millis")]
    window: Duration,

    #[serde(with = "as_secs")]
    timeout: Duration,
}
```

Serialized form:

```json
{ "window": 500, "timeout": 30 }
```

## How It Works

Both modules expose the four functions required by serde's `with` attribute:
`serialize`, `deserialize`. They convert between `Duration` and the corresponding
integer type (`u64` for milliseconds, `u64` for seconds) using `Duration::from_millis`
and `Duration::from_secs` respectively.

These adapters are used throughout Suon's configuration and protocol crates wherever
`Duration` values appear in TOML settings files.
