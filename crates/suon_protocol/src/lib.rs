//! Packet encoding and decoding types for the Suon protocol.

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
