use super::*;
use crate::{forward, forward_val, Interpreter, Realm};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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
    let u = Value::Undefined;
    assert_eq!(u.get_type(), Type::Undefined);
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

    assert_eq!(
        forward(
            &mut engine,
            "Number.POSITIVE_INFINITY === Number.POSITIVE_INFINITY"
        ),
        "true"
    );
    assert_eq!(
        forward(
            &mut engine,
            "Number.NEGAVIVE_INFINITY === Number.NEGAVIVE_INFINITY"
        ),
        "true"
    );
}

/// Helper function to get the hash of a `Value`.
fn hash_value(value: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn hash_undefined() {
    let value1 = Value::undefined();
    let value_clone = value1.clone();
    assert_eq!(value1, value_clone);

    let value2 = Value::undefined();
    assert_eq!(value1, value2);

    assert_eq!(hash_value(&value1), hash_value(&value_clone));
    assert_eq!(hash_value(&value2), hash_value(&value_clone));
}

#[test]
fn hash_rational() {
    let value1 = Value::rational(1.0);
    let value2 = Value::rational(1.0);
    assert_eq!(value1, value2);
    assert_eq!(hash_value(&value1), hash_value(&value2));

    let nan = Value::nan();
    assert_eq!(nan, nan);
    assert_eq!(hash_value(&nan), hash_value(&nan));
    assert_ne!(hash_value(&nan), hash_value(&Value::rational(1.0)));
}

#[test]
fn hash_object() {
    let object1 = Value::object(Object::default());
    assert_eq!(object1, object1);
    assert_eq!(object1, object1.clone());

    let object2 = Value::object(Object::default());
    assert_ne!(object1, object2);

    assert_eq!(hash_value(&object1), hash_value(&object1.clone()));
    assert_ne!(hash_value(&object1), hash_value(&object2));
}

#[test]
fn get_types() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    assert_eq!(
        forward_val(&mut engine, "undefined").unwrap().get_type(),
        Type::Undefined
    );
    assert_eq!(
        forward_val(&mut engine, "1").unwrap().get_type(),
        Type::Number
    );
    assert_eq!(
        forward_val(&mut engine, "1.5").unwrap().get_type(),
        Type::Number
    );
    assert_eq!(
        forward_val(&mut engine, "BigInt(\"123442424242424424242424242\")")
            .unwrap()
            .get_type(),
        Type::BigInt
    );
    assert_eq!(
        forward_val(&mut engine, "true").unwrap().get_type(),
        Type::Boolean
    );
    assert_eq!(
        forward_val(&mut engine, "false").unwrap().get_type(),
        Type::Boolean
    );
    assert_eq!(
        forward_val(&mut engine, "function foo() {console.log(\"foo\");}")
            .unwrap()
            .get_type(),
        Type::Undefined
    );
    assert_eq!(
        forward_val(&mut engine, "null").unwrap().get_type(),
        Type::Null
    );
    assert_eq!(
        forward_val(&mut engine, "var x = {arg: \"hi\", foo: \"hello\"}; x")
            .unwrap()
            .get_type(),
        Type::Object
    );
    assert_eq!(
        forward_val(&mut engine, "\"Hi\"").unwrap().get_type(),
        Type::String
    );
    assert_eq!(
        forward_val(&mut engine, "Symbol()").unwrap().get_type(),
        Type::Symbol
    );
}
