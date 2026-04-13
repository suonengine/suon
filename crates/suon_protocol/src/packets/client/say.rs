//! Client say packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeakClass {
    Say = 1,
    Whisper = 2,
    Yell = 3,
    PrivateFrom = 4,
    PrivateTo = 5,
    ChannelYellow = 7,
    ChannelOrange = 8,
    Spell = 9,
    PrivateNpc = 10,
    PrivateNpcConsole = 11,
    PrivatePlayerToNpc = 12,
    Broadcast = 13,
    ChannelRed = 14,
    PrivateRedFrom = 15,
    PrivateRedTo = 16,
    MonsterSay = 36,
    MonsterYell = 37,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Say {
    pub speech_kind: SpeakClass,
    pub receiver: Option<String>,
    pub channel_id: Option<u16>,
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
