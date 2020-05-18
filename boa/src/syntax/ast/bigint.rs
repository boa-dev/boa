//! This module implements the `BigInt` structure, which represents the BigInt values in JavaScript.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-bigint-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt

use gc::{unsafe_empty_trace, Finalize, Trace};
use num_traits::cast::{FromPrimitive, ToPrimitive};
use num_traits::pow::Pow;

use std::convert::TryFrom;
use std::str::FromStr;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BigInt(num_bigint::BigInt);

impl BigInt {
    /// Returns the inner bigint structure.
    #[inline]
    pub fn into_inner(self) -> num_bigint::BigInt {
        self.0
    }

    /// Converts a string to a BigInt with the specified radix.
    #[inline]
    pub fn from_str_radix(buf: &str, radix: u32) -> Option<Self> {
        num_bigint::BigInt::parse_bytes(buf.as_bytes(), radix).map(Self)
    }

    #[inline]
    pub fn pow(self, other: &Self) -> Self {
        Self(
            self.0.pow(
                other
                    .0
                    .to_biguint()
                    .expect("RangeError: \"BigInt negative exponent\""),
            ),
        )
    }

    /// Converts the BigInt to a f64 type.
    ///
    /// Returns `std::f64::INFINITY` if the BigInt is too big.
    #[inline]
    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().unwrap_or(std::f64::INFINITY)
    }
}

impl From<i64> for BigInt {
    fn from(n: i64) -> BigInt {
        BigInt(num_bigint::BigInt::from(n))
    }
}

impl From<i32> for BigInt {
    fn from(n: i32) -> BigInt {
        BigInt(num_bigint::BigInt::from(n))
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TryFromF64Error;

impl std::fmt::Display for TryFromF64Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert f64 value to a BigInt type")
    }
}

impl TryFrom<f64> for BigInt {
    type Error = TryFromF64Error;

    fn try_from(n: f64) -> Result<Self, Self::Error> {
        match num_bigint::BigInt::from_f64(n) {
            Some(bigint) => Ok(BigInt(bigint)),
            None => Err(TryFromF64Error),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ParseBigIntError;

impl std::fmt::Display for ParseBigIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not parse to BigInt type")
    }
}

impl FromStr for BigInt {
    type Err = ParseBigIntError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match num_bigint::BigInt::from_str(string) {
            Ok(bigint) => Ok(BigInt(bigint)),
            Err(_) => Err(ParseBigIntError),
        }
    }
}

macro_rules! impl_bigint_operator {
    ($op:ident, $op_method:ident, $assign_op:ident, $assign_op_method:ident) => {
        impl std::ops::$op for BigInt {
            type Output = Self;

            fn $op_method(mut self, other: Self) -> Self {
                std::ops::$assign_op::$assign_op_method(&mut self.0, other.0);
                self
            }
        }
    };
}

impl_bigint_operator!(Add, add, AddAssign, add_assign);
impl_bigint_operator!(Sub, sub, SubAssign, sub_assign);
impl_bigint_operator!(Mul, mul, MulAssign, mul_assign);
impl_bigint_operator!(Div, div, DivAssign, div_assign);
impl_bigint_operator!(Rem, rem, RemAssign, rem_assign);
impl_bigint_operator!(BitAnd, bitand, BitAndAssign, bitand_assign);
impl_bigint_operator!(BitOr, bitor, BitOrAssign, bitor_assign);
impl_bigint_operator!(BitXor, bitxor, BitXorAssign, bitxor_assign);

impl std::ops::Shr for BigInt {
    type Output = Self;

    fn shr(mut self, other: Self) -> Self::Output {
        use std::ops::ShlAssign;
        use std::ops::ShrAssign;

        if let Some(n) = other.0.to_i32() {
            if n > 0 {
                self.0.shr_assign(n as usize)
            } else {
                self.0.shl_assign(n.abs() as usize)
            }

            return self;
        }

        panic!("RangeError: Maximum BigInt size exceeded");
    }
}

impl std::ops::Shl for BigInt {
    type Output = Self;

    fn shl(mut self, other: Self) -> Self::Output {
        use std::ops::ShlAssign;
        use std::ops::ShrAssign;

        if let Some(n) = other.0.to_i32() {
            if n > 0 {
                self.0.shl_assign(n as usize)
            } else {
                self.0.shr_assign(n.abs() as usize)
            }

            return self;
        }

        panic!("RangeError: Maximum BigInt size exceeded");
    }
}

impl std::ops::Neg for BigInt {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
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

impl std::fmt::Debug for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::fmt::Display for BigInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::ops::Deref for BigInt {
    type Target = num_bigint::BigInt;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BigInt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Finalize for BigInt {}
unsafe impl Trace for BigInt {
    // BigInt type implements an empty trace becasue the inner structure 
    // `num_bigint::BigInt` does not implement `Trace` trait.
    // If it did this would be unsound.
    unsafe_empty_trace!();
}
