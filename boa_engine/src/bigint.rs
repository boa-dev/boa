//! This module implements the JavaScript bigint primitive rust type.

use crate::{builtins::Number, Context, JsValue};
use boa_gc::{unsafe_empty_trace, Finalize, Trace};
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
#[derive(Debug, Finalize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JsBigInt {
    inner: Rc<RawBigInt>,
}

// Safety: BigInt does not contain any objects which needs to be traced,
// so this is safe.
unsafe impl Trace for JsBigInt {
    unsafe_empty_trace!();
}

impl JsBigInt {
    /// Create a new [`JsBigInt`].
    #[inline]
    pub fn new<T: Into<Self>>(value: T) -> Self {
        value.into()
    }

    /// Create a [`JsBigInt`] with value `0`.
    #[inline]
    pub fn zero() -> Self {
        Self {
            inner: Rc::new(RawBigInt::zero()),
        }
    }

    /// Check if is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }

    /// Create a [`JsBigInt`] with value `1`.
    #[inline]
    pub fn one() -> Self {
        Self {
            inner: Rc::new(RawBigInt::one()),
        }
    }

    /// Check if is one.
    #[inline]
    pub fn is_one(&self) -> bool {
        self.inner.is_one()
    }

    /// Convert bigint to string with radix.
    #[inline]
    pub fn to_string_radix(&self, radix: u32) -> String {
        self.inner.to_str_radix(radix)
    }

    /// Converts the `BigInt` to a f64 type.
    ///
    /// Returns `f64::INFINITY` if the `BigInt` is too big.
    #[inline]
    pub fn to_f64(&self) -> f64 {
        self.inner.to_f64().unwrap_or(f64::INFINITY)
    }

    /// Converts a string to a `BigInt` with the specified radix.
    #[inline]
    pub fn from_string_radix(buf: &str, radix: u32) -> Option<Self> {
        Some(Self {
            inner: Rc::new(RawBigInt::parse_bytes(buf.as_bytes(), radix)?),
        })
    }

    /// This function takes a string and conversts it to `BigInt` type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringtobigint
    #[inline]
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
    pub fn equal(x: &Self, y: &Self) -> bool {
        x == y
    }

    #[inline]
    pub fn pow(x: &Self, y: &Self, context: &mut Context) -> Result<Self, JsValue> {
        let y = if let Some(y) = y.inner.to_biguint() {
            y
        } else {
            return context.throw_range_error("BigInt negative exponent");
        };

        let num_bits = (x.inner.bits() as f64
            * y.to_f64().expect("Unable to convert from BigUInt to f64"))
        .floor()
            + 1f64;

        if num_bits > 1_000_000_000f64 {
            return context.throw_range_error("Maximum BigInt size exceeded");
        }

        Ok(Self::new(x.inner.as_ref().clone().pow(y)))
    }

    #[inline]
    pub fn shift_right(x: &Self, y: &Self, context: &mut Context) -> Result<Self, JsValue> {
        if let Some(n) = y.inner.to_i32() {
            let inner = if n > 0 {
                x.inner.as_ref().clone().shr(n as usize)
            } else {
                x.inner.as_ref().clone().shl(n.unsigned_abs())
            };

            Ok(Self::new(inner))
        } else {
            context.throw_range_error("Maximum BigInt size exceeded")
        }
    }

    #[inline]
    pub fn shift_left(x: &Self, y: &Self, context: &mut Context) -> Result<Self, JsValue> {
        if let Some(n) = y.inner.to_i32() {
            let inner = if n > 0 {
                x.inner.as_ref().clone().shl(n as usize)
            } else {
                x.inner.as_ref().clone().shr(n.unsigned_abs())
            };

            Ok(Self::new(inner))
        } else {
            context.throw_range_error("Maximum BigInt size exceeded")
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
    pub fn mod_floor(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.mod_floor(&y.inner))
    }

    #[inline]
    pub fn add(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().add(y.inner.as_ref()))
    }

    #[inline]
    pub fn sub(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().sub(y.inner.as_ref()))
    }

    #[inline]
    pub fn mul(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().mul(y.inner.as_ref()))
    }

    #[inline]
    pub fn div(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().div(y.inner.as_ref()))
    }

    #[inline]
    pub fn rem(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().rem(y.inner.as_ref()))
    }

    #[inline]
    pub fn bitand(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().bitand(y.inner.as_ref()))
    }

    #[inline]
    pub fn bitor(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().bitor(y.inner.as_ref()))
    }

    #[inline]
    pub fn bitxor(x: &Self, y: &Self) -> Self {
        Self::new(x.inner.as_ref().clone().bitxor(y.inner.as_ref()))
    }

    #[inline]
    pub fn neg(x: &Self) -> Self {
        Self::new(x.as_inner().neg())
    }

    #[inline]
    pub fn not(x: &Self) -> Self {
        Self::new(!x.as_inner())
    }

    #[inline]
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
        match RawBigInt::from_f64(n) {
            Some(bigint) => Ok(Self::new(bigint)),
            None => Err(TryFromF64Error),
        }
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
        if other.fract() != 0.0 {
            return false;
        }

        self.inner.as_ref() == &RawBigInt::from(*other as i64)
    }
}

impl PartialEq<JsBigInt> for f64 {
    #[inline]
    fn eq(&self, other: &JsBigInt) -> bool {
        if self.fract() != 0.0 {
            return false;
        }

        &RawBigInt::from(*self as i64) == other.inner.as_ref()
    }
}
