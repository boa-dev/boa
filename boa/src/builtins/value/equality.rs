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

        match (self.data(), other.data()) {
            // 2. If Type(x) is Number or BigInt, then
            //    a. Return ! Type(x)::equal(x, y).
            (ValueData::BigInt(x), ValueData::BigInt(y)) => BigInt::equal(x, y),
            (ValueData::Rational(x), ValueData::Rational(y)) => Number::equal(*x, *y),
            (ValueData::Rational(x), ValueData::Integer(y)) => Number::equal(*x, f64::from(*y)),
            (ValueData::Integer(x), ValueData::Rational(y)) => Number::equal(f64::from(*x), *y),
            (ValueData::Integer(x), ValueData::Integer(y)) => x == y,

            //Null has to be handled specially because "typeof null" returns object and if we managed
            //this without a special case we would compare self and other as if they were actually
            //objects which unfortunately fails
            //Specification Link: https://tc39.es/ecma262/#sec-typeof-operator
            (ValueData::Null, ValueData::Null) => true,

            // 3. Return ! SameValueNonNumeric(x, y).
            (_, _) => same_value_non_numeric(self, other),
        }
    }

    /// Abstract equality comparison.
    ///
    /// This method is executed when doing abstract equality comparisons with the `==` operator.
    ///  For more information, check <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
    #[allow(clippy::float_cmp)]
    pub fn equals(&mut self, other: &mut Self, interpreter: &mut Interpreter) -> bool {
        // 1. If Type(x) is the same as Type(y), then
        //     a. Return the result of performing Strict Equality Comparison x === y.
        if self.get_type() == other.get_type() {
            return self.strict_equals(other);
        }

        match (self.data(), other.data()) {
            // 2. If x is null and y is undefined, return true.
            // 3. If x is undefined and y is null, return true.
            _ if self.is_null_or_undefined() && other.is_null_or_undefined() => true,

            // 3. If Type(x) is Number and Type(y) is String, return the result of the comparison x == ! ToNumber(y).
            // 4. If Type(x) is String and Type(y) is Number, return the result of the comparison ! ToNumber(x) == y.
            //
            // https://github.com/rust-lang/rust/issues/54883
            (ValueData::Integer(_), ValueData::String(_))
            | (ValueData::Rational(_), ValueData::String(_))
            | (ValueData::String(_), ValueData::Integer(_))
            | (ValueData::String(_), ValueData::Rational(_))
            | (ValueData::Rational(_), ValueData::Boolean(_))
            | (ValueData::Integer(_), ValueData::Boolean(_)) => {
                let a: &Value = self.borrow();
                let b: &Value = other.borrow();
                Number::equal(f64::from(a), f64::from(b))
            }

            // 6. If Type(x) is BigInt and Type(y) is String, then
            //    a. Let n be ! StringToBigInt(y).
            //    b. If n is NaN, return false.
            //    c. Return the result of the comparison x == n.
            (ValueData::BigInt(ref a), ValueData::String(ref b)) => match string_to_bigint(b) {
                Some(ref b) => a == b,
                None => false,
            },

            // 7. If Type(x) is String and Type(y) is BigInt, return the result of the comparison y == x.
            (ValueData::String(ref a), ValueData::BigInt(ref b)) => match string_to_bigint(a) {
                Some(ref a) => a == b,
                None => false,
            },

            // 8. If Type(x) is Boolean, return the result of the comparison ! ToNumber(x) == y.
            (ValueData::Boolean(_), _) => {
                other.equals(&mut Value::from(self.to_integer()), interpreter)
            }

            // 9. If Type(y) is Boolean, return the result of the comparison x == ! ToNumber(y).
            (_, ValueData::Boolean(_)) => {
                self.equals(&mut Value::from(other.to_integer()), interpreter)
            }

            // 10. If Type(x) is either String, Number, BigInt, or Symbol and Type(y) is Object, return the result
            // of the comparison x == ? ToPrimitive(y).
            (ValueData::Object(_), _) => {
                let mut primitive = interpreter.to_primitive(self, PreferredType::Default);
                primitive.equals(other, interpreter)
            }

            // 11. If Type(x) is Object and Type(y) is either String, Number, BigInt, or Symbol, return the result
            // of the comparison ? ToPrimitive(x) == y.
            (_, ValueData::Object(_)) => {
                let mut primitive = interpreter.to_primitive(other, PreferredType::Default);
                primitive.equals(self, interpreter)
            }

            // 12. If Type(x) is BigInt and Type(y) is Number, or if Type(x) is Number and Type(y) is BigInt, then
            //    a. If x or y are any of NaN, +∞, or -∞, return false.
            //    b. If the mathematical value of x is equal to the mathematical value of y, return true; otherwise return false.
            (ValueData::BigInt(ref a), ValueData::Rational(ref b)) => a == b,
            (ValueData::Rational(ref a), ValueData::BigInt(ref b)) => a == b,
            (ValueData::BigInt(ref a), ValueData::Integer(ref b)) => a == b,
            (ValueData::Integer(ref a), ValueData::BigInt(ref b)) => a == b,

            // 13. Return false.
            _ => false,
        }
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
/// Such a comparison is performed as follows:
///
/// https://tc39.es/ecma262/#sec-samevalue
/// strict mode currently compares the pointers
pub fn same_value(x: &Value, y: &Value, strict: bool) -> bool {
    if strict {
        // Do both Values point to the same underlying valueData?
        return std::ptr::eq(x.data(), y.data());
    }

    // 1. If Type(x) is different from Type(y), return false.
    if x.get_type() != y.get_type() {
        return false;
    }

    match (x.data(), y.data()) {
        // 2. If Type(x) is Number or BigInt, then
        //    a. Return ! Type(x)::SameValue(x, y).
        (ValueData::BigInt(x), ValueData::BigInt(y)) => BigInt::same_value(x, y),
        (ValueData::Rational(x), ValueData::Rational(y)) => Number::same_value(*x, *y),
        (ValueData::Rational(x), ValueData::Integer(y)) => Number::same_value(*x, f64::from(*y)),
        (ValueData::Integer(x), ValueData::Rational(y)) => Number::same_value(f64::from(*x), *y),
        (ValueData::Integer(x), ValueData::Integer(y)) => x == y,

        // 3. Return ! SameValueNonNumeric(x, y).
        (_, _) => same_value_non_numeric(x, y),
    }
}

/// The internal comparison abstract operation SameValueZero(x, y),
/// where x and y are ECMAScript language values, produces true or false.
/// SameValueZero differs from SameValue only in its treatment of +0 and -0.
///
/// Such a comparison is performed as follows:
///
/// <https://tc39.es/ecma262/#sec-samevaluezero>
pub fn same_value_zero(x: &Value, y: &Value) -> bool {
    if x.get_type() != y.get_type() {
        return false;
    }

    match (x.data(), y.data()) {
        // 2. If Type(x) is Number or BigInt, then
        //    a. Return ! Type(x)::SameValueZero(x, y).
        (ValueData::BigInt(x), ValueData::BigInt(y)) => BigInt::same_value_zero(x, y),

        (ValueData::Rational(x), ValueData::Rational(y)) => Number::same_value_zero(*x, *y),
        (ValueData::Rational(x), ValueData::Integer(y)) => {
            Number::same_value_zero(*x, f64::from(*y))
        }
        (ValueData::Integer(x), ValueData::Rational(y)) => {
            Number::same_value_zero(f64::from(*x), *y)
        }
        (ValueData::Integer(x), ValueData::Integer(y)) => x == y,

        // 3. Return ! SameValueNonNumeric(x, y).
        (_, _) => same_value_non_numeric(x, y),
    }
}

fn same_value_non_numeric(x: &Value, y: &Value) -> bool {
    debug_assert!(x.get_type() == y.get_type());
    match x.get_type() {
        Type::Null | Type::Undefined => true,
        Type::String => x.to_string() == y.to_string(),
        Type::Boolean => bool::from(x) == bool::from(y),
        Type::Object => std::ptr::eq(x.data(), y.data()),
        _ => false,
    }
}
