use super::BigInt;

use crate::{builtins::Number, Context, Value};
use num_traits::cast::{FromPrimitive, ToPrimitive};

use std::convert::TryFrom;
use std::str::FromStr;

impl BigInt {
    /// This function takes a string and conversts it to BigInt type.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-stringtobigint
    #[inline]
    pub(crate) fn from_string(string: &str, context: &mut Context) -> Result<Self, Value> {
        if string.trim().is_empty() {
            return Ok(BigInt::from(0));
        }

        let mut radix = 10;
        let mut string = string;
        if string.starts_with("0b") || string.starts_with("0B") {
            radix = 2;
            string = &string[2..];
        }
        if string.starts_with("0x") || string.starts_with("0X") {
            radix = 16;
            string = &string[2..];
        }
        if string.starts_with("0o") || string.starts_with("0O") {
            radix = 8;
            string = &string[2..];
        }

        BigInt::from_string_radix(string, radix).ok_or_else(|| {
            context.construct_syntax_error(format!("cannot convert {} to a BigInt", string))
        })
    }

    /// Converts a string to a BigInt with the specified radix.
    #[inline]
    pub fn from_string_radix(buf: &str, radix: u32) -> Option<Self> {
        num_bigint::BigInt::parse_bytes(buf.as_bytes(), radix).map(Self)
    }

    /// Convert bigint to string with radix.
    #[inline]
    pub fn to_string_radix(&self, radix: u32) -> String {
        self.0.to_str_radix(radix)
    }

    /// Converts the BigInt to a f64 type.
    ///
    /// Returns `std::f64::INFINITY` if the BigInt is too big.
    #[inline]
    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().unwrap_or(f64::INFINITY)
    }

    #[inline]
    pub(crate) fn from_str(string: &str) -> Option<Self> {
        match num_bigint::BigInt::from_str(string) {
            Ok(bigint) => Some(BigInt(bigint)),
            Err(_) => None,
        }
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
        // If the truncated version of the number is not the
        // same as the non-truncated version then the floating-point
        // number conains a fractional part.
        if !Number::equal(n.trunc(), n) {
            return Err(TryFromF64Error);
        }
        match num_bigint::BigInt::from_f64(n) {
            Some(bigint) => Ok(BigInt(bigint)),
            None => Err(TryFromF64Error),
        }
    }
}
