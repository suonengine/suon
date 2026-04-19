//! Server-to-client packet definitions for the Suon protocol.
//!
//! This crate groups the packet kinds and typed encoders used by the server when
//! writing packets to a client.

mod packets {
    pub(crate) mod encoder {
        pub use suon_protocol::prelude::Encoder;
    }

    pub(crate) const PACKET_KIND_SIZE: usize = suon_protocol::prelude::PACKET_KIND_SIZE;
}

mod server;

pub mod prelude {
    pub use crate::server::{
        Encodable, PacketKind,
        prelude::{ChallengePacket, KeepAlivePacket, PingLatencyPacket},
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_expose_server_protocol_api_through_prelude() {
        use crate::prelude::*;

        fn encode_keep_alive<T: Encodable>(packet: T) {
            let _ = packet.encode_with_kind();
        }

        let _ = std::mem::size_of::<KeepAlivePacket>();
        let _ = std::mem::size_of::<PacketKind>();

        encode_keep_alive(KeepAlivePacket);
    }
}
