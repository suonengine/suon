# suon_crypto

Cryptographic helpers for the Suon MMORPG framework.

`suon_crypto` provides:

- `sha1` - compute SHA-1 digest bytes
- `sha1_hex` - compute lowercase hexadecimal SHA-1 digests
- `hmac_sha1` - compute HMAC-SHA1 digests
- `hotp_sha1` - generate HOTP codes using HMAC-SHA1
- `totp_sha1` - generate TOTP codes using HMAC-SHA1
- `OtpError` - validation errors for OTP generation

## Installation

```toml
[dependencies]
suon_crypto = { path = "../suon_crypto" }
```

## Quick Start

```rust,ignore
use suon_crypto::prelude::*;

let digest = sha1("abc");
let hex = sha1_hex("abc");
let hmac = hmac_sha1("key", "message")?;

let hotp = hotp_sha1(b"12345678901234567890", 0, 6)?;
let totp = totp_sha1(b"12345678901234567890", 59, 30, 8)?;

assert_eq!(digest.len(), 20);
assert_eq!(hex, "a9993e364706816aba3e25717850c26c9cd0d89d");
assert_eq!(hmac.len(), 20);
assert_eq!(hotp, "755224");
assert_eq!(totp, "94287082");
```

## Security Notes

SHA-1 is included for legacy protocol compatibility and OTP standards. Do not use SHA-1
for new collision-resistant hashing. Prefer modern hash functions for new security-sensitive
designs.

HOTP and TOTP use HMAC-SHA1 as specified by RFC 4226 and RFC 6238. `digits` must be between
1 and 9, and TOTP `step_seconds` must be greater than zero.
