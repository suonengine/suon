//! Client say packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Wire type discriminator that classifies the speech mode of a [`Say`] packet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeakClass {
    /// Normal local speech (wire value `1`).
    Say = 1,
    /// Low-range whisper visible only to nearby players (wire value `2`).
    Whisper = 2,
    /// Extended-range yell visible to distant players (wire value `3`).
    Yell = 3,
    /// Incoming private message received by the client (wire value `4`).
    PrivateFrom = 4,
    /// Outgoing private message sent to a named player; carries a `receiver` field (wire value `5`).
    PrivateTo = 5,
    /// Normal text posted to a named channel; carries a `channel_id` field (wire value `7`).
    ChannelYellow = 7,
    /// Orange-coloured text posted to a named channel; carries a `channel_id` field (wire value `8`).
    ChannelOrange = 8,
    /// Spell incantation text (wire value `9`).
    Spell = 9,
    /// NPC private message received by the client (wire value `10`).
    PrivateNpc = 10,
    /// NPC message shown in the server console (wire value `11`).
    PrivateNpcConsole = 11,
    /// Player-to-NPC private message sent by the client (wire value `12`).
    PrivatePlayerToNpc = 12,
    /// Server-wide broadcast message (wire value `13`).
    Broadcast = 13,
    /// Red-coloured channel message (wire value `14`).
    ChannelRed = 14,
    /// Incoming red private message (wire value `15`).
    PrivateRedFrom = 15,
    /// Outgoing red private message; carries a `receiver` field (wire value `16`).
    PrivateRedTo = 16,
    /// Monster normal speech (wire value `36`).
    MonsterSay = 36,
    /// Monster extended-range speech (wire value `37`).
    MonsterYell = 37,
    /// Potion use message (wire value `52`).
    Potion = 52,
}

impl TryFrom<u8> for SpeakClass {
    type Error = DecodableError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Say),
            2 => Ok(Self::Whisper),
            3 => Ok(Self::Yell),
            4 => Ok(Self::PrivateFrom),
            5 => Ok(Self::PrivateTo),
            7 => Ok(Self::ChannelYellow),
            8 => Ok(Self::ChannelOrange),
            9 => Ok(Self::Spell),
            10 => Ok(Self::PrivateNpc),
            11 => Ok(Self::PrivateNpcConsole),
            12 => Ok(Self::PrivatePlayerToNpc),
            13 => Ok(Self::Broadcast),
            14 => Ok(Self::ChannelRed),
            15 => Ok(Self::PrivateRedFrom),
            16 => Ok(Self::PrivateRedTo),
            36 => Ok(Self::MonsterSay),
            37 => Ok(Self::MonsterYell),
            52 => Ok(Self::Potion),
            _ => Err(DecodableError::InvalidFieldValue {
                field: "speech_kind",
                value,
            }),
        }
    }
}

/// Packet sent by the client to emit text through one of the protocol speech routes.
///
/// The payload starts with a [`SpeakClass`] discriminator and conditionally
/// includes either a receiver name or a channel id before the final message
/// text, depending on how the speech should be routed by the server.
///
/// # Examples
///
/// ```rust
/// use suon_protocol::packets::client::prelude::{Decodable, PacketKind, Say, SpeakClass};
///
/// let mut payload: &[u8] = &[7, 0x34, 0x12, 2, 0, b'h', b'i'];
/// let packet = Say::decode(PacketKind::Say, &mut payload).unwrap();
///
/// assert_eq!(packet.speech_kind, SpeakClass::ChannelYellow);
/// assert_eq!(packet.channel_id, Some(0x1234));
/// assert_eq!(packet.text, "hi");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Say {
    /// Speech-route discriminator that decides which optional fields are present.
    pub speech_kind: SpeakClass,
    /// Destination character name for private outgoing speech kinds; otherwise `None`.
    pub receiver: Option<String>,
    /// Destination channel id for channel speech kinds; otherwise `None`.
    pub channel_id: Option<u16>,
    /// Final text payload, spell formula, or chat message content.
    pub text: String,
}

impl Decodable for Say {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let speech_kind = bytes.get_u8()?.try_into()?;

        let (receiver, channel_id, text) = match speech_kind {
            SpeakClass::PrivateTo | SpeakClass::PrivateRedTo => {
                (Some(bytes.get_string()?), None, bytes.get_string()?)
            }
            SpeakClass::ChannelYellow | SpeakClass::ChannelRed => {
                (None, Some(bytes.get_u16()?), bytes.get_string()?)
            }
            _ => (None, None, bytes.get_string()?),
        };

        Ok(Self {
            speech_kind,
            receiver,
            channel_id,
            text,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_decode_plain_say() {
        let mut payload: &[u8] = &[1, 5, 0, b'h', b'e', b'l', b'l', b'o'];
        assert_eq!(
            Say::decode(PacketKind::Say, &mut payload).unwrap().text,
            "hello"
        );
    }

    #[test]
    fn should_decode_private_say() {
        let mut payload: &[u8] = &[5, 4, 0, b'J', b'o', b'h', b'n', 2, 0, b'h', b'i'];
        let packet = Say::decode(PacketKind::Say, &mut payload).unwrap();
        assert_eq!(packet.speech_kind, SpeakClass::PrivateTo);
        assert_eq!(packet.receiver.as_deref(), Some("John"));
        assert_eq!(packet.text, "hi");
    }

    #[test]
    fn should_decode_channel_say() {
        let mut payload: &[u8] = &[7, 0x34, 0x12, 2, 0, b'h', b'i'];
        let packet = Say::decode(PacketKind::Say, &mut payload).unwrap();
        assert_eq!(packet.speech_kind, SpeakClass::ChannelYellow);
        assert_eq!(packet.channel_id, Some(0x1234));
        assert_eq!(packet.text, "hi");
    }
}
