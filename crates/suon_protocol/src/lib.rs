//! Shared wire-format codecs for the Suon protocol.
//!
//! # Examples
//! ```
//! use suon_protocol::prelude::*;
//!
//! let encoded = Encoder::new().put_u16(7).put_str("suon").finalize();
//! let mut slice = encoded.as_ref();
//!
//! assert_eq!((&mut slice).get_u16().unwrap(), 7);
//! assert_eq!((&mut slice).get_string().unwrap(), "suon");
//! ```

mod packets;

pub mod prelude {
    pub use crate::packets::{
        PACKET_KIND_SIZE,
        decoder::{Decoder, DecoderError},
        encoder::Encoder,
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn should_expose_codec_modules_from_crate_root() {
        let module_name = std::any::type_name::<crate::packets::encoder::Encoder>();

        assert!(
            module_name.contains("packets::encoder::Encoder"),
            "The crate should keep the encoder module available through the packets namespace"
        );
    }

    #[test]
    fn should_expose_protocol_codec_api_through_prelude() {
        use crate::prelude::*;

        fn assert_decodable<T: Decoder>() {}

        let _ = std::mem::size_of::<DecoderError>();
        let _ = std::mem::size_of::<Encoder>();

        assert_decodable::<&mut &[u8]>();

        assert_eq!(PACKET_KIND_SIZE, 1);
    }
}
