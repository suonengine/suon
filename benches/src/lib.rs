//! Workspace benchmark crate for Suon.
//!
//! This crate exists only to host Criterion benchmarks for the workspace crates.
//! The benchmark entry points live under `benches/` and exercise each crate in
//! isolation so performance regressions can be caught without publishing any
//! runtime library API from this package.

/// Builds a benchmark identifier scoped by the calling module.
#[macro_export]
macro_rules! bench {
    ($name:literal) => {
        concat!(module_path!(), "::", $name)
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_scope_benchmark_name_to_calling_module() {
        assert_eq!(
            crate::bench!("decode"),
            "benches::tests::decode",
            "The benchmark helper should prefix names with the calling module path"
        );
    }
}
