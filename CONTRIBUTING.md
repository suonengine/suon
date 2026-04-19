# Contributing to Suon

Thanks for helping improve Suon.

## Before you start

- Search existing issues and pull requests before opening a new one.
- For larger changes, open an issue first so we can align on scope and direction.
- Keep pull requests focused. Small, reviewable changes are easier to merge.

## Development setup

```sh
git clone https://github.com/suonengine/suon.git
cd suon
cargo run --example server
```

## Recommended checks

Run the relevant checks before opening a pull request:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
taplo fmt --check --diff
```

If your change affects a specific crate, example, benchmark, or workflow, please run the checks most relevant to that area as well.

## Style

- Follow the existing Rust style and naming conventions in the workspace.
- Prefer clear APIs, targeted docs, and focused commits.
- Add or update tests when behavior changes.
- Keep comments useful and concise.

## Pull requests

Please include:

- A short description of the problem and solution
- Notes about any tradeoffs or follow-up work
- Testing details describing what you ran

By submitting a contribution, you agree that it may be licensed under the repository's dual-license terms: `MIT OR Apache-2.0`.
