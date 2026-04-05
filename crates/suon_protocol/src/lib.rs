//! Packet encoding and decoding types for the Suon protocol.
//!
//! # Examples
//! ```
//! use suon_protocol::packets::{
//!     decoder::Decoder,
//!     encoder::Encoder,
//!     server::{Encodable, prelude::KeepAlivePacket},
//! };
//!
//! let encoded = Encoder::new().put_u16(7).put_str("suon").finalize();
//! let mut slice = encoded.as_ref();
//!
//! assert_eq!((&mut slice).get_u16().unwrap(), 7);
//! assert_eq!((&mut slice).get_string().unwrap(), "suon");
//! assert_eq!(KeepAlivePacket.encode_with_kind().as_ref(), &[29]);
//! ```

pub mod packets;

#[cfg(test)]
mod tests {
    #[test]
    fn should_expose_packets_module_from_crate_root() {
        let module_name = std::any::type_name::<crate::packets::encoder::Encoder>();

        assert!(
            module_name.contains("packets::encoder::Encoder"),
            "The crate root should expose packet primitives through the packets module"
        );
    }
}
