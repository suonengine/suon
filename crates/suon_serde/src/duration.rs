//! Duration serialization adapters.

/// Module for serializing and deserializing `Duration` as milliseconds.
pub mod as_millis {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    /// Serializes a `Duration` as milliseconds (u64).
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis = duration.as_millis() as u64;
        serializer.serialize_u64(millis)
    }

    /// Deserializes a `Duration` from milliseconds (u64).
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

/// Module for serializing and deserializing `Duration` as seconds.
pub mod as_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    /// Serializes a `Duration` as seconds (u64).
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let secs = duration.as_secs();
        serializer.serialize_u64(secs)
    }

    /// Deserializes a `Duration` from seconds (u64).
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
#[cfg(test)]
mod tests {
    use crate::duration::{as_millis, as_secs};
    use serde::{Deserialize, Serialize};
    use std::time::Duration;

    mod millis {
        use super::super::as_millis;
        use serde::{Deserialize, Serialize};
        use serde_json;
        use std::time::Duration;

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            #[serde(with = "as_millis")]
            duration: Duration,
        }

        #[test]
        fn should_serialize_duration_as_milliseconds() {
            const TEST_MILLIS: u64 = 1234;
            const VALUE: TestStruct = TestStruct {
                duration: Duration::from_millis(TEST_MILLIS),
            };

            let serialized = serde_json::to_string(&VALUE).unwrap();

            assert_eq!(serialized, format!(r#"{{"duration":{}}}"#, TEST_MILLIS));
        }

        #[test]
        fn should_deserialize_duration_from_milliseconds() {
            const JSON_MILLIS: u64 = 5678;

            let json_input = format!(r#"{{"duration":{}}}"#, JSON_MILLIS);
            let result: TestStruct = serde_json::from_str(&json_input).unwrap();

            assert_eq!(result.duration, Duration::from_millis(JSON_MILLIS));
        }
    }

    mod secs {
        use super::super::as_secs;
        use serde::{Deserialize, Serialize};
        use serde_json;
        use std::time::Duration;

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct TestStruct {
            #[serde(with = "as_secs")]
            duration: Duration,
        }

        #[test]
        fn should_serialize_duration_as_seconds() {
            const TEST_SECS: u64 = 42;
            const VALUE: TestStruct = TestStruct {
                duration: Duration::from_secs(TEST_SECS),
            };

            let serialized = serde_json::to_string(&VALUE).unwrap();

            assert_eq!(serialized, format!(r#"{{"duration":{}}}"#, TEST_SECS));
        }

        #[test]
        fn should_deserialize_duration_from_seconds() {
            const JSON_SECS: u64 = 99;

            let json_input: String = format!(r#"{{"duration":{}}}"#, JSON_SECS);
            let result: TestStruct = serde_json::from_str(&json_input).unwrap();

            assert_eq!(result.duration, Duration::from_secs(JSON_SECS));
        }
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct MillisContainer {
        #[serde(with = "as_millis")]
        duration: Duration,
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct SecsContainer {
        #[serde(with = "as_secs")]
        duration: Duration,
    }

    #[test]
    fn millis_serialization_truncates_sub_millisecond_precision() {
        let value = MillisContainer {
            duration: Duration::from_nanos(1_999_999),
        };

        let serialized = serde_json::to_string(&value).expect("Serialization should succeed");

        assert_eq!(
            serialized, r#"{"duration":1}"#,
            "as_millis should truncate sub-millisecond precision during serialization"
        );
    }

    #[test]
    fn secs_serialization_truncates_subsecond_precision() {
        let value = SecsContainer {
            duration: Duration::from_millis(1_999),
        };

        let serialized = serde_json::to_string(&value).expect("Serialization should succeed");

        assert_eq!(
            serialized, r#"{"duration":1}"#,
            "as_secs should truncate subsecond precision during serialization"
        );
    }

    #[test]
    fn duration_deserialization_rejects_invalid_json_type() {
        let millis_error =
            serde_json::from_str::<MillisContainer>(r#"{"duration":"fast"}"#).unwrap_err();
        let secs_error =
            serde_json::from_str::<SecsContainer>(r#"{"duration":"slow"}"#).unwrap_err();

        assert!(
            !millis_error.to_string().is_empty(),
            "Millis deserialization should fail for non-numeric JSON values"
        );

        assert!(
            !secs_error.to_string().is_empty(),
            "Seconds deserialization should fail for non-numeric JSON values"
        );
    }
}
