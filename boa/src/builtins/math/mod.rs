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
        value::{ResultValue, Value},
    },
    exec::Interpreter,
    BoaProfiler,
};
use rand::random;
use std::f64;

#[cfg(test)]
mod tests;

/// Get the absolute value of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.abs
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/abs
pub fn abs(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).abs()
    }))
}

/// Get the arccos of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.acos
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/acos
pub fn acos(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).acos()
    }))
}

/// Get the hyperbolic arccos of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.acosh
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/acosh
pub fn acosh(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).acosh()
    }))
}

/// Get the arcsine of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.asin
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/asin
pub fn asin(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).asin()
    }))
}

/// Get the hyperbolic arcsine of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.asinh
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/asinh
pub fn asinh(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).asinh()
    }))
}

/// Get the arctangent of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.atan
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/atan
pub fn atan(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).atan()
    }))
}

/// Get the hyperbolic arctangent of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.atanh
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/atanh
pub fn atanh(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).atanh()
    }))
}

/// Get the arctangent of a numbers.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.atan2
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/atan2
pub fn atan2(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument"))
            .atan2(args.get(1).expect("Could not get argument").to_number())
    }))
}

/// Get the cubic root of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.cbrt
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/cbrt
pub fn cbrt(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).cbrt()
    }))
}

/// Get lowest integer above a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.ceil
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/ceil
pub fn ceil(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).ceil()
    }))
}

/// Get the cosine of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.cos
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/cos
pub fn cos(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).cos()
    }))
}

/// Get the hyperbolic cosine of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.cosh
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/cosh
pub fn cosh(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).cosh()
    }))
}

/// Get the power to raise the natural logarithm to get the number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.exp
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/exp
pub fn exp(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).exp()
    }))
}

/// Get the highest integer below a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.floor
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/floor
pub fn floor(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).floor()
    }))
}

/// Get the natural logarithm of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.log
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log
pub fn log(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        let value = f64::from(args.get(0).expect("Could not get argument"));

        if value <= 0.0 {
            f64::NAN
        } else {
            value.log(f64::consts::E)
        }
    }))
}

/// Get the base 10 logarithm of the number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.log10
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log10
pub fn log10(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        let value = f64::from(args.get(0).expect("Could not get argument"));

        if value <= 0.0 {
            f64::NAN
        } else {
            value.log10()
        }
    }))
}

/// Get the base 2 logarithm of the number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.log2
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/log2
pub fn log2(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        let value = f64::from(args.get(0).expect("Could not get argument"));

        if value <= 0.0 {
            f64::NAN
        } else {
            value.log2()
        }
    }))
}

/// Get the maximum of several numbers.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.max
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/max
pub fn max(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let mut max = f64::NEG_INFINITY;
    for arg in args {
        let num = f64::from(arg);
        max = max.max(num);
    }
    Ok(Value::from(max))
}

/// Get the minimum of several numbers.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.min
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/min
pub fn min(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let mut max = f64::INFINITY;
    for arg in args {
        let num = f64::from(arg);
        max = max.min(num);
    }
    Ok(Value::from(max))
}

/// Raise a number to a power.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.pow
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/pow
pub fn pow(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.len() >= 2 {
        let num = f64::from(args.get(0).expect("Could not get argument"));
        let power = f64::from(args.get(1).expect("Could not get argument"));
        num.powf(power)
    } else {
        f64::NAN
    }))
}

/// Generate a random floating-point number between `0` and `1`.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.random
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/random
pub fn _random(_: &mut Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(random::<f64>()))
}

/// Round a number to the nearest integer.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.round
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/round
pub fn round(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).round()
    }))
}

/// Get the sign of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.sign
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sign
pub fn sign(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        let value = f64::from(args.get(0).expect("Could not get argument"));

        if value == 0.0 || value == -0.0 {
            value
        } else {
            value.signum()
        }
    }))
}

/// Get the sine of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.sin
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sin
pub fn sin(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).sin()
    }))
}

/// Get the hyperbolic sine of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.sinh
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sinh
pub fn sinh(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).sinh()
    }))
}

/// Get the square root of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.sqrt
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/sqrt
pub fn sqrt(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).sqrt()
    }))
}
/// Get the tangent of a number
pub fn tan(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).tan()
    }))
}

/// Get the hyperbolic tangent of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.tanh
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/tanh
pub fn tanh(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).tanh()
    }))
}

/// Get the integer part of a number.
///
/// More information:
///  - [ECMAScript reference][spec]
///  - [MDN documentation][mdn]
///
/// [spec]: https://tc39.es/ecma262/#sec-math.trunc
/// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/trunc
pub fn trunc(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(Value::from(if args.is_empty() {
        f64::NAN
    } else {
        f64::from(args.get(0).expect("Could not get argument")).trunc()
    }))
}

/// Create a new `Math` object
pub fn create(global: &Value) -> Value {
    let _timer = BoaProfiler::global().start_event("math:create", "init");
    let math = Value::new_object(Some(global));

    {
        let mut properties = math.as_object_mut().unwrap();
        properties.insert_field("E", Value::from(f64::consts::E));
        properties.insert_field("LN2", Value::from(f64::consts::LN_2));
        properties.insert_field("LN10", Value::from(f64::consts::LN_10));
        properties.insert_field("LOG2E", Value::from(f64::consts::LOG2_E));
        properties.insert_field("LOG10E", Value::from(f64::consts::LOG10_E));
        properties.insert_field("SQRT1_2", Value::from(0.5_f64.sqrt()));
        properties.insert_field("SQRT2", Value::from(f64::consts::SQRT_2));
        properties.insert_field("PI", Value::from(f64::consts::PI));
    }
    make_builtin_fn(abs, "abs", &math, 1);
    make_builtin_fn(acos, "acos", &math, 1);
    make_builtin_fn(acosh, "acosh", &math, 1);
    make_builtin_fn(asin, "asin", &math, 1);
    make_builtin_fn(asinh, "asinh", &math, 1);
    make_builtin_fn(atan, "atan", &math, 1);
    make_builtin_fn(atanh, "atanh", &math, 1);
    make_builtin_fn(atan2, "atan2", &math, 2);
    make_builtin_fn(cbrt, "cbrt", &math, 1);
    make_builtin_fn(ceil, "ceil", &math, 1);
    make_builtin_fn(cos, "cos", &math, 1);
    make_builtin_fn(cosh, "cosh", &math, 1);
    make_builtin_fn(exp, "exp", &math, 1);
    make_builtin_fn(floor, "floor", &math, 1);
    make_builtin_fn(log, "log", &math, 1);
    make_builtin_fn(log10, "log10", &math, 1);
    make_builtin_fn(log2, "log2", &math, 1);
    make_builtin_fn(max, "max", &math, 2);
    make_builtin_fn(min, "min", &math, 2);
    make_builtin_fn(pow, "pow", &math, 2);
    make_builtin_fn(_random, "random", &math, 0);
    make_builtin_fn(round, "round", &math, 1);
    make_builtin_fn(sign, "sign", &math, 1);
    make_builtin_fn(sin, "sin", &math, 1);
    make_builtin_fn(sinh, "sinh", &math, 1);
    make_builtin_fn(sqrt, "sqrt", &math, 1);
    make_builtin_fn(tan, "tan", &math, 1);
    make_builtin_fn(tanh, "tanh", &math, 1);
    make_builtin_fn(trunc, "trunc", &math, 1);

    math
}

/// Initialise the `Math` object on the global object.
#[inline]
pub fn init(global: &Value) {
    let _timer = BoaProfiler::global().start_event("math", "init");
    global.set_field("Math", create(global));
}
