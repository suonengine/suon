<!-- Badges -->
[![Rust](https://img.shields.io/badge/Rust-nightly-DAA520?logo=rust)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/Bevy-0.18-white?logo=Bevy)](https://bevyengine.org/)
[![MIT](https://img.shields.io/badge/license-MIT%2FApache--2.0-white.svg)](https://github.com/suonengine/suon#license)


[![Build](https://img.shields.io/github/actions/workflow/status/suonengine/suon/build.yml?label=build)](https://github.com/suonengine/suon/actions/workflows/build.yml)
[![Test](https://img.shields.io/github/actions/workflow/status/suonengine/suon/test.yml?label=test)](https://github.com/suonengine/suon/actions/workflows/test.yml)
[![Benchmarks](https://img.shields.io/github/actions/workflow/status/suonengine/suon/bench.yml?label=benchmarks)](https://github.com/suonengine/suon/actions/workflows/bench.yml)
[![Docs](https://docs.rs/suon/badge.svg)](https://docs.rs/suon)
[![Format](https://img.shields.io/github/actions/workflow/status/suonengine/suon/format.yml?label=format)](https://github.com/suonengine/suon/actions/workflows/format.yml)
[![Lint](https://img.shields.io/github/actions/workflow/status/suonengine/suon/lint.yml?label=lint)](https://github.com/suonengine/suon/actions/workflows/lint.yml)
[![Typos](https://img.shields.io/github/actions/workflow/status/suonengine/suon/typos.yml?label=typos)](https://github.com/suonengine/suon/actions/workflows/typos.yml)
[![Toml](https://img.shields.io/github/actions/workflow/status/suonengine/suon/toml.yml?label=toml)](https://github.com/suonengine/suon/actions/workflows/toml.yml)

[![Crates.io](https://img.shields.io/crates/v/suon.svg)](https://crates.io/crates/suon)
[![Downloads](https://img.shields.io/crates/d/suon.svg)](https://crates.io/crates/suon)
[![Discord](https://img.shields.io/discord/1417650719888248868.svg?label=&logo=discord&logoColor=white&color=DAA520&labelColor=DAA520)](https://discord.com/invite/EKhxe6gjQt)

## What is Suon?

An MMORPG server framework in Rust.

## Getting Started

Clone the repository and run the example server:

```sh
git clone https://github.com/suonengine/suon.git
cd suon

cargo run --example server
```

To bootstrap a minimal server app:

```rust
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
