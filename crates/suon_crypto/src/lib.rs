//! Shared cryptographic helpers for Suon systems.

mod sha1;

pub mod prelude {
    pub use crate::sha1::{OtpError, Sha1Digest, hmac_sha1, hotp_sha1, sha1, sha1_hex, totp_sha1};
}
