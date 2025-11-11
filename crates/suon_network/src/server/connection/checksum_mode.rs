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
