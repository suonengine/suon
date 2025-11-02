/// Represents a 32-bit Adler-32 checksum.
///
/// The Adler-32 algorithm is a fast checksum used for verifying data integrity.
/// It combines two 16-bit values (A and B) into a single 32-bit value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Adler32Checksum(u32);

impl Adler32Checksum {
    /// Adler-32 modulus constant.
    pub const MOD_ADLER: u32 = 65_521;

    /// The initial Adler-32 checksum value, used when no data has been processed.
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
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u32> for Adler32Checksum {
    /// Converts a `u32` directly into an `Adler32Checksum`.
    #[inline(always)]
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<&[u8]> for Adler32Checksum {
    /// Creates an `Adler32Checksum` from a byte slice using the calculation method.
    #[inline(always)]
    fn from(bytes: &[u8]) -> Self {
        Self::calculate(bytes)
    }
}

impl From<Vec<u8>> for Adler32Checksum {
    /// Creates an `Adler32Checksum` from a `Vec<u8>` using the calculation method.
    #[inline(always)]
    fn from(vec: Vec<u8>) -> Self {
        Self::calculate(&vec)
    }
}

impl<const N: usize> From<&[u8; N]> for Adler32Checksum {
    /// Creates an `Adler32Checksum` from a fixed-size byte array.
    #[inline(always)]
    fn from(array: &[u8; N]) -> Self {
        Self::calculate(array)
    }
}

impl std::fmt::Display for Adler32Checksum {
    /// Formats the checksum as an 8-digit uppercase hexadecimal string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08X}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adler32_checksum_computation_with_sample_data() {
        // Sample data for checksum calculation
        const TEST_DATA: &[u8] = b"Hello Checksum!";

        // Calculate the Adler-32 checksum
        let checksum = Adler32Checksum::calculate(TEST_DATA);

        // Retrieve the checksum value as u32
        let checksum_value: u32 = *checksum;

        // Ensure the checksum differs from the initial value
        assert_ne!(
            checksum_value,
            Adler32Checksum::INITIAL,
            "Checksum should be different from the initial value for non-empty data"
        );

        // Split checksum into 16-bit low and high parts
        let (low_16_bits, high_16_bits) = checksum.components();

        // Recombine the components to verify integrity
        let recombined_checksum = ((high_16_bits as u32) << 16) | (low_16_bits as u32);
        assert_eq!(
            recombined_checksum, checksum_value,
            "Recombined value should match the original checksum"
        );
    }

    #[test]
    fn test_adler32_checksum_with_empty_data_returns_initial() {
        // Empty input data
        const EMPTY_DATA: &[u8] = b"";

        // Calculate checksum for empty data
        let checksum = Adler32Checksum::calculate(EMPTY_DATA);

        // Verify that checksum equals the initial value
        assert_eq!(
            *checksum,
            Adler32Checksum::INITIAL,
            "Checksum for empty data should be the initial value"
        );

        // Confirm that is_initial() returns true
        assert!(
            checksum.is_initial(),
            "is_initial() should return true for the initial checksum"
        );
    }

    #[test]
    fn test_checksum_from_slice_trait() {
        // Sample input data slice
        const INPUT_SLICE: &[u8] = b"Hello Checksum!";

        // Create checksum from slice
        let checksum_from_slice = Adler32Checksum::from(INPUT_SLICE);

        // Calculate expected checksum directly
        let expected_checksum = Adler32Checksum::calculate(INPUT_SLICE);

        // Verify both methods produce the same result
        assert_eq!(
            checksum_from_slice, expected_checksum,
            "Checksum from slice should match direct calculation"
        );
    }

    #[test]
    fn test_checksum_from_vec_trait() {
        // Sample data as Vec<u8>
        const INPUT_DATA: &[u8] = b"Hello Checksum!";
        let data_vec: Vec<u8> = INPUT_DATA.to_vec();

        // Create checksum from Vec
        let checksum_from_vec = Adler32Checksum::from(data_vec);

        // Calculate expected checksum directly
        let expected_checksum = Adler32Checksum::calculate(INPUT_DATA);

        // Verify both results match
        assert_eq!(
            checksum_from_vec, expected_checksum,
            "Checksum from Vec should match direct calculation"
        );
    }

    #[test]
    fn test_checksum_from_array_trait() {
        // Fixed-size array data
        const ARRAY_DATA: &[u8; 15] = b"Hello Checksum!";

        // Create checksum from array
        let checksum_from_array = Adler32Checksum::from(ARRAY_DATA);

        // Calculate checksum directly
        let expected_checksum = Adler32Checksum::calculate(ARRAY_DATA);

        // Results should match
        assert_eq!(
            checksum_from_array, expected_checksum,
            "Checksum from array should match direct calculation"
        );
    }

    #[test]
    fn test_display_trait_formats_checksum_as_uppercase_hex() {
        // Sample data for checksum
        const SAMPLE_DATA: &[u8] = b"Hello Checksum!";

        // Calculate checksum
        let checksum = Adler32Checksum::calculate(SAMPLE_DATA);

        // Format checksum as uppercase hexadecimal string
        let formatted_checksum = format!("{}", checksum);

        // Check string length and uppercase formatting
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
    fn test_components_and_recombine() {
        // Sample data
        const SAMPLE_DATA: &[u8] = b"Hello Checksum!";

        // Calculate checksum
        let checksum = Adler32Checksum::calculate(SAMPLE_DATA);

        // Split into 16-bit parts
        let (low_16_bits, high_16_bits) = checksum.components();

        // Recombine to verify correctness
        let recombined_checksum = ((high_16_bits as u32) << 16) | (low_16_bits as u32);

        // Confirm recombined value matches original checksum
        assert_eq!(
            recombined_checksum, *checksum,
            "Recombined value should match the original checksum"
        );
    }
}
