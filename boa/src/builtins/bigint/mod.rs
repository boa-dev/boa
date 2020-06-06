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
        value::{ResultValue, Value, ValueData},
    },
    exec::Interpreter,
    BoaProfiler,
};

use gc::{unsafe_empty_trace, Finalize, Trace};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod conversions;
pub mod equality;
pub mod operations;

pub use conversions::*;
pub use equality::*;
pub use operations::*;

#[cfg(test)]
mod tests;

/// `BigInt` implementation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BigInt(num_bigint::BigInt);

impl BigInt {
    /// The abstract operation thisBigIntValue takes argument value.
    ///
    /// The phrase “this BigInt value” within the specification of a method refers to the
    /// result returned by calling the abstract operation thisBigIntValue with the `this` value
    /// of the method invocation passed as the argument.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thisbigintvalue
    #[inline]
    fn this_bigint_value(value: &Value, ctx: &mut Interpreter) -> Result<Self, Value> {
        match value.data() {
            // 1. If Type(value) is BigInt, return value.
            ValueData::BigInt(ref bigint) => return Ok(bigint.clone()),

            // 2. If Type(value) is Object and value has a [[BigIntData]] internal slot, then
            //    a. Assert: Type(value.[[BigIntData]]) is BigInt.
            //    b. Return value.[[BigIntData]].
            ValueData::Object(_) => {
                let bigint = value.get_internal_slot("BigIntData");
                if let ValueData::BigInt(bigint) = bigint.data() {
                    return Ok(bigint.clone());
                }
            }
            _ => {}
        }

        // 3. Throw a TypeError exception.
        ctx.throw_type_error("'this' is not a BigInt")?;
        unreachable!();
    }

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
            Some(ref value) => Value::from(ctx.to_bigint(value)?),
            None => Value::from(Self::from(0)),
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
            return ctx
                .throw_range_error("radix must be an integer at least 2 and no greater than 36");
        }
        Ok(Value::from(
            Self::this_bigint_value(this, ctx)?.to_string_radix(radix as u32),
        ))
    }

    /// `BigInt.prototype.valueOf()`
    ///
    /// The `valueOf()` method returns the wrapped primitive value of a Number object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint.prototype.valueof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/valueOf
    pub(crate) fn value_of(
        this: &mut Value,
        _args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        Ok(Value::from(Self::this_bigint_value(this, ctx)?))
    }

    /// Create a new `Number` object
    pub(crate) fn create(global: &Value) -> Value {
        let prototype = Value::new_object(Some(global));
        prototype.set_internal_slot("BigIntData", Value::from(Self::from(0)));

        make_builtin_fn(Self::to_string, "toString", &prototype, 1);
        make_builtin_fn(Self::value_of, "valueOf", &prototype, 0);

        make_constructor_fn("BigInt", 1, Self::make_bigint, global, prototype, false)
    }

    /// Initialise the `BigInt` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) {
        let _timer = BoaProfiler::global().start_event("bigint", "init");
        global.set_field("BigInt", Self::create(global));
    }
}

impl Finalize for BigInt {}
unsafe impl Trace for BigInt {
    // BigInt type implements an empty trace becasue the inner structure
    // `num_bigint::BigInt` does not implement `Trace` trait.
    // If it did this would be unsound.
    unsafe_empty_trace!();
}
