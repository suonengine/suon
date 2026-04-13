//! Client login packet.

use crate::packets::decoder::{Decoder, DecoderError};
use thiserror::Error;

use super::prelude::*;

/// Second handshake packet sent by the client during login negotiation.
///
/// The wire packet contains a plain header followed by an RSA-encrypted login
/// block. That RSA block carries the negotiated XTEA key used only for packets
/// sent after login. This type preserves the original payload and exposes
/// higher-level decoding helpers for the layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Login {
    /// Raw login payload bytes after the `0x0A` packet kind.
    pub payload: Vec<u8>,
}

/// Decoded fixed header used by the login packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestLoginHeader {
    /// Client operating system identifier.
    pub operating_system: u16,

    /// Protocol version sent by the client.
    pub protocol_version: u16,

    /// Numeric client version.
    pub client_version: u32,

    /// Stringified client version sent by newer clients.
    pub client_version_string: String,

    /// Appearance hash string sent by 13.34+ clients.
    pub appearances_hash: String,

    /// Game preview-state flag.
    pub preview_state: u8,
}

/// Decrypted RSA block used by the login packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestLoginCredentials {
    /// First RSA byte, expected to be zero.
    pub leading_zero: u8,

    /// Negotiated XTEA key embedded in the RSA-protected login block.
    pub xtea_key: [u32; 4],

    /// Gamemaster flag sent by the client.
    pub is_game_master: u8,

    /// Session key string.
    pub session_key: String,

    /// Character name selected during login.
    pub character_name: String,

    /// Challenge timestamp echoed by the client.
    pub challenge_timestamp: u32,

    /// Challenge random byte echoed by the client.
    pub challenge_random: u8,

    /// Optional extra login data returned by Lua hooks.
    pub extended_data: Option<String>,
}

/// Fully decoded latest-version login packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestLogin {
    /// Plain, non-encrypted login header.
    pub header: LatestLoginHeader,

    /// Decoded RSA-protected credentials block.
    pub credentials: LatestLoginCredentials,
}

/// Abstraction used by `Login` to decrypt the RSA-protected login block.
///
/// The login packet does not carry an additional XTEA-encrypted
/// inner block. Instead, the decrypted RSA payload contains the XTEA key that
/// becomes active for packets sent after the login handshake completes.
pub trait LoginBlockDecoder {
    /// Decrypts the encrypted login block and returns the resulting plaintext.
    fn decode_login_block(&self, encrypted_block: &[u8]) -> Result<Vec<u8>, LoginDecodeError>;
}

/// Errors raised while decoding the higher-level login packet structure.
#[derive(Debug, Error)]
pub enum LoginDecodeError {
    /// Wraps lower-level decoder failures.
    #[error("failed to decode login packet: {0}")]
    Decoder(#[from] DecoderError),

    /// Returned when the encrypted block could not be decrypted.
    #[error("failed to decrypt login block: {message}")]
    RsaDecryption {
        /// Human-readable failure message.
        message: String,
    },

    /// Returned when the decrypted login block contains an unexpected marker.
    #[error("invalid value {value} for field '{field}'")]
    InvalidFieldValue {
        /// Logical field name being decoded.
        field: &'static str,
        /// Raw value received from the wire.
        value: u8,
    },
}

impl Login {
    /// Decodes the login layout using the provided RSA decoder.
    ///
    /// 1. plain header fields
    /// 2. RSA-encrypted block containing XTEA key and credentials
    pub fn decode_latest<D>(&self, block_decoder: &D) -> Result<LatestLogin, LoginDecodeError>
    where
        D: LoginBlockDecoder,
    {
        let mut bytes = self.payload.as_slice();

        let header = LatestLoginHeader {
            operating_system: (&mut bytes).get_u16()?,
            protocol_version: (&mut bytes).get_u16()?,
            client_version: (&mut bytes).get_u32()?,
            client_version_string: (&mut bytes).get_string()?,
            appearances_hash: (&mut bytes).get_string()?,
            preview_state: (&mut bytes).get_u8()?,
        };

        let encrypted_block = bytes;
        let decrypted_block = block_decoder.decode_login_block(encrypted_block)?;
        let mut decrypted = decrypted_block.as_slice();

        let leading_zero = (&mut decrypted).get_u8()?;
        if leading_zero != 0 {
            return Err(LoginDecodeError::InvalidFieldValue {
                field: "leading_zero",
                value: leading_zero,
            });
        }

        let credentials = LatestLoginCredentials {
            leading_zero,
            xtea_key: [
                (&mut decrypted).get_u32()?,
                (&mut decrypted).get_u32()?,
                (&mut decrypted).get_u32()?,
                (&mut decrypted).get_u32()?,
            ],
            is_game_master: (&mut decrypted).get_u8()?,
            session_key: (&mut decrypted).get_string()?,
            character_name: (&mut decrypted).get_string()?,
            challenge_timestamp: (&mut decrypted).get_u32()?,
            challenge_random: (&mut decrypted).get_u8()?,
            extended_data: if decrypted.is_empty() {
                None
            } else {
                Some((&mut decrypted).get_string()?)
            },
        };

        Ok(LatestLogin {
            header,
            credentials,
        })
    }
}

impl Decodable for Login {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            payload: bytes.take_remaining().to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PassthroughBlockDecoder;

    impl LoginBlockDecoder for PassthroughBlockDecoder {
        fn decode_login_block(&self, encrypted_block: &[u8]) -> Result<Vec<u8>, LoginDecodeError> {
            Ok(encrypted_block.to_vec())
        }
    }

    #[test]
    fn should_decode_login_as_raw_payload() {
        let mut payload: &[u8] = &[1, 2, 3, 4];

        let packet = Login::decode(PacketKind::Login, &mut payload)
            .expect("Login packets should preserve raw payload");

        assert_eq!(packet.payload, vec![1, 2, 3, 4]);
        assert!(payload.is_empty());
    }

    #[test]
    fn should_decode_latest_login_layout_from_the_packet() {
        let mut payload: &[u8] = &[
            0x34, 0x12, 0x78, 0x56, 0xEF, 0xCD, 0xAB, 0x90, 4, 0, b'1', b'4', b'.', b'0', 6, 0,
            b'h', b'a', b's', b'h', b'e', b'd', 1, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0,
            0, 0, 3, 0, b'k', b'e', b'y', 4, 0, b'N', b'a', b'm', b'e', 0x44, 0x33, 0x22, 0x11, 9,
            5, 0, b'e', b'x', b't', b'r', b'a',
        ];

        let packet = Login::decode(PacketKind::Login, &mut payload)
            .expect("Login packets should preserve the wire payload before contextual decoding");

        let decoded = packet
            .decode_latest(&PassthroughBlockDecoder)
            .expect("decode_latest should parse the login layout");

        assert_eq!(decoded.header.operating_system, 0x1234);
        assert_eq!(decoded.header.protocol_version, 0x5678);
        assert_eq!(decoded.header.client_version, 0x90ABCDEF);
        assert_eq!(decoded.header.client_version_string, "14.0");
        assert_eq!(decoded.header.appearances_hash, "hashed");
        assert_eq!(decoded.header.preview_state, 1);
        assert_eq!(decoded.credentials.xtea_key, [1, 2, 3, 4]);
        assert_eq!(decoded.credentials.session_key, "key");
        assert_eq!(decoded.credentials.character_name, "Name");
        assert_eq!(decoded.credentials.challenge_timestamp, 0x11223344);
        assert_eq!(decoded.credentials.challenge_random, 9);
        assert_eq!(decoded.credentials.extended_data.as_deref(), Some("extra"));
    }

    #[test]
    fn should_reject_non_zero_leading_rsa_byte() {
        let packet = Login {
            payload: vec![
                1, 0, 2, 0, 3, 0, 0, 0, 1, 0, b'v', 1, 0, b'h', 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
        };

        let error = packet
            .decode_latest(&PassthroughBlockDecoder)
            .expect_err("decode_latest should reject unexpected leading RSA markers");

        assert!(matches!(
            error,
            LoginDecodeError::InvalidFieldValue {
                field: "leading_zero",
                value: 1,
            }
        ));
    }
}
