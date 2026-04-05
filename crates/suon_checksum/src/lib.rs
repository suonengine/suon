//! Checksum helpers used across the Suon workspace.
//!
//! At the moment this crate exposes a small Adler-32 wrapper type that keeps the
//! checksum API explicit and easy to pass around without losing formatting and
//! component helpers.
//!
//! # Examples
//! ```
//! use suon_checksum::Adler32Checksum;
//!
//! let checksum = Adler32Checksum::calculate(b"hello");
//!
//! assert_eq!(checksum.components(), (0x0215, 0x062C));
//! assert_eq!(checksum.to_string(), "062C0215");
//! assert!(!checksum.is_initial());
//! ```

pub use adler32::Adler32Checksum;

mod adler32;

#[cfg(test)]
mod tests {
    use super::Adler32Checksum;

    #[test]
    fn should_reexport_adler32_checksum_from_crate_root() {
        let checksum = Adler32Checksum::calculate(b"root-export");

        assert_eq!(
            checksum,
            Adler32Checksum::from(b"root-export".as_slice()),
            "The crate root should re-export Adler32Checksum for direct use"
        );
    }
}
