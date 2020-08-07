use super::*;
use crate::{forward, forward_val, Interpreter, Realm};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn is_object() {
    let val = Value::new_object(None);
    assert_eq!(val.is_object(), true);
}

#[test]
fn string_to_value() {
    let s = String::from("Hello");
    let v = Value::from(s);
    assert_eq!(v.is_string(), true);
    assert_eq!(v.is_null(), false);
}

#[test]
fn undefined() {
    let u = Value::Undefined;
    assert_eq!(u.get_type(), Type::Undefined);
    assert_eq!(u.to_string(), "undefined");
}

#[test]
fn get_set_field() {
    let obj = Value::new_object(None);
    // Create string and convert it to a Value
    let s = Value::from("bar");
    obj.set_field("foo", s);
    assert_eq!(obj.get_field("foo").to_string(), "\"bar\"");
}

#[test]
fn integer_is_true() {
    assert_eq!(Value::from(1).to_boolean(), true);
    assert_eq!(Value::from(0).to_boolean(), false);
    assert_eq!(Value::from(-1).to_boolean(), true);
}

#[test]
fn number_is_true() {
    assert_eq!(Value::from(1.0).to_boolean(), true);
    assert_eq!(Value::from(0.1).to_boolean(), true);
    assert_eq!(Value::from(0.0).to_boolean(), false);
    assert_eq!(Value::from(-0.0).to_boolean(), false);
    assert_eq!(Value::from(-1.0).to_boolean(), true);
    assert_eq!(Value::from(NAN).to_boolean(), false);
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

#[test]
fn to_string() {
    let f64_to_str = |f| Value::Rational(f).to_string();

    assert_eq!(f64_to_str(f64::NAN), "NaN");
    assert_eq!(f64_to_str(0.0), "0");
    assert_eq!(f64_to_str(f64::INFINITY), "Infinity");
    assert_eq!(f64_to_str(f64::NEG_INFINITY), "-Infinity");
    assert_eq!(f64_to_str(90.12), "90.12");
    assert_eq!(f64_to_str(111111111111111111111.0), "111111111111111110000");
    assert_eq!(
        f64_to_str(1111111111111111111111.0),
        "1.1111111111111111e+21"
    );

    assert_eq!(f64_to_str(-90.12), "-90.12");

    assert_eq!(
        f64_to_str(-111111111111111111111.0),
        "-111111111111111110000"
    );
    assert_eq!(
        f64_to_str(-1111111111111111111111.0),
        "-1.1111111111111111e+21"
    );

    assert_eq!(f64_to_str(0.0000001), "1e-7");
    assert_eq!(f64_to_str(0.000001), "0.000001");
    assert_eq!(f64_to_str(0.0000002), "2e-7");
    assert_eq!(f64_to_str(-0.0000001), "-1e-7");

    assert_eq!(f64_to_str(3e50), "3e+50");
}

#[test]
fn add_number_and_number() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "1 + 2").unwrap();
    let value = engine.to_int32(&value).unwrap();
    assert_eq!(value, 3);
}

#[test]
fn add_number_and_string() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "1 + \" + 2 = 3\"").unwrap();
    let value = engine.to_string(&value).unwrap();
    assert_eq!(value, "1 + 2 = 3");
}

#[test]
fn add_string_and_string() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "\"Hello\" + \", world\"").unwrap();
    let value = engine.to_string(&value).unwrap();
    assert_eq!(value, "Hello, world");
}

#[test]
fn add_number_object_and_number() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "new Number(10) + 6").unwrap();
    let value = engine.to_int32(&value).unwrap();
    assert_eq!(value, 16);
}

#[test]
fn add_number_object_and_string_object() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "new Number(10) + new String(\"0\")").unwrap();
    let value = engine.to_string(&value).unwrap();
    assert_eq!(value, "100");
}

#[test]
fn sub_number_and_number() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "1 - 999").unwrap();
    let value = engine.to_int32(&value).unwrap();
    assert_eq!(value, -998);
}

#[test]
fn sub_number_object_and_number_object() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "new Number(1) - new Number(999)").unwrap();
    let value = engine.to_int32(&value).unwrap();
    assert_eq!(value, -998);
}

#[test]
fn sub_string_and_number_object() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "'Hello' - new Number(999)").unwrap();
    let value = engine.to_number(&value).unwrap();
    assert!(value.is_nan());
}

#[test]
fn bitand_integer_and_integer() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "0xFFFF & 0xFF").unwrap();
    let value = engine.to_int32(&value).unwrap();
    assert_eq!(value, 255);
}

#[test]
fn bitand_integer_and_rational() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "0xFFFF & 255.5").unwrap();
    let value = engine.to_int32(&value).unwrap();
    assert_eq!(value, 255);
}

#[test]
fn bitand_rational_and_rational() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "255.772 & 255.5").unwrap();
    let value = engine.to_int32(&value).unwrap();
    assert_eq!(value, 255);
}

#[test]
fn display_string() {
    let s = String::from("Hello");
    let v = Value::from(s);
    assert_eq!(v.to_string(), "\"Hello\"");
}

#[test]
fn display_array_string() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let value = forward_val(&mut engine, "[\"Hello\"]").unwrap();
    assert_eq!(value.to_string(), "[ \"Hello\" ]");
}

#[test]
fn display_boolean_object() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let d_obj = r#"
        let bool = new Boolean(0);
        bool
    "#;
    let value = forward_val(&mut engine, d_obj).unwrap();
    assert_eq!(value.to_string(), "Boolean { false }")
}

#[test]
fn display_number_object() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let d_obj = r#"
        let num = new Number(3.14);
        num
    "#;
    let value = forward_val(&mut engine, d_obj).unwrap();
    assert_eq!(value.to_string(), "Number { 3.14 }")
}

#[test]
fn display_negative_zero_object() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let d_obj = r#"
        let num = new Number(-0);
        num
    "#;
    let value = forward_val(&mut engine, d_obj).unwrap();
    assert_eq!(value.to_string(), "Number { -0 }")
}

#[test]
fn debug_object() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let value = forward_val(&mut engine, "new Array([new Date()])").unwrap();

    // We don't care about the contents of the debug display (it is *debug* after all).
    // In the commit that this test was added, this would cause a stack overflow, so
    // executing Debug::fmt is the assertion.
    let _ = format!("{:?}", value);
}

#[test]
#[ignore] // TODO: Once objects are printed in a simpler way this test can be simplified and used
fn display_object() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let d_obj = r#"
        let o = {a: 'a'};
        o
    "#;
    let value = forward_val(&mut engine, d_obj).unwrap();
    assert_eq!(
        value.to_string(),
        r#"{
   a: "a",
__proto__: {
constructor: {
setPrototypeOf: {
          length: 2
            },
   prototype: [Cycle],
        name: "Object",
      length: 1,
defineProperty: {
          length: 3
            },
getPrototypeOf: {
          length: 1
            },
          is: {
          length: 2
            },
   __proto__: {
     constructor: {
                name: "Function",
           prototype: [Cycle],
              length: 1,
           __proto__: undefined
                },
       __proto__: undefined
            }
        },
hasOwnProperty: {
      length: 0
        },
propertyIsEnumerable: {
      length: 0
        },
toString: {
      length: 0
        }
    }
}"#
    );
}
