# suon_protocol_server

Server-to-client packet definitions for the Suon MMORPG framework.

`suon_protocol_server` provides:

- Typed structs for every packet the server sends to game clients
- The `Encodable` trait for serializing packets into wire-format bytes
- The `PacketKind` trait mapping packets to their 1-byte discriminant
- `encode_with_kind()` to prepend the kind byte automatically

## Installation

```toml
[dependencies]
suon_protocol_server = { path = "../suon_protocol_server" }
suon_protocol = { path = "../suon_protocol" }
```

## Quick Start

```rust,ignore
use suon_protocol_server::prelude::*;

fn send_challenge(connection: &mut Connection) {
    let packet = ChallengePacket { key: [0xDE, 0xAD, 0xBE, 0xEF] };
    let bytes = packet.encode_with_kind();
    connection.send(bytes);
}
```

## How It Works

Each packet struct implements `Encodable` and `PacketKind`. Call `encode_with_kind()` to
get a `Bytes` buffer with the 1-byte kind discriminant prepended, ready to write to the
TCP stream.

The underlying encoding uses `suon_protocol::Encoder`, so field order in the wire format
matches the struct field order in the implementation.

## Packet Reference

| Packet | Description |
|---|---|
| `ChallengePacket` | Sent during the handshake to establish the XTEA session key |
| `KeepAlivePacket` | Server-initiated heartbeat to detect dead connections |
| `PingLatencyPacket` | Round-trip latency measurement response |
