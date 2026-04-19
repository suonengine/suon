# suon_network

Networking layer and session management for the Suon MMORPG framework.

`suon_network` provides:

- Async TCP connection management via `smol` and `tokio`
- Packet framing, checksum modes, and connection abstractions
- Per-session rate limiting, throttle policies, and quotas
- ECS integration through `IncomingConnections` and `OutgoingConnections` resources
- A `NetworkPlugins` plugin group that wires everything into the Bevy schedule

## Installation

```toml
[dependencies]
bevy = "0.18"
suon_network = { path = "../suon_network" }
```

## Quick Start

```rust,ignore
use bevy::prelude::*;
use suon_network::prelude::*;

fn handle_incoming(mut connections: ResMut<IncomingConnections>) {
    for (connection, packet) in connections.drain() {
        println!("Received {} bytes from {:?}", packet.len(), connection);
    }
}

fn main() {
    App::new()
        .add_plugins((MinimalPlugins, NetworkPlugins))
        .add_systems(Update, handle_incoming)
        .run();
}
```

## How It Works

`NetworkPlugins` starts an async listener on startup. Incoming TCP streams are accepted
and wrapped in `Connection` handles backed by `crossbeam-channel` queues. Each frame,
the ECS resources drain those queues so systems can process packets as plain Rust values.

### Connection

`Connection` is the core handle for sending and receiving data. It manages packet framing
and exposes a synchronous API usable from Bevy systems without blocking the main thread.

### Session Policies

Policies control how each connection is admitted and throttled:

| Type | Role |
|---|---|
| `ChecksumMode` | Enable or skip Adler-32 verification on incoming packets |
| `Limiter` | Token-bucket rate limiter per connection |
| `SessionQuota` | Maximum allowed packets / bytes per time window |
| `IncomingPacketPolicy` | Policy applied to packets the server receives |
| `OutgoingPacketPolicy` | Policy applied to packets the server sends |
| `ThrottlePolicy` | Action to take when a session exceeds its quota (drop, delay, kick) |
| `PacketPolicyPenalty` | Penalty applied on policy violation |

### Configuration

Network settings are loaded from `settings/NetworkSettings.toml` via TOML deserialization.
Missing fields fall back to sensible defaults.
