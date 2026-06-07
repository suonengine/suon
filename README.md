<!-- Badges -->
[![Rust](https://img.shields.io/badge/Rust-nightly-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Lua](https://img.shields.io/badge/Lua-5.5-%23000080.svg?style=for-the-badge&logo=lua&logoColor=white)](https://lua.org/)
[![MIT](https://img.shields.io/badge/license-MIT%2FApache--2.0-white.svg?style=for-the-badge)](https://github.com/suonengine/suon#license)

[![Build](https://img.shields.io/github/actions/workflow/status/suonengine/suon/build.yml?style=for-the-badge&label=build)](https://github.com/suonengine/suon/actions/workflows/build.yml)
[![Test](https://img.shields.io/github/actions/workflow/status/suonengine/suon/test.yml?style=for-the-badge&label=test)](https://github.com/suonengine/suon/actions/workflows/test.yml)
[![Benchmarks](https://img.shields.io/github/actions/workflow/status/suonengine/suon/bench.yml?style=for-the-badge&label=benchmarks)](https://github.com/suonengine/suon/actions/workflows/bench.yml)
[![Docs](https://img.shields.io/badge/docs-stable-%23DAA520.svg?style=for-the-badge)](https://docs.rs/suon)
[![Format](https://img.shields.io/github/actions/workflow/status/suonengine/suon/format.yml?style=for-the-badge&label=format)](https://github.com/suonengine/suon/actions/workflows/format.yml)
[![Lint](https://img.shields.io/github/actions/workflow/status/suonengine/suon/lint.yml?style=for-the-badge&label=lint)](https://github.com/suonengine/suon/actions/workflows/lint.yml)
[![Typos](https://img.shields.io/github/actions/workflow/status/suonengine/suon/typos.yml?style=for-the-badge&label=typos)](https://github.com/suonengine/suon/actions/workflows/typos.yml)
[![Toml](https://img.shields.io/github/actions/workflow/status/suonengine/suon/toml.yml?style=for-the-badge&label=toml)](https://github.com/suonengine/suon/actions/workflows/toml.yml)

[![Crates.io](https://img.shields.io/badge/Crates.io-%23E16B31.svg?style=for-the-badge&logo=rust&logoColor=white)](https://crates.io/crates/suon)
[![Downloads](https://img.shields.io/crates/d/suon.svg?style=for-the-badge)](https://crates.io/crates/suon)
[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/invite/EKhxe6gjQt)

## What is Suon?

An MMORPG server written in Rust with Lua scripting, built as a modular
workspace.

## Getting Started

### Prerequisites

- **Rust nightly**. Install via [rustup](https://rustup.rs/):

  ```sh
  rustup toolchain install nightly
  ```
- **Lua 5.5**

### Run

```sh
git clone https://github.com/suonengine/suon.git
cd suon
cargo run
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
