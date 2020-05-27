use super::*;
use crate::{forward, Interpreter, Realm};

#[test]
fn check_is_object() {
    let val = Value::new_object(None);
    assert_eq!(val.is_object(), true);
}

#[test]
fn check_string_to_value() {
    let s = String::from("Hello");
    let v = Value::from(s);
    assert_eq!(v.is_string(), true);
    assert_eq!(v.is_null(), false);
}

#[test]
fn check_undefined() {
    let u = ValueData::Undefined;
    assert_eq!(u.get_type(), "undefined");
    assert_eq!(u.to_string(), "undefined");
}

#[test]
fn check_get_set_field() {
    let obj = Value::new_object(None);
    // Create string and convert it to a Value
    let s = Value::from("bar");
    obj.set_field("foo", s);
    assert_eq!(obj.get_field("foo").to_string(), "bar");
}

#[test]
fn check_integer_is_true() {
    assert_eq!(Value::from(1).is_true(), true);
    assert_eq!(Value::from(0).is_true(), false);
    assert_eq!(Value::from(-1).is_true(), true);
}

#[test]
fn check_number_is_true() {
    assert_eq!(Value::from(1.0).is_true(), true);
    assert_eq!(Value::from(0.1).is_true(), true);
    assert_eq!(Value::from(0.0).is_true(), false);
    assert_eq!(Value::from(-0.0).is_true(), false);
    assert_eq!(Value::from(-1.0).is_true(), true);
    assert_eq!(Value::from(NAN).is_true(), false);
}

// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Equality_comparisons_and_sameness
#[test]
fn abstract_equality_comparison() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(forward(&mut engine, "undefined == undefined"), "true");
    assert_eq!(forward(&mut engine, "null == null"), "true");
    assert_eq!(forward(&mut engine, "true == true"), "true");
    assert_eq!(forward(&mut engine, "false == false"), "true");
    assert_eq!(forward(&mut engine, "'foo' == 'foo'"), "true");
    assert_eq!(forward(&mut engine, "0 == 0"), "true");
    assert_eq!(forward(&mut engine, "+0 == -0"), "true");
    assert_eq!(forward(&mut engine, "+0 == 0"), "true");
    assert_eq!(forward(&mut engine, "-0 == 0"), "true");
    assert_eq!(forward(&mut engine, "0 == false"), "true");
    assert_eq!(forward(&mut engine, "'' == false"), "true");
    assert_eq!(forward(&mut engine, "'' == 0"), "true");
    assert_eq!(forward(&mut engine, "'17' == 17"), "true");
    assert_eq!(forward(&mut engine, "[1,2] == '1,2'"), "true");
    assert_eq!(forward(&mut engine, "new String('foo') == 'foo'"), "true");
    assert_eq!(forward(&mut engine, "null == undefined"), "true");
    assert_eq!(forward(&mut engine, "undefined == null"), "true");
    assert_eq!(forward(&mut engine, "null == false"), "false");
    assert_eq!(forward(&mut engine, "[] == ![]"), "true");
    assert_eq!(
        forward(&mut engine, "a = { foo: 'bar' }; b = { foo: 'bar'}; a == b"),
        "false"
    );
    assert_eq!(
        forward(&mut engine, "new String('foo') == new String('foo')"),
        "false"
    );
    assert_eq!(forward(&mut engine, "0 == null"), "false");

    assert_eq!(forward(&mut engine, "0 == '-0'"), "true");
    assert_eq!(forward(&mut engine, "0 == '+0'"), "true");
    assert_eq!(forward(&mut engine, "'+0' == 0"), "true");
    assert_eq!(forward(&mut engine, "'-0' == 0"), "true");

    assert_eq!(forward(&mut engine, "0 == NaN"), "false");
    assert_eq!(forward(&mut engine, "'foo' == NaN"), "false");
    assert_eq!(forward(&mut engine, "NaN == NaN"), "false");
}
