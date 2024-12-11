use super::{InnerValue, JsBigInt, JsObject, JsResult, JsValue, PreferredType};
use crate::{builtins::Number, Context};

impl JsValue {
    /// Strict equality comparison.
    ///
    /// This method is executed when doing strict equality comparisons with the `===` operator.
    /// For more information, check <https://tc39.es/ecma262/#sec-strict-equality-comparison>.
    #[must_use]
    pub fn strict_equals(&self, other: &Self) -> bool {
        // 1. If Type(x) is different from Type(y), return false.
        if self.get_type() != other.get_type() {
            return false;
        }

        match (&self.inner, &other.inner) {
            // 2. If Type(x) is Number or BigInt, then
            //    a. Return ! Type(x)::equal(x, y).
            (InnerValue::BigInt(x), InnerValue::BigInt(y)) => JsBigInt::equal(x, y),
            (InnerValue::Float64(x), InnerValue::Float64(y)) => Number::equal(*x, *y),
            (InnerValue::Float64(x), InnerValue::Integer32(y)) => Number::equal(*x, f64::from(*y)),
            (InnerValue::Integer32(x), InnerValue::Float64(y)) => Number::equal(f64::from(*x), *y),
            (InnerValue::Integer32(x), InnerValue::Integer32(y)) => x == y,

            //Null has to be handled specially because "typeof null" returns object and if we managed
            //this without a special case we would compare self and other as if they were actually
            //objects which unfortunately fails
            //Specification Link: https://tc39.es/ecma262/#sec-typeof-operator
            (InnerValue::Null, InnerValue::Null) => true,

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

        Ok(match (&self.inner, &other.inner) {
            // 2. If x is null and y is undefined, return true.
            // 3. If x is undefined and y is null, return true.
            (InnerValue::Null, InnerValue::Undefined)
            | (InnerValue::Undefined, InnerValue::Null) => true,

            // 3. If Type(x) is Number and Type(y) is String, return the result of the comparison x == ! ToNumber(y).
            // 4. If Type(x) is String and Type(y) is Number, return the result of the comparison ! ToNumber(x) == y.
            //
            // https://github.com/rust-lang/rust/issues/54883
            (
                InnerValue::Integer32(_) | InnerValue::Float64(_),
                InnerValue::String(_) | InnerValue::Boolean(_),
            )
            | (InnerValue::String(_), InnerValue::Integer32(_) | InnerValue::Float64(_)) => {
                let x = self.to_number(context)?;
                let y = other.to_number(context)?;
                Number::equal(x, y)
            }

            // 6. If Type(x) is BigInt and Type(y) is String, then
            //    a. Let n be ! StringToBigInt(y).
            //    b. If n is NaN, return false.
            //    c. Return the result of the comparison x == n.
            (InnerValue::BigInt(ref a), InnerValue::String(ref b)) => JsBigInt::from_js_string(b)
                .as_ref()
                .map_or(false, |b| a == b),

            // 7. If Type(x) is String and Type(y) is BigInt, return the result of the comparison y == x.
            (InnerValue::String(ref a), InnerValue::BigInt(ref b)) => JsBigInt::from_js_string(a)
                .as_ref()
                .map_or(false, |a| a == b),

            // 8. If Type(x) is Boolean, return the result of the comparison ! ToNumber(x) == y.
            (InnerValue::Boolean(x), _) => {
                return other.equals(&JsValue::new(i32::from(*x)), context)
            }

            // 9. If Type(y) is Boolean, return the result of the comparison x == ! ToNumber(y).
            (_, InnerValue::Boolean(y)) => {
                return self.equals(&JsValue::new(i32::from(*y)), context)
            }

            // 10. If Type(x) is either String, Number, BigInt, or Symbol and Type(y) is Object, return the result
            // of the comparison x == ? ToPrimitive(y).
            (
                InnerValue::Object(_),
                InnerValue::String(_)
                | InnerValue::Float64(_)
                | InnerValue::Integer32(_)
                | InnerValue::BigInt(_)
                | InnerValue::Symbol(_),
            ) => {
                let primitive = self.to_primitive(context, PreferredType::Default)?;
                return Ok(primitive
                    .equals(other, context)
                    .expect("should not fail according to spec"));
            }

            // 11. If Type(x) is Object and Type(y) is either String, Number, BigInt, or Symbol, return the result
            // of the comparison ? ToPrimitive(x) == y.
            (
                InnerValue::String(_)
                | InnerValue::Float64(_)
                | InnerValue::Integer32(_)
                | InnerValue::BigInt(_)
                | InnerValue::Symbol(_),
                InnerValue::Object(_),
            ) => {
                let primitive = other.to_primitive(context, PreferredType::Default)?;
                return Ok(primitive
                    .equals(self, context)
                    .expect("should not fail according to spec"));
            }

            // 12. If Type(x) is BigInt and Type(y) is Number, or if Type(x) is Number and Type(y) is BigInt, then
            //    a. If x or y are any of NaN, +∞, or -∞, return false.
            //    b. If the mathematical value of x is equal to the mathematical value of y, return true; otherwise return false.
            (InnerValue::BigInt(ref a), InnerValue::Float64(ref b)) => a == b,
            (InnerValue::Float64(ref a), InnerValue::BigInt(ref b)) => a == b,
            (InnerValue::BigInt(ref a), InnerValue::Integer32(ref b)) => a == b,
            (InnerValue::Integer32(ref a), InnerValue::BigInt(ref b)) => a == b,

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
    #[must_use]
    pub fn same_value(x: &Self, y: &Self) -> bool {
        // 1. If Type(x) is different from Type(y), return false.
        if x.get_type() != y.get_type() {
            return false;
        }

        match (&x.inner, &y.inner) {
            // 2. If Type(x) is Number or BigInt, then
            //    a. Return ! Type(x)::SameValue(x, y).
            (InnerValue::BigInt(x), InnerValue::BigInt(y)) => JsBigInt::same_value(x, y),
            (InnerValue::Float64(x), InnerValue::Float64(y)) => Number::same_value(*x, *y),
            (InnerValue::Float64(x), InnerValue::Integer32(y)) => {
                Number::same_value(*x, f64::from(*y))
            }
            (InnerValue::Integer32(x), InnerValue::Float64(y)) => {
                Number::same_value(f64::from(*x), *y)
            }
            (InnerValue::Integer32(x), InnerValue::Integer32(y)) => x == y,

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
    #[must_use]
    pub fn same_value_zero(x: &Self, y: &Self) -> bool {
        if x.get_type() != y.get_type() {
            return false;
        }

        match (&x.inner, &y.inner) {
            // 2. If Type(x) is Number or BigInt, then
            //    a. Return ! Type(x)::SameValueZero(x, y).
            (InnerValue::BigInt(x), InnerValue::BigInt(y)) => JsBigInt::same_value_zero(x, y),

            (InnerValue::Float64(x), InnerValue::Float64(y)) => Number::same_value_zero(*x, *y),
            (InnerValue::Float64(x), InnerValue::Integer32(y)) => {
                Number::same_value_zero(*x, f64::from(*y))
            }
            (InnerValue::Integer32(x), InnerValue::Float64(y)) => {
                Number::same_value_zero(f64::from(*x), *y)
            }
            (InnerValue::Integer32(x), InnerValue::Integer32(y)) => x == y,

            // 3. Return ! SameValueNonNumeric(x, y).
            (_, _) => Self::same_value_non_numeric(x, y),
        }
    }

    fn same_value_non_numeric(x: &Self, y: &Self) -> bool {
        debug_assert!(x.get_type() == y.get_type());
        match (&x.inner, &y.inner) {
            (InnerValue::Null, InnerValue::Null)
            | (InnerValue::Undefined, InnerValue::Undefined) => true,
            (InnerValue::String(x), InnerValue::String(y)) => x == y,
            (InnerValue::Boolean(x), InnerValue::Boolean(y)) => x == y,
            (InnerValue::Object(x), InnerValue::Object(y)) => JsObject::equals(x, y),
            (InnerValue::Symbol(x), InnerValue::Symbol(y)) => x == y,
            _ => false,
        }
    }
}
