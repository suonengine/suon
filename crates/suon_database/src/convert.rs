//! Conversion helpers used by persistence mappers.
//!
//! These extension traits keep row-to-domain conversions small while attaching
//! field names to overflow and epoch-related errors.

use anyhow::{Context, Result};
use std::time::SystemTime;

/// Converts signed database integers into unsigned domain values with field context.
pub trait I64DatabaseConvertExt {
    /// Converts an `i64` to `u16`, returning the field name on overflow.
    fn try_u16_field(self, field: &str) -> Result<u16>;

    /// Converts an `i64` to `u32`, returning the field name on overflow.
    fn try_u32_field(self, field: &str) -> Result<u32>;

    /// Converts an `i64` to `u64`, returning the field name on overflow.
    fn try_u64_field(self, field: &str) -> Result<u64>;
}

impl I64DatabaseConvertExt for i64 {
    fn try_u16_field(self, field: &str) -> Result<u16> {
        u16::try_from(self)
            .with_context(|| format!("Field '{field}' value {self} does not fit u16"))
    }

    fn try_u32_field(self, field: &str) -> Result<u32> {
        u32::try_from(self)
            .with_context(|| format!("Field '{field}' value {self} does not fit u32"))
    }

    fn try_u64_field(self, field: &str) -> Result<u64> {
        u64::try_from(self)
            .with_context(|| format!("Field '{field}' value {self} does not fit u64"))
    }
}

pub trait U64DatabaseConvertExt {
    /// Converts a `u64` to `i64`, returning the field name on overflow.
    fn try_i64_field(self, field: &str) -> Result<i64>;
}

impl U64DatabaseConvertExt for u64 {
    fn try_i64_field(self, field: &str) -> Result<i64> {
        i64::try_from(self)
            .with_context(|| format!("Field '{field}' value {self} does not fit i64"))
    }
}

pub trait SystemTimeDatabaseConvertExt {
    /// Converts a `SystemTime` to UNIX seconds stored as `i64`.
    fn try_i64_secs_field(self, field: &str) -> Result<i64>;
}

impl SystemTimeDatabaseConvertExt for SystemTime {
    fn try_i64_secs_field(self, field: &str) -> Result<i64> {
        self.duration_since(SystemTime::UNIX_EPOCH)
            .with_context(|| format!("Field '{field}' was before UNIX_EPOCH"))?
            .as_secs()
            .try_i64_field(field)
    }
}

#[cfg(test)]
mod tests {
    use super::{I64DatabaseConvertExt, SystemTimeDatabaseConvertExt, U64DatabaseConvertExt};
    use std::time::SystemTime;

    #[test]
    fn should_convert_i64_values_with_field_context() {
        assert_eq!(
            42_i64.try_u16_field("demo").unwrap(),
            42,
            "try_u16_field should preserve in-range i64 values"
        );

        assert_eq!(
            42_i64.try_u32_field("demo").unwrap(),
            42,
            "try_u32_field should preserve in-range i64 values"
        );

        assert_eq!(
            42_i64.try_u64_field("demo").unwrap(),
            42,
            "try_u64_field should preserve in-range i64 values"
        );
    }

    #[test]
    fn should_convert_u64_values_with_field_context() {
        assert_eq!(
            42_u64.try_i64_field("demo").unwrap(),
            42,
            "try_i64_field should preserve in-range u64 values"
        );
    }

    #[test]
    fn should_convert_system_time_to_unix_seconds() {
        assert_eq!(
            std::time::UNIX_EPOCH.try_i64_secs_field("demo").unwrap(),
            0,
            "try_i64_secs_field should map UNIX_EPOCH to zero seconds"
        );
    }

    #[test]
    fn should_fail_when_i64_to_unsigned_conversion_overflows() {
        let error = (-1_i64)
            .try_u16_field("demo")
            .expect_err("negative values should not convert to u16");

        assert!(
            error.to_string().contains("demo"),
            "Overflow errors should include the field name for easier debugging"
        );
    }

    #[test]
    fn should_fail_when_u64_to_i64_conversion_overflows() {
        let error = (i64::MAX as u64 + 1)
            .try_i64_field("demo")
            .expect_err("out-of-range u64 values should not convert to i64");

        assert!(
            error.to_string().contains("demo"),
            "Overflow errors should include the field name for easier debugging"
        );
    }

    #[test]
    fn should_fail_when_system_time_is_before_unix_epoch() {
        let error = SystemTime::UNIX_EPOCH
            .checked_sub(std::time::Duration::from_secs(1))
            .expect("UNIX_EPOCH minus one second should exist")
            .try_i64_secs_field("demo")
            .expect_err("times before the UNIX epoch should be rejected");

        assert!(
            error.to_string().contains("demo"),
            "Epoch conversion errors should include the field name for easier debugging"
        );
    }

    #[test]
    fn should_report_field_context_for_each_integer_conversion_error() {
        let u32_error = (-1_i64)
            .try_u32_field("health")
            .expect_err("negative values should not convert to u32");
        let u64_error = (-1_i64)
            .try_u64_field("gold")
            .expect_err("negative values should not convert to u64");

        assert!(
            u32_error.to_string().contains("health"),
            "try_u32_field should include the field name in overflow errors"
        );

        assert!(
            u64_error.to_string().contains("gold"),
            "try_u64_field should include the field name in overflow errors"
        );
    }
}
