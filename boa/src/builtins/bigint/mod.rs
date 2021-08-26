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
    builtins::BuiltIn, object::ConstructorBuilder, property::Attribute, symbol::WellKnownSymbols,
    value::IntegerOrInfinity, BoaProfiler, Context, JsBigInt, JsResult, JsValue,
};
#[cfg(test)]
mod tests;

/// `BigInt` implementation.
#[derive(Debug, Clone, Copy)]
pub struct BigInt;

impl BuiltIn for BigInt {
    const NAME: &'static str = "BigInt";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let to_string_tag = WellKnownSymbols::to_string_tag();

        let bigint_object = ConstructorBuilder::with_standard_object(
            context,
            Self::constructor,
            context.standard_objects().bigint_object().clone(),
        )
        .name(Self::NAME)
        .length(Self::LENGTH)
        .method(Self::to_string, "toString", 0)
        .method(Self::value_of, "valueOf", 0)
        .static_method(Self::as_int_n, "asIntN", 2)
        .static_method(Self::as_uint_n, "asUintN", 2)
        .callable(true)
        .constructable(false)
        .property(
            to_string_tag,
            Self::NAME,
            Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        )
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
    fn constructor(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let data = match args.get(0) {
            Some(value) => value.to_bigint(context)?,
            None => JsBigInt::zero(),
        };
        Ok(JsValue::new(data))
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
    fn this_bigint_value(value: &JsValue, context: &mut Context) -> JsResult<JsBigInt> {
        match value {
            // 1. If Type(value) is BigInt, return value.
            JsValue::BigInt(ref bigint) => return Ok(bigint.clone()),

            // 2. If Type(value) is Object and value has a [[BigIntData]] internal slot, then
            //    a. Assert: Type(value.[[BigIntData]]) is BigInt.
            //    b. Return value.[[BigIntData]].
            JsValue::Object(ref object) => {
                if let Some(bigint) = object.borrow().as_bigint() {
                    return Ok(bigint.clone());
                }
            }
            _ => {}
        }

        // 3. Throw a TypeError exception.
        Err(context.construct_type_error("'this' is not a BigInt"))
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
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let x be ? thisBigIntValue(this value).
        let x = Self::this_bigint_value(this, context)?;

        let radix = args.get(0).cloned().unwrap_or_default();

        // 2. If radix is undefined, let radixMV be 10.
        let radix_mv = if radix.is_undefined() {
            // 5. If radixMV = 10, return ! ToString(x).
            // Note: early return optimization.
            return Ok(x.to_string().into());
        // 3. Else, let radixMV be ? ToIntegerOrInfinity(radix).
        } else {
            radix.to_integer_or_infinity(context)?
        };

        // 4. If radixMV < 2 or radixMV > 36, throw a RangeError exception.
        let radix_mv = match radix_mv {
            IntegerOrInfinity::Integer(i) if (2..=36).contains(&i) => i,
            _ => {
                return context.throw_range_error(
                    "radix must be an integer at least 2 and no greater than 36",
                )
            }
        };

        // 5. If radixMV = 10, return ! ToString(x).
        if radix_mv == 10 {
            return Ok(x.to_string().into());
        }

        // 1. Let x be ? thisBigIntValue(this value).
        // 6. Return the String representation of this Number value using the radix specified by radixMV.
        //    Letters a-z are used for digits with values 10 through 35.
        //    The precise algorithm is implementation-defined, however the algorithm should be a generalization of that specified in 6.1.6.2.23.
        Ok(JsValue::new(x.to_string_radix(radix_mv as u32)))
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
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Ok(JsValue::new(Self::this_bigint_value(this, context)?))
    }

    /// `BigInt.asIntN()`
    ///
    /// The `BigInt.asIntN()` method wraps the value of a `BigInt` to a signed integer between `-2**(width - 1)` and `2**(width-1) - 1`.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint.asintn
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/asIntN
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn as_int_n(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let (modulo, bits) = Self::calculate_as_uint_n(args, context)?;

        if bits > 0
            && modulo >= JsBigInt::pow(&JsBigInt::new(2), &JsBigInt::new(bits as i64 - 1), context)?
        {
            Ok(JsValue::new(JsBigInt::sub(
                &modulo,
                &JsBigInt::pow(&JsBigInt::new(2), &JsBigInt::new(bits as i64), context)?,
            )))
        } else {
            Ok(JsValue::new(modulo))
        }
    }

    /// `BigInt.asUintN()`
    ///
    /// The `BigInt.asUintN()` method wraps the value of a `BigInt` to an unsigned integer between `0` and `2**(width) - 1`.
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint.asuintn
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/asUintN
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn as_uint_n(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let (modulo, _) = Self::calculate_as_uint_n(args, context)?;

        Ok(JsValue::new(modulo))
    }

    /// Helper function to wrap the value of a `BigInt` to an unsigned integer.
    ///
    /// This function expects the same arguments as `as_uint_n` and wraps the value of a `BigInt`.
    /// Additionally to the wrapped unsigned value it returns the converted `bits` argument, so it
    /// can be reused from the `as_int_n` method.
    fn calculate_as_uint_n(args: &[JsValue], context: &mut Context) -> JsResult<(JsBigInt, u32)> {
        use std::convert::TryFrom;

        let undefined_value = JsValue::undefined();

        let bits_arg = args.get(0).unwrap_or(&undefined_value);
        let bigint_arg = args.get(1).unwrap_or(&undefined_value);

        let bits = bits_arg.to_index(context)?;
        let bits = u32::try_from(bits).unwrap_or(u32::MAX);

        let bigint = bigint_arg.to_bigint(context)?;

        Ok((
            JsBigInt::mod_floor(
                &bigint,
                &JsBigInt::pow(&JsBigInt::new(2), &JsBigInt::new(bits as i64), context)?,
            ),
            bits,
        ))
    }
}
