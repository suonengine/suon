/// Specifies the checksum calculation method for validating packets.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ChecksumMode {
    /// Use the Adler-32 checksum algorithm.
    Adler32,

    /// Use a sequence-based checksum, typically incremented for each packet.
    Sequence(usize),
}

impl std::fmt::Display for ChecksumMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_format_checksum_modes_like_debug_output() {
        assert_eq!(
            ChecksumMode::Adler32.to_string(),
            "Adler32",
            "Display should keep the human-readable Adler-32 mode name"
        );

        assert_eq!(
            ChecksumMode::Sequence(7).to_string(),
            "Sequence(7)",
            "Display should include the sequence counter for sequence-based checksums"
        );
    }
}
