//! Client-to-server packet definitions for the Suon protocol.
//!
//! This crate groups the packet kinds and typed decoders used by the server when
//! reading packets sent from a client.

mod packets {
    pub(crate) mod decoder {
        pub use suon_protocol::prelude::*;
    }
}

mod client;

pub mod prelude {
    pub use crate::client::{
        Decodable, DecodableError, PacketKind,
        prelude::{
            AcceptTradePacket, CancelStepsPacket, ChangeSharedPartyExperiencePacket,
            CloseTradePacket, CreateBuddyPacket, DeleteBuddyPacket, FacePacket, InspectTradePacket,
            InviteToPartyPacket, JoinPartyPacket, KeepAlivePacket, LeavePartyPacket,
            MarketBrowseKind, MarketOfferKind, MarketPacket, PassPartyLeadershipPacket,
            PingLatencyPacket, RequestTradePacket, RevokePartyInvitePacket, StepPacket,
            StepsPacket, UpdateBuddyPacket,
        },
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_expose_client_protocol_api_through_prelude() {
        use crate::prelude::*;

        fn assert_decodable<T: Decodable>() {}

        let _ = std::mem::size_of::<DecodableError>();
        let _ = std::mem::size_of::<KeepAlivePacket>();
        let _ = std::mem::size_of::<PacketKind>();

        assert_decodable::<KeepAlivePacket>();
    }
}
