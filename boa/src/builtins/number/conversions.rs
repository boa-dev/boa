use super::Number;

impl Number {
    const SIGN_MASK: u64 = 0x8000000000000000;
    const EXPONENT_MASK: u64 = 0x7FF0000000000000;
    const SIGNIFICAND_MASK: u64 = 0x000FFFFFFFFFFFFF;
    const HIDDEN_BIT: u64 = 0x0010000000000000;
    const PHYSICAL_SIGNIFICAND_SIZE: i32 = 52; // Excludes the hidden bit.
    const SIGNIFICAND_SIZE: i32 = 53;

    const EXPONENT_BIAS: i32 = 0x3FF + Self::PHYSICAL_SIGNIFICAND_SIZE;
    const DENORMAL_EXPONENT: i32 = -Self::EXPONENT_BIAS + 1;

    #[inline]
    pub(crate) fn is_denormal(self) -> bool {
        (self.0.to_bits() & Self::EXPONENT_MASK) == 0
    }

    #[inline]
    pub(crate) fn sign(self) -> i64 {
        if (self.0.to_bits() & Self::SIGN_MASK) == 0 {
            1
        } else {
            -1
        }
    }

    pub(crate) fn significand(self) -> u64 {
        let d64 = self.0.to_bits();
        let significand = d64 & Self::SIGNIFICAND_MASK;

        if !self.is_denormal() {
            significand + Self::HIDDEN_BIT
        } else {
            significand
        }
    }

    #[inline]
    pub(crate) fn exponent(self) -> i32 {
        if self.is_denormal() {
            return Self::DENORMAL_EXPONENT;
        }

        let d64 = self.0.to_bits();
        let biased_e = ((d64 & Self::EXPONENT_MASK) >> Self::PHYSICAL_SIGNIFICAND_SIZE) as i32;

        biased_e - Self::EXPONENT_BIAS
    }

    /// Converts a 64-bit floating point number to an `i32` according to the [`ToInt32`][ToInt32] algorithm.
    ///
    /// [ToInt32]: https://tc39.es/ecma262/#sec-toint32
    #[inline]
    #[allow(clippy::float_cmp)]
    pub(crate) fn to_int32(self) -> i32 {
        if self.0.is_finite() && self.0 <= f64::from(i32::MAX) && self.0 >= f64::from(i32::MIN) {
            let i = self.0 as i32;
            if f64::from(i) == self.0 {
                return i;
            }
        }

        // let exponent = ((bits >> 52) & 0x7ff);
        let exponent = self.exponent();
        let bits = if exponent < 0 {
            if exponent <= -Self::SIGNIFICAND_SIZE {
                return 0;
            }

            self.significand() >> -exponent
        } else {
            if exponent > 31 {
                return 0;
            }

            (self.significand() << exponent) & 0xFFFFFFFF
        };

        (self.sign() * (bits as i64)) as i32
    }
}
