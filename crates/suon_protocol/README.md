# suon_protocol

Binary protocol encoding and decoding for the Suon MMORPG framework.

`suon_protocol` provides:

- A fluent `Encoder` for building wire-format packet buffers
- A `Decoder` trait on `&mut &[u8]` for reading typed values from raw bytes
- A `DecoderError` type for decoding failures
- The `PACKET_KIND_SIZE` constant (1 byte) for packet framing

## Installation

```toml
[dependencies]
suon_protocol = { path = "../suon_protocol" }
```

## Quick Start

### Encoding

```rust,ignore
use suon_protocol::prelude::*;

let bytes = Encoder::new()
    .put_u8(0x01)          // packet kind
    .put_u16(1337)         // field
    .put_str("hello")      // length-prefixed UTF-8 string
    .finalize();
```

### Decoding

```rust,ignore
use suon_protocol::prelude::*;

fn decode(buf: &[u8]) -> Result<(u16, String), DecoderError> {
    let mut cursor = buf;
    let value = cursor.get_u16()?;
    let text  = cursor.get_string()?;
    Ok((value, text))
}
```

## How It Works

`suon_protocol` is codec-agnostic — it defines the wire-format building blocks
used by `suon_protocol_client` and `suon_protocol_server`. It does not know about
specific packet kinds; that knowledge lives in the higher-level protocol crates.

### Encoder

`Encoder` accumulates bytes into an internal buffer using a builder pattern. Call
`finalize()` to get the completed `Bytes` frame.

### Decoder

The `Decoder` trait is implemented on `&mut &[u8]`. Each `get_*` call advances the
cursor and returns the decoded value, or `DecoderError` if the buffer is exhausted or
malformed.

### Packet Kind Size

`PACKET_KIND_SIZE = 1` — the first byte of every packet is its kind discriminant.
Both encoder and decoder callers are responsible for reading or writing this byte.
