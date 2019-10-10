use crate::{
    exec::Interpreter,
    js::{
        function::NativeFunctionData,
        value::{from_value, to_value, ResultValue, Value, ValueData},
    },
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
        let num: f64 = from_value(args.get(0).expect("Could not get argument").clone()).expect("Could not convert argument to f64");
        let power: f64 = from_value(args.get(1).expect("Could not get argument").clone()).expect("Could not convert argument to f64");
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
pub fn _create(global: &Value) -> Value {
    let math = ValueData::new_obj(Some(global));
    math.set_field_slice("E", to_value(f64::consts::E));
    math.set_field_slice("LN2", to_value(f64::consts::LN_2));
    math.set_field_slice("LN10", to_value(f64::consts::LN_10));
    math.set_field_slice("LOG2E", to_value(f64::consts::LOG2_E));
    math.set_field_slice("LOG10E", to_value(f64::consts::LOG10_E));
    math.set_field_slice("SQRT1_2", to_value(0.5_f64.sqrt()));
    math.set_field_slice("SQRT2", to_value(f64::consts::SQRT_2));
    math.set_field_slice("PI", to_value(f64::consts::PI));
    math.set_field_slice("abs", to_value(abs as NativeFunctionData));
    math.set_field_slice("acos", to_value(acos as NativeFunctionData));
    math.set_field_slice("asin", to_value(asin as NativeFunctionData));
    math.set_field_slice("atan", to_value(atan as NativeFunctionData));
    math.set_field_slice("atan2", to_value(atan2 as NativeFunctionData));
    math.set_field_slice("cbrt", to_value(cbrt as NativeFunctionData));
    math.set_field_slice("ceil", to_value(ceil as NativeFunctionData));
    math.set_field_slice("cos", to_value(cos as NativeFunctionData));
    math.set_field_slice("exp", to_value(exp as NativeFunctionData));
    math.set_field_slice("floor", to_value(floor as NativeFunctionData));
    math.set_field_slice("log", to_value(log as NativeFunctionData));
    math.set_field_slice("max", to_value(max as NativeFunctionData));
    math.set_field_slice("min", to_value(min as NativeFunctionData));
    math.set_field_slice("pow", to_value(pow as NativeFunctionData));
    math.set_field_slice("random", to_value(_random as NativeFunctionData));
    math.set_field_slice("round", to_value(round as NativeFunctionData));
    math.set_field_slice("sin", to_value(sin as NativeFunctionData));
    math.set_field_slice("sqrt", to_value(sqrt as NativeFunctionData));
    math.set_field_slice("tan", to_value(tan as NativeFunctionData));
    math
}
/// Initialise the `Math` object on the global object
pub fn init(global: &Value) {
    global.set_field_slice("Math", _create(global));
}
