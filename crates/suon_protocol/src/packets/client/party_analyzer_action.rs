//! Client party-analyzer-action packet.

use crate::packets::decoder::Decoder;

use super::prelude::*;

/// Party-analyzer action requested by the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartyAnalyzerActionKind {
    /// Resets the analyzer data.
    Reset,

    /// Switches the analyzer price type.
    SwitchPriceType,

    /// Updates item custom prices.
    UpdatePrices {
        /// Item-price pairs being updated.
        entries: Vec<(u16, u64)>,
    },
}

/// Packet sent by the client to manage the party analyzer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartyAnalyzerAction {
    /// Analyzer action requested by the client.
    pub action: PartyAnalyzerActionKind,
}

impl Decodable for PartyAnalyzerAction {
    fn decode(_: PacketKind, mut bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        let action = match bytes.get_u8()? {
            0 => PartyAnalyzerActionKind::Reset,
            1 => PartyAnalyzerActionKind::SwitchPriceType,
            2 => {
                let count = bytes.get_u16()?;
                let mut entries = Vec::with_capacity(count as usize);
                for _ in 0..count {
                    entries.push((bytes.get_u16()?, bytes.get_u64()?));
                }

                PartyAnalyzerActionKind::UpdatePrices { entries }
            }
            value => {
                return Err(DecodableError::InvalidFieldValue {
                    field: "action",
                    value,
                });
            }
        };

        Ok(Self { action })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_decode_party_analyzer_price_updates() {
        let mut payload: &[u8] = &[
            2, 2, 0, 0x34, 0x12, 8, 7, 6, 5, 4, 3, 2, 1, 0x78, 0x56, 1, 0, 0, 0, 0, 0, 0, 0,
        ];

        let packet = PartyAnalyzerAction::decode(PacketKind::PartyAnalyzerAction, &mut payload)
            .expect("PartyAnalyzerAction packets should decode price updates");

        assert_eq!(
            packet.action,
            PartyAnalyzerActionKind::UpdatePrices {
                entries: vec![(0x1234, 0x0102030405060708), (0x5678, 1)],
            }
        );
    }
}
