//! This module implements the global `Math` object.
//!
//! `Math` is a built-in object that has properties and methods for mathematical constants and functions. It’s not a function object.
//!
//! `Math` works with the `Number` type. It doesn't work with `BigInt`.
//!
//! More information:
//!  - [ECMAScript reference][spec]
//!  - [MDN documentation][mdn]
//!
//! [spec]: https://tc39.es/ecma262/#sec-math-object
//! [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math

use crate::{
    builtins::{
        function::make_builtin_fn,
        property::Attribute,
        value::{ResultValue, Value},
    },
    exec::Interpreter,
    BoaProfiler,
};
use std::borrow::BorrowMut;
use std::f64;

#[cfg(test)]
mod tests;

/// Javascript `Math` object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Math;

impl Math {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "Math";

    /// Get the absolute value of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.abs
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/abs
    pub(crate) fn abs(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::abs)
            .into())
    }

    /// Get the arccos of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.acos
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/acos
    pub(crate) fn acos(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::acos)
            .into())
    }

    /// Get the hyperbolic arccos of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.acosh
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/acosh
    pub(crate) fn acosh(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::acosh)
            .into())
    }

    /// Get the arcsine of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.asin
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/asin
    pub(crate) fn asin(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::asin)
            .into())
    }

    /// Get the hyperbolic arcsine of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.asinh
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/asinh
    pub(crate) fn asinh(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::asinh)
            .into())
    }

    /// Get the arctangent of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.atan
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/atan
    pub(crate) fn atan(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::atan)
            .into())
    }

    /// Get the hyperbolic arctangent of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.atanh
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/atanh
    pub(crate) fn atanh(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::atanh)
            .into())
    }

    /// Get the arctangent of a numbers.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.atan2
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/atan2
    pub(crate) fn atan2(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(match (
            args.get(0).map(|x| ctx.to_number(x)).transpose()?,
            args.get(1).map(|x| ctx.to_number(x)).transpose()?,
        ) {
            (Some(x), Some(y)) => x.atan2(y),
            (_, _) => f64::NAN,
        }
        .into())
    }

    /// Get the cubic root of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.cbrt
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/cbrt
    pub(crate) fn cbrt(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::cbrt)
            .into())
    }

    /// Get lowest integer above a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.ceil
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/ceil
    pub(crate) fn ceil(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::ceil)
            .into())
    }

    /// Get the number of leading zeros in the 32 bit representation of a number
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.clz32
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/clz32
    pub(crate) fn clz32(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_uint32(x))
            .transpose()?
            .map(u32::leading_zeros)
            .unwrap_or(32)
            .into())
    }

    /// Get the cosine of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.cos
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/cos
    pub(crate) fn cos(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::cos)
            .into())
    }

    /// Get the hyperbolic cosine of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.cosh
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/cosh
    pub(crate) fn cosh(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::cosh)
            .into())
    }

    /// Get the power to raise the natural logarithm to get the number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.exp
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/exp
    pub(crate) fn exp(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::exp)
            .into())
    }

    /// The Math.expm1() function returns e^x - 1, where x is the argument, and e the base of
    /// the natural logarithms. The result is computed in a way that is accurate even when the
    /// value of x is close 0
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.expm1
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/expm1
    pub(crate) fn expm1(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::exp_m1)
            .into())
    }

    /// Get the highest integer below a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.floor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/floor
    pub(crate) fn floor(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::floor)
            .into())
    }

    /// Get the nearest 32-bit single precision float representation of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.fround
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/fround
    pub(crate) fn fround(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, |x| (x as f32) as f64)
            .into())
    }

    /// Get an approximation of the square root of the sum of squares of all arguments.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.hypot
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/hypot
    pub(crate) fn hypot(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let mut result = 0f64;
        for arg in args {
            let x = ctx.to_number(arg)?;
            result = result.hypot(x);
        }
        Ok(result.into())
    }

    /// Get the result of the C-like 32-bit multiplication of the two parameters.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.imul
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/imul
    pub(crate) fn imul(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(match (
            args.get(0).map(|x| ctx.to_uint32(x)).transpose()?,
            args.get(1).map(|x| ctx.to_uint32(x)).transpose()?,
        ) {
            (Some(x), Some(y)) => x.wrapping_mul(y) as i32,
            (_, _) => 0,
        }
        .into())
    }

    /// Get the natural logarithm of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.log
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log
    pub(crate) fn log(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, |x| if x <= 0.0 { f64::NAN } else { x.ln() })
            .into())
    }

    /// Get approximation to the natural logarithm of 1 + x.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.log1p
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log1p
    pub(crate) fn log1p(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::ln_1p)
            .into())
    }

    /// Get the base 10 logarithm of the number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.log10
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log10
    pub(crate) fn log10(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, |x| if x <= 0.0 { f64::NAN } else { x.log10() })
            .into())
    }

    /// Get the base 2 logarithm of the number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.log2
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log2
    pub(crate) fn log2(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, |x| if x <= 0.0 { f64::NAN } else { x.log2() })
            .into())
    }

    /// Get the maximum of several numbers.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.max
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/max
    pub(crate) fn max(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let mut max = f64::NEG_INFINITY;
        for arg in args {
            let num = ctx.to_number(arg)?;
            max = max.max(num);
        }
        Ok(max.into())
    }

    /// Get the minimum of several numbers.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.min
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/min
    pub(crate) fn min(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let mut min = f64::INFINITY;
        for arg in args {
            let num = ctx.to_number(arg)?;
            min = min.min(num);
        }
        Ok(min.into())
    }

    /// Raise a number to a power.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.pow
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/pow
    pub(crate) fn pow(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(match (
            args.get(0).map(|x| ctx.to_number(x)).transpose()?,
            args.get(1).map(|x| ctx.to_number(x)).transpose()?,
        ) {
            (Some(x), Some(y)) => x.powf(y),
            (_, _) => f64::NAN,
        }
        .into())
    }

    /// Generate a random floating-point number between `0` and `1`.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.random
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/random
    pub(crate) fn random(_: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
        Ok(rand::random::<f64>().into())
    }

    /// Round a number to the nearest integer.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.round
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/round
    pub(crate) fn round(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::round)
            .into())
    }

    /// Get the sign of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.sign
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sign
    pub(crate) fn sign(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(
                f64::NAN,
                |x| {
                    if x == 0.0 || x == -0.0 {
                        x
                    } else {
                        x.signum()
                    }
                },
            )
            .into())
    }

    /// Get the sine of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.sin
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sin
    pub(crate) fn sin(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::sin)
            .into())
    }

    /// Get the hyperbolic sine of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.sinh
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sinh
    pub(crate) fn sinh(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::sinh)
            .into())
    }

    /// Get the square root of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.sqrt
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sqrt
    pub(crate) fn sqrt(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::sqrt)
            .into())
    }

    /// Get the tangent of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.tan
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/tan
    pub(crate) fn tan(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::tan)
            .into())
    }

    /// Get the hyperbolic tangent of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.tanh
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/tanh
    pub(crate) fn tanh(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::tanh)
            .into())
    }

    /// Get the integer part of a number.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-math.trunc
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/trunc
    pub(crate) fn trunc(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(args
            .get(0)
            .map(|x| ctx.to_number(x))
            .transpose()?
            .map_or(f64::NAN, f64::trunc)
            .into())
    }

    /// Create a new `Math` object
    pub(crate) fn create(interpreter: &mut Interpreter) -> Value {
        let global = interpreter.global();
        let _timer = BoaProfiler::global().start_event("math:create", "init");
        let math = Value::new_object(Some(global));

        {
            let mut properties = math.as_object_mut().unwrap();
            let attribute = Attribute::default();
            properties.insert_property("E", Value::from(f64::consts::E), attribute);
            properties.insert_property("LN2", Value::from(f64::consts::LN_2), attribute);
            properties.insert_property("LN10", Value::from(f64::consts::LN_10), attribute);
            properties.insert_property("LOG2E", Value::from(f64::consts::LOG2_E), attribute);
            properties.insert_property("LOG10E", Value::from(f64::consts::LOG10_E), attribute);
            properties.insert_property("SQRT1_2", Value::from(0.5_f64.sqrt()), attribute);
            properties.insert_property("SQRT2", Value::from(f64::consts::SQRT_2), attribute);
            properties.insert_property("PI", Value::from(f64::consts::PI), attribute);
        }

        make_builtin_fn(Self::abs, "abs", &math, 1, interpreter);
        make_builtin_fn(Self::acos, "acos", &math, 1, interpreter);
        make_builtin_fn(Self::acosh, "acosh", &math, 1, interpreter);
        make_builtin_fn(Self::asin, "asin", &math, 1, interpreter);
        make_builtin_fn(Self::asinh, "asinh", &math, 1, interpreter);
        make_builtin_fn(Self::atan, "atan", &math, 1, interpreter);
        make_builtin_fn(Self::atanh, "atanh", &math, 1, interpreter);
        make_builtin_fn(Self::atan2, "atan2", &math, 2, interpreter);
        make_builtin_fn(Self::cbrt, "cbrt", &math, 1, interpreter);
        make_builtin_fn(Self::ceil, "ceil", &math, 1, interpreter);
        make_builtin_fn(Self::clz32, "clz32", &math, 1, interpreter);
        make_builtin_fn(Self::cos, "cos", &math, 1, interpreter);
        make_builtin_fn(Self::cosh, "cosh", &math, 1, interpreter);
        make_builtin_fn(Self::exp, "exp", &math, 1, interpreter);
        make_builtin_fn(Self::expm1, "expm1", &math, 1, interpreter);
        make_builtin_fn(Self::floor, "floor", &math, 1, interpreter);
        make_builtin_fn(Self::fround, "fround", &math, 1, interpreter);
        make_builtin_fn(Self::hypot, "hypot", &math, 1, interpreter);
        make_builtin_fn(Self::imul, "imul", &math, 1, interpreter);
        make_builtin_fn(Self::log, "log", &math, 1, interpreter);
        make_builtin_fn(Self::log1p, "log1p", &math, 1, interpreter);
        make_builtin_fn(Self::log10, "log10", &math, 1, interpreter);
        make_builtin_fn(Self::log2, "log2", &math, 1, interpreter);
        make_builtin_fn(Self::max, "max", &math, 2, interpreter);
        make_builtin_fn(Self::min, "min", &math, 2, interpreter);
        make_builtin_fn(Self::pow, "pow", &math, 2, interpreter);
        make_builtin_fn(Self::random, "random", &math, 0, interpreter);
        make_builtin_fn(Self::round, "round", &math, 1, interpreter);
        make_builtin_fn(Self::sign, "sign", &math, 1, interpreter);
        make_builtin_fn(Self::sin, "sin", &math, 1, interpreter);
        make_builtin_fn(Self::sinh, "sinh", &math, 1, interpreter);
        make_builtin_fn(Self::sqrt, "sqrt", &math, 1, interpreter);
        make_builtin_fn(Self::tan, "tan", &math, 1, interpreter);
        make_builtin_fn(Self::tanh, "tanh", &math, 1, interpreter);
        make_builtin_fn(Self::trunc, "trunc", &math, 1, interpreter);

        math
    }

    /// Initialise the `Math` object on the global object.
    #[inline]
    pub(crate) fn init(interpreter: &mut Interpreter) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");
        let math = Self::create(interpreter);
        let mut global = interpreter.global().as_object_mut().expect("Expect object");
        global.borrow_mut().insert_property(
            Self::NAME,
            math,
            Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE,
        );
    }
}
