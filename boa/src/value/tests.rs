#![allow(clippy::float_cmp)]

use super::*;
use crate::{forward, forward_val, Context};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn is_object() {
    let context = Context::new();
    let val = Value::new_object(&context);
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
    assert_eq!(u.display().to_string(), "undefined");
}

#[test]
fn get_set_field() {
    let mut context = Context::new();
    let obj = Value::new_object(&context);
    // Create string and convert it to a Value
    let s = Value::from("bar");
    obj.set_field("foo", s, &mut context).unwrap();
    assert_eq!(
        obj.get_field("foo", &mut context)
            .unwrap()
            .display()
            .to_string(),
        "\"bar\""
    );
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
    let mut context = Context::new();

    assert_eq!(forward(&mut context, "undefined == undefined"), "true");
    assert_eq!(forward(&mut context, "null == null"), "true");
    assert_eq!(forward(&mut context, "true == true"), "true");
    assert_eq!(forward(&mut context, "false == false"), "true");
    assert_eq!(forward(&mut context, "'foo' == 'foo'"), "true");
    assert_eq!(forward(&mut context, "0 == 0"), "true");
    assert_eq!(forward(&mut context, "+0 == -0"), "true");
    assert_eq!(forward(&mut context, "+0 == 0"), "true");
    assert_eq!(forward(&mut context, "-0 == 0"), "true");
    assert_eq!(forward(&mut context, "0 == false"), "true");
    assert_eq!(forward(&mut context, "'' == false"), "true");
    assert_eq!(forward(&mut context, "'' == 0"), "true");
    assert_eq!(forward(&mut context, "'17' == 17"), "true");
    assert_eq!(forward(&mut context, "[1,2] == '1,2'"), "true");
    assert_eq!(forward(&mut context, "new String('foo') == 'foo'"), "true");
    assert_eq!(forward(&mut context, "null == undefined"), "true");
    assert_eq!(forward(&mut context, "undefined == null"), "true");
    assert_eq!(forward(&mut context, "null == false"), "false");
    assert_eq!(forward(&mut context, "[] == ![]"), "true");
    assert_eq!(
        forward(
            &mut context,
            "a = { foo: 'bar' }; b = { foo: 'bar'}; a == b"
        ),
        "false"
    );
    assert_eq!(
        forward(&mut context, "new String('foo') == new String('foo')"),
        "false"
    );
    assert_eq!(forward(&mut context, "0 == null"), "false");

    assert_eq!(forward(&mut context, "0 == '-0'"), "true");
    assert_eq!(forward(&mut context, "0 == '+0'"), "true");
    assert_eq!(forward(&mut context, "'+0' == 0"), "true");
    assert_eq!(forward(&mut context, "'-0' == 0"), "true");

    assert_eq!(forward(&mut context, "0 == NaN"), "false");
    assert_eq!(forward(&mut context, "'foo' == NaN"), "false");
    assert_eq!(forward(&mut context, "NaN == NaN"), "false");

    assert_eq!(
        forward(
            &mut context,
            "Number.POSITIVE_INFINITY === Number.POSITIVE_INFINITY"
        ),
        "true"
    );
    assert_eq!(
        forward(
            &mut context,
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
    let mut context = Context::new();

    assert_eq!(
        forward_val(&mut context, "undefined").unwrap().get_type(),
        Type::Undefined
    );
    assert_eq!(
        forward_val(&mut context, "1").unwrap().get_type(),
        Type::Number
    );
    assert_eq!(
        forward_val(&mut context, "1.5").unwrap().get_type(),
        Type::Number
    );
    assert_eq!(
        forward_val(&mut context, "BigInt(\"123442424242424424242424242\")")
            .unwrap()
            .get_type(),
        Type::BigInt
    );
    assert_eq!(
        forward_val(&mut context, "true").unwrap().get_type(),
        Type::Boolean
    );
    assert_eq!(
        forward_val(&mut context, "false").unwrap().get_type(),
        Type::Boolean
    );
    assert_eq!(
        forward_val(&mut context, "function foo() {console.log(\"foo\");}")
            .unwrap()
            .get_type(),
        Type::Undefined
    );
    assert_eq!(
        forward_val(&mut context, "null").unwrap().get_type(),
        Type::Null
    );
    assert_eq!(
        forward_val(&mut context, "var x = {arg: \"hi\", foo: \"hello\"}; x")
            .unwrap()
            .get_type(),
        Type::Object
    );
    assert_eq!(
        forward_val(&mut context, "\"Hi\"").unwrap().get_type(),
        Type::String
    );
    assert_eq!(
        forward_val(&mut context, "Symbol()").unwrap().get_type(),
        Type::Symbol
    );
}

#[test]
fn to_string() {
    let f64_to_str = |f| Value::Rational(f).display().to_string();

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
fn string_length_is_not_enumerable() {
    let mut context = Context::new();

    let object = Value::from("foo").to_object(&mut context).unwrap();
    let length_desc = object
        .get_own_property(&PropertyKey::from("length"))
        .unwrap();
    assert!(!length_desc.enumerable());
}

#[test]
fn string_length_is_in_utf16_codeunits() {
    let mut context = Context::new();

    // 😀 is one Unicode code point, but 2 UTF-16 code units
    let object = Value::from("😀").to_object(&mut context).unwrap();
    let length_desc = object
        .get_own_property(&PropertyKey::from("length"))
        .unwrap();
    assert_eq!(
        length_desc
            .as_data_descriptor()
            .unwrap()
            .value()
            .to_integer_or_infinity(&mut context)
            .unwrap(),
        IntegerOrInfinity::Integer(2)
    );
}

#[test]
fn add_number_and_number() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "1 + 2").unwrap();
    let value = value.to_i32(&mut context).unwrap();
    assert_eq!(value, 3);
}

#[test]
fn add_number_and_string() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "1 + \" + 2 = 3\"").unwrap();
    let value = value.to_string(&mut context).unwrap();
    assert_eq!(value, "1 + 2 = 3");
}

#[test]
fn add_string_and_string() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "\"Hello\" + \", world\"").unwrap();
    let value = value.to_string(&mut context).unwrap();
    assert_eq!(value, "Hello, world");
}

#[test]
fn add_number_object_and_number() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "new Number(10) + 6").unwrap();
    let value = value.to_i32(&mut context).unwrap();
    assert_eq!(value, 16);
}

#[test]
fn add_number_object_and_string_object() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "new Number(10) + new String(\"0\")").unwrap();
    let value = value.to_string(&mut context).unwrap();
    assert_eq!(value, "100");
}

#[test]
fn sub_number_and_number() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "1 - 999").unwrap();
    let value = value.to_i32(&mut context).unwrap();
    assert_eq!(value, -998);
}

#[test]
fn sub_number_object_and_number_object() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "new Number(1) - new Number(999)").unwrap();
    let value = value.to_i32(&mut context).unwrap();
    assert_eq!(value, -998);
}

#[test]
fn sub_string_and_number_object() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "'Hello' - new Number(999)").unwrap();
    let value = value.to_number(&mut context).unwrap();
    assert!(value.is_nan());
}

#[test]
fn div_by_zero() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "1 / 0").unwrap();
    let value = value.to_number(&mut context).unwrap();
    assert!(value.is_infinite());
}

#[test]
fn rem_by_zero() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "1 % 0").unwrap();
    let value = value.to_number(&mut context).unwrap();
    assert!(value.is_nan());
}

#[test]
fn bitand_integer_and_integer() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "0xFFFF & 0xFF").unwrap();
    let value = value.to_i32(&mut context).unwrap();
    assert_eq!(value, 255);
}

#[test]
fn bitand_integer_and_rational() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "0xFFFF & 255.5").unwrap();
    let value = value.to_i32(&mut context).unwrap();
    assert_eq!(value, 255);
}

#[test]
fn bitand_rational_and_rational() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "255.772 & 255.5").unwrap();
    let value = value.to_i32(&mut context).unwrap();
    assert_eq!(value, 255);
}

#[test]
#[allow(clippy::float_cmp)]
fn pow_number_and_number() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "3 ** 3").unwrap();
    let value = value.to_number(&mut context).unwrap();
    assert_eq!(value, 27.0);
}

#[test]
fn pow_number_and_string() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "3 ** 'Hello'").unwrap();
    let value = value.to_number(&mut context).unwrap();
    assert!(value.is_nan());
}

#[test]
fn assign_pow_number_and_string() {
    let mut context = Context::new();

    let value = forward_val(
        &mut context,
        r"
        let a = 3;
        a **= 'Hello'
        a
    ",
    )
    .unwrap();
    let value = value.to_number(&mut context).unwrap();
    assert!(value.is_nan());
}

#[test]
fn display_string() {
    let s = String::from("Hello");
    let v = Value::from(s);
    assert_eq!(v.display().to_string(), "\"Hello\"");
}

#[test]
fn display_array_string() {
    let mut context = Context::new();

    let value = forward_val(&mut context, "[\"Hello\"]").unwrap();
    assert_eq!(value.display().to_string(), "[ \"Hello\" ]");
}

#[test]
fn display_boolean_object() {
    let mut context = Context::new();
    let d_obj = r#"
        let bool = new Boolean(0);
        bool
    "#;
    let value = forward_val(&mut context, d_obj).unwrap();
    assert_eq!(value.display().to_string(), "Boolean { false }")
}

#[test]
fn display_number_object() {
    let mut context = Context::new();
    let d_obj = r#"
        let num = new Number(3.14);
        num
    "#;
    let value = forward_val(&mut context, d_obj).unwrap();
    assert_eq!(value.display().to_string(), "Number { 3.14 }")
}

#[test]
fn display_negative_zero_object() {
    let mut context = Context::new();
    let d_obj = r#"
        let num = new Number(-0);
        num
    "#;
    let value = forward_val(&mut context, d_obj).unwrap();
    assert_eq!(value.display().to_string(), "Number { -0 }")
}

#[test]
fn debug_object() {
    let mut context = Context::new();
    let value = forward_val(&mut context, "new Array([new Date()])").unwrap();

    // We don't care about the contents of the debug display (it is *debug* after all). In the commit that this test was
    // added, this would cause a stack overflow, so executing Debug::fmt is the assertion.
    //
    // However, we want to make sure that no data is being left in the internal hashset, so executing this twice should
    // result in the same output.
    assert_eq!(format!("{:?}", value), format!("{:?}", value));
}

#[test]
#[ignore] // TODO: Once objects are printed in a simpler way this test can be simplified and used
fn display_object() {
    let mut context = Context::new();
    let d_obj = r#"
        let o = {a: 'a'};
        o
    "#;
    let value = forward_val(&mut context, d_obj).unwrap();
    assert_eq!(
        value.display().to_string(),
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

#[test]
fn to_integer_or_infinity() {
    let mut context = Context::new();

    assert_eq!(
        Value::undefined().to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::Integer(0))
    );
    assert_eq!(
        Value::from(NAN).to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::Integer(0))
    );
    assert_eq!(
        Value::from(0.0).to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::Integer(0))
    );
    assert_eq!(
        Value::from(-0.0).to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::Integer(0))
    );

    assert_eq!(
        Value::from(f64::INFINITY).to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::PositiveInfinity)
    );
    assert_eq!(
        Value::from(f64::NEG_INFINITY).to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::NegativeInfinity)
    );

    assert_eq!(
        Value::from(10).to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::Integer(10))
    );
    assert_eq!(
        Value::from(11.0).to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::Integer(11))
    );
    assert_eq!(
        Value::from("12").to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::Integer(12))
    );
    assert_eq!(
        Value::from(true).to_integer_or_infinity(&mut context),
        Ok(IntegerOrInfinity::Integer(1))
    );
}

#[test]
fn test_accessors() {
    let mut context = Context::new();
    let src = r#"
            let arr = [];
            let a = { get b() { return "c" }, set b(value) { arr = arr.concat([value]) }} ;
            a.b = "a";
        "#;
    context.eval(src).unwrap();
    assert_eq!(forward(&mut context, "a.b"), r#""c""#);
    assert_eq!(forward(&mut context, "arr"), r#"[ "a" ]"#);
}

/// Test cyclic conversions that previously caused stack overflows
/// Relevant mitigations for these are in `GcObject::ordinary_to_primitive` and
/// `GcObject::to_json`
mod cyclic_conversions {
    use super::*;

    #[test]
    fn to_json_cyclic() {
        let mut context = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            JSON.stringify(a)
        "#;

        assert_eq!(
            forward(&mut context, src),
            r#"Uncaught "TypeError": "cyclic object value""#,
        );
    }

    #[test]
    fn to_json_noncyclic() {
        let mut context = Context::new();
        let src = r#"
            let b = [];
            let a = [b, b];
            JSON.stringify(a)
        "#;

        let value = forward_val(&mut context, src).unwrap();
        let result = value.as_string().unwrap();
        assert_eq!(result, "[[],[]]",);
    }

    // These tests don't throw errors. Instead we mirror Chrome / Firefox behavior for these conversions
    #[test]
    fn to_string_cyclic() {
        let mut context = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            a.toString()
        "#;

        let value = forward_val(&mut context, src).unwrap();
        let result = value.as_string().unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn to_number_cyclic() {
        let mut context = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            +a
        "#;

        let value = forward_val(&mut context, src).unwrap();
        let result = value.as_number().unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn to_boolean_cyclic() {
        // this already worked before the mitigation, but we don't want to cause a regression
        let mut context = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            !!a
        "#;

        let value = forward_val(&mut context, src).unwrap();
        // There isn't an as_boolean function for some reason?
        assert_eq!(value, Value::Boolean(true));
    }

    #[test]
    fn to_bigint_cyclic() {
        let mut context = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            BigInt(a)
        "#;

        let value = forward_val(&mut context, src).unwrap();
        let result = value.as_bigint().unwrap().to_f64();
        assert_eq!(result, 0.);
    }

    #[test]
    fn to_u32_cyclic() {
        let mut context = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            a | 0
        "#;

        let value = forward_val(&mut context, src).unwrap();
        let result = value.as_number().unwrap();
        assert_eq!(result, 0.);
    }

    #[test]
    fn console_log_cyclic() {
        let mut context = Context::new();
        let src = r#"
            let a = [1];
            a[1] = a;
            console.log(a);
        "#;

        let _ = forward(&mut context, src);
        // Should not stack overflow
    }
}

mod abstract_relational_comparison {
    use super::*;
    macro_rules! check_comparison {
        ($context:ident, $string:expr => $expect:expr) => {
            assert_eq!(
                forward_val(&mut $context, $string).unwrap().to_boolean(),
                $expect
            );
        };
    }

    #[test]
    fn number_less_than_number() {
        let mut context = Context::new();
        check_comparison!(context, "1 < 2" => true);
        check_comparison!(context, "2 < 2" => false);
        check_comparison!(context, "3 < 2" => false);
        check_comparison!(context, "2 < 2.5" => true);
        check_comparison!(context, "2.5 < 2" => false);
    }

    #[test]
    fn string_less_than_number() {
        let mut context = Context::new();
        check_comparison!(context, "'1' < 2" => true);
        check_comparison!(context, "'2' < 2" => false);
        check_comparison!(context, "'3' < 2" => false);
        check_comparison!(context, "'2' < 2.5" => true);
        check_comparison!(context, "'2.5' < 2" => false);
    }

    #[test]
    fn number_less_than_string() {
        let mut context = Context::new();
        check_comparison!(context, "1 < '2'" => true);
        check_comparison!(context, "2 < '2'" => false);
        check_comparison!(context, "3 < '2'" => false);
        check_comparison!(context, "2 < '2.5'" => true);
        check_comparison!(context, "2.5 < '2'" => false);
    }

    #[test]
    fn number_object_less_than_number() {
        let mut context = Context::new();
        check_comparison!(context, "new Number(1) < '2'" => true);
        check_comparison!(context, "new Number(2) < '2'" => false);
        check_comparison!(context, "new Number(3) < '2'" => false);
        check_comparison!(context, "new Number(2) < '2.5'" => true);
        check_comparison!(context, "new Number(2.5) < '2'" => false);
    }

    #[test]
    fn number_object_less_than_number_object() {
        let mut context = Context::new();
        check_comparison!(context, "new Number(1) < new Number(2)" => true);
        check_comparison!(context, "new Number(2) < new Number(2)" => false);
        check_comparison!(context, "new Number(3) < new Number(2)" => false);
        check_comparison!(context, "new Number(2) < new Number(2.5)" => true);
        check_comparison!(context, "new Number(2.5) < new Number(2)" => false);
    }

    #[test]
    fn string_less_than_string() {
        let mut context = Context::new();
        check_comparison!(context, "'hello' < 'hello'" => false);
        check_comparison!(context, "'hell' < 'hello'" => true);
        check_comparison!(context, "'hello, world' < 'world'" => true);
        check_comparison!(context, "'aa' < 'ab'" => true);
    }

    #[test]
    fn string_object_less_than_string() {
        let mut context = Context::new();
        check_comparison!(context, "new String('hello') < 'hello'" => false);
        check_comparison!(context, "new String('hell') < 'hello'" => true);
        check_comparison!(context, "new String('hello, world') < 'world'" => true);
        check_comparison!(context, "new String('aa') < 'ab'" => true);
    }

    #[test]
    fn string_object_less_than_string_object() {
        let mut context = Context::new();
        check_comparison!(context, "new String('hello') < new String('hello')" => false);
        check_comparison!(context, "new String('hell') < new String('hello')" => true);
        check_comparison!(context, "new String('hello, world') < new String('world')" => true);
        check_comparison!(context, "new String('aa') < new String('ab')" => true);
    }

    #[test]
    fn bigint_less_than_number() {
        let mut context = Context::new();
        check_comparison!(context, "1n < 10" => true);
        check_comparison!(context, "10n < 10" => false);
        check_comparison!(context, "100n < 10" => false);
        check_comparison!(context, "10n < 10.9" => true);
    }

    #[test]
    fn number_less_than_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "10 < 1n" => false);
        check_comparison!(context, "1 < 1n" => false);
        check_comparison!(context, "-1 < -1n" => false);
        check_comparison!(context, "-1.9 < -1n" => true);
    }

    #[test]
    fn negative_infnity_less_than_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "-Infinity < -10000000000n" => true);
        check_comparison!(context, "-Infinity < (-1n << 100n)" => true);
    }

    #[test]
    fn bigint_less_than_infinity() {
        let mut context = Context::new();
        check_comparison!(context, "1000n < NaN" => false);
        check_comparison!(context, "(1n << 100n) < NaN" => false);
    }

    #[test]
    fn nan_less_than_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "NaN < -10000000000n" => false);
        check_comparison!(context, "NaN < (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_less_than_nan() {
        let mut context = Context::new();
        check_comparison!(context, "1000n < Infinity" => true);
        check_comparison!(context, "(1n << 100n) < Infinity" => true);
    }

    #[test]
    fn bigint_less_than_string() {
        let mut context = Context::new();
        check_comparison!(context, "1000n < '1000'" => false);
        check_comparison!(context, "1000n < '2000'" => true);
        check_comparison!(context, "1n < '-1'" => false);
        check_comparison!(context, "2n < '-1'" => false);
        check_comparison!(context, "-100n < 'InvalidBigInt'" => false);
    }

    #[test]
    fn string_less_than_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "'1000' < 1000n" => false);
        check_comparison!(context, "'2000' < 1000n" => false);
        check_comparison!(context, "'500' < 1000n" => true);
        check_comparison!(context, "'-1' < 1n" => true);
        check_comparison!(context, "'-1' < 2n" => true);
        check_comparison!(context, "'InvalidBigInt' < -100n" => false);
    }

    // -------------------------------------------

    #[test]
    fn number_less_than_or_equal_number() {
        let mut context = Context::new();
        check_comparison!(context, "1 <= 2" => true);
        check_comparison!(context, "2 <= 2" => true);
        check_comparison!(context, "3 <= 2" => false);
        check_comparison!(context, "2 <= 2.5" => true);
        check_comparison!(context, "2.5 <= 2" => false);
    }

    #[test]
    fn string_less_than_or_equal_number() {
        let mut context = Context::new();
        check_comparison!(context, "'1' <= 2" => true);
        check_comparison!(context, "'2' <= 2" => true);
        check_comparison!(context, "'3' <= 2" => false);
        check_comparison!(context, "'2' <= 2.5" => true);
        check_comparison!(context, "'2.5' < 2" => false);
    }

    #[test]
    fn number_less_than_or_equal_string() {
        let mut context = Context::new();
        check_comparison!(context, "1 <= '2'" => true);
        check_comparison!(context, "2 <= '2'" => true);
        check_comparison!(context, "3 <= '2'" => false);
        check_comparison!(context, "2 <= '2.5'" => true);
        check_comparison!(context, "2.5 <= '2'" => false);
    }

    #[test]
    fn number_object_less_than_or_equal_number() {
        let mut context = Context::new();
        check_comparison!(context, "new Number(1) <= '2'" => true);
        check_comparison!(context, "new Number(2) <= '2'" => true);
        check_comparison!(context, "new Number(3) <= '2'" => false);
        check_comparison!(context, "new Number(2) <= '2.5'" => true);
        check_comparison!(context, "new Number(2.5) <= '2'" => false);
    }

    #[test]
    fn number_object_less_than_number_or_equal_object() {
        let mut context = Context::new();
        check_comparison!(context, "new Number(1) <= new Number(2)" => true);
        check_comparison!(context, "new Number(2) <= new Number(2)" => true);
        check_comparison!(context, "new Number(3) <= new Number(2)" => false);
        check_comparison!(context, "new Number(2) <= new Number(2.5)" => true);
        check_comparison!(context, "new Number(2.5) <= new Number(2)" => false);
    }

    #[test]
    fn string_less_than_or_equal_string() {
        let mut context = Context::new();
        check_comparison!(context, "'hello' <= 'hello'" => true);
        check_comparison!(context, "'hell' <= 'hello'" => true);
        check_comparison!(context, "'hello, world' <= 'world'" => true);
        check_comparison!(context, "'aa' <= 'ab'" => true);
    }

    #[test]
    fn string_object_less_than_or_equal_string() {
        let mut context = Context::new();
        check_comparison!(context, "new String('hello') <= 'hello'" => true);
        check_comparison!(context, "new String('hell') <= 'hello'" => true);
        check_comparison!(context, "new String('hello, world') <= 'world'" => true);
        check_comparison!(context, "new String('aa') <= 'ab'" => true);
    }

    #[test]
    fn string_object_less_than_string_or_equal_object() {
        let mut context = Context::new();
        check_comparison!(context, "new String('hello') <= new String('hello')" => true);
        check_comparison!(context, "new String('hell') <= new String('hello')" => true);
        check_comparison!(context, "new String('hello, world') <= new String('world')" => true);
        check_comparison!(context, "new String('aa') <= new String('ab')" => true);
    }

    #[test]
    fn bigint_less_than_or_equal_number() {
        let mut context = Context::new();
        check_comparison!(context, "1n <= 10" => true);
        check_comparison!(context, "10n <= 10" => true);
        check_comparison!(context, "100n <= 10" => false);
        check_comparison!(context, "10n <= 10.9" => true);
    }

    #[test]
    fn number_less_than_or_equal_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "10 <= 1n" => false);
        check_comparison!(context, "1 <= 1n" => true);
        check_comparison!(context, "-1 <= -1n" => true);
        check_comparison!(context, "-1.9 <= -1n" => true);
    }

    #[test]
    fn negative_infnity_less_than_or_equal_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "-Infinity <= -10000000000n" => true);
        check_comparison!(context, "-Infinity <= (-1n << 100n)" => true);
    }

    #[test]
    fn bigint_less_than_or_equal_infinity() {
        let mut context = Context::new();
        check_comparison!(context, "1000n <= NaN" => false);
        check_comparison!(context, "(1n << 100n) <= NaN" => false);
    }

    #[test]
    fn nan_less_than_or_equal_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "NaN <= -10000000000n" => false);
        check_comparison!(context, "NaN <= (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_less_than_or_equal_nan() {
        let mut context = Context::new();
        check_comparison!(context, "1000n <= Infinity" => true);
        check_comparison!(context, "(1n << 100n) <= Infinity" => true);
    }

    #[test]
    fn bigint_less_than_or_equal_string() {
        let mut context = Context::new();
        check_comparison!(context, "1000n <= '1000'" => true);
        check_comparison!(context, "1000n <= '2000'" => true);
        check_comparison!(context, "1n <= '-1'" => false);
        check_comparison!(context, "2n <= '-1'" => false);
        check_comparison!(context, "-100n <= 'InvalidBigInt'" => false);
    }

    #[test]
    fn string_less_than_or_equal_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "'1000' <= 1000n" => true);
        check_comparison!(context, "'2000' <= 1000n" => false);
        check_comparison!(context, "'500' <= 1000n" => true);
        check_comparison!(context, "'-1' <= 1n" => true);
        check_comparison!(context, "'-1' <= 2n" => true);
        check_comparison!(context, "'InvalidBigInt' <= -100n" => false);
    }

    // -------------------------------------------

    #[test]
    fn number_greater_than_number() {
        let mut context = Context::new();
        check_comparison!(context, "1 > 2" => false);
        check_comparison!(context, "2 > 2" => false);
        check_comparison!(context, "3 > 2" => true);
        check_comparison!(context, "2 > 2.5" => false);
        check_comparison!(context, "2.5 > 2" => true);
    }

    #[test]
    fn string_greater_than_number() {
        let mut context = Context::new();
        check_comparison!(context, "'1' > 2" => false);
        check_comparison!(context, "'2' > 2" => false);
        check_comparison!(context, "'3' > 2" => true);
        check_comparison!(context, "'2' > 2.5" => false);
        check_comparison!(context, "'2.5' > 2" => true);
    }

    #[test]
    fn number_less_greater_string() {
        let mut context = Context::new();
        check_comparison!(context, "1 > '2'" => false);
        check_comparison!(context, "2 > '2'" => false);
        check_comparison!(context, "3 > '2'" => true);
        check_comparison!(context, "2 > '2.5'" => false);
        check_comparison!(context, "2.5 > '2'" => true);
    }

    #[test]
    fn number_object_greater_than_number() {
        let mut context = Context::new();
        check_comparison!(context, "new Number(1) > '2'" => false);
        check_comparison!(context, "new Number(2) > '2'" => false);
        check_comparison!(context, "new Number(3) > '2'" => true);
        check_comparison!(context, "new Number(2) > '2.5'" => false);
        check_comparison!(context, "new Number(2.5) > '2'" => true);
    }

    #[test]
    fn number_object_greater_than_number_object() {
        let mut context = Context::new();
        check_comparison!(context, "new Number(1) > new Number(2)" => false);
        check_comparison!(context, "new Number(2) > new Number(2)" => false);
        check_comparison!(context, "new Number(3) > new Number(2)" => true);
        check_comparison!(context, "new Number(2) > new Number(2.5)" => false);
        check_comparison!(context, "new Number(2.5) > new Number(2)" => true);
    }

    #[test]
    fn string_greater_than_string() {
        let mut context = Context::new();
        check_comparison!(context, "'hello' > 'hello'" => false);
        check_comparison!(context, "'hell' > 'hello'" => false);
        check_comparison!(context, "'hello, world' > 'world'" => false);
        check_comparison!(context, "'aa' > 'ab'" => false);
        check_comparison!(context, "'ab' > 'aa'" => true);
    }

    #[test]
    fn string_object_greater_than_string() {
        let mut context = Context::new();
        check_comparison!(context, "new String('hello') > 'hello'" => false);
        check_comparison!(context, "new String('hell') > 'hello'" => false);
        check_comparison!(context, "new String('hello, world') > 'world'" => false);
        check_comparison!(context, "new String('aa') > 'ab'" => false);
        check_comparison!(context, "new String('ab') > 'aa'" => true);
    }

    #[test]
    fn string_object_greater_than_string_object() {
        let mut context = Context::new();
        check_comparison!(context, "new String('hello') > new String('hello')" => false);
        check_comparison!(context, "new String('hell') > new String('hello')" => false);
        check_comparison!(context, "new String('hello, world') > new String('world')" => false);
        check_comparison!(context, "new String('aa') > new String('ab')" => false);
        check_comparison!(context, "new String('ab') > new String('aa')" => true);
    }

    #[test]
    fn bigint_greater_than_number() {
        let mut context = Context::new();
        check_comparison!(context, "1n > 10" => false);
        check_comparison!(context, "10n > 10" => false);
        check_comparison!(context, "100n > 10" => true);
        check_comparison!(context, "10n > 10.9" => false);
    }

    #[test]
    fn number_greater_than_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "10 > 1n" => true);
        check_comparison!(context, "1 > 1n" => false);
        check_comparison!(context, "-1 > -1n" => false);
        check_comparison!(context, "-1.9 > -1n" => false);
    }

    #[test]
    fn negative_infnity_greater_than_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "-Infinity > -10000000000n" => false);
        check_comparison!(context, "-Infinity > (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_greater_than_infinity() {
        let mut context = Context::new();
        check_comparison!(context, "1000n > NaN" => false);
        check_comparison!(context, "(1n << 100n) > NaN" => false);
    }

    #[test]
    fn nan_greater_than_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "NaN > -10000000000n" => false);
        check_comparison!(context, "NaN > (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_greater_than_nan() {
        let mut context = Context::new();
        check_comparison!(context, "1000n > Infinity" => false);
        check_comparison!(context, "(1n << 100n) > Infinity" => false);
    }

    #[test]
    fn bigint_greater_than_string() {
        let mut context = Context::new();
        check_comparison!(context, "1000n > '1000'" => false);
        check_comparison!(context, "1000n > '2000'" => false);
        check_comparison!(context, "1n > '-1'" => true);
        check_comparison!(context, "2n > '-1'" => true);
        check_comparison!(context, "-100n > 'InvalidBigInt'" => false);
    }

    #[test]
    fn string_greater_than_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "'1000' > 1000n" => false);
        check_comparison!(context, "'2000' > 1000n" => true);
        check_comparison!(context, "'500' > 1000n" => false);
        check_comparison!(context, "'-1' > 1n" => false);
        check_comparison!(context, "'-1' > 2n" => false);
        check_comparison!(context, "'InvalidBigInt' > -100n" => false);
    }

    // ----------------------------------------------

    #[test]
    fn number_greater_than_or_equal_number() {
        let mut context = Context::new();
        check_comparison!(context, "1 >= 2" => false);
        check_comparison!(context, "2 >= 2" => true);
        check_comparison!(context, "3 >= 2" => true);
        check_comparison!(context, "2 >= 2.5" => false);
        check_comparison!(context, "2.5 >= 2" => true);
    }

    #[test]
    fn string_greater_than_or_equal_number() {
        let mut context = Context::new();
        check_comparison!(context, "'1' >= 2" => false);
        check_comparison!(context, "'2' >= 2" => true);
        check_comparison!(context, "'3' >= 2" => true);
        check_comparison!(context, "'2' >= 2.5" => false);
        check_comparison!(context, "'2.5' >= 2" => true);
    }

    #[test]
    fn number_less_greater_or_equal_string() {
        let mut context = Context::new();
        check_comparison!(context, "1 >= '2'" => false);
        check_comparison!(context, "2 >= '2'" => true);
        check_comparison!(context, "3 >= '2'" => true);
        check_comparison!(context, "2 >= '2.5'" => false);
        check_comparison!(context, "2.5 >= '2'" => true);
    }

    #[test]
    fn number_object_greater_than_or_equal_number() {
        let mut context = Context::new();
        check_comparison!(context, "new Number(1) >= '2'" => false);
        check_comparison!(context, "new Number(2) >= '2'" => true);
        check_comparison!(context, "new Number(3) >= '2'" => true);
        check_comparison!(context, "new Number(2) >= '2.5'" => false);
        check_comparison!(context, "new Number(2.5) >= '2'" => true);
    }

    #[test]
    fn number_object_greater_than_or_equal_number_object() {
        let mut context = Context::new();
        check_comparison!(context, "new Number(1) >= new Number(2)" => false);
        check_comparison!(context, "new Number(2) >= new Number(2)" => true);
        check_comparison!(context, "new Number(3) >= new Number(2)" => true);
        check_comparison!(context, "new Number(2) >= new Number(2.5)" => false);
        check_comparison!(context, "new Number(2.5) >= new Number(2)" => true);
    }

    #[test]
    fn string_greater_than_or_equal_string() {
        let mut context = Context::new();
        check_comparison!(context, "'hello' >= 'hello'" => true);
        check_comparison!(context, "'hell' >= 'hello'" => false);
        check_comparison!(context, "'hello, world' >= 'world'" => false);
        check_comparison!(context, "'aa' >= 'ab'" => false);
        check_comparison!(context, "'ab' >= 'aa'" => true);
    }

    #[test]
    fn string_object_greater_or_equal_than_string() {
        let mut context = Context::new();
        check_comparison!(context, "new String('hello') >= 'hello'" => true);
        check_comparison!(context, "new String('hell') >= 'hello'" => false);
        check_comparison!(context, "new String('hello, world') >= 'world'" => false);
        check_comparison!(context, "new String('aa') >= 'ab'" => false);
        check_comparison!(context, "new String('ab') >= 'aa'" => true);
    }

    #[test]
    fn string_object_greater_than_or_equal_string_object() {
        let mut context = Context::new();
        check_comparison!(context, "new String('hello') >= new String('hello')" => true);
        check_comparison!(context, "new String('hell') >= new String('hello')" => false);
        check_comparison!(context, "new String('hello, world') >= new String('world')" => false);
        check_comparison!(context, "new String('aa') >= new String('ab')" => false);
        check_comparison!(context, "new String('ab') >= new String('aa')" => true);
    }

    #[test]
    fn bigint_greater_than_or_equal_number() {
        let mut context = Context::new();
        check_comparison!(context, "1n >= 10" => false);
        check_comparison!(context, "10n >= 10" => true);
        check_comparison!(context, "100n >= 10" => true);
        check_comparison!(context, "10n >= 10.9" => false);
    }

    #[test]
    fn number_greater_than_or_equal_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "10 >= 1n" => true);
        check_comparison!(context, "1 >= 1n" => true);
        check_comparison!(context, "-1 >= -1n" => true);
        check_comparison!(context, "-1.9 >= -1n" => false);
    }

    #[test]
    fn negative_infnity_greater_or_equal_than_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "-Infinity >= -10000000000n" => false);
        check_comparison!(context, "-Infinity >= (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_greater_than_or_equal_infinity() {
        let mut context = Context::new();
        check_comparison!(context, "1000n >= NaN" => false);
        check_comparison!(context, "(1n << 100n) >= NaN" => false);
    }

    #[test]
    fn nan_greater_than_or_equal_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "NaN >= -10000000000n" => false);
        check_comparison!(context, "NaN >= (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_greater_than_or_equal_nan() {
        let mut context = Context::new();
        check_comparison!(context, "1000n >= Infinity" => false);
        check_comparison!(context, "(1n << 100n) >= Infinity" => false);
    }

    #[test]
    fn bigint_greater_than_or_equal_string() {
        let mut context = Context::new();
        check_comparison!(context, "1000n >= '1000'" => true);
        check_comparison!(context, "1000n >= '2000'" => false);
        check_comparison!(context, "1n >= '-1'" => true);
        check_comparison!(context, "2n >= '-1'" => true);
        check_comparison!(context, "-100n >= 'InvalidBigInt'" => false);
    }

    #[test]
    fn string_greater_than_or_equal_bigint() {
        let mut context = Context::new();
        check_comparison!(context, "'1000' >= 1000n" => true);
        check_comparison!(context, "'2000' >= 1000n" => true);
        check_comparison!(context, "'500' >= 1000n" => false);
        check_comparison!(context, "'-1' >= 1n" => false);
        check_comparison!(context, "'-1' >= 2n" => false);
        check_comparison!(context, "'InvalidBigInt' >= -100n" => false);
    }
}
