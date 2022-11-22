use std::cmp::Ordering;

/// Represents the result of `ToIntegerOrInfinity` operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IntegerOrInfinity {
    PositiveInfinity,
    Integer(i64),
    NegativeInfinity,
}

impl IntegerOrInfinity {
    /// Clamps an `IntegerOrInfinity` between two `i64`, effectively converting
    /// it to an i64.
    pub fn clamp_finite(self, min: i64, max: i64) -> i64 {
        assert!(min <= max);

        match self {
            Self::Integer(i) => i.clamp(min, max),
            Self::PositiveInfinity => max,
            Self::NegativeInfinity => min,
        }
    }

    /// Gets the wrapped `i64` if the variant is an `Integer`.
    pub fn as_integer(self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(i),
            _ => None,
        }
    }
}

impl From<f64> for IntegerOrInfinity {
    fn from(number: f64) -> Self {
        // `ToIntegerOrInfinity ( argument )`
        if number.is_nan() || number == 0.0 {
            // 2. If number is NaN, +0𝔽, or -0𝔽, return 0.
            Self::Integer(0)
        } else if number == f64::INFINITY {
            // 3. If number is +∞𝔽, return +∞.
            Self::PositiveInfinity
        } else if number == f64::NEG_INFINITY {
            // 4. If number is -∞𝔽, return -∞.
            Self::NegativeInfinity
        } else {
            // 5. Let integer be floor(abs(ℝ(number))).
            // 6. If number < +0𝔽, set integer to -integer.
            let integer = number.abs().floor().copysign(number) as i64;

            // 7. Return integer.
            Self::Integer(integer)
        }
    }
}

impl PartialEq<i64> for IntegerOrInfinity {
    fn eq(&self, other: &i64) -> bool {
        match self {
            Self::Integer(i) => i == other,
            _ => false,
        }
    }
}

impl PartialEq<IntegerOrInfinity> for i64 {
    fn eq(&self, other: &IntegerOrInfinity) -> bool {
        match other {
            IntegerOrInfinity::Integer(i) => i == other,
            _ => false,
        }
    }
}

impl PartialOrd<i64> for IntegerOrInfinity {
    fn partial_cmp(&self, other: &i64) -> Option<Ordering> {
        match self {
            Self::PositiveInfinity => Some(Ordering::Greater),
            Self::Integer(i) => i.partial_cmp(other),
            Self::NegativeInfinity => Some(Ordering::Less),
        }
    }
}

impl PartialOrd<IntegerOrInfinity> for i64 {
    fn partial_cmp(&self, other: &IntegerOrInfinity) -> Option<Ordering> {
        match other {
            IntegerOrInfinity::PositiveInfinity => Some(Ordering::Less),
            IntegerOrInfinity::Integer(i) => self.partial_cmp(i),
            IntegerOrInfinity::NegativeInfinity => Some(Ordering::Greater),
        }
    }
}
