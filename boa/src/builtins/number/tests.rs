#![allow(clippy::float_cmp)]

use crate::{builtins::Number, forward, forward_val, value::AbstractRelation, Context};

#[test]
fn integer_number_primitive_to_number_object() {
    let mut context = Context::default();

    let scenario = r#"
        (100).toString() === "100"
    "#;

    assert_eq!(forward(&mut context, scenario), "true");
}

#[test]
fn call_number() {
    let mut context = Context::default();
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

    eprintln!("{}", forward(&mut context, init));
    let default_zero = forward_val(&mut context, "default_zero").unwrap();
    let int_one = forward_val(&mut context, "int_one").unwrap();
    let float_two = forward_val(&mut context, "float_two").unwrap();
    let str_three = forward_val(&mut context, "str_three").unwrap();
    let bool_one = forward_val(&mut context, "bool_one").unwrap();
    let bool_zero = forward_val(&mut context, "bool_zero").unwrap();
    let invalid_nan = forward_val(&mut context, "invalid_nan").unwrap();
    let from_exp = forward_val(&mut context, "from_exp").unwrap();

    assert_eq!(default_zero.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(int_one.to_number(&mut context).unwrap(), 1_f64);
    assert_eq!(float_two.to_number(&mut context).unwrap(), 2.1);
    assert_eq!(str_three.to_number(&mut context).unwrap(), 3.2);
    assert_eq!(bool_one.to_number(&mut context).unwrap(), 1_f64);
    assert!(invalid_nan.to_number(&mut context).unwrap().is_nan());
    assert_eq!(bool_zero.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(from_exp.to_number(&mut context).unwrap(), 234_f64);
}

#[test]
fn to_exponential() {
    let mut context = Context::default();
    let init = r#"
        var default_exp = Number().toExponential();
        var int_exp = Number(5).toExponential();
        var float_exp = Number(1.234).toExponential();
        var big_exp = Number(1234).toExponential();
        var nan_exp = Number("I am also not a number").toExponential();
        var noop_exp = Number("1.23e+2").toExponential();
        "#;

    eprintln!("{}", forward(&mut context, init));
    let default_exp = forward(&mut context, "default_exp");
    let int_exp = forward(&mut context, "int_exp");
    let float_exp = forward(&mut context, "float_exp");
    let big_exp = forward(&mut context, "big_exp");
    let nan_exp = forward(&mut context, "nan_exp");
    let noop_exp = forward(&mut context, "noop_exp");

    assert_eq!(default_exp, "\"0e+0\"");
    assert_eq!(int_exp, "\"5e+0\"");
    assert_eq!(float_exp, "\"1.234e+0\"");
    assert_eq!(big_exp, "\"1.234e+3\"");
    assert_eq!(nan_exp, "\"NaN\"");
    assert_eq!(noop_exp, "\"1.23e+2\"");
}

#[test]
fn to_fixed() {
    let mut context = Context::default();
    let init = r#"
        var default_fixed = Number().toFixed();
        var pos_fixed = Number("3.456e+4").toFixed();
        var neg_fixed = Number("3.456e-4").toFixed();
        var noop_fixed = Number(5).toFixed();
        var nan_fixed = Number("I am not a number").toFixed();
        "#;

    eprintln!("{}", forward(&mut context, init));
    let default_fixed = forward(&mut context, "default_fixed");
    let pos_fixed = forward(&mut context, "pos_fixed");
    let neg_fixed = forward(&mut context, "neg_fixed");
    let noop_fixed = forward(&mut context, "noop_fixed");
    let nan_fixed = forward(&mut context, "nan_fixed");

    assert_eq!(default_fixed, "\"0\"");
    assert_eq!(pos_fixed, "\"34560\"");
    assert_eq!(neg_fixed, "\"0\"");
    assert_eq!(noop_fixed, "\"5\"");
    assert_eq!(nan_fixed, "\"NaN\"");
}

#[test]
fn to_locale_string() {
    let mut context = Context::default();
    let init = r#"
        var default_locale = Number().toLocaleString();
        var small_locale = Number(5).toLocaleString();
        var big_locale = Number("345600").toLocaleString();
        var neg_locale = Number(-25).toLocaleString();
        "#;

    // TODO: We don't actually do any locale checking here
    // To honor the spec we should print numbers according to user locale.

    eprintln!("{}", forward(&mut context, init));
    let default_locale = forward(&mut context, "default_locale");
    let small_locale = forward(&mut context, "small_locale");
    let big_locale = forward(&mut context, "big_locale");
    let neg_locale = forward(&mut context, "neg_locale");

    assert_eq!(default_locale, "\"0\"");
    assert_eq!(small_locale, "\"5\"");
    assert_eq!(big_locale, "\"345600\"");
    assert_eq!(neg_locale, "\"-25\"");
}

#[test]
fn to_precision() {
    let mut context = Context::default();
    let init = r#"
        var infinity = (1/0).toPrecision(3);
        var default_precision = Number().toPrecision();
        var explicit_ud_precision = Number().toPrecision(undefined);
        var low_precision = (123456789).toPrecision(1);
        var more_precision = (123456789).toPrecision(4);
        var exact_precision = (123456789).toPrecision(9);
        var over_precision = (123456789).toPrecision(50);
        var neg_precision = (-123456789).toPrecision(4);
        var neg_exponent = (0.1).toPrecision(4);
        var ieee754_limits = (1/3).toPrecision(60);
        "#;

    eprintln!("{}", forward(&mut context, init));
    let infinity = forward(&mut context, "infinity");
    let default_precision = forward(&mut context, "default_precision");
    let explicit_ud_precision = forward(&mut context, "explicit_ud_precision");
    let low_precision = forward(&mut context, "low_precision");
    let more_precision = forward(&mut context, "more_precision");
    let exact_precision = forward(&mut context, "exact_precision");
    let over_precision = forward(&mut context, "over_precision");
    let neg_precision = forward(&mut context, "neg_precision");
    let neg_exponent = forward(&mut context, "neg_exponent");
    let ieee754_limits = forward(&mut context, "ieee754_limits");

    assert_eq!(infinity, String::from("\"Infinity\""));
    assert_eq!(default_precision, String::from("\"0\""));
    assert_eq!(explicit_ud_precision, String::from("\"0\""));
    assert_eq!(low_precision, String::from("\"1e+8\""));
    assert_eq!(more_precision, String::from("\"1.235e+8\""));
    assert_eq!(exact_precision, String::from("\"123456789\""));
    assert_eq!(neg_precision, String::from("\"-1.235e+8\""));
    assert_eq!(
        over_precision,
        String::from("\"123456789.00000000000000000000000000000000000000000\"")
    );
    assert_eq!(neg_exponent, String::from("\"0.1000\""));
    assert_eq!(
        ieee754_limits,
        String::from("\"0.333333333333333314829616256247390992939472198486328125000000\"")
    );

    let expected = "Uncaught \"RangeError\": \"precision must be an integer at least 1 and no greater than 100\"";

    let range_error_1 = r#"(1).toPrecision(101);"#;
    let range_error_2 = r#"(1).toPrecision(0);"#;
    let range_error_3 = r#"(1).toPrecision(-2000);"#;
    let range_error_4 = r#"(1).toPrecision('%');"#;

    assert_eq!(forward(&mut context, range_error_1), expected);
    assert_eq!(forward(&mut context, range_error_2), expected);
    assert_eq!(forward(&mut context, range_error_3), expected);
    assert_eq!(forward(&mut context, range_error_4), expected);
}

#[test]
fn to_string() {
    let mut context = Context::default();

    assert_eq!("\"NaN\"", &forward(&mut context, "Number(NaN).toString()"));
    assert_eq!(
        "\"Infinity\"",
        &forward(&mut context, "Number(1/0).toString()")
    );
    assert_eq!(
        "\"-Infinity\"",
        &forward(&mut context, "Number(-1/0).toString()")
    );
    assert_eq!("\"0\"", &forward(&mut context, "Number(0).toString()"));
    assert_eq!("\"9\"", &forward(&mut context, "Number(9).toString()"));
    assert_eq!("\"90\"", &forward(&mut context, "Number(90).toString()"));
    assert_eq!(
        "\"90.12\"",
        &forward(&mut context, "Number(90.12).toString()")
    );
    assert_eq!("\"0.1\"", &forward(&mut context, "Number(0.1).toString()"));
    assert_eq!(
        "\"0.01\"",
        &forward(&mut context, "Number(0.01).toString()")
    );
    assert_eq!(
        "\"0.0123\"",
        &forward(&mut context, "Number(0.0123).toString()")
    );
    assert_eq!(
        "\"0.00001\"",
        &forward(&mut context, "Number(0.00001).toString()")
    );
    assert_eq!(
        "\"0.000001\"",
        &forward(&mut context, "Number(0.000001).toString()")
    );
    assert_eq!(
        "\"NaN\"",
        &forward(&mut context, "Number(NaN).toString(16)")
    );
    assert_eq!(
        "\"Infinity\"",
        &forward(&mut context, "Number(1/0).toString(16)")
    );
    assert_eq!(
        "\"-Infinity\"",
        &forward(&mut context, "Number(-1/0).toString(16)")
    );
    assert_eq!("\"0\"", &forward(&mut context, "Number(0).toString(16)"));
    assert_eq!("\"9\"", &forward(&mut context, "Number(9).toString(16)"));
    assert_eq!("\"5a\"", &forward(&mut context, "Number(90).toString(16)"));
    assert_eq!(
        "\"5a.1eb851eb852\"",
        &forward(&mut context, "Number(90.12).toString(16)")
    );
    assert_eq!(
        "\"0.1999999999999a\"",
        &forward(&mut context, "Number(0.1).toString(16)")
    );
    assert_eq!(
        "\"0.028f5c28f5c28f6\"",
        &forward(&mut context, "Number(0.01).toString(16)")
    );
    assert_eq!(
        "\"0.032617c1bda511a\"",
        &forward(&mut context, "Number(0.0123).toString(16)")
    );
    assert_eq!(
        "\"605f9f6dd18bc8000\"",
        &forward(&mut context, "Number(111111111111111111111).toString(16)")
    );
    assert_eq!(
        "\"3c3bc3a4a2f75c0000\"",
        &forward(&mut context, "Number(1111111111111111111111).toString(16)")
    );
    assert_eq!(
        "\"25a55a46e5da9a00000\"",
        &forward(&mut context, "Number(11111111111111111111111).toString(16)")
    );
    assert_eq!(
        "\"0.0000a7c5ac471b4788\"",
        &forward(&mut context, "Number(0.00001).toString(16)")
    );
    assert_eq!(
        "\"0.000010c6f7a0b5ed8d\"",
        &forward(&mut context, "Number(0.000001).toString(16)")
    );
    assert_eq!(
        "\"0.000001ad7f29abcaf48\"",
        &forward(&mut context, "Number(0.0000001).toString(16)")
    );
    assert_eq!(
        "\"0.000002036565348d256\"",
        &forward(&mut context, "Number(0.00000012).toString(16)")
    );
    assert_eq!(
        "\"0.0000021047ee22aa466\"",
        &forward(&mut context, "Number(0.000000123).toString(16)")
    );
    assert_eq!(
        "\"0.0000002af31dc4611874\"",
        &forward(&mut context, "Number(0.00000001).toString(16)")
    );
    assert_eq!(
        "\"0.000000338a23b87483be\"",
        &forward(&mut context, "Number(0.000000012).toString(16)")
    );
    assert_eq!(
        "\"0.00000034d3fe36aaa0a2\"",
        &forward(&mut context, "Number(0.0000000123).toString(16)")
    );

    assert_eq!("\"0\"", &forward(&mut context, "Number(-0).toString(16)"));
    assert_eq!("\"-9\"", &forward(&mut context, "Number(-9).toString(16)"));
    assert_eq!(
        "\"-5a\"",
        &forward(&mut context, "Number(-90).toString(16)")
    );
    assert_eq!(
        "\"-5a.1eb851eb852\"",
        &forward(&mut context, "Number(-90.12).toString(16)")
    );
    assert_eq!(
        "\"-0.1999999999999a\"",
        &forward(&mut context, "Number(-0.1).toString(16)")
    );
    assert_eq!(
        "\"-0.028f5c28f5c28f6\"",
        &forward(&mut context, "Number(-0.01).toString(16)")
    );
    assert_eq!(
        "\"-0.032617c1bda511a\"",
        &forward(&mut context, "Number(-0.0123).toString(16)")
    );
    assert_eq!(
        "\"-605f9f6dd18bc8000\"",
        &forward(&mut context, "Number(-111111111111111111111).toString(16)")
    );
    assert_eq!(
        "\"-3c3bc3a4a2f75c0000\"",
        &forward(&mut context, "Number(-1111111111111111111111).toString(16)")
    );
    assert_eq!(
        "\"-25a55a46e5da9a00000\"",
        &forward(
            &mut context,
            "Number(-11111111111111111111111).toString(16)"
        )
    );
    assert_eq!(
        "\"-0.0000a7c5ac471b4788\"",
        &forward(&mut context, "Number(-0.00001).toString(16)")
    );
    assert_eq!(
        "\"-0.000010c6f7a0b5ed8d\"",
        &forward(&mut context, "Number(-0.000001).toString(16)")
    );
    assert_eq!(
        "\"-0.000001ad7f29abcaf48\"",
        &forward(&mut context, "Number(-0.0000001).toString(16)")
    );
    assert_eq!(
        "\"-0.000002036565348d256\"",
        &forward(&mut context, "Number(-0.00000012).toString(16)")
    );
    assert_eq!(
        "\"-0.0000021047ee22aa466\"",
        &forward(&mut context, "Number(-0.000000123).toString(16)")
    );
    assert_eq!(
        "\"-0.0000002af31dc4611874\"",
        &forward(&mut context, "Number(-0.00000001).toString(16)")
    );
    assert_eq!(
        "\"-0.000000338a23b87483be\"",
        &forward(&mut context, "Number(-0.000000012).toString(16)")
    );
    assert_eq!(
        "\"-0.00000034d3fe36aaa0a2\"",
        &forward(&mut context, "Number(-0.0000000123).toString(16)")
    );
}

#[test]
fn num_to_string_exponential() {
    let mut context = Context::default();

    assert_eq!("\"0\"", forward(&mut context, "(0).toString()"));
    assert_eq!("\"0\"", forward(&mut context, "(-0).toString()"));
    assert_eq!(
        "\"111111111111111110000\"",
        forward(&mut context, "(111111111111111111111).toString()")
    );
    assert_eq!(
        "\"1.1111111111111111e+21\"",
        forward(&mut context, "(1111111111111111111111).toString()")
    );
    assert_eq!(
        "\"1.1111111111111111e+22\"",
        forward(&mut context, "(11111111111111111111111).toString()")
    );
    assert_eq!("\"1e-7\"", forward(&mut context, "(0.0000001).toString()"));
    assert_eq!(
        "\"1.2e-7\"",
        forward(&mut context, "(0.00000012).toString()")
    );
    assert_eq!(
        "\"1.23e-7\"",
        forward(&mut context, "(0.000000123).toString()")
    );
    assert_eq!("\"1e-8\"", forward(&mut context, "(0.00000001).toString()"));
    assert_eq!(
        "\"1.2e-8\"",
        forward(&mut context, "(0.000000012).toString()")
    );
    assert_eq!(
        "\"1.23e-8\"",
        forward(&mut context, "(0.0000000123).toString()")
    );
}

#[test]
fn value_of() {
    let mut context = Context::default();
    // TODO: In addition to parsing numbers from strings, parse them bare As of October 2019
    // the parser does not understand scientific e.g., Xe+Y or -Xe-Y notation.
    let init = r#"
        var default_val = Number().valueOf();
        var int_val = Number("123").valueOf();
        var float_val = Number(1.234).valueOf();
        var exp_val = Number("1.2e+4").valueOf()
        var neg_val = Number("-1.2e+4").valueOf()
        "#;

    eprintln!("{}", forward(&mut context, init));
    let default_val = forward_val(&mut context, "default_val").unwrap();
    let int_val = forward_val(&mut context, "int_val").unwrap();
    let float_val = forward_val(&mut context, "float_val").unwrap();
    let exp_val = forward_val(&mut context, "exp_val").unwrap();
    let neg_val = forward_val(&mut context, "neg_val").unwrap();

    assert_eq!(default_val.to_number(&mut context).unwrap(), 0_f64);
    assert_eq!(int_val.to_number(&mut context).unwrap(), 123_f64);
    assert_eq!(float_val.to_number(&mut context).unwrap(), 1.234);
    assert_eq!(exp_val.to_number(&mut context).unwrap(), 12_000_f64);
    assert_eq!(neg_val.to_number(&mut context).unwrap(), -12_000_f64);
}

#[test]
fn equal() {
    assert!(Number::equal(0.0, 0.0));
    assert!(Number::equal(-0.0, 0.0));
    assert!(Number::equal(0.0, -0.0));
    assert!(!Number::equal(f64::NAN, -0.0));
    assert!(!Number::equal(0.0, f64::NAN));

    assert!(Number::equal(1.0, 1.0));
}

#[test]
fn same_value() {
    assert!(Number::same_value(0.0, 0.0));
    assert!(!Number::same_value(-0.0, 0.0));
    assert!(!Number::same_value(0.0, -0.0));
    assert!(!Number::same_value(f64::NAN, -0.0));
    assert!(!Number::same_value(0.0, f64::NAN));
    assert!(Number::equal(1.0, 1.0));
}

#[test]
fn less_than() {
    assert_eq!(
        Number::less_than(f64::NAN, 0.0),
        AbstractRelation::Undefined
    );
    assert_eq!(
        Number::less_than(0.0, f64::NAN),
        AbstractRelation::Undefined
    );
    assert_eq!(
        Number::less_than(f64::NEG_INFINITY, 0.0),
        AbstractRelation::True
    );
    assert_eq!(
        Number::less_than(0.0, f64::NEG_INFINITY),
        AbstractRelation::False
    );
    assert_eq!(
        Number::less_than(f64::INFINITY, 0.0),
        AbstractRelation::False
    );
    assert_eq!(
        Number::less_than(0.0, f64::INFINITY),
        AbstractRelation::True
    );
}

#[test]
fn same_value_zero() {
    assert!(Number::same_value_zero(0.0, 0.0));
    assert!(Number::same_value_zero(-0.0, 0.0));
    assert!(Number::same_value_zero(0.0, -0.0));
    assert!(!Number::same_value_zero(f64::NAN, -0.0));
    assert!(!Number::same_value_zero(0.0, f64::NAN));
    assert!(Number::equal(1.0, 1.0));
}

#[test]
fn from_bigint() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "Number(0n)"), "0",);
    assert_eq!(&forward(&mut context, "Number(100000n)"), "100000",);
    assert_eq!(&forward(&mut context, "Number(100000n)"), "100000",);
    assert_eq!(&forward(&mut context, "Number(1n << 1240n)"), "Infinity",);
}

#[test]
fn number_constants() {
    let mut context = Context::default();

    assert!(!forward_val(&mut context, "Number.EPSILON")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut context, "Number.MAX_SAFE_INTEGER")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut context, "Number.MIN_SAFE_INTEGER")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut context, "Number.MAX_VALUE")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut context, "Number.MIN_VALUE")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut context, "Number.NEGATIVE_INFINITY")
        .unwrap()
        .is_null_or_undefined());
    assert!(!forward_val(&mut context, "Number.POSITIVE_INFINITY")
        .unwrap()
        .is_null_or_undefined());
}

#[test]
fn parse_int_simple() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(\"6\")"), "6");
}

#[test]
fn parse_int_negative() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(\"-9\")"), "-9");
}

#[test]
fn parse_int_already_int() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(100)"), "100");
}

#[test]
fn parse_int_float() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(100.5)"), "100");
}

#[test]
fn parse_int_float_str() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(\"100.5\")"), "100");
}

#[test]
fn parse_int_inferred_hex() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(\"0xA\")"), "10");
}

/// This test demonstrates that this version of parseInt treats strings starting with 0 to be parsed with
/// a radix 10 if no radix is specified. Some alternative implementations default to a radix of 8.
#[test]
fn parse_int_zero_start() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(\"018\")"), "18");
}

#[test]
fn parse_int_varying_radix() {
    let mut context = Context::default();

    let base_str = "1000";

    for radix in 2..36 {
        let expected = i32::from_str_radix(base_str, radix).unwrap();

        assert_eq!(
            forward(
                &mut context,
                &format!("parseInt(\"{}\", {} )", base_str, radix)
            ),
            expected.to_string()
        );
    }
}

#[test]
fn parse_int_negative_varying_radix() {
    let mut context = Context::default();

    let base_str = "-1000";

    for radix in 2..36 {
        let expected = i32::from_str_radix(base_str, radix).unwrap();

        assert_eq!(
            forward(
                &mut context,
                &format!("parseInt(\"{}\", {} )", base_str, radix)
            ),
            expected.to_string()
        );
    }
}

#[test]
fn parse_int_malformed_str() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(\"hello\")"), "NaN");
}

#[test]
fn parse_int_undefined() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(undefined)"), "NaN");
}

/// Shows that no arguments to parseInt is treated the same as if undefined was
/// passed as the first argument.
#[test]
fn parse_int_no_args() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt()"), "NaN");
}

/// Shows that extra arguments to parseInt are ignored.
#[test]
fn parse_int_too_many_args() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseInt(\"100\", 10, 10)"), "100");
}

#[test]
fn parse_float_simple() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseFloat(\"6.5\")"), "6.5");
}

#[test]
fn parse_float_int() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseFloat(10)"), "10");
}

#[test]
fn parse_float_int_str() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseFloat(\"8\")"), "8");
}

#[test]
fn parse_float_already_float() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseFloat(17.5)"), "17.5");
}

#[test]
fn parse_float_negative() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseFloat(\"-99.7\")"), "-99.7");
}

#[test]
fn parse_float_malformed_str() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseFloat(\"hello\")"), "NaN");
}

#[test]
fn parse_float_undefined() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseFloat(undefined)"), "NaN");
}

/// No arguments to parseFloat is treated the same as passing undefined as the first argument.
#[test]
fn parse_float_no_args() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseFloat()"), "NaN");
}

/// Shows that the parseFloat function ignores extra arguments.
#[test]
fn parse_float_too_many_args() {
    let mut context = Context::default();

    assert_eq!(&forward(&mut context, "parseFloat(\"100.5\", 10)"), "100.5");
}

#[test]
fn global_is_finite() {
    let mut context = Context::default();

    assert_eq!("false", &forward(&mut context, "isFinite(Infinity)"));
    assert_eq!("false", &forward(&mut context, "isFinite(NaN)"));
    assert_eq!("false", &forward(&mut context, "isFinite(-Infinity)"));
    assert_eq!("true", &forward(&mut context, "isFinite(0)"));
    assert_eq!("true", &forward(&mut context, "isFinite(2e64)"));
    assert_eq!("true", &forward(&mut context, "isFinite(910)"));
    assert_eq!("true", &forward(&mut context, "isFinite(null)"));
    assert_eq!("true", &forward(&mut context, "isFinite('0')"));
    assert_eq!("false", &forward(&mut context, "isFinite()"));
}

#[test]
fn global_is_nan() {
    let mut context = Context::default();

    assert_eq!("true", &forward(&mut context, "isNaN(NaN)"));
    assert_eq!("true", &forward(&mut context, "isNaN('NaN')"));
    assert_eq!("true", &forward(&mut context, "isNaN(undefined)"));
    assert_eq!("true", &forward(&mut context, "isNaN({})"));
    assert_eq!("false", &forward(&mut context, "isNaN(true)"));
    assert_eq!("false", &forward(&mut context, "isNaN(null)"));
    assert_eq!("false", &forward(&mut context, "isNaN(37)"));
    assert_eq!("false", &forward(&mut context, "isNaN('37')"));
    assert_eq!("false", &forward(&mut context, "isNaN('37.37')"));
    assert_eq!("true", &forward(&mut context, "isNaN('37,5')"));
    assert_eq!("true", &forward(&mut context, "isNaN('123ABC')"));
    // Incorrect due to ToNumber implementation inconsistencies.
    //assert_eq!("false", &forward(&mut context, "isNaN('')"));
    //assert_eq!("false", &forward(&mut context, "isNaN(' ')"));
    assert_eq!("true", &forward(&mut context, "isNaN('blabla')"));
}

#[test]
fn number_is_finite() {
    let mut context = Context::default();

    assert_eq!("false", &forward(&mut context, "Number.isFinite(Infinity)"));
    assert_eq!("false", &forward(&mut context, "Number.isFinite(NaN)"));
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isFinite(-Infinity)")
    );
    assert_eq!("true", &forward(&mut context, "Number.isFinite(0)"));
    assert_eq!("true", &forward(&mut context, "Number.isFinite(2e64)"));
    assert_eq!("true", &forward(&mut context, "Number.isFinite(910)"));
    assert_eq!("false", &forward(&mut context, "Number.isFinite(null)"));
    assert_eq!("false", &forward(&mut context, "Number.isFinite('0')"));
    assert_eq!("false", &forward(&mut context, "Number.isFinite()"));
    assert_eq!("false", &forward(&mut context, "Number.isFinite({})"));
    assert_eq!("true", &forward(&mut context, "Number.isFinite(Number(5))"));
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isFinite(new Number(5))")
    );
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isFinite(new Number(NaN))")
    );
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isFinite(BigInt(5))")
    );
}

#[test]
fn number_is_integer() {
    let mut context = Context::default();

    assert_eq!("true", &forward(&mut context, "Number.isInteger(0)"));
    assert_eq!("true", &forward(&mut context, "Number.isInteger(1)"));
    assert_eq!("true", &forward(&mut context, "Number.isInteger(-100000)"));
    assert_eq!(
        "true",
        &forward(&mut context, "Number.isInteger(99999999999999999999999)")
    );
    assert_eq!("false", &forward(&mut context, "Number.isInteger(0.1)"));
    assert_eq!("false", &forward(&mut context, "Number.isInteger(Math.PI)"));
    assert_eq!("false", &forward(&mut context, "Number.isInteger(NaN)"));
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isInteger(Infinity)")
    );
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isInteger(-Infinity)")
    );
    assert_eq!("false", &forward(&mut context, "Number.isInteger('10')"));
    assert_eq!("false", &forward(&mut context, "Number.isInteger(true)"));
    assert_eq!("false", &forward(&mut context, "Number.isInteger(false)"));
    assert_eq!("false", &forward(&mut context, "Number.isInteger([1])"));
    assert_eq!("true", &forward(&mut context, "Number.isInteger(5.0)"));
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isInteger(5.000000000000001)")
    );
    assert_eq!(
        "true",
        &forward(&mut context, "Number.isInteger(5.0000000000000001)")
    );
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isInteger(Number(5.000000000000001))")
    );
    assert_eq!(
        "true",
        &forward(&mut context, "Number.isInteger(Number(5.0000000000000001))")
    );
    assert_eq!("false", &forward(&mut context, "Number.isInteger()"));
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isInteger(new Number(5))")
    );
}

#[test]
fn number_is_nan() {
    let mut context = Context::default();

    assert_eq!("true", &forward(&mut context, "Number.isNaN(NaN)"));
    assert_eq!("true", &forward(&mut context, "Number.isNaN(Number.NaN)"));
    assert_eq!("true", &forward(&mut context, "Number.isNaN(0 / 0)"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN(undefined)"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN({})"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN(true)"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN(null)"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN(37)"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN('37')"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN('37.37')"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN('37,5')"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN('123ABC')"));
    // Incorrect due to ToNumber implementation inconsistencies.
    //assert_eq!("false", &forward(&mut context, "Number.isNaN('')"));
    //assert_eq!("false", &forward(&mut context, "Number.isNaN(' ')"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN('blabla')"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN(Number(5))"));
    assert_eq!("true", &forward(&mut context, "Number.isNaN(Number(NaN))"));
    assert_eq!("false", &forward(&mut context, "Number.isNaN(BigInt(5))"));
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isNaN(new Number(5))")
    );
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isNaN(new Number(NaN))")
    );
}

#[test]
fn number_is_safe_integer() {
    let mut context = Context::default();

    assert_eq!("true", &forward(&mut context, "Number.isSafeInteger(3)"));
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isSafeInteger(Math.pow(2, 53))")
    );
    assert_eq!(
        "true",
        &forward(&mut context, "Number.isSafeInteger(Math.pow(2, 53) - 1)")
    );
    assert_eq!("false", &forward(&mut context, "Number.isSafeInteger(NaN)"));
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isSafeInteger(Infinity)")
    );
    assert_eq!("false", &forward(&mut context, "Number.isSafeInteger('3')"));
    assert_eq!("false", &forward(&mut context, "Number.isSafeInteger(3.1)"));
    assert_eq!("true", &forward(&mut context, "Number.isSafeInteger(3.0)"));
    assert_eq!(
        "false",
        &forward(&mut context, "Number.isSafeInteger(new Number(5))")
    );
}
