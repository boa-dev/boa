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
    builtins::BuiltIn,
    object::{ConstructorBuilder, ObjectData},
    property::Attribute,
    value::{RcBigInt, Value},
    BoaProfiler, Context, Result,
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

impl BuiltIn for BigInt {
    const NAME: &'static str = "BigInt";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, Value, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let bigint_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().bigint_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::to_string, "toString", 1)
        .method(Self::value_of, "valueOf", 0)
        .static_method(Self::as_int_n, "asIntN", 2)
        .static_method(Self::as_uint_n, "asUintN", 2)
        .callable(true)
        .constructable(false)
        .build();

        (Self::NAME, bigint_object.into(), Self::attribute())
    }
}

impl BigInt {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 1;

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
    fn constructor(_: &Value, args: &[Value], context: &mut Context) -> Result<Value> {
        let data = match args.get(0) {
            Some(ref value) => value.to_bigint(context)?,
            None => RcBigInt::from(Self::from(0)),
        };
        Ok(Value::from(data))
    }

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
    fn this_bigint_value(value: &Value, ctx: &mut Context) -> Result<RcBigInt> {
        match value {
            // 1. If Type(value) is BigInt, return value.
            Value::BigInt(ref bigint) => return Ok(bigint.clone()),

            // 2. If Type(value) is Object and value has a [[BigIntData]] internal slot, then
            //    a. Assert: Type(value.[[BigIntData]]) is BigInt.
            //    b. Return value.[[BigIntData]].
            Value::Object(ref object) => {
                if let ObjectData::BigInt(ref bigint) = object.borrow().data {
                    return Ok(bigint.clone());
                }
            }
            _ => {}
        }

        // 3. Throw a TypeError exception.
        Err(ctx.construct_type_error("'this' is not a BigInt"))
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
    pub(crate) fn to_string(this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let radix = if !args.is_empty() {
            args[0].to_integer(ctx)? as i32
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
    pub(crate) fn value_of(this: &Value, _args: &[Value], ctx: &mut Context) -> Result<Value> {
        Ok(Value::from(Self::this_bigint_value(this, ctx)?))
    }

    /// `BigInt.asIntN()`
    ///
    /// The `BigInt.asIntN()` method wraps the value of a `BigInt` to a signed integer between `-2**(width - 1)` and `2**(width-1) - 1`.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint.asintn
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/asIntN
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn as_int_n(_this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let (modulo, bits) = Self::calculate_as_uint_n(args, ctx)?;

        if bits > 0 && modulo >= BigInt::from(2).pow(&BigInt::from(bits as i64 - 1)) {
            Ok(Value::from(
                modulo - BigInt::from(2).pow(&BigInt::from(bits as i64)),
            ))
        } else {
            Ok(Value::from(modulo))
        }
    }

    /// `BigInt.asUintN()`
    ///
    /// The `BigInt.asUintN()` method wraps the value of a `BigInt` to an unsigned integer between `0` and `2**(width) - 1`.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint.asuintn
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/asUintN
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn as_uint_n(_this: &Value, args: &[Value], ctx: &mut Context) -> Result<Value> {
        let (modulo, _) = Self::calculate_as_uint_n(args, ctx)?;

        Ok(Value::from(modulo))
    }

    /// Helper function to wrap the value of a `BigInt` to an unsigned integer.
    ///
    /// This function expects the same arguments as `as_uint_n` and wraps the value of a `BigInt`.
    /// Additionally to the wrapped unsigned value it returns the converted `bits` argument, so it
    /// can be reused from the `as_int_n` method.
    fn calculate_as_uint_n(args: &[Value], ctx: &mut Context) -> Result<(BigInt, u32)> {
        use std::convert::TryFrom;

        let undefined_value = Value::undefined();

        let bits_arg = args.get(0).unwrap_or(&undefined_value);
        let bigint_arg = args.get(1).unwrap_or(&undefined_value);

        let bits = bits_arg.to_index(ctx)?;
        let bits = u32::try_from(bits).unwrap_or(u32::MAX);

        let bigint = bigint_arg.to_bigint(ctx)?;

        Ok((
            bigint
                .as_inner()
                .clone()
                .mod_floor(&BigInt::from(2).pow(&BigInt::from(bits as i64))),
            bits,
        ))
    }
}

impl Finalize for BigInt {}
unsafe impl Trace for BigInt {
    // BigInt type implements an empty trace becasue the inner structure
    // `num_bigint::BigInt` does not implement `Trace` trait.
    // If it did this would be unsound.
    unsafe_empty_trace!();
}
