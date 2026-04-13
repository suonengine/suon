//! Client exiva-restrictions packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Packet sent by the client to update exiva restrictions in no-pvp worlds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExivaRestrictions {
    /// Whether all players should be allowed by default.
    pub allow_all: bool,

    /// Whether players from the same guild are allowed.
    pub allow_own_guild: bool,

    /// Whether players from the same party are allowed.
    pub allow_own_party: bool,

    /// Whether the VIP list is allowed.
    pub allow_vip_list: bool,

    /// Whether the player whitelist feature is enabled.
    pub allow_player_whitelist: bool,

    /// Whether the guild whitelist feature is enabled.
    pub allow_guild_whitelist: bool,

    /// Player names to add to the whitelist.
    pub added_player_names: Vec<String>,

    /// Player names to remove from the whitelist.
    pub removed_player_names: Vec<String>,

    /// Guild names to add to the whitelist.
    pub added_guild_names: Vec<String>,

    /// Guild names to remove from the whitelist.
    pub removed_guild_names: Vec<String>,
}

impl Decodable for ExivaRestrictions {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Ok(Self {
            allow_all: bytes.get_bool()?,
            allow_own_guild: bytes.get_bool()?,
            allow_own_party: bytes.get_bool()?,
            allow_vip_list: bytes.get_bool()?,
            allow_player_whitelist: bytes.get_bool()?,
            allow_guild_whitelist: bytes.get_bool()?,
            added_player_names: decode_string_list(&mut bytes)?,
            removed_player_names: decode_string_list(&mut bytes)?,
            added_guild_names: decode_string_list(&mut bytes)?,
            removed_guild_names: decode_string_list(&mut bytes)?,
        })
    }
}

fn decode_string_list(bytes: &mut &mut &[u8]) -> Result<Vec<String>, DecodableError> {
    let count = bytes.get_u16()? as usize;
    let mut entries = Vec::with_capacity(count);

    for _ in 0..count {
        entries.push(bytes.get_string()?);
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_exiva_restrictions() {
        let mut payload: &[u8] = &[
            1, 0, 1, 0, 1, 0, 1, 0, 1, 0, b'A', 0, 0, 1, 0, 1, 0, b'B', 1, 0, 1, 0, b'C',
        ];

        let packet = ExivaRestrictions::decode(PacketKind::ExivaRestrictions, &mut payload)
            .expect("ExivaRestrictions packets should decode restriction flags and lists");

        assert!(packet.allow_all);
        assert!(!packet.allow_own_guild);
        assert!(packet.allow_own_party);
        assert_eq!(packet.added_player_names, vec!["A"]);
        assert_eq!(packet.removed_player_names, Vec::<String>::new());
        assert_eq!(packet.added_guild_names, vec!["B"]);
        assert_eq!(packet.removed_guild_names, vec!["C"]);
        assert!(payload.is_empty());
    }
}
