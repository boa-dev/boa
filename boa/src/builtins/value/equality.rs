use super::*;
use crate::{builtins::Number, exec::PreferredType, Interpreter};

use std::borrow::Borrow;

impl Value {
    /// Strict equality comparison.
    ///
    /// This method is executed when doing strict equality comparisons with the `===` operator.
    /// For more information, check <https://tc39.es/ecma262/#sec-strict-equality-comparison>.
    pub fn strict_equals(&self, other: &Self) -> bool {
        // 1. If Type(x) is different from Type(y), return false.
        if self.get_type() != other.get_type() {
            return false;
        }

        match (self, other) {
            // 2. If Type(x) is Number or BigInt, then
            //    a. Return ! Type(x)::equal(x, y).
            (Self::BigInt(x), Self::BigInt(y)) => BigInt::equal(x, y),
            (Self::Rational(x), Self::Rational(y)) => Number::equal(*x, *y),
            (Self::Rational(x), Self::Integer(y)) => Number::equal(*x, f64::from(*y)),
            (Self::Integer(x), Self::Rational(y)) => Number::equal(f64::from(*x), *y),
            (Self::Integer(x), Self::Integer(y)) => x == y,

            //Null has to be handled specially because "typeof null" returns object and if we managed
            //this without a special case we would compare self and other as if they were actually
            //objects which unfortunately fails
            //Specification Link: https://tc39.es/ecma262/#sec-typeof-operator
            (Self::Null, Self::Null) => true,

            // 3. Return ! SameValueNonNumeric(x, y).
            (_, _) => same_value_non_numeric(self, other),
        }
    }

    /// Abstract equality comparison.
    ///
    /// This method is executed when doing abstract equality comparisons with the `==` operator.
    ///  For more information, check <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
    #[allow(clippy::float_cmp)]
    pub fn equals(&self, other: &Self, interpreter: &mut Interpreter) -> Result<bool, Value> {
        // 1. If Type(x) is the same as Type(y), then
        //     a. Return the result of performing Strict Equality Comparison x === y.
        if self.get_type() == other.get_type() {
            return Ok(self.strict_equals(other));
        }

        Ok(match (self, other) {
            // 2. If x is null and y is undefined, return true.
            // 3. If x is undefined and y is null, return true.
            (Self::Null, Self::Undefined) | (Self::Undefined, Self::Null) => true,

            // 3. If Type(x) is Number and Type(y) is String, return the result of the comparison x == ! ToNumber(y).
            // 4. If Type(x) is String and Type(y) is Number, return the result of the comparison ! ToNumber(x) == y.
            //
            // https://github.com/rust-lang/rust/issues/54883
            (Self::Integer(_), Self::String(_))
            | (Self::Rational(_), Self::String(_))
            | (Self::String(_), Self::Integer(_))
            | (Self::String(_), Self::Rational(_))
            | (Self::Rational(_), Self::Boolean(_))
            | (Self::Integer(_), Self::Boolean(_)) => {
                let a: &Value = self.borrow();
                let b: &Value = other.borrow();
                Number::equal(f64::from(a), f64::from(b))
            }

            // 6. If Type(x) is BigInt and Type(y) is String, then
            //    a. Let n be ! StringToBigInt(y).
            //    b. If n is NaN, return false.
            //    c. Return the result of the comparison x == n.
            (Self::BigInt(ref a), Self::String(ref b)) => match string_to_bigint(b) {
                Some(ref b) => a.as_inner() == b,
                None => false,
            },

            // 7. If Type(x) is String and Type(y) is BigInt, return the result of the comparison y == x.
            (Self::String(ref a), Self::BigInt(ref b)) => match string_to_bigint(a) {
                Some(ref a) => a == b.as_inner(),
                None => false,
            },

            // 8. If Type(x) is Boolean, return the result of the comparison ! ToNumber(x) == y.
            (Self::Boolean(_), _) => {
                return other.equals(&Value::from(self.to_integer()), interpreter)
            }

            // 9. If Type(y) is Boolean, return the result of the comparison x == ! ToNumber(y).
            (_, Self::Boolean(_)) => {
                return self.equals(&Value::from(other.to_integer()), interpreter)
            }

            // 10. If Type(x) is either String, Number, BigInt, or Symbol and Type(y) is Object, return the result
            // of the comparison x == ? ToPrimitive(y).
            (Self::Object(_), _) => {
                let primitive = interpreter.to_primitive(self, PreferredType::Default)?;
                return primitive.equals(other, interpreter);
            }

            // 11. If Type(x) is Object and Type(y) is either String, Number, BigInt, or Symbol, return the result
            // of the comparison ? ToPrimitive(x) == y.
            (_, Self::Object(_)) => {
                let primitive = interpreter.to_primitive(other, PreferredType::Default)?;
                return primitive.equals(self, interpreter);
            }

            // 12. If Type(x) is BigInt and Type(y) is Number, or if Type(x) is Number and Type(y) is BigInt, then
            //    a. If x or y are any of NaN, +∞, or -∞, return false.
            //    b. If the mathematical value of x is equal to the mathematical value of y, return true; otherwise return false.
            (Self::BigInt(ref a), Self::Rational(ref b)) => a.as_inner() == b,
            (Self::Rational(ref a), Self::BigInt(ref b)) => a == b.as_inner(),
            (Self::BigInt(ref a), Self::Integer(ref b)) => a.as_inner() == b,
            (Self::Integer(ref a), Self::BigInt(ref b)) => a == b.as_inner(),

            // 13. Return false.
            _ => false,
        })
    }
}

/// This function takes a string and conversts it to BigInt type.
///
/// If the result is `NaN` than `None` is returned.
///
/// More information:
///  - [ECMAScript reference][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-stringtobigint
pub fn string_to_bigint(string: &str) -> Option<BigInt> {
    if string.is_empty() {
        return Some(BigInt::from(0));
    }

    BigInt::from_str(string)
}

/// The internal comparison abstract operation SameValue(x, y),
/// where x and y are ECMAScript language values, produces true or false.
///
/// More information:
///  - [ECMAScript][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-samevalue
pub fn same_value(x: &Value, y: &Value) -> bool {
    // 1. If Type(x) is different from Type(y), return false.
    if x.get_type() != y.get_type() {
        return false;
    }

    match (x, y) {
        // 2. If Type(x) is Number or BigInt, then
        //    a. Return ! Type(x)::SameValue(x, y).
        (Value::BigInt(x), Value::BigInt(y)) => BigInt::same_value(x, y),
        (Value::Rational(x), Value::Rational(y)) => Number::same_value(*x, *y),
        (Value::Rational(x), Value::Integer(y)) => Number::same_value(*x, f64::from(*y)),
        (Value::Integer(x), Value::Rational(y)) => Number::same_value(f64::from(*x), *y),
        (Value::Integer(x), Value::Integer(y)) => x == y,

        // 3. Return ! SameValueNonNumeric(x, y).
        (_, _) => same_value_non_numeric(x, y),
    }
}

/// The internal comparison abstract operation `SameValueZero(x, y)`,
/// where `x` and `y` are ECMAScript language values, produces `true` or `false`.
///
/// `SameValueZero` differs from SameValue only in its treatment of `+0` and `-0`.
///
/// More information:
///  - [ECMAScript][spec]
///
/// [spec]: https://tc39.es/ecma262/#sec-samevaluezero
pub fn same_value_zero(x: &Value, y: &Value) -> bool {
    if x.get_type() != y.get_type() {
        return false;
    }

    match (x, y) {
        // 2. If Type(x) is Number or BigInt, then
        //    a. Return ! Type(x)::SameValueZero(x, y).
        (Value::BigInt(x), Value::BigInt(y)) => BigInt::same_value_zero(x, y),

        (Value::Rational(x), Value::Rational(y)) => Number::same_value_zero(*x, *y),
        (Value::Rational(x), Value::Integer(y)) => Number::same_value_zero(*x, f64::from(*y)),
        (Value::Integer(x), Value::Rational(y)) => Number::same_value_zero(f64::from(*x), *y),
        (Value::Integer(x), Value::Integer(y)) => x == y,

        // 3. Return ! SameValueNonNumeric(x, y).
        (_, _) => same_value_non_numeric(x, y),
    }
}

fn same_value_non_numeric(x: &Value, y: &Value) -> bool {
    debug_assert!(x.get_type() == y.get_type());
    match (x, y) {
        (Value::Null, Value::Null) | (Value::Undefined, Value::Undefined) => true,
        (Value::String(ref x), Value::String(ref y)) => x == y,
        (Value::Boolean(x), Value::Boolean(y)) => x == y,
        (Value::Object(ref x), Value::Object(ref y)) => GcObject::equals(x, y),
        _ => false,
    }
}
