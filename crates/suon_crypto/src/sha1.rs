use hmac::{Hmac, KeyInit, Mac};
use sha1::{Digest, Sha1};
use thiserror::Error;

/// SHA-1 digest bytes.
pub type Sha1Digest = [u8; 20];

/// Error returned when an OTP code cannot be generated from the requested settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OtpError {
    #[error("OTP digits must be between 1 and 9, got {digits}")]
    InvalidDigits { digits: usize },

    #[error("TOTP step duration must be greater than zero")]
    ZeroStep,

    #[error("HMAC key was rejected")]
    InvalidKey,
}

/// Computes a SHA-1 digest.
///
/// SHA-1 is provided for compatibility with legacy protocols and OTP standards.
/// Do not use it for new collision-resistant hashing.
///
/// # Examples
/// ```rust
/// use suon_crypto::prelude::*;
///
/// assert_eq!(sha1("abc").len(), 20);
/// ```
pub fn sha1(input: impl AsRef<[u8]>) -> Sha1Digest {
    Sha1::digest(input.as_ref()).into()
}

/// Computes a lowercase hexadecimal SHA-1 digest.
///
/// # Examples
/// ```rust
/// use suon_crypto::prelude::*;
///
/// assert_eq!(
///     sha1_hex("abc"),
///     "a9993e364706816aba3e25717850c26c9cd0d89d"
/// );
/// ```
pub fn sha1_hex(input: impl AsRef<[u8]>) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let bytes = sha1(input);
    let mut out = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        out.push(char::from(HEX[usize::from(byte >> 4)]));
        out.push(char::from(HEX[usize::from(byte & 0x0F)]));
    }

    out
}

/// Computes HMAC-SHA1.
///
/// # Examples
/// ```rust
/// use suon_crypto::prelude::*;
///
/// let digest = hmac_sha1("key", "message").unwrap();
///
/// assert_eq!(digest.len(), 20);
/// ```
pub fn hmac_sha1(
    key: impl AsRef<[u8]>,
    message: impl AsRef<[u8]>,
) -> Result<Sha1Digest, hmac::digest::InvalidLength> {
    let mut mac = Hmac::<Sha1>::new_from_slice(key.as_ref())?;
    mac.update(message.as_ref());

    Ok(mac.finalize().into_bytes().into())
}

/// Generates an HOTP code with HMAC-SHA1.
///
/// `digits` must be between 1 and 9.
///
/// # Examples
/// ```rust
/// use suon_crypto::prelude::*;
///
/// let code = hotp_sha1(b"12345678901234567890", 0, 6).unwrap();
///
/// assert_eq!(code, "755224");
/// ```
pub fn hotp_sha1(key: impl AsRef<[u8]>, counter: u64, digits: usize) -> Result<String, OtpError> {
    if !(1..=9).contains(&digits) {
        return Err(OtpError::InvalidDigits { digits });
    }

    let digest = hmac_sha1(key, counter.to_be_bytes()).map_err(|_| OtpError::InvalidKey)?;

    let offset = usize::from(digest[digest.len() - 1] & 0x0F);
    let code = (u32::from(digest[offset] & 0x7F) << 24)
        | (u32::from(digest[offset + 1]) << 16)
        | (u32::from(digest[offset + 2]) << 8)
        | u32::from(digest[offset + 3]);

    Ok(format!("{:0digits$}", code % 10_u32.pow(digits as u32)))
}

/// Generates a TOTP code with HMAC-SHA1.
///
/// `step_seconds` must be greater than zero and `digits` must be between 1 and 9.
///
/// # Examples
/// ```rust
/// use suon_crypto::prelude::*;
///
/// let code = totp_sha1(b"12345678901234567890", 59, 30, 8).unwrap();
///
/// assert_eq!(code, "94287082");
/// ```
pub fn totp_sha1(
    key: impl AsRef<[u8]>,
    unix_seconds: u64,
    step_seconds: u64,
    digits: usize,
) -> Result<String, OtpError> {
    if step_seconds == 0 {
        return Err(OtpError::ZeroStep);
    }

    hotp_sha1(key, unix_seconds / step_seconds, digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RFC_4226_KEY: &[u8] = b"12345678901234567890";

    #[test]
    fn should_compute_sha1_and_hmac() {
        assert_eq!(sha1_hex("abc"), "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(sha1_hex(""), "da39a3ee5e6b4b0d3255bfef95601890afd80709");

        assert_eq!(
            hmac_sha1("key", "The quick brown fox jumps over the lazy dog").unwrap(),
            [
                0xDE, 0x7C, 0x9B, 0x85, 0xB8, 0xB7, 0x8A, 0xA6, 0xBC, 0x8A, 0x7A, 0x36, 0xF7, 0x0A,
                0x90, 0x70, 0x1C, 0x9D, 0xB4, 0xD9
            ]
        );
    }

    #[test]
    fn should_generate_hotp_from_rfc_4226_vectors() {
        let expected = [
            "755224", "287082", "359152", "969429", "338314", "254676", "287922", "162583",
            "399871", "520489",
        ];

        for (counter, expected) in expected.into_iter().enumerate() {
            assert_eq!(
                hotp_sha1(RFC_4226_KEY, counter as u64, 6).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn should_generate_totp_from_rfc_6238_vectors() {
        let vectors = [
            (59u64, "94287082"),
            (1_111_111_109, "07081804"),
            (1_111_111_111, "14050471"),
            (1_234_567_890, "89005924"),
            (2_000_000_000, "69279037"),
            (20_000_000_000, "65353130"),
        ];

        for (unix_seconds, expected) in vectors {
            assert_eq!(
                totp_sha1(RFC_4226_KEY, unix_seconds, 30, 8).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn should_reject_invalid_otp_settings() {
        assert_eq!(
            hotp_sha1(RFC_4226_KEY, 0, 0),
            Err(OtpError::InvalidDigits { digits: 0 })
        );

        assert_eq!(
            hotp_sha1(RFC_4226_KEY, 0, 10),
            Err(OtpError::InvalidDigits { digits: 10 })
        );

        assert_eq!(totp_sha1(RFC_4226_KEY, 59, 0, 6), Err(OtpError::ZeroStep));
    }

    #[test]
    fn should_format_otp_errors() {
        assert_eq!(
            OtpError::InvalidDigits { digits: 10 }.to_string(),
            "OTP digits must be between 1 and 9, got 10"
        );
        assert_eq!(
            OtpError::ZeroStep.to_string(),
            "TOTP step duration must be greater than zero"
        );
    }
}
