//! Serde helpers shared across the Suon workspace.
//!
//! The modules in this crate provide focused serialization adapters that can be
//! reused from multiple configuration and protocol crates without repeating
//! serde glue code.

pub mod duration;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    #[test]
    fn should_expose_duration_module_from_crate_root() {
        let serialized = crate::duration::as_secs::serialize(
            &Duration::from_secs(5),
            serde_json::value::Serializer,
        )
        .expect("Root-exported duration helpers should be callable");

        assert_eq!(
            serialized,
            serde_json::Value::from(5),
            "The crate root should expose the duration serde helpers"
        );
    }
}
