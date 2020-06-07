use super::BigInt;

impl BigInt {
    /// Checks for `SameValueZero` equality.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-numeric-types-bigint-equal
    #[inline]
    pub(crate) fn same_value_zero(x: &Self, y: &Self) -> bool {
        // Return BigInt::equal(x, y)
        Self::equal(x, y)
    }

    /// Checks for `SameValue` equality.
    ///
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-numeric-types-bigint-sameValue
    #[inline]
    pub(crate) fn same_value(x: &Self, y: &Self) -> bool {
        // Return BigInt::equal(x, y)
        Self::equal(x, y)
    }

    /// Checks for mathematical equality.
    ///
    /// The abstract operation BigInt::equal takes arguments x (a `BigInt`) and y (a `BigInt`).
    /// It returns `true` if x and y have the same mathematical integer value and false otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-numeric-types-bigint-sameValueZero
    #[inline]
    pub(crate) fn equal(x: &Self, y: &Self) -> bool {
        x == y
    }
}

impl PartialEq<i32> for BigInt {
    fn eq(&self, other: &i32) -> bool {
        self.0 == num_bigint::BigInt::from(*other)
    }
}

impl PartialEq<BigInt> for i32 {
    fn eq(&self, other: &BigInt) -> bool {
        num_bigint::BigInt::from(*self) == other.0
    }
}

impl PartialEq<f64> for BigInt {
    fn eq(&self, other: &f64) -> bool {
        if other.fract() != 0.0 {
            return false;
        }

        self.0 == num_bigint::BigInt::from(*other as i64)
    }
}

impl PartialEq<BigInt> for f64 {
    fn eq(&self, other: &BigInt) -> bool {
        if self.fract() != 0.0 {
            return false;
        }

        num_bigint::BigInt::from(*self as i64) == other.0
    }
}
