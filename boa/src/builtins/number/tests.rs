#![allow(clippy::float_cmp)]

use crate::{
    builtins::{Number, Value},
    exec::Interpreter,
    forward, forward_val,
    realm::Realm,
};

#[test]
fn check_number_constructor_is_function() {
    let global = Value::new_object(None);
    let number_constructor = Number::create(&global);
    assert_eq!(number_constructor.is_function(), true);
}

#[test]
fn call_number() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
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

    eprintln!("{}", forward(&mut engine, init));
    let default_zero = forward_val(&mut engine, "default_zero").unwrap();
    let int_one = forward_val(&mut engine, "int_one").unwrap();
    let float_two = forward_val(&mut engine, "float_two").unwrap();
    let str_three = forward_val(&mut engine, "str_three").unwrap();
    let bool_one = forward_val(&mut engine, "bool_one").unwrap();
    let bool_zero = forward_val(&mut engine, "bool_zero").unwrap();
    let invalid_nan = forward_val(&mut engine, "invalid_nan").unwrap();
    let from_exp = forward_val(&mut engine, "from_exp").unwrap();

    assert_eq!(default_zero.to_number(), 0_f64);
    assert_eq!(int_one.to_number(), 1_f64);
    assert_eq!(float_two.to_number(), 2.1);
    assert_eq!(str_three.to_number(), 3.2);
    assert_eq!(bool_one.to_number(), 1_f64);
    assert!(invalid_nan.to_number().is_nan());
    assert_eq!(bool_zero.to_number(), 0_f64);
    assert_eq!(from_exp.to_number(), 234_f64);
}

#[test]
fn to_exponential() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var default_exp = Number().toExponential();
        var int_exp = Number(5).toExponential();
        var float_exp = Number(1.234).toExponential();
        var big_exp = Number(1234).toExponential();
        var nan_exp = Number("I am also not a number").toExponential();
        var noop_exp = Number("1.23e+2").toExponential();
        "#;

    eprintln!("{}", forward(&mut engine, init));
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
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var default_fixed = Number().toFixed();
        var pos_fixed = Number("3.456e+4").toFixed();
        var neg_fixed = Number("3.456e-4").toFixed();
        var noop_fixed = Number(5).toFixed();
        var nan_fixed = Number("I am not a number").toFixed();
        "#;

    eprintln!("{}", forward(&mut engine, init));
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
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var default_locale = Number().toLocaleString();
        var small_locale = Number(5).toLocaleString();
        var big_locale = Number("345600").toLocaleString();
        var neg_locale = Number(-25).toLocaleString();
        "#;

    // TODO: We don't actually do any locale checking here
    // To honor the spec we should print numbers according to user locale.

    eprintln!("{}", forward(&mut engine, init));
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
    let mut engine = Interpreter::new(realm);
    let init = r#"
        var default_precision = Number().toPrecision();
        var low_precision = Number(123456789).toPrecision(1);
        var more_precision = Number(123456789).toPrecision(4);
        var exact_precision = Number(123456789).toPrecision(9);
        var over_precision = Number(123456789).toPrecision(50);
        var neg_precision = Number(-123456789).toPrecision(4);
        "#;

    eprintln!("{}", forward(&mut engine, init));
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
    let mut engine = Interpreter::new(realm);

    assert_eq!("NaN", &forward(&mut engine, "Number(NaN).toString()"));
    assert_eq!("Infinity", &forward(&mut engine, "Number(1/0).toString()"));
    assert_eq!(
        "-Infinity",
        &forward(&mut engine, "Number(-1/0).toString()")
    );
    assert_eq!("0", &forward(&mut engine, "Number(0).toString()"));
    assert_eq!("9", &forward(&mut engine, "Number(9).toString()"));
    assert_eq!("90", &forward(&mut engine, "Number(90).toString()"));
    assert_eq!("90.12", &forward(&mut engine, "Number(90.12).toString()"));
    assert_eq!("0.1", &forward(&mut engine, "Number(0.1).toString()"));
    assert_eq!("0.01", &forward(&mut engine, "Number(0.01).toString()"));
    assert_eq!("0.0123", &forward(&mut engine, "Number(0.0123).toString()"));
    assert_eq!(
        "0.00001",
        &forward(&mut engine, "Number(0.00001).toString()")
    );
    assert_eq!(
        "0.000001",
        &forward(&mut engine, "Number(0.000001).toString()")
    );
    assert_eq!("NaN", &forward(&mut engine, "Number(NaN).toString(16)"));
    assert_eq!(
        "Infinity",
        &forward(&mut engine, "Number(1/0).toString(16)")
    );
    assert_eq!(
        "-Infinity",
        &forward(&mut engine, "Number(-1/0).toString(16)")
    );
    assert_eq!("0", &forward(&mut engine, "Number(0).toString(16)"));
    assert_eq!("9", &forward(&mut engine, "Number(9).toString(16)"));
    assert_eq!("5a", &forward(&mut engine, "Number(90).toString(16)"));
    assert_eq!(
        "5a.1eb851eb852",
        &forward(&mut engine, "Number(90.12).toString(16)")
    );
    assert_eq!(
        "0.1999999999999a",
        &forward(&mut engine, "Number(0.1).toString(16)")
    );
    assert_eq!(
        "0.028f5c28f5c28f6",
        &forward(&mut engine, "Number(0.01).toString(16)")
    );
    assert_eq!(
        "0.032617c1bda511a",
        &forward(&mut engine, "Number(0.0123).toString(16)")
    );
    assert_eq!(
        "605f9f6dd18bc8000",
        &forward(&mut engine, "Number(111111111111111111111).toString(16)")
    );
    assert_eq!(
        "3c3bc3a4a2f75c0000",
        &forward(&mut engine, "Number(1111111111111111111111).toString(16)")
    );
    assert_eq!(
        "25a55a46e5da9a00000",
        &forward(&mut engine, "Number(11111111111111111111111).toString(16)")
    );
    assert_eq!(
        "0.0000a7c5ac471b4788",
        &forward(&mut engine, "Number(0.00001).toString(16)")
    );
    assert_eq!(
        "0.000010c6f7a0b5ed8d",
        &forward(&mut engine, "Number(0.000001).toString(16)")
    );
    assert_eq!(
        "0.000001ad7f29abcaf48",
        &forward(&mut engine, "Number(0.0000001).toString(16)")
    );
    assert_eq!(
        "0.000002036565348d256",
        &forward(&mut engine, "Number(0.00000012).toString(16)")
    );
    assert_eq!(
        "0.0000021047ee22aa466",
        &forward(&mut engine, "Number(0.000000123).toString(16)")
    );
    assert_eq!(
        "0.0000002af31dc4611874",
        &forward(&mut engine, "Number(0.00000001).toString(16)")
    );
    assert_eq!(
        "0.000000338a23b87483be",
        &forward(&mut engine, "Number(0.000000012).toString(16)")
    );
    assert_eq!(
        "0.00000034d3fe36aaa0a2",
        &forward(&mut engine, "Number(0.0000000123).toString(16)")
    );

    assert_eq!("0", &forward(&mut engine, "Number(-0).toString(16)"));
    assert_eq!("-9", &forward(&mut engine, "Number(-9).toString(16)"));
    assert_eq!("-5a", &forward(&mut engine, "Number(-90).toString(16)"));
    assert_eq!(
        "-5a.1eb851eb852",
        &forward(&mut engine, "Number(-90.12).toString(16)")
    );
    assert_eq!(
        "-0.1999999999999a",
        &forward(&mut engine, "Number(-0.1).toString(16)")
    );
    assert_eq!(
        "-0.028f5c28f5c28f6",
        &forward(&mut engine, "Number(-0.01).toString(16)")
    );
    assert_eq!(
        "-0.032617c1bda511a",
        &forward(&mut engine, "Number(-0.0123).toString(16)")
    );
    assert_eq!(
        "-605f9f6dd18bc8000",
        &forward(&mut engine, "Number(-111111111111111111111).toString(16)")
    );
    assert_eq!(
        "-3c3bc3a4a2f75c0000",
        &forward(&mut engine, "Number(-1111111111111111111111).toString(16)")
    );
    assert_eq!(
        "-25a55a46e5da9a00000",
        &forward(&mut engine, "Number(-11111111111111111111111).toString(16)")
    );
    assert_eq!(
        "-0.0000a7c5ac471b4788",
        &forward(&mut engine, "Number(-0.00001).toString(16)")
    );
    assert_eq!(
        "-0.000010c6f7a0b5ed8d",
        &forward(&mut engine, "Number(-0.000001).toString(16)")
    );
    assert_eq!(
        "-0.000001ad7f29abcaf48",
        &forward(&mut engine, "Number(-0.0000001).toString(16)")
    );
    assert_eq!(
        "-0.000002036565348d256",
        &forward(&mut engine, "Number(-0.00000012).toString(16)")
    );
    assert_eq!(
        "-0.0000021047ee22aa466",
        &forward(&mut engine, "Number(-0.000000123).toString(16)")
    );
    assert_eq!(
        "-0.0000002af31dc4611874",
        &forward(&mut engine, "Number(-0.00000001).toString(16)")
    );
    assert_eq!(
        "-0.000000338a23b87483be",
        &forward(&mut engine, "Number(-0.000000012).toString(16)")
    );
    assert_eq!(
        "-0.00000034d3fe36aaa0a2",
        &forward(&mut engine, "Number(-0.0000000123).toString(16)")
    );
}

#[test]
#[ignore]
// This tests fail for now since the Rust's default formatting for exponential format does not match the js spec.
// https://github.com/jasonwilliams/boa/pull/381#discussion_r422458544
fn num_to_string_exponential() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(
        String::from("111111111111111110000"),
        forward(&mut engine, "Number(111111111111111111111).toString()")
    );
    assert_eq!(
        String::from("1.1111111111111111e+21"),
        forward(&mut engine, "Number(1111111111111111111111).toString()")
    );
    assert_eq!(
        String::from("1.1111111111111111e+22"),
        forward(&mut engine, "Number(11111111111111111111111).toString()")
    );
    assert_eq!(
        String::from("1e-7"),
        forward(&mut engine, "Number(0.0000001).toString()")
    );
    assert_eq!(
        String::from("1.2e-7"),
        forward(&mut engine, "Number(0.00000012).toString()")
    );
    assert_eq!(
        String::from("1.23e-7"),
        forward(&mut engine, "Number(0.000000123).toString()")
    );
    assert_eq!(
        String::from("1e-8"),
        forward(&mut engine, "Number(0.00000001).toString()")
    );
    assert_eq!(
        String::from("1.2e-8"),
        forward(&mut engine, "Number(0.000000012).toString()")
    );
    assert_eq!(
        String::from("1.23e-8"),
        forward(&mut engine, "Number(0.0000000123).toString()")
    );
}

#[test]
fn value_of() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    // TODO: In addition to parsing numbers from strings, parse them bare As of October 2019
    // the parser does not understand scientific e.g., Xe+Y or -Xe-Y notation.
    let init = r#"
        var default_val = Number().valueOf();
        var int_val = Number("123").valueOf();
        var float_val = Number(1.234).valueOf();
        var exp_val = Number("1.2e+4").valueOf()
        var neg_val = Number("-1.2e+4").valueOf()
        "#;

    eprintln!("{}", forward(&mut engine, init));
    let default_val = forward_val(&mut engine, "default_val").unwrap();
    let int_val = forward_val(&mut engine, "int_val").unwrap();
    let float_val = forward_val(&mut engine, "float_val").unwrap();
    let exp_val = forward_val(&mut engine, "exp_val").unwrap();
    let neg_val = forward_val(&mut engine, "neg_val").unwrap();

    assert_eq!(default_val.to_number(), 0_f64);
    assert_eq!(int_val.to_number(), 123_f64);
    assert_eq!(float_val.to_number(), 1.234);
    assert_eq!(exp_val.to_number(), 12_000_f64);
    assert_eq!(neg_val.to_number(), -12_000_f64);
}

#[test]
fn equal() {
    assert_eq!(Number::equals(0.0, 0.0), true);
    assert_eq!(Number::equals(-0.0, 0.0), true);
    assert_eq!(Number::equals(0.0, -0.0), true);
    assert_eq!(Number::equals(f64::NAN, -0.0), false);
    assert_eq!(Number::equals(0.0, f64::NAN), false);

    assert_eq!(Number::equals(1.0, 1.0), true);
}

#[test]
fn same_value() {
    assert_eq!(Number::same_value(0.0, 0.0), true);
    assert_eq!(Number::same_value(-0.0, 0.0), false);
    assert_eq!(Number::same_value(0.0, -0.0), false);
    assert_eq!(Number::same_value(f64::NAN, -0.0), false);
    assert_eq!(Number::same_value(0.0, f64::NAN), false);
    assert_eq!(Number::equals(1.0, 1.0), true);
}

#[test]
fn same_value_zero() {
    assert_eq!(Number::same_value_zero(0.0, 0.0), true);
    assert_eq!(Number::same_value_zero(-0.0, 0.0), true);
    assert_eq!(Number::same_value_zero(0.0, -0.0), true);
    assert_eq!(Number::same_value_zero(f64::NAN, -0.0), false);
    assert_eq!(Number::same_value_zero(0.0, f64::NAN), false);
    assert_eq!(Number::equals(1.0, 1.0), true);
}

#[test]
fn from_bigint() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(&forward(&mut engine, "Number(0n)"), "0",);
    assert_eq!(&forward(&mut engine, "Number(100000n)"), "100000",);
    assert_eq!(&forward(&mut engine, "Number(100000n)"), "100000",);
    assert_eq!(&forward(&mut engine, "Number(1n << 1240n)"), "Infinity",);
}

#[test]
fn number_constants() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert!(!forward_val(&mut engine, "Number.EPSILON")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut engine, "Number.MAX_SAFE_INTEGER")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut engine, "Number.MIN_SAFE_INTEGER")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut engine, "Number.MAX_VALUE")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut engine, "Number.MIN_VALUE")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut engine, "Number.NEGATIVE_INFINITY")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut engine, "Number.POSITIVE_INFINITY")
        .unwrap()
        .is_null_or_undefined());
}
