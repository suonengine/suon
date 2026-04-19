# suon_checksum

Packet checksum utilities for the Suon MMORPG framework.

`suon_checksum` provides:

- Adler-32 checksum calculation over raw packet bytes
- Checksum extraction from encoded packets
- Formatted string representation for debugging

## Installation

```toml
[dependencies]
suon_checksum = { path = "../suon_checksum" }
```

## Quick Start

```rust,ignore
use suon_checksum::prelude::*;

// Calculate a checksum for a packet payload
let payload = &[0x0A, 0x00, 0x01, 0xFF];
let checksum = Adler32Checksum::calculate(payload);

println!("{checksum}");
```

## How It Works

`suon_checksum` wraps the [Adler-32](https://en.wikipedia.org/wiki/Adler-32) algorithm, a
rolling checksum widely used in network protocols for lightweight integrity verification.

The core type is `Adler32Checksum`, which exposes:

- `Adler32Checksum::calculate(&[u8])` — computes the checksum over a byte slice
- Component accessors to extract the two 16-bit halves (A and B) of the Adler-32 sum
- `Display` impl for human-readable output during debugging
