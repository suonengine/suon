//! Checksum helpers used across the Suon workspace.
//!
//! At the moment this crate exposes a small Adler-32 wrapper type that keeps the
//! checksum API explicit and easy to pass around without losing formatting and
//! component helpers.

pub use adler32::Adler32Checksum;

mod adler32;
