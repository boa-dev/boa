use crate::{
    builtins::{
        function::NativeFunctionData,
        value::{from_value, to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use rand::random;
use std::f64;

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
        from_value::<f64>(args.get(0).expect("Could not get argument").clone())
            .expect("Could not convert argument to f64")
            .log(f64::consts::E)
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
    make_fn!(abs, named "abs", of math);
    make_fn!(acos, named "acos", of math);
    make_fn!(asin, named "asin", of math);
    make_fn!(atan, named "atan", of math);
    make_fn!(atan2, named "atan2", of math);
    make_fn!(cbrt, named "cbrt", of math);
    make_fn!(ceil, named "ceil", of math);
    make_fn!(cos,  named "cos", of math);
    make_fn!(exp, named "exp", of math);
    make_fn!(floor, named "floor", of math);
    make_fn!(log, named "log", of math);
    make_fn!(max, named "max", of math);
    make_fn!(min, named "min", of math);
    make_fn!(pow, named "pow", of math);
    make_fn!(_random, named "random", of math);
    make_fn!(round, named "round" , of math);
    make_fn!(sin, named "sin", of math);
    make_fn!(sqrt, named "sqrt", of math);
    make_fn!(tan, named "tan", of math);
    math
}
