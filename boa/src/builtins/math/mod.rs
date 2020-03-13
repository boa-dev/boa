use crate::{
    builtins::{
        function::NativeFunctionData,
        value::{from_value, to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use rand::random;
use std::f64;

#[cfg(test)]
mod tests;

/// Get the absolute value of a number
pub fn abs(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .abs()
    }))
}
/// Get the arccos of a number
pub fn acos(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .acos()
    }))
}
/// Get the hyperbolic arccos of a number
pub fn acosh(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .acosh()
    }))
}
/// Get the arcsine of a number
pub fn asin(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .asin()
    }))
}
/// Get the hyperbolic arcsine of a number
pub fn asinh(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .asinh()
    }))
}
/// Get the arctangent of a number
pub fn atan(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .atan()
    }))
}
/// Get the hyperbolic arctangent of a number
pub fn atanh(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .atanh()
    }))
}
/// Get the arctangent of a numbers
pub fn atan2(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .atan2(args.get(1).expect("Could not get argument").to_num())
    }))
}
/// Get the cubic root of a number
pub fn cbrt(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .cbrt()
    }))
}
/// Get lowest integer above a number
pub fn ceil(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .ceil()
    }))
}
/// Get the cosine of a number
pub fn cos(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .cos()
    }))
}
/// Get the hyperbolic cosine of a number
pub fn cosh(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .cosh()
    }))
}
/// Get the power to raise the natural logarithm to get the number
pub fn exp(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .exp()
    }))
}
/// Get the highest integer below a number
pub fn floor(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .floor()
    }))
}
/// Get the natural logarithm of a number
pub fn log(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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
/// Get the base 10 logarithm of the number
pub fn log10(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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
/// Get the base 2 logarithm of the number
pub fn log2(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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
/// Get the maximum of several numbers
pub fn max(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let mut max = f64::NEG_INFINITY;
    for arg in args {
        let num = arg.to_num();
        max = max.max(num);
    }
    Ok(to_value(max))
}
/// Get the minimum of several numbers
pub fn min(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    let mut max = f64::INFINITY;
    for arg in args {
        let num = arg.to_num();
        max = max.min(num);
    }
    Ok(to_value(max))
}
/// Raise a number to a power
pub fn pow(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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
/// Generate a random floating-point number between 0 and 1
pub fn _random(_: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(random::<f64>()))
}
/// Round a number to the nearest integer
pub fn round(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .round()
    }))
}
/// Get the sign of a number
pub fn sign(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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
/// Get the sine of a number
pub fn sin(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .sin()
    }))
}
/// Get the hyperbolic sine of a number
pub fn sinh(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .sinh()
    }))
}
/// Get the square root of a number
pub fn sqrt(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .sqrt()
    }))
}
/// Get the tangent of a number
pub fn tan(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .tan()
    }))
}
/// Get the hyperbolic tangent of a number
pub fn tanh(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .tanh()
    }))
}
/// Get the integer part of a number
pub fn trunc(_: &Value, args: &[Value], _: &mut Interpreter) -> ResultValue {
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
