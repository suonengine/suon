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
        fn serialize_duration_as_milliseconds() {
            // Define a constant test value in milliseconds
            const TEST_MILLIS: u64 = 1234;
            const VALUE: TestStruct = TestStruct {
                duration: Duration::from_millis(TEST_MILLIS),
            };

            // Serialize the struct to JSON
            let serialized = serde_json::to_string(&VALUE).unwrap();

            // Ensure the JSON string contains the correct millisecond value
            assert_eq!(serialized, format!(r#"{{"duration":{}}}"#, TEST_MILLIS));
        }

        #[test]
        fn deserialize_duration_from_milliseconds() {
            // Define a constant JSON input value
            const JSON_MILLIS: u64 = 5678;

            let json_input = format!(r#"{{"duration":{}}}"#, JSON_MILLIS);

            // Deserialize the JSON into a TestStruct
            let result: TestStruct = serde_json::from_str(&json_input).unwrap();

            // Verify the deserialized duration matches the expected value
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
        fn serialize_duration_as_seconds() {
            // Define a constant test value in seconds
            const TEST_SECS: u64 = 42;
            const VALUE: TestStruct = TestStruct {
                duration: Duration::from_secs(TEST_SECS),
            };

            // Serialize the struct to JSON
            let serialized = serde_json::to_string(&VALUE).unwrap();

            // Ensure the JSON string contains the correct second value
            assert_eq!(serialized, format!(r#"{{"duration":{}}}"#, TEST_SECS));
        }

        #[test]
        fn deserialize_duration_from_seconds() {
            // Define a constant JSON input value in seconds
            const JSON_SECS: u64 = 99;

            let json_input: String = format!(r#"{{"duration":{}}}"#, JSON_SECS);

            // Deserialize the JSON into a TestStruct
            let result: TestStruct = serde_json::from_str(&json_input).unwrap();

            // Verify the deserialized duration matches the expected value
            assert_eq!(result.duration, Duration::from_secs(JSON_SECS));
        }
    }
}
