use super::{JsBigInt, JsObject, JsResult, JsValue, PreferredType};
use crate::{builtins::Number, Context};

impl JsValue {
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
            (Self::BigInt(x), Self::BigInt(y)) => JsBigInt::equal(x, y),
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
            (_, _) => Self::same_value_non_numeric(self, other),
        }
    }

    /// Abstract equality comparison.
    ///
    /// This method is executed when doing abstract equality comparisons with the `==` operator.
    ///  For more information, check <https://tc39.es/ecma262/#sec-abstract-equality-comparison>
    #[allow(clippy::float_cmp)]
    pub fn equals(&self, other: &Self, context: &mut Context) -> JsResult<bool> {
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
            (Self::Integer(_) | Self::Rational(_), Self::String(_) | Self::Boolean(_))
            | (Self::String(_), Self::Integer(_) | Self::Rational(_)) => {
                let x = self.to_number(context)?;
                let y = other.to_number(context)?;
                Number::equal(x, y)
            }

            // 6. If Type(x) is BigInt and Type(y) is String, then
            //    a. Let n be ! StringToBigInt(y).
            //    b. If n is NaN, return false.
            //    c. Return the result of the comparison x == n.
            (Self::BigInt(ref a), Self::String(ref b)) => match JsBigInt::from_string(b) {
                Some(ref b) => a == b,
                None => false,
            },

            // 7. If Type(x) is String and Type(y) is BigInt, return the result of the comparison y == x.
            (Self::String(ref a), Self::BigInt(ref b)) => match JsBigInt::from_string(a) {
                Some(ref a) => a == b,
                None => false,
            },

            // 8. If Type(x) is Boolean, return the result of the comparison ! ToNumber(x) == y.
            (Self::Boolean(x), _) => return other.equals(&Self::new(i32::from(*x)), context),

            // 9. If Type(y) is Boolean, return the result of the comparison x == ! ToNumber(y).
            (_, Self::Boolean(y)) => return self.equals(&Self::new(i32::from(*y)), context),

            // 10. If Type(x) is either String, Number, BigInt, or Symbol and Type(y) is Object, return the result
            // of the comparison x == ? ToPrimitive(y).
            (
                Self::Object(_),
                Self::String(_)
                | Self::Rational(_)
                | Self::Integer(_)
                | Self::BigInt(_)
                | Self::Symbol(_),
            ) => {
                let primitive = self.to_primitive(context, PreferredType::Default)?;
                return Ok(primitive
                    .equals(other, context)
                    .expect("should not fail according to spec"));
            }

            // 11. If Type(x) is Object and Type(y) is either String, Number, BigInt, or Symbol, return the result
            // of the comparison ? ToPrimitive(x) == y.
            (
                Self::String(_)
                | Self::Rational(_)
                | Self::Integer(_)
                | Self::BigInt(_)
                | Self::Symbol(_),
                Self::Object(_),
            ) => {
                let primitive = other.to_primitive(context, PreferredType::Default)?;
                return Ok(primitive
                    .equals(self, context)
                    .expect("should not fail according to spec"));
            }

            // 12. If Type(x) is BigInt and Type(y) is Number, or if Type(x) is Number and Type(y) is BigInt, then
            //    a. If x or y are any of NaN, +∞, or -∞, return false.
            //    b. If the mathematical value of x is equal to the mathematical value of y, return true; otherwise return false.
            (Self::BigInt(ref a), Self::Rational(ref b)) => a == b,
            (Self::Rational(ref a), Self::BigInt(ref b)) => a == b,
            (Self::BigInt(ref a), Self::Integer(ref b)) => a == b,
            (Self::Integer(ref a), Self::BigInt(ref b)) => a == b,

            // 13. Return false.
            _ => false,
        })
    }

    /// The internal comparison abstract operation SameValue(x, y),
    /// where x and y are ECMAScript language values, produces true or false.
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-samevalue
    pub fn same_value(x: &Self, y: &Self) -> bool {
        // 1. If Type(x) is different from Type(y), return false.
        if x.get_type() != y.get_type() {
            return false;
        }

        match (x, y) {
            // 2. If Type(x) is Number or BigInt, then
            //    a. Return ! Type(x)::SameValue(x, y).
            (Self::BigInt(x), Self::BigInt(y)) => JsBigInt::same_value(x, y),
            (Self::Rational(x), Self::Rational(y)) => Number::same_value(*x, *y),
            (Self::Rational(x), Self::Integer(y)) => Number::same_value(*x, f64::from(*y)),
            (Self::Integer(x), Self::Rational(y)) => Number::same_value(f64::from(*x), *y),
            (Self::Integer(x), Self::Integer(y)) => x == y,

            // 3. Return ! SameValueNonNumeric(x, y).
            (_, _) => Self::same_value_non_numeric(x, y),
        }
    }

    /// The internal comparison abstract operation `SameValueZero(x, y)`,
    /// where `x` and `y` are ECMAScript language values, produces `true` or `false`.
    ///
    /// `SameValueZero` differs from `SameValue` only in its treatment of `+0` and `-0`.
    ///
    /// More information:
    ///  - [ECMAScript][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-samevaluezero
    pub fn same_value_zero(x: &Self, y: &Self) -> bool {
        if x.get_type() != y.get_type() {
            return false;
        }

        match (x, y) {
            // 2. If Type(x) is Number or BigInt, then
            //    a. Return ! Type(x)::SameValueZero(x, y).
            (JsValue::BigInt(x), JsValue::BigInt(y)) => JsBigInt::same_value_zero(x, y),

            (JsValue::Rational(x), JsValue::Rational(y)) => Number::same_value_zero(*x, *y),
            (JsValue::Rational(x), JsValue::Integer(y)) => {
                Number::same_value_zero(*x, f64::from(*y))
            }
            (JsValue::Integer(x), JsValue::Rational(y)) => {
                Number::same_value_zero(f64::from(*x), *y)
            }
            (JsValue::Integer(x), JsValue::Integer(y)) => x == y,

            // 3. Return ! SameValueNonNumeric(x, y).
            (_, _) => Self::same_value_non_numeric(x, y),
        }
    }

    fn same_value_non_numeric(x: &Self, y: &Self) -> bool {
        debug_assert!(x.get_type() == y.get_type());
        match (x, y) {
            (JsValue::Null, JsValue::Null) | (JsValue::Undefined, JsValue::Undefined) => true,
            (JsValue::String(ref x), JsValue::String(ref y)) => x == y,
            (JsValue::Boolean(x), JsValue::Boolean(y)) => x == y,
            (JsValue::Object(ref x), JsValue::Object(ref y)) => JsObject::equals(x, y),
            (JsValue::Symbol(ref x), JsValue::Symbol(ref y)) => x == y,
            _ => false,
        }
    }
}
