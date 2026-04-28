//! Conversion helpers used by tables that bridge domain and Diesel types.
//!
//! These extension traits keep row-to-domain conversions small while attaching
//! field names to overflow and epoch-related errors.

use std::time::SystemTime;

use anyhow::{Context, Result};

/// Converts one field value into another type while attaching field context.
pub trait FieldTryIntoExt: Sized + Copy + std::fmt::Display {
    /// Converts the current value into `T`, returning the field name on overflow.
    fn try_field<T>(self, field: &str) -> Result<T>
    where
        T: TryFrom<Self>,
        <T as TryFrom<Self>>::Error: std::error::Error + Send + Sync + 'static;
}

impl<S> FieldTryIntoExt for S
where
    S: Sized + Copy + std::fmt::Display,
{
    fn try_field<T>(self, field: &str) -> Result<T>
    where
        T: TryFrom<Self>,
        <T as TryFrom<Self>>::Error: std::error::Error + Send + Sync + 'static,
    {
        T::try_from(self)
            .with_context(|| format!("Field '{field}' value {self} does not fit the target type"))
    }
}

/// Converts a [`SystemTime`] field into UNIX seconds with field context.
pub trait SystemTimeDbExt {
    /// Converts a [`SystemTime`] to UNIX seconds stored as `i64`.
    fn try_i64_secs_field(self, field: &str) -> Result<i64>;
}

impl SystemTimeDbExt for SystemTime {
    fn try_i64_secs_field(self, field: &str) -> Result<i64> {
        self.duration_since(SystemTime::UNIX_EPOCH)
            .with_context(|| format!("Field '{field}' was before UNIX_EPOCH"))?
            .as_secs()
            .try_field(field)
    }
}

#[cfg(test)]
mod tests {
    use super::{FieldTryIntoExt, SystemTimeDbExt};
    use std::time::SystemTime;

    #[test]
    fn should_convert_i64_values_with_field_context() {
        assert_eq!(42_i64.try_field::<u16>("demo").unwrap(), 42);
        assert_eq!(42_i64.try_field::<u32>("demo").unwrap(), 42);
        assert_eq!(42_i64.try_field::<u64>("demo").unwrap(), 42);
    }

    #[test]
    fn should_convert_u64_values_with_field_context() {
        assert_eq!(42_u64.try_field::<i64>("demo").unwrap(), 42);
    }

    #[test]
    fn should_convert_system_time_to_unix_seconds() {
        assert_eq!(std::time::UNIX_EPOCH.try_i64_secs_field("demo").unwrap(), 0);
    }

    #[test]
    fn should_fail_when_i64_to_unsigned_conversion_overflows() {
        let error = (-1_i64)
            .try_field::<u16>("demo")
            .expect_err("negative values should not convert to u16");

        assert!(error.to_string().contains("demo"));
    }

    #[test]
    fn should_fail_when_u64_to_i64_conversion_overflows() {
        let error = (i64::MAX as u64 + 1)
            .try_field::<i64>("demo")
            .expect_err("out-of-range u64 values should not convert to i64");

        assert!(error.to_string().contains("demo"));
    }

    #[test]
    fn should_fail_when_system_time_is_before_unix_epoch() {
        let error = SystemTime::UNIX_EPOCH
            .checked_sub(std::time::Duration::from_secs(1))
            .expect("UNIX_EPOCH minus one second should exist")
            .try_i64_secs_field("demo")
            .expect_err("times before the UNIX epoch should be rejected");

        assert!(error.to_string().contains("demo"));
    }

    #[test]
    fn should_report_field_context_for_each_integer_conversion_error() {
        let u32_error = (-1_i64)
            .try_field::<u32>("health")
            .expect_err("negative values should not convert to u32");

        let u64_error = (-1_i64)
            .try_field::<u64>("gold")
            .expect_err("negative values should not convert to u64");

        assert!(u32_error.to_string().contains("health"));
        assert!(u64_error.to_string().contains("gold"));
    }
}
