# suon_xtea

XTEA cipher implementation for the Suon MMORPG framework.

`suon_xtea` provides:

- `expand_key` — derive round keys from a 128-bit XTEA key
- `encrypt` — encrypt a packet payload using expanded round keys
- `decrypt` — decrypt a packet payload, returning `XTEADecryptError` on failure
- `XTEAKey = [u32; 4]` — a 128-bit key as four 32-bit words

## Installation

```toml
[dependencies]
suon_xtea = { path = "../suon_xtea" }
```

## Quick Start

```rust,ignore
use suon_xtea::prelude::*;

let key: XTEAKey = [0xDEAD_BEEF, 0xCAFE_BABE, 0x1234_5678, 0xABCD_EF01];

// Encryption
let plaintext = b"hello world!"; // must be padded to a multiple of 8 bytes
let encrypted = encrypt(plaintext, &key);

// Decryption
match decrypt(&encrypted, &key) {
    Ok(decrypted) => println!("{:?}", &decrypted[..]),
    Err(e) => eprintln!("Decrypt failed: {e}"),
}
```

## How It Works

XTEA (eXtended TEA) is a 64-bit block cipher with a 128-bit key operating over 32 rounds.
Suon uses it to encrypt the payload of game packets after the initial XTEA handshake.

### Wire Format

Every encrypted buffer carries a two-byte **little-endian length prefix** followed by the
ciphertext:

```
[ length: u16 LE ] [ ciphertext: (length) bytes ]
```

The payload must be a multiple of 8 bytes (`XTEA_BLOCK_SIZE`). Callers are responsible for
adding padding before encryption and stripping it after decryption.

### Algorithm Constants

| Constant | Value | Description |
|---|---|---|
| `XTEA_DELTA` | `0x9E3779B9` | Golden-ratio-derived round constant |
| `XTEA_NUM_ROUNDS` | `32` | Number of Feistel rounds |
| `XTEA_BLOCK_SIZE` | `8` | Block size in bytes |

### Key Expansion

```rust,ignore
let round_keys = expand_key(&key);
// round_keys can be cached and reused across multiple encrypt/decrypt calls.
```

Pre-expanding the key avoids repeated work when processing many packets per tick.
