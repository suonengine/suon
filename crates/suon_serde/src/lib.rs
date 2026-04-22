//! Serde helpers shared across the Suon workspace.
//!
//! The modules in this crate provide focused serialization adapters that can be
//! reused from multiple configuration and protocol crates without repeating
//! serde glue code.
//!
//! # Examples
//! ```
//! use serde::{Deserialize, Serialize};
//! use std::time::Duration;
//! use suon_serde::prelude::*;
//!
//! #[derive(Serialize, Deserialize)]
//! struct Durations {
//!     #[serde(with = "as_millis")]
//!     retry_after: Duration,
//!     #[serde(with = "as_secs")]
//!     timeout: Duration,
//! }
//!
//! let json = serde_json::to_string(&Durations {
//!     retry_after: Duration::from_millis(1500),
//!     timeout: Duration::from_secs(3),
//! })
//! .unwrap();
//!
//! assert_eq!(json, r#"{"retry_after":1500,"timeout":3}"#);
//! ```

mod duration;

pub mod prelude {
    pub use crate::duration::{as_millis, as_secs};
}

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

    #[test]
    fn should_expose_duration_helpers_through_prelude() {
        use crate::prelude::*;

        let millis =
            as_millis::serialize(&Duration::from_millis(12), serde_json::value::Serializer)
                .expect("Prelude should expose the millisecond serializer");

        let secs = as_secs::serialize(&Duration::from_secs(3), serde_json::value::Serializer)
            .expect("Prelude should expose the second serializer");

        assert_eq!(millis, serde_json::Value::from(12));
        assert_eq!(secs, serde_json::Value::from(3));
    }
}
