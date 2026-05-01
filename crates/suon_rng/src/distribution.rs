use rand_core::Rng;

pub(crate) trait RngDistributionExt: Rng {
    fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        assert!(min <= max, "min must be less than or equal to max");

        let span = u64::from(max) - u64::from(min) + 1;
        let threshold = u64::MAX - (u64::MAX % span);

        loop {
            let value = self.next_u64();
            if value < threshold {
                return min + (value % span) as u32;
            }
        }
    }

    fn chance(&mut self, numerator: u32, denominator: u32) -> bool {
        assert!(denominator > 0, "denominator must be greater than zero");
        assert!(
            numerator <= denominator,
            "numerator must be less than or equal to denominator"
        );

        numerator > 0 && self.range_u32(1, denominator) <= numerator
    }
}

impl<T: Rng + ?Sized> RngDistributionExt for T {}
