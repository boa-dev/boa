//! Boa's implementation of ECMAScript's global `BigInt` object.
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
    builtins::BuiltInObject,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    error::JsNativeError,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
    value::{IntegerOrInfinity, PreferredType},
    Context, JsArgs, JsBigInt, JsResult, JsValue,
};
use boa_profiler::Profiler;
use num_bigint::ToBigInt;

use super::{BuiltInBuilder, BuiltInConstructor, IntrinsicObject};

#[cfg(test)]
mod tests;

/// `BigInt` implementation.
#[derive(Debug, Clone, Copy)]
pub struct BigInt;

impl IntrinsicObject for BigInt {
    fn init(realm: &Realm) {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::to_string, "toString", 0)
            .method(Self::value_of, "valueOf", 0)
            .static_method(Self::as_int_n, "asIntN", 2)
            .static_method(Self::as_uint_n, "asUintN", 2)
            .property(
                JsSymbol::to_string_tag(),
                Self::NAME,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for BigInt {
    const NAME: &'static str = "BigInt";
}

impl BuiltInConstructor for BigInt {
    const LENGTH: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::bigint;

    /// `BigInt()`
    ///
    /// The `BigInt()` constructor is used to create `BigInt` objects.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-bigint-objects
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt/BigInt
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. If NewTarget is not undefined, throw a TypeError exception.
        if !new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("BigInt is not a constructor")
                .into());
        }

        let value = args.get_or_undefined(0);

        // 2. Let prim be ? ToPrimitive(value, number).
        let prim = value.to_primitive(context, PreferredType::Number)?;

        // 3. If Type(prim) is Number, return ? NumberToBigInt(prim).
        if let Some(number) = prim.as_number() {
            return Self::number_to_bigint(number);
        }

        // 4. Otherwise, return ? ToBigInt(prim).
        Ok(prim.to_bigint(context)?.into())
    }
}

impl BigInt {
    /// `NumberToBigInt ( number )`
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-numbertobigint
    fn number_to_bigint(number: f64) -> JsResult<JsValue> {
        // 1. If IsIntegralNumber(number) is false, throw a RangeError exception.
        if number.is_nan() || number.is_infinite() || number.fract() != 0.0 {
            return Err(JsNativeError::range()
                .with_message(format!("cannot convert {number} to a BigInt"))
                .into());
        }

        // 2. Return the BigInt value that represents ℝ(number).
        Ok(JsBigInt::from(number.to_bigint().expect("This conversion must be safe")).into())
    }

    /// The abstract operation `thisBigIntValue` takes argument value.
    ///
    /// The phrase “this `BigInt` value” within the specification of a method refers to the
    /// result returned by calling the abstract operation `thisBigIntValue` with the `this` value
    /// of the method invocation passed as the argument.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thisbigintvalue
    fn this_bigint_value(value: &JsValue) -> JsResult<JsBigInt> {
        value
            // 1. If Type(value) is BigInt, return value.
            .as_bigint()
            .cloned()
            // 2. If Type(value) is Object and value has a [[BigIntData]] internal slot, then
            //    a. Assert: Type(value.[[BigIntData]]) is BigInt.
            //    b. Return value.[[BigIntData]].
            .or_else(|| {
                value
                    .as_object()
                    .and_then(|obj| obj.borrow().as_bigint().cloned())
            })
            // 3. Throw a TypeError exception.
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("'this' is not a BigInt")
                    .into()
            })
    }

    /// `BigInt.prototype.toString( [radix] )`
    ///
    /// The `toString()` method returns a string representing the specified `BigInt` object.
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
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        // 1. Let x be ? thisBigIntValue(this value).
        let x = Self::this_bigint_value(this)?;

        let radix = args.get_or_undefined(0);

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
                return Err(JsNativeError::range()
                    .with_message("radix must be an integer at least 2 and no greater than 36")
                    .into())
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
        _: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        Ok(JsValue::new(Self::this_bigint_value(this)?))
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
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let (modulo, bits) = Self::calculate_as_uint_n(args, context)?;

        if bits > 0
            && modulo >= JsBigInt::pow(&JsBigInt::new(2), &JsBigInt::new(i64::from(bits) - 1))?
        {
            Ok(JsValue::new(JsBigInt::sub(
                &modulo,
                &JsBigInt::pow(&JsBigInt::new(2), &JsBigInt::new(i64::from(bits)))?,
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
        context: &mut Context<'_>,
    ) -> JsResult<JsValue> {
        let (modulo, _) = Self::calculate_as_uint_n(args, context)?;

        Ok(JsValue::new(modulo))
    }

    /// Helper function to wrap the value of a `BigInt` to an unsigned integer.
    ///
    /// This function expects the same arguments as `as_uint_n` and wraps the value of a `BigInt`.
    /// Additionally to the wrapped unsigned value it returns the converted `bits` argument, so it
    /// can be reused from the `as_int_n` method.
    fn calculate_as_uint_n(
        args: &[JsValue],
        context: &mut Context<'_>,
    ) -> JsResult<(JsBigInt, u32)> {
        let bits_arg = args.get_or_undefined(0);
        let bigint_arg = args.get_or_undefined(1);

        let bits = bits_arg.to_index(context)?;
        let bits = u32::try_from(bits).unwrap_or(u32::MAX);

        let bigint = bigint_arg.to_bigint(context)?;

        Ok((
            JsBigInt::mod_floor(
                &bigint,
                &JsBigInt::pow(&JsBigInt::new(2), &JsBigInt::new(i64::from(bits)))?,
            ),
            bits,
        ))
    }
}
