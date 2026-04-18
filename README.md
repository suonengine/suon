<!-- Badges -->
[![Build](https://github.com/suonengine/suon/actions/workflows/build.yml/badge.svg)](https://github.com/suonengine/suon/actions/workflows/build.yml)
[![Test](https://github.com/suonengine/suon/actions/workflows/test.yml/badge.svg)](https://github.com/suonengine/suon/actions/workflows/test.yml)
[![Format](https://github.com/suonengine/suon/actions/workflows/format.yml/badge.svg)](https://github.com/suonengine/suon/actions/workflows/format.yml)
[![Lint](https://github.com/suonengine/suon/actions/workflows/lint.yml/badge.svg)](https://github.com/suonengine/suon/actions/workflows/lint.yml)
[![Docs](https://img.shields.io/badge/docs-rustdoc-blue.svg)](https://docs.rs/suon)
[![Benchmarks](https://github.com/suonengine/suon/actions/workflows/bench.yml/badge.svg)](https://github.com/suonengine/suon/actions/workflows/bench.yml)
[![Typos](https://github.com/suonengine/suon/actions/workflows/typos.yml/badge.svg)](https://github.com/suonengine/suon/actions/workflows/typos.yml)
[![Toml](https://github.com/suonengine/suon/actions/workflows/toml.yml/badge.svg)](https://github.com/suonengine/suon/actions/workflows/toml.yml)
[![Rust](https://img.shields.io/badge/Rust-1.85%2B-000000?logo=rust)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/Bevy-0.18-232326)](https://bevyengine.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](./LICENSE-MIT)

## What is Suon?

Suon is an MMORPG server framework in Rust.

## Getting Started

Clone the repository and run the example server:

```sh
git clone https://github.com/suonengine/suon.git
cd suon

cargo run --example server
```

To bootstrap a minimal server app:

```rust
use bevy::prelude::*;
use suon::prelude::*;

fn main() {
    App::new()
        .add_plugins(SuonPlugin)
        .run();
}
```

## License

Suon is free, open source, and dual-licensed under either:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

### Your contributions

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
