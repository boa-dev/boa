use js::function::Function;
use js::value::{from_value, to_value, ResultValue, Value};
use rand::random;
use std::f64;

/// Get the absolute value of a number
pub fn abs(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .abs()
    } else {
        f64::NAN
    }))
}
/// Get the arccos of a number
pub fn acos(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .acos()
    } else {
        f64::NAN
    }))
}
/// Get the arcsine of a number
pub fn asin(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .asin()
    } else {
        f64::NAN
    }))
}
/// Get the arctangent of a number
pub fn atan(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .atan()
    } else {
        f64::NAN
    }))
}
/// Get the arctangent of a numbers
pub fn atan2(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .atan2(args.get(1).unwrap().to_num())
    } else {
        f64::NAN
    }))
}
/// Get the cubic root of a number
pub fn cbrt(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .cbrt()
    } else {
        f64::NAN
    }))
}
/// Get lowest integer above a number
pub fn ceil(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .ceil()
    } else {
        f64::NAN
    }))
}
/// Get the cosine of a number
pub fn cos(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .cos()
    } else {
        f64::NAN
    }))
}
/// Get the power to raise the natural logarithm to get the number
pub fn exp(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .exp()
    } else {
        f64::NAN
    }))
}
/// Get the highest integer below a number
pub fn floor(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .floor()
    } else {
        f64::NAN
    }))
}
/// Get the natural logarithm of a number
pub fn log(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .log(f64::consts::E)
    } else {
        f64::NAN
    }))
}
/// Get the maximum of several numbers
pub fn max(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    let mut max = f64::NEG_INFINITY;
    for arg in args.iter() {
        let num = arg.to_num();
        max = max.max(num);
    }
    Ok(to_value(max))
}
/// Get the minimum of several numbers
pub fn min(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    let mut max = f64::INFINITY;
    for arg in args.iter() {
        let num = arg.to_num();
        max = max.min(num);
    }
    Ok(to_value(max))
}
/// Raise a number to a power
pub fn pow(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 2 {
        let num: f64 = from_value(args.get(0).unwrap().clone()).unwrap();
        let power: f64 = from_value(args.get(1).unwrap().clone()).unwrap();
        num.powf(power)
    } else {
        f64::NAN
    }))
}
/// Generate a random floating-point number between 0 and 1
pub fn _random(_: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(random::<f64>()))
}
/// Round a number to the nearest integer
pub fn round(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .round()
    } else {
        f64::NAN
    }))
}
/// Get the sine of a number
pub fn sin(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .sin()
    } else {
        f64::NAN
    }))
}
/// Get the square root of a number
pub fn sqrt(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .sqrt()
    } else {
        f64::NAN
    }))
}
/// Get the tangent of a number
pub fn tan(args: Vec<Value>, _: Value, _: Value, _: Value) -> ResultValue {
    Ok(to_value(if args.len() >= 1 {
        from_value::<f64>(args.get(0).unwrap().clone())
            .unwrap()
            .tan()
    } else {
        f64::NAN
    }))
}
/// Create a new `Math` object
pub fn _create(global: Value) -> Value {
    let math = Value::new_obj(Some(global));
    math.set_field_slice("E", to_value(f64::consts::E));
    math.set_field_slice("LN2", to_value(f64::consts::LN_2));
    math.set_field_slice("LN10", to_value(f64::consts::LN_10));
    math.set_field_slice("LOG2E", to_value(f64::consts::LOG2_E));
    math.set_field_slice("LOG10E", to_value(f64::consts::LOG10_E));
    math.set_field_slice("SQRT1_2", to_value(0.5f64.sqrt()));
    math.set_field_slice("SQRT2", to_value(f64::consts::SQRT_2));
    math.set_field_slice("PI", to_value(f64::consts::PI));
    math.set_field_slice("abs", Function::make(abs, &["num1", "num2"]));
    math.set_field_slice("acos", Function::make(acos, &["num1", "num2"]));
    math.set_field_slice("asin", Function::make(asin, &["num1", "num2"]));
    math.set_field_slice("atan", Function::make(atan, &["num1", "num2"]));
    math.set_field_slice("atan2", Function::make(atan2, &["num1", "num2"]));
    math.set_field_slice("cbrt", Function::make(cbrt, &["num1", "num2"]));
    math.set_field_slice("ceil", Function::make(ceil, &["num1", "num2"]));
    math.set_field_slice("cos", Function::make(cos, &["num1", "num2"]));
    math.set_field_slice("exp", Function::make(exp, &["num1", "num2"]));
    math.set_field_slice("floor", Function::make(floor, &["num"]));
    math.set_field_slice("log", Function::make(log, &["num1", "num2"]));
    math.set_field_slice("max", Function::make(max, &["num1", "num2"]));
    math.set_field_slice("min", Function::make(min, &["num1", "num2"]));
    math.set_field_slice("pow", Function::make(pow, &["num1", "num2"]));
    math.set_field_slice("random", Function::make(_random, &[]));
    math.set_field_slice("round", Function::make(round, &["num"]));
    math.set_field_slice("sin", Function::make(sin, &["num"]));
    math.set_field_slice("sqrt", Function::make(sqrt, &["num"]));
    math.set_field_slice("tan", Function::make(tan, &["num"]));
    math
}
/// Initialise the `Math` object on the global object
pub fn init(global: Value) {
    global.set_field_slice("Math", _create(global.clone()));
}
