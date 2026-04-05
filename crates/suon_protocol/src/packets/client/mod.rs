use thiserror::Error;

mod accept_market_offer;
mod accept_trade;
mod browse_market;
mod cancel_market_offer;
mod cancel_steps;
mod change_shared_party_experience;
mod close_trade;
mod create_buddy;
mod create_market_offer;
mod delete_buddy;
mod face;
mod inspect_trade;
mod invite_to_party;
mod join_party;
mod keep_alive;
mod leave_market;
mod leave_party;
mod movement;
mod pass_party_leadership;
mod ping_latency;
mod request_trade;
mod revoke_party_invite;
mod steps;
mod update_buddy;

pub mod prelude {
    pub use super::{
        Decodable, DecodableError, PacketKind,
        accept_market_offer::AcceptMarketOfferPacket,
        accept_trade::AcceptTradePacket,
        browse_market::BrowseMarketPacket,
        cancel_market_offer::CancelMarketOfferPacket,
        cancel_steps::CancelStepsPacket,
        change_shared_party_experience::ChangeSharedPartyExperiencePacket,
        close_trade::CloseTradePacket,
        create_buddy::CreateBuddyPacket,
        create_market_offer::{CreateMarketOfferPacket, MarketOfferKind},
        delete_buddy::DeleteBuddyPacket,
        face::FacePacket,
        inspect_trade::InspectTradePacket,
        invite_to_party::InviteToPartyPacket,
        join_party::JoinPartyPacket,
        keep_alive::KeepAlivePacket,
        leave_market::LeaveMarketPacket,
        leave_party::LeavePartyPacket,
        movement::StepPacket,
        pass_party_leadership::PassPartyLeadershipPacket,
        ping_latency::PingLatencyPacket,
        request_trade::RequestTradePacket,
        revoke_party_invite::RevokePartyInvitePacket,
        steps::StepsPacket,
        update_buddy::UpdateBuddyPacket,
    };
}

/// Errors that can occur while decoding a packet.
#[derive(Debug, Error)]
pub enum DecodableError {
    /// Wraps a lower-level decoding error.
    #[error("failed to decode packet: {0}")]
    Decoder(#[from] crate::packets::decoder::DecoderError),

    /// The payload contained an unsupported value for a typed packet field.
    #[error("invalid value {value} for field '{field}'")]
    InvalidFieldValue {
        /// Logical field name being decoded.
        field: &'static str,
        /// Raw value received from the wire.
        value: u8,
    },
}

/// Represents a packet that can be decoded from a binary buffer.
///
/// This trait defines how a packet is reconstructed from raw bytes received
/// over a network or read from storage. Each packet type has a unique [`PacketKind`]
/// that identifies it and allows the system to dispatch the correct decoding logic.
///
/// # Associated Constant
/// - [`Self::KIND`]: The unique [`PacketKind`] that identifies this packet type.
///
/// # Methods
/// - [`Self::decode`]: Decodes the packet instance from a raw byte slice.
///
/// # Example
/// ```
/// use suon_protocol::packets::client::{Decodable, DecodableError, PacketKind};
///
/// struct LoginPacket {
///     username: String,
/// }
///
/// impl Decodable for LoginPacket {
///     const KIND: PacketKind = PacketKind::Login;
///
///     fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError> {
///         use suon_protocol::packets::decoder::Decoder;
///
///         let username = (&mut *bytes).get_string()?;
///         Ok(LoginPacket { username })
///     }
/// }
///
/// let mut buffer: &[u8] = &[5, 0, b'A', b'l', b'i', b'c', b'e'];
/// let packet = LoginPacket::decode(&mut buffer).unwrap();
///
/// assert_eq!(packet.username, "Alice");
/// ```
///
/// This trait is typically paired with the server-side
/// [`crate::packets::server::Encodable`] trait to allow symmetric serialization
/// and deserialization of packet types.
pub trait Decodable: Sized {
    /// Unique kind identifier for this packet type.
    const KIND: PacketKind;

    /// Returns whether this packet type can be decoded from the provided wire kind.
    fn accepts_kind(kind: PacketKind) -> bool {
        kind == Self::KIND
    }

    /// Decodes the packet instance from a raw byte slice.
    ///
    /// Implementers should read the buffer according to the expected packet structure.
    /// Returns an error if the buffer is incomplete or contains invalid data.
    fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError>;

    /// Decodes the packet instance while preserving the originating wire kind.
    fn decode_with_kind(_: PacketKind, bytes: &mut &[u8]) -> Result<Self, DecodableError> {
        Self::decode(bytes)
    }
}

/// Defines the possible kinds or categories of network packets.
///
/// Each [`PacketKind`] corresponds to a specific packet type that implements
/// the [`Decodable`] trait. This allows the system to determine how to
/// deserialize and distinguish different packet variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PacketKind {
    /// Internal packet sent by the client as the **first message**.
    ServerName = 0,

    /// Sent when a client attempts to log in.
    Login = 10,
    /// Sent when a client logs out.
    Logout = 20,

    /// Requests a multi-step path.
    Steps = 100,
    /// Requests a one-tile step to the north.
    StepNorth = 101,
    /// Requests a one-tile step to the east.
    StepEast = 102,
    /// Requests a one-tile step to the south.
    StepSouth = 103,
    /// Requests a one-tile step to the west.
    StepWest = 104,
    /// Requests that any active step sequence be canceled.
    CancelSteps = 105,
    /// Requests a one-tile step to the north-east.
    StepNorthEast = 106,
    /// Requests a one-tile step to the south-east.
    StepSouthEast = 107,
    /// Requests a one-tile step to the south-west.
    StepSouthWest = 108,
    /// Requests a one-tile step to the north-west.
    StepNorthWest = 109,

    /// Faces north.
    FaceNorth = 111,
    /// Faces east.
    FaceEast = 112,
    /// Faces south.
    FaceSouth = 113,
    /// Faces west.
    FaceWest = 114,

    /// Sent to measure latency between client and server.
    PingLatency = 29,
    /// Keeps the connection alive.
    KeepAlive = 30,

    /// Requests a trade with another player for a specific item.
    RequestTrade = 125,
    /// Inspects one of the items shown in the trade window.
    InspectTrade = 126,
    /// Accepts the current trade.
    AcceptTrade = 127,
    /// Closes the current trade.
    CloseTrade = 128,

    /// Creates a buddy entry.
    CreateBuddy = 220,
    /// Deletes a buddy entry.
    DeleteBuddy = 221,
    /// Updates a buddy entry.
    UpdateBuddy = 222,

    /// Invites a player to the party.
    InviteToParty = 163,
    /// Joins a party through a target player's invitation.
    JoinParty = 164,
    /// Revokes a previously sent party invite.
    RevokePartyInvite = 165,
    /// Passes party leadership to another player.
    PassPartyLeadership = 166,
    /// Leaves the current party.
    LeaveParty = 167,
    /// Changes the shared party experience state.
    ChangeSharedPartyExperience = 168,

    /// Leaves the market view.
    LeaveMarket = 244,
    /// Browses a market category, own offers, or own history.
    BrowseMarket = 245,
    /// Creates a market offer.
    CreateMarketOffer = 246,
    /// Cancels a market offer.
    CancelMarketOffer = 247,
    /// Accepts a market offer.
    AcceptMarketOffer = 248,
}

impl TryFrom<u8> for PacketKind {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::ServerName),

            10 => Ok(Self::Login),
            20 => Ok(Self::Logout),

            29 => Ok(Self::PingLatency),
            30 => Ok(Self::KeepAlive),

            100 => Ok(Self::Steps),
            101 => Ok(Self::StepNorth),
            102 => Ok(Self::StepEast),
            103 => Ok(Self::StepSouth),
            104 => Ok(Self::StepWest),
            105 => Ok(Self::CancelSteps),
            106 => Ok(Self::StepNorthEast),
            107 => Ok(Self::StepSouthEast),
            108 => Ok(Self::StepSouthWest),
            109 => Ok(Self::StepNorthWest),

            111 => Ok(Self::FaceNorth),
            112 => Ok(Self::FaceEast),
            113 => Ok(Self::FaceSouth),
            114 => Ok(Self::FaceWest),

            125 => Ok(Self::RequestTrade),
            126 => Ok(Self::InspectTrade),
            127 => Ok(Self::AcceptTrade),
            128 => Ok(Self::CloseTrade),

            163 => Ok(Self::InviteToParty),
            164 => Ok(Self::JoinParty),
            165 => Ok(Self::RevokePartyInvite),
            166 => Ok(Self::PassPartyLeadership),
            167 => Ok(Self::LeaveParty),
            168 => Ok(Self::ChangeSharedPartyExperience),

            220 => Ok(Self::CreateBuddy),
            221 => Ok(Self::DeleteBuddy),
            222 => Ok(Self::UpdateBuddy),

            244 => Ok(Self::LeaveMarket),
            245 => Ok(Self::BrowseMarket),
            246 => Ok(Self::CreateMarketOffer),
            247 => Ok(Self::CancelMarketOffer),
            248 => Ok(Self::AcceptMarketOffer),

            _ => Err(value),
        }
    }
}

impl std::fmt::Display for PacketKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} (0x{:02X})", self, *self as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Packet;

    impl Decodable for Packet {
        const KIND: PacketKind = PacketKind::PingLatency;

        fn decode(bytes: &mut &[u8]) -> Result<Self, DecodableError> {
            if bytes.is_empty() {
                Err(DecodableError::Decoder(
                    crate::packets::decoder::DecoderError::Incomplete {
                        expected: 1,
                        available: 0,
                    },
                ))
            } else {
                Ok(Packet)
            }
        }
    }

    #[test]
    fn decode_packet_returns_error_on_empty_buffer() {
        let mut buffer: &[u8] = &[];

        let error = Packet::decode(&mut buffer)
            .expect_err("Expected DecoderError::Incomplete for empty buffer");

        match error {
            DecodableError::Decoder(crate::packets::decoder::DecoderError::Incomplete {
                expected,
                available,
            }) => {
                assert!(
                    expected == 1,
                    "Expected 1 byte to be required, got {}",
                    expected
                );
                assert!(
                    available == 0,
                    "Expected 0 bytes available, got {}",
                    available
                );
            }
            other => {
                panic!("Unexpected error variant: {:?}", other);
            }
        }
    }

    #[test]
    fn decode_packet_succeeds_with_non_empty_buffer() {
        const PAYLOAD: &[u8] = &[42];

        let mut buffer: &[u8] = PAYLOAD;

        let packet_result = Packet::decode(&mut buffer);
        assert!(
            packet_result.is_ok(),
            "Decoding should succeed with non-empty buffer"
        );

        let packet = packet_result.unwrap();
        assert!(
            matches!(packet, Packet),
            "Decoded packet should be of type Packet"
        );
    }

    #[test]
    fn packet_kind_should_convert_from_wire_values_and_format_for_logs() {
        assert_eq!(
            PacketKind::try_from(30),
            Ok(PacketKind::KeepAlive),
            "Wire value 30 should decode to the keep-alive client packet kind"
        );
        assert_eq!(
            PacketKind::try_from(101),
            Ok(PacketKind::StepNorth),
            "Wire value 101 should decode to the step-north client packet kind"
        );
        assert_eq!(
            PacketKind::PingLatency.to_string(),
            "PingLatency (0x1D)",
            "Display should include both the variant name and hexadecimal id"
        );
        assert_eq!(
            PacketKind::try_from(222),
            Ok(PacketKind::UpdateBuddy),
            "Wire value 222 should decode to the update-buddy client packet kind"
        );
    }
}
