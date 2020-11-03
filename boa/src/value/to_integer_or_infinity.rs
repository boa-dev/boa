use super::*;

use std::convert::TryFrom;

/// Represents the result of ToIntegerOrInfinity operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegerOrInfinity {
    Integer(isize),
    Undefined,
    PositiveInfinity,
    NegativeInfinity,
}

impl IntegerOrInfinity {
    /// Represents the algorithm to calculate `relativeStart` (or `k`) in array functions.
    pub fn as_relative_start(self, len: usize) -> usize {
        match self {
            // 1. If relativeStart is -∞, let k be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 2. Else if relativeStart < 0, let k be max(len + relativeStart, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => Self::offset(len, i),
            // 3. Else, let k be min(relativeStart, len).
            IntegerOrInfinity::Integer(i) => (i as usize).min(len),

            // Special case - postive infinity. `len` is always smaller than +inf, thus from (3)
            IntegerOrInfinity::PositiveInfinity => len,
            // Special case - `undefined` is treated like 0
            IntegerOrInfinity::Undefined => 0,
        }
    }

    /// Represents the algorithm to calculate `relativeEnd` (or `final`) in array functions.
    pub fn as_relative_end(self, len: usize) -> usize {
        match self {
            // 1. If end is undefined, let relativeEnd be len
            IntegerOrInfinity::Undefined => len,

            // 1. cont, else let relativeEnd be ? ToIntegerOrInfinity(end).

            // 2. If relativeEnd is -∞, let final be 0.
            IntegerOrInfinity::NegativeInfinity => 0,
            // 3. Else if relativeEnd < 0, let final be max(len + relativeEnd, 0).
            IntegerOrInfinity::Integer(i) if i < 0 => Self::offset(len, i),
            // 4. Else, let final be min(relativeEnd, len).
            IntegerOrInfinity::Integer(i) => (i as usize).min(len),

            // Special case - postive infinity. `len` is always smaller than +inf, thus from (4)
            IntegerOrInfinity::PositiveInfinity => len,
        }
    }

    fn offset(len: usize, i: isize) -> usize {
        debug_assert!(i < 0);
        if i == isize::MIN {
            len.saturating_sub(isize::MAX as usize).saturating_sub(1)
        } else {
            len.saturating_sub(i.saturating_neg() as usize)
        }
    }
}

impl Value {
    /// Converts argument to an integer, +∞, or -∞.
    ///
    /// See: <https://tc39.es/ecma262/#sec-tointegerorinfinity>
    pub fn to_integer_or_infinity(&self, context: &mut Context) -> Result<IntegerOrInfinity> {
        // Special case - `undefined`
        if self.is_undefined() {
            return Ok(IntegerOrInfinity::Undefined);
        }

        // 1. Let number be ? ToNumber(argument).
        let number = self.to_number(context)?;

        // 2. If number is NaN, +0𝔽, or -0𝔽, return 0.
        if number.is_nan() || number == 0.0 || number == -0.0 {
            Ok(IntegerOrInfinity::Integer(0))
        } else if number.is_infinite() && number.is_sign_positive() {
            // 3. If number is +∞𝔽, return +∞.
            Ok(IntegerOrInfinity::PositiveInfinity)
        } else if number.is_infinite() && number.is_sign_negative() {
            // 4. If number is -∞𝔽, return -∞.
            Ok(IntegerOrInfinity::NegativeInfinity)
        } else {
            // 5. Let integer be floor(abs(ℝ(number))).
            let integer = number.abs().floor();
            let integer = integer.min(Number::MAX_SAFE_INTEGER) as i64;
            let integer = isize::try_from(integer)?;

            // 6. If number < +0𝔽, set integer to -integer.
            // 7. Return integer.
            if number < 0.0 {
                Ok(IntegerOrInfinity::Integer(-integer))
            } else {
                Ok(IntegerOrInfinity::Integer(integer))
            }
        }
    }
}
