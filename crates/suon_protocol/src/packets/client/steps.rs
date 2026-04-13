//! Client multi-step packet.

use suon_position::direction::Direction;

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to request a multi-step path.
///
/// # Examples
/// ```
/// use suon_position::direction::Direction;
/// use suon_protocol::packets::client::{Decodable, PacketKind, prelude::Steps};
///
/// let mut payload: &[u8] = &[3, 1, 3, 5];
/// let packet = Steps::decode(PacketKind::Steps, &mut payload).unwrap();
///
/// assert_eq!(
///     packet.path,
///     vec![Direction::East, Direction::North, Direction::West]
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Steps {
    /// Ordered path requested by the client.
    pub path: Vec<Direction>,
}

impl Steps {
    fn direction_from_wire(value: u8) -> Option<Direction> {
        match value {
            1 => Some(Direction::East),
            2 => Some(Direction::NorthEast),
            3 => Some(Direction::North),
            4 => Some(Direction::NorthWest),
            5 => Some(Direction::West),
            6 => Some(Direction::SouthWest),
            7 => Some(Direction::South),
            8 => Some(Direction::SouthEast),
            _ => None,
        }
    }
}

impl Decodable for Steps {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let path_length = bytes.get_u8()?;
        if path_length == 0 {
            return Err(DecodableError::InvalidFieldValue {
                field: "path_length",
                value: 0,
            });
        }

        let mut path = Vec::with_capacity(path_length as usize);
        for _ in 0..path_length {
            let raw_direction = bytes.get_u8()?;
            if let Some(direction) = Self::direction_from_wire(raw_direction) {
                path.push(direction);
            }
        }

        if path.is_empty() {
            return Err(DecodableError::InvalidFieldValue {
                field: "path",
                value: 0,
            });
        }

        Ok(Self { path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_steps_path() {
        let mut payload: &[u8] = &[3, 1, 3, 5];

        let packet = Steps::decode(PacketKind::Steps, &mut payload)
            .expect("Steps packets should decode valid direction sequences");

        assert_eq!(
            packet.path,
            vec![Direction::East, Direction::North, Direction::West]
        );
        assert!(
            payload.is_empty(),
            "Steps decoding should consume the whole payload"
        );
    }

    #[test]
    fn should_ignore_unknown_directions_while_preserving_valid_steps() {
        let mut payload: &[u8] = &[4, 1, 0, 9, 7];

        let packet = Steps::decode(PacketKind::Steps, &mut payload)
            .expect("Steps packets should keep valid directions even when some bytes are unknown");

        assert_eq!(packet.path, vec![Direction::East, Direction::South]);
    }

    #[test]
    fn should_reject_empty_steps_paths() {
        let mut payload: &[u8] = &[0];

        let error = Steps::decode(PacketKind::Steps, &mut payload)
            .expect_err("Steps packets should reject zero-length paths");

        assert!(matches!(
            error,
            DecodableError::InvalidFieldValue {
                field: "path_length",
                value: 0
            }
        ));
    }
}
