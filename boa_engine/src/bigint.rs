//! Boa's implementation of ECMAScript's bigint primitive type.

use crate::{builtins::Number, error::JsNativeError, JsResult};
use num_integer::Integer;
use num_traits::{pow::Pow, FromPrimitive, One, ToPrimitive, Zero};
use std::{
    fmt::{self, Display},
    ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub},
    rc::Rc,
};

/// The raw bigint type.
pub type RawBigInt = num_bigint::BigInt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

/// JavaScript bigint primitive rust type.
#[cfg_attr(feature = "deser", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsBigInt {
    inner: Rc<RawBigInt>,
}

impl JsBigInt {
    /// Create a new [`JsBigInt`].
    #[must_use]
    pub fn new<T: Into<Self>>(value: T) -> Self {
        value.into()
    }

    /// Create a [`JsBigInt`] with value `0`.
    #[inline]
    #[must_use]
    pub fn zero() -> Self {
        Self {
            inner: Rc::new(RawBigInt::zero()),
        }
    }

    /// Check if is zero.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }

    /// Create a [`JsBigInt`] with value `1`.
    #[inline]
    #[must_use]
    pub fn one() -> Self {
        Self {
            inner: Rc::new(RawBigInt::one()),
        }
    }

    /// Check if is one.
    #[inline]
    #[must_use]
    pub fn is_one(&self) -> bool {
        self.inner.is_one()
    }

    /// Convert bigint to string with radix.
    #[inline]
    #[must_use]
    pub fn to_string_radix(&self, radix: u32) -> String {
        self.inner.to_str_radix(radix)
    }

    /// Converts the `BigInt` to a f64 type.
    ///
    /// Returns `f64::INFINITY` if the `BigInt` is too big.
    #[inline]
    #[must_use]
    pub fn to_f64(&self) -> f64 {
        self.inner.to_f64().unwrap_or(f64::INFINITY)
    }

    /// Converts a string to a `BigInt` with the specified radix.
    #[inline]
    #[must_use]
    pub fn from_string_radix(buf: &str, radix: u32) -> Option<Self> {
        Some(Self {
            inner: Rc::new(RawBigInt::parse_bytes(buf.as_bytes(), radix)?),
        })
    }

    /// This function takes a string and converts it to `BigInt` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringtobigint
    #[inline]
    #[must_use]
    pub fn from_string(mut string: &str) -> Option<Self> {
        string = string.trim();

        if string.is_empty() {
            return Some(Self::zero());
        }

        let mut radix = 10;
        if string.starts_with("0b") || string.starts_with("0B") {
            radix = 2;
            string = &string[2..];
        } else if string.starts_with("0x") || string.starts_with("0X") {
            radix = 16;
            string = &string[2..];
        } else if string.starts_with("0o") || string.starts_with("0O") {
            radix = 8;
            string = &string[2..];
        }

        Self::from_string_radix(string, radix)
    }

    /// Checks for `SameValueZero` equality.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-numeric-types-bigint-equal
    #[inline]
    #[must_use]
    pub fn same_value_zero(x: &Self, y: &Self) -> bool {
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
    #[must_use]
    pub fn same_value(x: &Self, y: &Self) -> bool {
        // Return BigInt::equal(x, y)
        Self::equal(x, y)
    }

    /// Checks for mathematical equality.
    ///
    /// The abstract operation `BigInt::equal` takes arguments x (a `BigInt`) and y (a `BigInt`).
    /// It returns `true` if x and y have the same mathematical integer value and false otherwise.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-numeric-types-bigint-sameValueZero
    #[inline]
    #[must_use]
    pub fn equal(x: &Self, y: &Self) -> bool {
        x == y
    }

    /// Returns `x` to the power `y`.
    #[inline]
    pub fn pow(x: &Self, y: &Self) -> JsResult<Self> {
        let y = y
            .inner
            .to_biguint()
            .ok_or_else(|| JsNativeError::range().with_message("BigInt negative exponent"))?;

        let num_bits = (x.inner.bits() as f64
            * y.to_f64().expect("Unable to convert from BigUInt to f64"))
        .floor()
            + 1f64;

        if num_bits > 1_000_000_000f64 {
            return Err(JsNativeError::range()
                .with_message("Maximum BigInt size exceeded")
                .into());
        }

        Ok(Self::new(x.inner.as_ref().clone().pow(y)))
    }

    /// Performs the `>>` operation.
    #[inline]
    pub fn shift_right(x: &Self, y: &Self) -> JsResult<Self> {
        match y.inner.to_i32() {
            Some(n) if n > 0 => Ok(Self::new(x.inner.as_ref().clone().shr(n as usize))),
            Some(n) => Ok(Self::new(x.inner.as_ref().clone().shl(n.unsigned_abs()))),
            None => Err(JsNativeError::range()
                .with_message("Maximum BigInt size exceeded")
                .into()),
        }
    }

    /// Performs the `<<` operation.
    #[inline]
    pub fn shift_left(x: &Self, y: &Self) -> JsResult<Self> {
        match y.inner.to_i32() {
            Some(n) if n > 0 => Ok(Self::new(x.inner.as_ref().clone().shl(n as usize))),
            Some(n) => Ok(Self::new(x.inner.as_ref().clone().shr(n.unsigned_abs()))),
            None => Err(JsNativeError::range()
                .with_message("Maximum BigInt size exceeded")
                .into()),
        }
    }

    /// Floored integer modulo.
    ///
    /// # Examples
    /// ```
    /// # use num_integer::Integer;
    /// assert_eq!((8).mod_floor(&3), 2);
    /// assert_eq!((8).mod_floor(&-3), -1);
    /// ```
    #[inline]
    #[must_use]
    pub fn mod_floor(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.mod_floor(&y.inner))
    }

    /// Performs the `+` operation.
    #[inline]
    #[must_use]
    pub fn add(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().add(y.inner.as_ref()))
    }

    /// Performs the `-` operation.
    #[inline]
    #[must_use]
    pub fn sub(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().sub(y.inner.as_ref()))
    }

    /// Performs the `*` operation.
    #[inline]
    #[must_use]
    pub fn mul(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().mul(y.inner.as_ref()))
    }

    /// Performs the `/` operation.
    #[inline]
    #[must_use]
    pub fn div(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().div(y.inner.as_ref()))
    }

    /// Performs the `%` operation.
    #[inline]
    #[must_use]
    pub fn rem(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().rem(y.inner.as_ref()))
    }

    /// Performs the `&` operation.
    #[inline]
    #[must_use]
    pub fn bitand(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().bitand(y.inner.as_ref()))
    }

    /// Performs the `|` operation.
    #[inline]
    #[must_use]
    pub fn bitor(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().bitor(y.inner.as_ref()))
    }

    /// Performs the `^` operation.
    #[inline]
    #[must_use]
    pub fn bitxor(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().bitxor(y.inner.as_ref()))
    }

    /// Performs the unary `-` operation.
    #[inline]
    #[must_use]
    pub fn neg(x: &Self) -> Self {
        Self::new(x.as_inner().neg())
    }

    /// Performs the unary `!` operation.
    #[inline]
    #[must_use]
    pub fn not(x: &Self) -> Self {
        Self::new(!x.as_inner())
    }

    pub(crate) fn as_inner(&self) -> &RawBigInt {
        &self.inner
    }
}

impl Display for JsBigInt {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<RawBigInt> for JsBigInt {
    #[inline]
    fn from(value: RawBigInt) -> Self {
        Self {
            inner: Rc::new(value),
        }
    }
}

impl From<Box<RawBigInt>> for JsBigInt {
    #[inline]
    fn from(value: Box<RawBigInt>) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl From<i8> for JsBigInt {
    #[inline]
    fn from(value: i8) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

impl From<u8> for JsBigInt {
    #[inline]
    fn from(value: u8) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

impl From<i16> for JsBigInt {
    #[inline]
    fn from(value: i16) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

impl From<u16> for JsBigInt {
    #[inline]
    fn from(value: u16) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

impl From<i32> for JsBigInt {
    #[inline]
    fn from(value: i32) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

impl From<u32> for JsBigInt {
    #[inline]
    fn from(value: u32) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

impl From<i64> for JsBigInt {
    #[inline]
    fn from(value: i64) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

impl From<u64> for JsBigInt {
    #[inline]
    fn from(value: u64) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

impl From<isize> for JsBigInt {
    #[inline]
    fn from(value: isize) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

impl From<usize> for JsBigInt {
    #[inline]
    fn from(value: usize) -> Self {
        Self {
            inner: Rc::new(RawBigInt::from(value)),
        }
    }
}

/// The error indicates that the conversion from [`f64`] to [`JsBigInt`] failed.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TryFromF64Error;

impl Display for TryFromF64Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not convert f64 value to a BigInt type")
    }
}

impl TryFrom<f64> for JsBigInt {
    type Error = TryFromF64Error;

    #[inline]
    fn try_from(n: f64) -> Result<Self, Self::Error> {
        // If the truncated version of the number is not the
        // same as the non-truncated version then the floating-point
        // number conains a fractional part.
        if !Number::equal(n.trunc(), n) {
            return Err(TryFromF64Error);
        }
        RawBigInt::from_f64(n).map_or(Err(TryFromF64Error), |bigint| Ok(Self::new(bigint)))
    }
}

impl PartialEq<i32> for JsBigInt {
    #[inline]
    fn eq(&self, other: &i32) -> bool {
        self.inner.as_ref() == &RawBigInt::from(*other)
    }
}

impl PartialEq<JsBigInt> for i32 {
    #[inline]
    fn eq(&self, other: &JsBigInt) -> bool {
        &RawBigInt::from(*self) == other.inner.as_ref()
    }
}

impl PartialEq<f64> for JsBigInt {
    #[inline]
    fn eq(&self, other: &f64) -> bool {
        other.fract().is_zero()
            && RawBigInt::from_f64(*other).map_or(false, |bigint| self.inner.as_ref() == &bigint)
    }
}

impl PartialEq<JsBigInt> for f64 {
    #[inline]
    fn eq(&self, other: &JsBigInt) -> bool {
        self.fract().is_zero()
            && RawBigInt::from_f64(*self).map_or(false, |bigint| other.inner.as_ref() == &bigint)
    }
}
