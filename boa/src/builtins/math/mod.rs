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
    builtins::value::{from_value, to_value, ResultValue, Value, ValueData},
    exec::Interpreter,
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .abs()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .acos()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .acosh()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .asin()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .asinh()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .atan()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .atanh()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .atan2(args.get(1).expect("Could not get argument").to_num())
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .cbrt()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .ceil()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .cos()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .cosh()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .exp()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .floor()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        let value = from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64");

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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        let value = from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64");

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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        let value = from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64");

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
        let num = arg.to_num();
        max = max.max(num);
    }
    Ok(to_value(max))
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
        let num = arg.to_num();
        max = max.min(num);
    }
    Ok(to_value(max))
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
    Ok(to_value(if args.len() >= 2 {
        let num: f64 = from_value(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64");
        let power: f64 = from_value(args.get(1).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64");
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
    Ok(to_value(random::<f64>()))
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .round()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        let value = from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64");

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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .sin()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .sinh()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .sqrt()
    }))
}
/// Get the tangent of a number
pub fn tan(_: &mut Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .tan()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .tanh()
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
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .trunc()
    }))
}

/// Create a new `Math` object
pub fn create_constructor(global: &Value) -> Value {
    let math = ValueData::new_obj(Some(global));
    math.set_field_slice("E", to_value(f64::consts::E));
    math.set_field_slice("LN2", to_value(f64::consts::LN_2));
    math.set_field_slice("LN10", to_value(f64::consts::LN_10));
    math.set_field_slice("LOG2E", to_value(f64::consts::LOG2_E));
    math.set_field_slice("LOG10E", to_value(f64::consts::LOG10_E));
    math.set_field_slice("SQRT1_2", to_value(0.5_f64.sqrt()));
    math.set_field_slice("SQRT2", to_value(f64::consts::SQRT_2));
    math.set_field_slice("PI", to_value(f64::consts::PI));
    make_builtin_fn!(abs, named "abs", with length 1, of math);
    make_builtin_fn!(acos, named "acos", with length 1, of math);
    make_builtin_fn!(acosh, named "acosh", with length 1, of math);
    make_builtin_fn!(asin, named "asin", with length 1, of math);
    make_builtin_fn!(asinh, named "asinh", with length 1, of math);
    make_builtin_fn!(atan, named "atan", with length 1, of math);
    make_builtin_fn!(atanh, named "atanh", with length 1, of math);
    make_builtin_fn!(atan2, named "atan2", with length 2, of math);
    make_builtin_fn!(cbrt, named "cbrt", with length 1, of math);
    make_builtin_fn!(ceil, named "ceil", with length 1, of math);
    make_builtin_fn!(cos,  named "cos", with length 1, of math);
    make_builtin_fn!(cosh,  named "cosh", with length 1, of math);
    make_builtin_fn!(exp, named "exp", with length 1, of math);
    make_builtin_fn!(floor, named "floor", with length 1, of math);
    make_builtin_fn!(log, named "log", with length 1, of math);
    make_builtin_fn!(log10, named "log10", with length 1, of math);
    make_builtin_fn!(log2, named "log2", with length 1, of math);
    make_builtin_fn!(max, named "max", with length 2, of math);
    make_builtin_fn!(min, named "min", with length 2, of math);
    make_builtin_fn!(pow, named "pow", with length 2, of math);
    make_builtin_fn!(_random, named "random", of math);
    make_builtin_fn!(round, named "round", with length 1, of math);
    make_builtin_fn!(sign, named "sign", with length 1, of math);
    make_builtin_fn!(sin, named "sin", with length 1, of math);
    make_builtin_fn!(sinh, named "sinh", with length 1, of math);
    make_builtin_fn!(sqrt, named "sqrt", with length 1, of math);
    make_builtin_fn!(tan, named "tan", with length 1, of math);
    make_builtin_fn!(tanh, named "tanh", with length 1, of math);
    make_builtin_fn!(trunc, named "trunc", with length 1, of math);
    math
}
