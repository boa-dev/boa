//! This module implements the global `BigInt` object.
//!
//! `BigInt` is a built-in object that provides a way to represent whole numbers larger
//! than the largest number JavaScript can reliably represent with the Number primitive
//! and represented by the `Number.MAX_SAFE_INTEGER` constant.
//! `BigInt` can be used for arbitrarily large integers.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-bigint-objects
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt

use crate::{
    builtins::{
        function::{make_builtin_fn, make_constructor_fn},
        value::{ResultValue, Value},
        RangeError,
    },
    exec::Interpreter,
    syntax::ast::bigint::BigInt as AstBigInt,
};

#[cfg(test)]
mod tests;

/// `BigInt` implementation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct BigInt;

impl BigInt {
    /// `BigInt()`
    ///
    /// The `BigInt()` constructor is used to create BigInt objects.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint-objects
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/BigInt
    pub(crate) fn make_bigint(
        _this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        let data = match args.get(0) {
            Some(ref value) => {
                if let Some(bigint) = value.to_bigint() {
                    Value::from(bigint)
                } else {
                    return Err(RangeError::run_new(
                        format!(
                            "{} can't be converted to BigInt because it isn't an integer",
                            value
                        ),
                        ctx,
                    )?);
                }
            }
            None => Value::from(AstBigInt::from(0)),
        };
        Ok(data)
    }

    /// `BigInt.prototype.toString( [radix] )`
    ///
    /// The `toString()` method returns a string representing the specified BigInt object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/toString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(
        this: &mut Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        let radix = if !args.is_empty() {
            args[0].to_integer()
        } else {
            10
        };
        if radix < 2 && radix > 36 {
            return Err(RangeError::run_new(
                "radix must be an integer at least 2 and no greater than 36",
                ctx,
            )?);
        }
        Ok(Value::from(
            this.to_bigint().unwrap().to_str_radix(radix as u32),
        ))
    }

    // /// `BigInt.prototype.valueOf()`
    // ///
    // /// The `valueOf()` method returns the wrapped primitive value of a Number object.
    // ///
    // /// More information:
    // ///  - [ECMAScript reference][spec]
    // ///  - [MDN documentation][mdn]
    // ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint.prototype.valueof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/valueOf
    pub(crate) fn value_of(
        this: &mut Value,
        _args: &[Value],
        _ctx: &mut Interpreter,
    ) -> ResultValue {
        Ok(Value::from(
            this.to_bigint().expect("BigInt.prototype.valueOf"),
        ))
    }

    /// Create a new `Number` object
    pub(crate) fn create(global: &Value) -> Value {
        let prototype = Value::new_object(Some(global));
        prototype.set_internal_slot("BigIntData", Value::from(AstBigInt::from(0)));

        make_builtin_fn(Self::to_string, "toString", &prototype, 1);
        make_builtin_fn(Self::value_of, "valueOf", &prototype, 0);

        make_constructor_fn(Self::make_bigint, global, prototype)
    }

    /// Initialise the `BigInt` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) {
        global.set_field("BigInt", Self::create(global));
    }
}
