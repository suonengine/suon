//! Workspace benchmark crate for Suon.
//!
//! This crate exists only to host Criterion benchmarks for the workspace crates.
//! The benchmark entry points live under `benches/` and exercise each crate in
//! isolation so performance regressions can be caught without publishing any
//! runtime library API from this package.

/// Returns the benchmark package name used across the workspace.
pub const fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_expose_workspace_benchmark_crate_name() {
        assert_eq!(
            crate_name(),
            "suon_benches",
            "The benchmark support crate name should stay stable for workspace tooling"
        );
    }
}
