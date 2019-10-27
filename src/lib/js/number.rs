use crate::{
    exec::Interpreter,
    js::value::{ResultValue, Value},
};

/// Create a new number [[Construct]]
pub fn make_number(_this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number-constructor-number-value
pub fn call_number(_this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.toexponential
pub fn to_expotential(_this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.tofixed
pub fn to_fixed(_this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.tolocalestring
pub fn to_locale_string(_this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.toprecision
pub fn to_precision(_this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.tostring
pub fn to_string(_this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// https://tc39.es/ecma262/#sec-number.prototype.valueof
pub fn value_of(_this: &Value, _args: &[Value], _ctx: &mut Interpreter) -> ResultValue {
    unimplemented!()
}

/// Create a new `Number` object
pub fn create_constructor(_global: &Value) -> Value {
    unimplemented!()
}

/// Iniitalize the `Number` object on the global object
pub fn init(_global: &Value) {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{exec::Executor, forward, forward_val, js::value::ValueData, realm::Realm};
    use std::f64::NAN;

    #[test]
    fn check_number_constructor_is_function() {
        let global = ValueData::new_obj(None);
        let number_constructor = create_constructor(&global);
        assert_eq!(number_constructor.is_function(), true);
    }

    #[test]
    fn make_number() {
        unimplemented!()
    }

    #[test]
    pub fn call_number() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const default_zero = new Number();
        const int_one = new Number(1);
        const float_two = new Number(2.1);
        const str_three = new Number('3.2');
        const bool_one = new Number(true);
        const bool_zero = new Number(true);
        const invalid_nan = new Number("I am not a number");
        const from_exp = new Number("2.34e+2");
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
        assert_eq!(bool_zero.to_num(), f64::from(0));
        assert_eq!(invalid_nan.to_num(), NAN);
        assert_eq!(from_exp.to_num(), f64::from(234));

        for v in vec![
            &default_zero,
            &int_one,
            &float_two,
            &str_three,
            &bool_one,
            &invalid_nan,
            &from_exp,
        ]
        .iter()
        {
            assert_eq!(v.is_object(), true);
            assert_eq!(v.is_num(), true);
        }
    }

    #[test]
    pub fn to_expotential() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const default_exp = Number().toExponential();
        const int_exp = Number(5).toExponential();
        const float_exp = Number(1.234).toExponential();
        cont big_exp = Number(1234).toExponential();
        cont nan_exp = Number("I am also not a number").toExponential();
        cont noop_exp = Number("1.23e+2").toExponential();
        "#;

        forward(&mut engine, init);
        let default_exp = forward(&mut engine, "default_exp");
        let int_exp = forward(&mut engine, "int_exp");
        let float_exp = forward(&mut engine, "float_exp");
        let big_exp = forward(&mut engine, "big_exp");
        let nan_exp = forward(&mut engine, "nan_exp");
        let noop_exp = forward(&mut engine, "noop_exp");

        assert_eq!(default_exp, String::from("0e+0"));
        assert_eq!(int_exp, String::from("5"));
        assert_eq!(float_exp, String::from("1.234e+0"));
        assert_eq!(big_exp, String::from("5e+0"));
        assert_eq!(nan_exp, String::from("1.234e+3"));
        assert_eq!(default_exp, String::from("Nan"));
        assert_eq!(noop_exp, String::from("1.23e+2"));
    }

    #[test]
    pub fn to_fixed() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const default_fixed = Number().toFixed();
        const pos_fixed = Number("3.456e+4").toFixed();
        const neg_fixed = Number("3.456e-4").toFixed();
        const noop_fixed = Number(5).toFixed();
        const nan_fixed = Number("I am not a number").toFixed();
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
    pub fn to_locale_string() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const default_locale = Number().toLocaleString();
        const small_locale = Number(5).toLocaleString();
        const big_locale = Number("345600").toLocaleString();
        const neg_locale = Number(-25).toLocaleString();
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
        assert_eq!(neg_locale, String::from("345600"));
    }

    #[test]
    pub fn to_precision() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const default_precision = Number().toPrecision();
        const low_precision = Number(123456789).toPrecision(1);
        const more_precision = Number(123456789).toPrecision(4);
        const exact_precision = Number(123456789).toPrecision(9);
        const over_precision = Number(123456789).toPrecision(50);
        const neg_precision = Number(-123456789).toPrecision(4);
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
    pub fn to_string() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const default_string = Number().toString();
        const int_string = Number(123).toString();
        const float_string = Number(1.234).toString();
        const exp_string = Number(1.2e+4).toString();
        const neg_string = Number(-1.2).toString();
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
        assert_eq!(exp_string, String::from("1200"));
        assert_eq!(neg_string, String::from("-1.2"));
    }

    #[test]
    pub fn value_of() {
        let realm = Realm::create();
        let mut engine = Executor::new(realm);
        let init = r#"
        const default_val = Number().valueOf();
        const int_val = Number("123").valueOf();
        const float_val = Number(1.234).valueOf();
        const exp_val = Number(1.2e+4).valueOf()
        const neg_val = Number(-1.2e+4).valueOf()
        "#;

        forward(&mut engine, init);
        let default_val = forward_val(&mut engine, "default_val").unwrap();
        let int_val = forward_val(&mut engine, "int_val").unwrap();
        let float_val = forward_val(&mut engine, "float_val").unwrap();
        let exp_val = forward_val(&mut engine, "exp_val").unwrap();
        let neg_val = forward_val(&mut engine, "neg_val").unwrap();

        for v in vec![&default_val, &int_val, &float_val, &exp_val, &neg_val].iter() {
            assert_eq!(v.is_object(), true);
            assert_eq!(v.is_num(), true);
        }

        assert_eq!(default_val.to_num(), f64::from(0));
        assert_eq!(int_val.to_num(), f64::from(123));
        assert_eq!(float_val.to_num(), f64::from(1.234));
        assert_eq!(exp_val.to_num(), f64::from(1200));
        assert_eq!(neg_val.to_num(), f64::from(-1200));
    }
}
