#![allow(clippy::needless_pass_by_value)]

use crate::js::{
    function::NativeFunctionData,
    value::{from_value, to_value, ResultValue, Value, ValueData},
};
use rand::random;
use std::f64;

/// Get the absolute value of a number
pub fn abs(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .abs()
    }))
}
/// Get the arccos of a number
pub fn acos(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .acos()
    }))
}
/// Get the arcsine of a number
pub fn asin(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .asin()
    }))
}
/// Get the arctangent of a number
pub fn atan(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .atan()
    }))
}
/// Get the arctangent of a numbers
pub fn atan2(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .atan2(args.get(1).unwrap().to_num())
    }))
}
/// Get the cubic root of a number
pub fn cbrt(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .cbrt()
    }))
}
/// Get lowest integer above a number
pub fn ceil(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .ceil()
    }))
}
/// Get the cosine of a number
pub fn cos(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .cos()
    }))
}
/// Get the power to raise the natural logarithm to get the number
pub fn exp(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .exp()
    }))
}
/// Get the highest integer below a number
pub fn floor(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .floor()
    }))
}
/// Get the natural logarithm of a number
pub fn log(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .log(f64::consts::E)
    }))
}
/// Get the maximum of several numbers
pub fn max(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let mut max = f64::NEG_INFINITY;
    for arg in &args {
        let num = arg.to_num();
        max = max.max(num);
    }
    Ok(to_value(max))
}
/// Get the minimum of several numbers
pub fn min(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    let mut max = f64::INFINITY;
    for arg in &args {
        let num = arg.to_num();
        max = max.min(num);
    }
    Ok(to_value(max))
}
/// Raise a number to a power
pub fn pow(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.len() >= 2 {
        let num: f64 = from_value(args.get(0).unwrap().clone()).unwrap();
        let power: f64 = from_value(args.get(1).unwrap().clone()).unwrap();
        num.powf(power)
    } else {
        f64::NAN
    }))
}
/// Generate a random floating-point number between 0 and 1
pub fn _random(_: Value, _: Value, _args: Vec<Value>) -> ResultValue {
    Ok(to_value(random::<f64>()))
}
/// Round a number to the nearest integer
pub fn round(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .round()
    }))
}
/// Get the sine of a number
pub fn sin(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .sin()
    }))
}
/// Get the square root of a number
pub fn sqrt(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .sqrt()
    }))
}
/// Get the tangent of a number
pub fn tan(_: Value, _: Value, args: Vec<Value>) -> ResultValue {
    Ok(to_value(if args.is_empty() {
        f64::NAN
    } else {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
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
