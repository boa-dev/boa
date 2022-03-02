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
            IntegerOrInfinity::Integer(i) => i.clamp(min, max),
            IntegerOrInfinity::PositiveInfinity => max,
            IntegerOrInfinity::NegativeInfinity => min,
        }
    }

    /// Gets the wrapped `i64` if the variant is an `Integer`.
    pub fn as_integer(self) -> Option<i64> {
        match self {
            IntegerOrInfinity::Integer(i) => Some(i),
            _ => None,
        }
    }
}

impl PartialEq<i64> for IntegerOrInfinity {
    fn eq(&self, other: &i64) -> bool {
        match self {
            IntegerOrInfinity::Integer(i) => i == other,
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
            IntegerOrInfinity::PositiveInfinity => Some(Ordering::Greater),
            IntegerOrInfinity::Integer(i) => i.partial_cmp(other),
            IntegerOrInfinity::NegativeInfinity => Some(Ordering::Less),
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
