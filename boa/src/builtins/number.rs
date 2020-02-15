use crate::{
    builtins::{
        function::NativeFunctionData,
        object::{Object, ObjectKind, PROTOTYPE},
        value::{to_value, ResultValue, Value, ValueData},
    },
    exec::Interpreter,
};
use std::{borrow::Borrow, f64, ops::Deref};

/// Helper function: to_number(value: &Value) -> Value
///
/// Converts a Value to a Number.
fn to_number(value: &Value) -> Value {
    match *value.deref().borrow() {
        ValueData::Boolean(b) => {
            if b {
                to_value(1)
            } else {
                to_value(0)
            }
        }
        ValueData::Function(_) | ValueData::Symbol(_) | ValueData::Undefined => to_value(f64::NAN),
        ValueData::Integer(i) => to_value(f64::from(i)),
        ValueData::Object(ref o) => (o).deref().borrow().get_internal_slot("NumberData"),
        ValueData::Null => to_value(0),
        ValueData::Number(n) => to_value(n),
        ValueData::String(ref s) => match s.parse::<f64>() {
            Ok(n) => to_value(n),
            Err(_) => to_value(f64::NAN),
        },
    }
}

/// Helper function: num_to_exponential(n: f64) -> String
///
/// Formats a float as a ES6-style exponential number string.
fn num_to_exponential(n: f64) -> String {
    match n.abs() {
        x if x > 1.0 => format!("{:e}", n).replace("e", "e+"),
        x if x == 0.0 => format!("{:e}", n).replace("e", "e+"),
        _ => format!("{:e}", n),
    }
}

/// Number(arg)
///
/// Create a new number [[Construct]]
pub fn make_number(this: &Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let data = match args.get(0) {
        Some(ref value) => to_number(value),
        None => to_number(&to_value(0)),
    };
    this.set_internal_slot("NumberData", data);
    Ok(this.clone())
}

/// Number()
///
/// https://tc39.es/ecma262/#sec-number-constructor-number-value
pub fn call_number(_this: &Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let data = match args.get(0) {
        Some(ref value) => to_number(value),
        None => to_number(&to_value(0)),
    };
    Ok(data)
}

/// Number().toExponential()
///
/// https://tc39.es/ecma262/#sec-number.prototype.toexponential
pub fn to_exponential(this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this).to_num();
    let this_str_num = num_to_exponential(this_num);
    Ok(to_value(this_str_num))
}

/// https://tc39.es/ecma262/#sec-number.prototype.tofixed
pub fn to_fixed(this: &Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this).to_num();
    let precision = match args.get(0) {
        Some(n) => match n.to_int() {
            x if x > 0 => n.to_int() as usize,
            _ => 0,
        },
        None => 0,
    };
    let this_fixed_num = format!("{:.*}", precision, this_num);
    Ok(to_value(this_fixed_num))
}

/// Number().toLocaleString()
///
/// https://tc39.es/ecma262/#sec-number.prototype.tolocalestring
///
/// Note that while this technically conforms to the Ecma standard, it does no actual
/// internationalization logic.
pub fn to_locale_string(this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    let this_num = to_number(this).to_num();
    let this_str_num = format!("{}", this_num);
    Ok(to_value(this_str_num))
}

/// Number().toPrecision(p)
///
/// https://tc39.es/ecma262/#sec-number.prototype.toprecision
pub fn to_precision(this: &Value, args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    println!("Number::to_precision()");
    let this_num = to_number(this);
    let _num_str_len = format!("{}", this_num.to_num()).len();
    let _precision = match args.get(0) {
        Some(n) => match n.to_int() {
            x if x > 0 => n.to_int() as usize,
            _ => 0,
        },
        None => 0,
    };
    // TODO: Implement toPrecision
    unimplemented!();
}

/// Number().toString()
///
/// https://tc39.es/ecma262/#sec-number.prototype.tostring
pub fn to_string(this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    Ok(to_value(format!("{}", to_number(this).to_num())))
}

/// Number().valueOf()
///
/// https://tc39.es/ecma262/#sec-number.prototype.valueof
pub fn value_of(this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    Ok(to_number(this))
}

/// Create a new `Number` object
pub fn create_constructor(global: &Value) -> Value {
    let mut number_constructor = Object::default();
    number_constructor.kind = ObjectKind::Function;

    number_constructor.set_internal_method("construct", make_number);
    number_constructor.set_internal_method("call", call_number);

    let number_prototype = ValueData::new_obj(Some(global));

    number_prototype.set_internal_slot("NumberData", to_value(0));

    make_builtin_fn!(to_exponential, named "toExponential", with length 1, of number_prototype);
    make_builtin_fn!(to_fixed, named "toFixed", with length 1, of number_prototype);
    make_builtin_fn!(to_locale_string, named "toLocaleString", of number_prototype);
    make_builtin_fn!(to_precision, named "toPrecision", with length 1, of number_prototype);
    make_builtin_fn!(to_string, named "toString", with length 1, of number_prototype);
    make_builtin_fn!(value_of, named "valueOf", of number_prototype);

    let number = to_value(number_constructor);
    number_prototype.set_field_slice("constructor", number.clone());
    number.set_field_slice(PROTOTYPE, number_prototype);
    number
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{builtins::value::ValueData, exec::Executor, forward, forward_val, realm::Realm};
    use std::f64;

    #[test]
    fn check_number_constructor_is_function() {
        let global = ValueData::new_obj(None);
        let number_constructor = create_constructor(&global);
        assert_eq!(number_constructor.is_function(), true);
    }

    #[test]
    fn call_number() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var default_zero = Number();
        var int_one = Number(1);
        var float_two = Number(2.1);
        var str_three = Number('3.2');
        var bool_one = Number(true);
        var bool_zero = Number(false);
        var invalid_nan = Number("I am not a number");
        var from_exp = Number("2.34e+2");
        "#;

        forward(&mut engine, init);
        let default_zero = forward_val(&mut engine, "default_zero").unwrap();
        let int_one = forward_val(&mut engine, "int_one").unwrap();
        let float_two = forward_val(&mut engine, "float_two").unwrap();
        let str_three = forward_val(&mut engine, "str_three").unwrap();
        let bool_one = forward_val(&mut engine, "bool_one").unwrap();
        let bool_zero = forward_val(&mut engine, "bool_zero").unwrap();
        let invalid_nan = forward_val(&mut engine, "invalid_nan").unwrap();
        let from_exp = forward_val(&mut engine, "from_exp").unwrap();

        assert_eq!(default_zero.to_num(), f64::from(0));
        assert_eq!(int_one.to_num(), f64::from(1));
        assert_eq!(float_two.to_num(), f64::from(2.1));
        assert_eq!(str_three.to_num(), f64::from(3.2));
        assert_eq!(bool_one.to_num(), f64::from(1));
        assert!(invalid_nan.to_num().is_nan());
        assert_eq!(bool_zero.to_num(), f64::from(0));
        assert_eq!(from_exp.to_num(), f64::from(234));
    }

    #[test]
    fn to_exponential() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var default_exp = Number().toExponential();
        var int_exp = Number(5).toExponential();
        var float_exp = Number(1.234).toExponential();
        var big_exp = Number(1234).toExponential();
        var nan_exp = Number("I am also not a number").toExponential();
        var noop_exp = Number("1.23e+2").toExponential();
        "#;

        forward(&mut engine, init);
        let default_exp = forward(&mut engine, "default_exp");
        let int_exp = forward(&mut engine, "int_exp");
        let float_exp = forward(&mut engine, "float_exp");
        let big_exp = forward(&mut engine, "big_exp");
        let nan_exp = forward(&mut engine, "nan_exp");
        let noop_exp = forward(&mut engine, "noop_exp");

        assert_eq!(default_exp, String::from("0e+0"));
        assert_eq!(int_exp, String::from("5e+0"));
        assert_eq!(float_exp, String::from("1.234e+0"));
        assert_eq!(big_exp, String::from("1.234e+3"));
        assert_eq!(nan_exp, String::from("NaN"));
        assert_eq!(noop_exp, String::from("1.23e+2"));
    }

    #[test]
    fn to_fixed() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var default_fixed = Number().toFixed();
        var pos_fixed = Number("3.456e+4").toFixed();
        var neg_fixed = Number("3.456e-4").toFixed();
        var noop_fixed = Number(5).toFixed();
        var nan_fixed = Number("I am not a number").toFixed();
        "#;

        forward(&mut engine, init);
        let default_fixed = forward(&mut engine, "default_fixed");
        let pos_fixed = forward(&mut engine, "pos_fixed");
        let neg_fixed = forward(&mut engine, "neg_fixed");
        let noop_fixed = forward(&mut engine, "noop_fixed");
        let nan_fixed = forward(&mut engine, "nan_fixed");

        assert_eq!(default_fixed, String::from("0"));
        assert_eq!(pos_fixed, String::from("34560"));
        assert_eq!(neg_fixed, String::from("0"));
        assert_eq!(noop_fixed, String::from("5"));
        assert_eq!(nan_fixed, String::from("NaN"));
    }

    #[test]
    fn to_locale_string() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var default_locale = Number().toLocaleString();
        var small_locale = Number(5).toLocaleString();
        var big_locale = Number("345600").toLocaleString();
        var neg_locale = Number(-25).toLocaleString();
        "#;

        // TODO: We don't actually do any locale checking here
        // To honor the spec we should print numbers according to user locale.

        forward(&mut engine, init);
        let default_locale = forward(&mut engine, "default_locale");
        let small_locale = forward(&mut engine, "small_locale");
        let big_locale = forward(&mut engine, "big_locale");
        let neg_locale = forward(&mut engine, "neg_locale");

        assert_eq!(default_locale, String::from("0"));
        assert_eq!(small_locale, String::from("5"));
        assert_eq!(big_locale, String::from("345600"));
        assert_eq!(neg_locale, String::from("-25"));
    }

    #[test]
    #[ignore]
    fn to_precision() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var default_precision = Number().toPrecision();
        var low_precision = Number(123456789).toPrecision(1);
        var more_precision = Number(123456789).toPrecision(4);
        var exact_precision = Number(123456789).toPrecision(9);
        var over_precision = Number(123456789).toPrecision(50);
        var neg_precision = Number(-123456789).toPrecision(4);
        "#;

        forward(&mut engine, init);
        let default_precision = forward(&mut engine, "default_precision");
        let low_precision = forward(&mut engine, "low_precision");
        let more_precision = forward(&mut engine, "more_precision");
        let exact_precision = forward(&mut engine, "exact_precision");
        let over_precision = forward(&mut engine, "over_precision");
        let neg_precision = forward(&mut engine, "neg_precision");

        assert_eq!(default_precision, String::from("0"));
        assert_eq!(low_precision, String::from("1e+8"));
        assert_eq!(more_precision, String::from("1.235e+8"));
        assert_eq!(exact_precision, String::from("123456789"));
        assert_eq!(
            over_precision,
            String::from("123456789.00000000000000000000000000000000000000000")
        );
        assert_eq!(neg_precision, String::from("-1.235e+8"));
    }

    #[test]
    fn to_string() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        var default_string = Number().toString();
        var int_string = Number(123).toString();
        var float_string = Number(1.234).toString();
        var exp_string = Number("1.2e+4").toString();
        var neg_string = Number(-1.2).toString();
        "#;

        forward(&mut engine, init);
        let default_string = forward(&mut engine, "default_string");
        let int_string = forward(&mut engine, "int_string");
        let float_string = forward(&mut engine, "float_string");
        let exp_string = forward(&mut engine, "exp_string");
        let neg_string = forward(&mut engine, "neg_string");

        assert_eq!(default_string, String::from("0"));
        assert_eq!(int_string, String::from("123"));
        assert_eq!(float_string, String::from("1.234"));
        assert_eq!(exp_string, String::from("12000"));
        assert_eq!(neg_string, String::from("-1.2"));
    }

    #[test]
    fn value_of() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        // TODO: In addition to parsing numbers from strings, parse them bare As of October 2019
        // the parser does not understand scientific e.g., Xe+Y or -Xe-Y notation.
        let init = r#"
        var default_val = Number().valueOf();
        var int_val = Number("123").valueOf();
        var float_val = Number(1.234).valueOf();
        var exp_val = Number("1.2e+4").valueOf()
        var neg_val = Number("-1.2e+4").valueOf()
        "#;

        forward(&mut engine, init);
        let default_val = forward_val(&mut engine, "default_val").unwrap();
        let int_val = forward_val(&mut engine, "int_val").unwrap();
        let float_val = forward_val(&mut engine, "float_val").unwrap();
        let exp_val = forward_val(&mut engine, "exp_val").unwrap();
        let neg_val = forward_val(&mut engine, "neg_val").unwrap();

        assert_eq!(default_val.to_num(), f64::from(0));
        assert_eq!(int_val.to_num(), f64::from(123));
        assert_eq!(float_val.to_num(), f64::from(1.234));
        assert_eq!(exp_val.to_num(), f64::from(12000));
        assert_eq!(neg_val.to_num(), f64::from(-12000));
    }
}
