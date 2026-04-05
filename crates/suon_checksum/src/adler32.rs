/// Represents a 32-bit Adler-32 checksum.
///
/// The Adler-32 algorithm is a fast checksum used for verifying data integrity.
/// It combines two 16-bit values (A and B) into a single 32-bit value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Adler32Checksum(u32);

impl Adler32Checksum {
    /// Adler-32 modulus constant.
    ///
    /// # Examples
    /// ```
    /// assert_eq!(suon_checksum::Adler32Checksum::MOD_ADLER, 65_521);
    /// ```
    pub const MOD_ADLER: u32 = 65_521;

    /// The initial Adler-32 checksum value, used when no data has been processed.
    ///
    /// # Examples
    /// ```
    /// let checksum = suon_checksum::Adler32Checksum::calculate(b"");
    ///
    /// assert_eq!(*checksum, suon_checksum::Adler32Checksum::INITIAL);
    /// assert!(checksum.is_initial());
    /// ```
    pub const INITIAL: u32 = 1;

    /// Calculates the Adler-32 checksum for the given byte slice.
    ///
    /// # Parameters
    /// - `data`: Byte slice to Calculate the checksum for.
    ///
    /// # Returns
    /// - An `Adler32Checksum` instance containing the computed checksum.
    ///
    /// # Example
    /// ```
    /// let checksum = suon_checksum::Adler32Checksum::calculate(b"hello");
    /// assert_eq!(*checksum, 0x062C0215);
    /// assert_eq!(checksum.to_string(), "062C0215");
    /// ```
    #[inline]
    pub fn calculate(data: &[u8]) -> Self {
        let mut sum_low: u32 = Self::INITIAL;
        let mut sum_high: u32 = 0;

        for &byte in data {
            sum_low = (sum_low + byte as u32) % Self::MOD_ADLER;
            sum_high = (sum_high + sum_low) % Self::MOD_ADLER;
        }

        Self::from((sum_high << 16) | sum_low)
    }

    /// Checks if the checksum is the initial value (i.e., no data processed).
    ///
    /// # Returns
    /// - `true` if checksum is initial; `false` otherwise.
    ///
    /// # Examples
    /// ```
    /// assert!(suon_checksum::Adler32Checksum::calculate(b"").is_initial());
    /// assert!(!suon_checksum::Adler32Checksum::calculate(b"hello").is_initial());
    /// ```
    #[inline(always)]
    pub const fn is_initial(&self) -> bool {
        self.0 == Self::INITIAL
    }

    /// Retrieves the individual 16-bit components of the Adler-32 checksum.
    ///
    /// - The first element is the lower 16 bits (A).
    /// - The second element is the higher 16 bits (B).
    ///
    /// # Returns
    /// - Tuple `(a_component, b_component)`
    ///
    /// # Examples
    /// ```
    /// let checksum = suon_checksum::Adler32Checksum::calculate(b"hello");
    ///
    /// assert_eq!(checksum.components(), (0x0215, 0x062C));
    /// ```
    #[inline(always)]
    pub const fn components(&self) -> (u16, u16) {
        let a = (self.0 & 0xFFFF) as u16;
        let b = ((self.0 >> 16) & 0xFFFF) as u16;
        (a, b)
    }
}

impl std::ops::Deref for Adler32Checksum {
    type Target = u32;

    /// Dereferences to the internal `u32` checksum value.
    ///
    /// # Examples
    /// ```
    /// let checksum = suon_checksum::Adler32Checksum::from(0x1234_5678);
    ///
    /// assert_eq!(*checksum, 0x1234_5678);
    /// ```
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for Adler32Checksum {
    /// Converts a `u32` directly into an `Adler32Checksum`.
    ///
    /// # Examples
    /// ```
    /// let checksum = suon_checksum::Adler32Checksum::from(0xABCD_1234);
    ///
    /// assert_eq!(checksum.components(), (0x1234, 0xABCD));
    /// ```
    #[inline(always)]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for Adler32Checksum {
    /// Creates an `Adler32Checksum` from a byte slice using the calculation method.
    ///
    /// # Examples
    /// ```
    /// let checksum = suon_checksum::Adler32Checksum::from(b"hello".as_slice());
    ///
    /// assert_eq!(*checksum, 0x062C0215);
    /// ```
    #[inline(always)]
    fn from(bytes: &[u8]) -> Self {
        Self::calculate(bytes)
    }
}

impl From<Vec<u8>> for Adler32Checksum {
    /// Creates an `Adler32Checksum` from a `Vec<u8>` using the calculation method.
    ///
    /// # Examples
    /// ```
    /// let checksum = suon_checksum::Adler32Checksum::from(b"hello".to_vec());
    ///
    /// assert_eq!(checksum.to_string(), "062C0215");
    /// ```
    #[inline(always)]
    fn from(vec: Vec<u8>) -> Self {
        Self::calculate(&vec)
    }
}

impl<const N: usize> From<&[u8; N]> for Adler32Checksum {
    /// Creates an `Adler32Checksum` from a fixed-size byte array.
    ///
    /// # Examples
    /// ```
    /// let checksum = suon_checksum::Adler32Checksum::from(b"hello");
    ///
    /// assert_eq!(checksum.components(), (0x0215, 0x062C));
    /// ```
    #[inline(always)]
    fn from(array: &[u8; N]) -> Self {
        Self::calculate(array)
    }
}

impl std::fmt::Display for Adler32Checksum {
    /// Formats the checksum as an 8-digit uppercase hexadecimal string.
    ///
    /// # Examples
    /// ```
    /// let checksum = suon_checksum::Adler32Checksum::calculate(b"hello");
    ///
    /// assert_eq!(checksum.to_string(), "062C0215");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DATA: &[u8] = b"Hello Checksum!";

    #[test]
    fn should_compute_checksum_for_sample_data() {
        let checksum = Adler32Checksum::calculate(SAMPLE_DATA);
        let checksum_value: u32 = *checksum;
        let (low_16_bits, high_16_bits) = checksum.components();
        let recombined_checksum = ((high_16_bits as u32) << 16) | (low_16_bits as u32);

        assert_ne!(
            checksum_value,
            Adler32Checksum::INITIAL,
            "Checksum should be different from the initial value for non-empty data"
        );

        assert_eq!(
            recombined_checksum, checksum_value,
            "Recombined value should match the original checksum"
        );
    }

    #[test]
    fn should_return_initial_checksum_for_empty_data() {
        const EMPTY_DATA: &[u8] = b"";

        let checksum = Adler32Checksum::calculate(EMPTY_DATA);

        assert_eq!(
            *checksum,
            Adler32Checksum::INITIAL,
            "Checksum for empty data should be the initial value"
        );

        assert!(
            checksum.is_initial(),
            "is_initial() should return true for the initial checksum"
        );
    }

    #[test]
    fn should_expose_adler32_constants() {
        assert_eq!(
            Adler32Checksum::MOD_ADLER,
            65_521,
            "MOD_ADLER should expose the standard Adler-32 modulus"
        );

        assert_eq!(
            Adler32Checksum::INITIAL,
            1,
            "INITIAL should expose the standard Adler-32 starting value"
        );
    }

    #[test]
    fn should_build_checksum_from_slice() {
        let checksum_from_slice = Adler32Checksum::from(SAMPLE_DATA);
        let expected_checksum = Adler32Checksum::calculate(SAMPLE_DATA);

        assert_eq!(
            checksum_from_slice, expected_checksum,
            "Checksum from slice should match direct calculation"
        );
    }

    #[test]
    fn should_build_checksum_from_vec() {
        let data_vec: Vec<u8> = SAMPLE_DATA.to_vec();
        let checksum_from_vec = Adler32Checksum::from(data_vec);
        let expected_checksum = Adler32Checksum::calculate(SAMPLE_DATA);

        assert_eq!(
            checksum_from_vec, expected_checksum,
            "Checksum from Vec should match direct calculation"
        );
    }

    #[test]
    fn should_build_checksum_from_array() {
        const ARRAY_DATA: &[u8; 15] = b"Hello Checksum!";

        let checksum_from_array = Adler32Checksum::from(ARRAY_DATA);
        let expected_checksum = Adler32Checksum::calculate(ARRAY_DATA);

        assert_eq!(
            checksum_from_array, expected_checksum,
            "Checksum from array should match direct calculation"
        );
    }

    #[test]
    fn should_format_checksum_as_uppercase_hex() {
        let checksum = Adler32Checksum::calculate(SAMPLE_DATA);
        let formatted_checksum = format!("{}", checksum);

        assert_eq!(
            formatted_checksum.len(),
            8,
            "Formatted checksum should be 8 characters long"
        );

        assert_eq!(
            formatted_checksum,
            formatted_checksum.to_uppercase(),
            "Formatted checksum should be uppercase"
        );
    }

    #[test]
    fn should_format_known_checksum_as_expected_hex() {
        let checksum = Adler32Checksum::calculate(b"hello");

        assert_eq!(
            checksum.to_string(),
            "062C0215",
            "Display should emit the exact uppercase hexadecimal checksum"
        );
    }

    #[test]
    fn should_split_components_and_recombine_them() {
        let checksum = Adler32Checksum::calculate(SAMPLE_DATA);
        let (low_16_bits, high_16_bits) = checksum.components();
        let recombined_checksum = ((high_16_bits as u32) << 16) | (low_16_bits as u32);

        assert_eq!(
            recombined_checksum, *checksum,
            "Recombined value should match the original checksum"
        );
    }

    #[test]
    fn should_match_known_adler32_value_for_hello() {
        const HELLO: &[u8] = b"hello";
        const EXPECTED: u32 = 0x062C0215;

        let checksum = Adler32Checksum::calculate(HELLO);

        assert_eq!(
            *checksum, EXPECTED,
            "The checksum should match the known Adler-32 value for \"hello\""
        );
    }

    #[test]
    fn should_preserve_components_and_initial_state_when_built_from_u32() {
        const RAW: u32 = 0xABCD1234;

        let checksum = Adler32Checksum::from(RAW);

        assert_eq!(
            *checksum, RAW,
            "Converting from u32 should preserve the raw checksum value"
        );

        assert_eq!(
            checksum.components(),
            (0x1234, 0xABCD),
            "components should split the low and high 16-bit halves correctly"
        );

        assert!(
            !checksum.is_initial(),
            "A non-initial raw checksum value should not report as initial"
        );
    }
}
