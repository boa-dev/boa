#![allow(clippy::float_cmp)]

use super::*;
use crate::{forward, forward_val, Context};

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
    assert_eq!(u.display().to_string(), "undefined");
}

#[test]
fn get_set_field() {
    let obj = Value::new_object(None);
    // Create string and convert it to a Value
    let s = Value::from("bar");
    obj.set_field("foo", s);
    assert_eq!(obj.get_field("foo").display().to_string(), "\"bar\"");
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
    let mut engine = Context::new();

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
    let mut engine = Context::new();

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
fn add_number_and_number() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "1 + 2").unwrap();
    let value = value.to_i32(&mut engine).unwrap();
    assert_eq!(value, 3);
}

#[test]
fn add_number_and_string() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "1 + \" + 2 = 3\"").unwrap();
    let value = value.to_string(&mut engine).unwrap();
    assert_eq!(value, "1 + 2 = 3");
}

#[test]
fn add_string_and_string() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "\"Hello\" + \", world\"").unwrap();
    let value = value.to_string(&mut engine).unwrap();
    assert_eq!(value, "Hello, world");
}

#[test]
fn add_number_object_and_number() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "new Number(10) + 6").unwrap();
    let value = value.to_i32(&mut engine).unwrap();
    assert_eq!(value, 16);
}

#[test]
fn add_number_object_and_string_object() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "new Number(10) + new String(\"0\")").unwrap();
    let value = value.to_string(&mut engine).unwrap();
    assert_eq!(value, "100");
}

#[test]
fn sub_number_and_number() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "1 - 999").unwrap();
    let value = value.to_i32(&mut engine).unwrap();
    assert_eq!(value, -998);
}

#[test]
fn sub_number_object_and_number_object() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "new Number(1) - new Number(999)").unwrap();
    let value = value.to_i32(&mut engine).unwrap();
    assert_eq!(value, -998);
}

#[test]
fn sub_string_and_number_object() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "'Hello' - new Number(999)").unwrap();
    let value = value.to_number(&mut engine).unwrap();
    assert!(value.is_nan());
}

#[test]
fn bitand_integer_and_integer() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "0xFFFF & 0xFF").unwrap();
    let value = value.to_i32(&mut engine).unwrap();
    assert_eq!(value, 255);
}

#[test]
fn bitand_integer_and_rational() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "0xFFFF & 255.5").unwrap();
    let value = value.to_i32(&mut engine).unwrap();
    assert_eq!(value, 255);
}

#[test]
fn bitand_rational_and_rational() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "255.772 & 255.5").unwrap();
    let value = value.to_i32(&mut engine).unwrap();
    assert_eq!(value, 255);
}

#[test]
#[allow(clippy::float_cmp)]
fn pow_number_and_number() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "3 ** 3").unwrap();
    let value = value.to_number(&mut engine).unwrap();
    assert_eq!(value, 27.0);
}

#[test]
fn pow_number_and_string() {
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "3 ** 'Hello'").unwrap();
    let value = value.to_number(&mut engine).unwrap();
    assert!(value.is_nan());
}

#[test]
fn assign_pow_number_and_string() {
    let mut engine = Context::new();

    let value = forward_val(
        &mut engine,
        r"
        let a = 3;
        a **= 'Hello'
        a
    ",
    )
    .unwrap();
    let value = value.to_number(&mut engine).unwrap();
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
    let mut engine = Context::new();

    let value = forward_val(&mut engine, "[\"Hello\"]").unwrap();
    assert_eq!(value.display().to_string(), "[ \"Hello\" ]");
}

#[test]
fn display_boolean_object() {
    let mut engine = Context::new();
    let d_obj = r#"
        let bool = new Boolean(0);
        bool
    "#;
    let value = forward_val(&mut engine, d_obj).unwrap();
    assert_eq!(value.display().to_string(), "Boolean { false }")
}

#[test]
fn display_number_object() {
    let mut engine = Context::new();
    let d_obj = r#"
        let num = new Number(3.14);
        num
    "#;
    let value = forward_val(&mut engine, d_obj).unwrap();
    assert_eq!(value.display().to_string(), "Number { 3.14 }")
}

#[test]
fn display_negative_zero_object() {
    let mut engine = Context::new();
    let d_obj = r#"
        let num = new Number(-0);
        num
    "#;
    let value = forward_val(&mut engine, d_obj).unwrap();
    assert_eq!(value.display().to_string(), "Number { -0 }")
}

#[test]
fn debug_object() {
    let mut engine = Context::new();
    let value = forward_val(&mut engine, "new Array([new Date()])").unwrap();

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
    let mut engine = Context::new();
    let d_obj = r#"
        let o = {a: 'a'};
        o
    "#;
    let value = forward_val(&mut engine, d_obj).unwrap();
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

/// Test cyclic conversions that previously caused stack overflows
/// Relevant mitigations for these are in `GcObject::ordinary_to_primitive` and
/// `GcObject::to_json`
mod cyclic_conversions {
    use super::*;

    #[test]
    fn to_json_cyclic() {
        let mut engine = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            JSON.stringify(a)
        "#;

        assert_eq!(
            forward(&mut engine, src),
            r#"Uncaught "TypeError": "cyclic object value""#,
        );
    }

    #[test]
    fn to_json_noncyclic() {
        let mut engine = Context::new();
        let src = r#"
            let b = [];
            let a = [b, b];
            JSON.stringify(a)
        "#;

        let value = forward_val(&mut engine, src).unwrap();
        let result = value.as_string().unwrap();
        assert_eq!(result, "[[],[]]",);
    }

    // These tests don't throw errors. Instead we mirror Chrome / Firefox behavior for these conversions
    #[test]
    fn to_string_cyclic() {
        let mut engine = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            a.toString()
        "#;

        let value = forward_val(&mut engine, src).unwrap();
        let result = value.as_string().unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn to_number_cyclic() {
        let mut engine = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            +a
        "#;

        let value = forward_val(&mut engine, src).unwrap();
        let result = value.as_number().unwrap();
        assert_eq!(result, 0.0);
    }

    #[test]
    fn to_boolean_cyclic() {
        // this already worked before the mitigation, but we don't want to cause a regression
        let mut engine = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            !!a
        "#;

        let value = forward_val(&mut engine, src).unwrap();
        // There isn't an as_boolean function for some reason?
        assert_eq!(value, Value::Boolean(true));
    }

    #[test]
    fn to_bigint_cyclic() {
        let mut engine = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            BigInt(a)
        "#;

        let value = forward_val(&mut engine, src).unwrap();
        let result = value.as_bigint().unwrap().to_f64();
        assert_eq!(result, 0.);
    }

    #[test]
    fn to_u32_cyclic() {
        let mut engine = Context::new();
        let src = r#"
            let a = [];
            a[0] = a;
            a | 0
        "#;

        let value = forward_val(&mut engine, src).unwrap();
        let result = value.as_number().unwrap();
        assert_eq!(result, 0.);
    }

    #[test]
    fn console_log_cyclic() {
        let mut engine = Context::new();
        let src = r#"
            let a = [1];
            a[1] = a;
            console.log(a);
        "#;

        let _ = forward(&mut engine, src);
        // Should not stack overflow
    }
}

mod abstract_relational_comparison {
    use super::*;
    macro_rules! check_comparison {
        ($engine:ident, $string:expr => $expect:expr) => {
            assert_eq!(
                forward_val(&mut $engine, $string).unwrap().to_boolean(),
                $expect
            );
        };
    }

    #[test]
    fn number_less_than_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "1 < 2" => true);
        check_comparison!(engine, "2 < 2" => false);
        check_comparison!(engine, "3 < 2" => false);
        check_comparison!(engine, "2 < 2.5" => true);
        check_comparison!(engine, "2.5 < 2" => false);
    }

    #[test]
    fn string_less_than_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "'1' < 2" => true);
        check_comparison!(engine, "'2' < 2" => false);
        check_comparison!(engine, "'3' < 2" => false);
        check_comparison!(engine, "'2' < 2.5" => true);
        check_comparison!(engine, "'2.5' < 2" => false);
    }

    #[test]
    fn number_less_than_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "1 < '2'" => true);
        check_comparison!(engine, "2 < '2'" => false);
        check_comparison!(engine, "3 < '2'" => false);
        check_comparison!(engine, "2 < '2.5'" => true);
        check_comparison!(engine, "2.5 < '2'" => false);
    }

    #[test]
    fn number_object_less_than_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "new Number(1) < '2'" => true);
        check_comparison!(engine, "new Number(2) < '2'" => false);
        check_comparison!(engine, "new Number(3) < '2'" => false);
        check_comparison!(engine, "new Number(2) < '2.5'" => true);
        check_comparison!(engine, "new Number(2.5) < '2'" => false);
    }

    #[test]
    fn number_object_less_than_number_object() {
        let mut engine = Context::new();
        check_comparison!(engine, "new Number(1) < new Number(2)" => true);
        check_comparison!(engine, "new Number(2) < new Number(2)" => false);
        check_comparison!(engine, "new Number(3) < new Number(2)" => false);
        check_comparison!(engine, "new Number(2) < new Number(2.5)" => true);
        check_comparison!(engine, "new Number(2.5) < new Number(2)" => false);
    }

    #[test]
    fn string_less_than_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "'hello' < 'hello'" => false);
        check_comparison!(engine, "'hell' < 'hello'" => true);
        check_comparison!(engine, "'hello, world' < 'world'" => true);
        check_comparison!(engine, "'aa' < 'ab'" => true);
    }

    #[test]
    fn string_object_less_than_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "new String('hello') < 'hello'" => false);
        check_comparison!(engine, "new String('hell') < 'hello'" => true);
        check_comparison!(engine, "new String('hello, world') < 'world'" => true);
        check_comparison!(engine, "new String('aa') < 'ab'" => true);
    }

    #[test]
    fn string_object_less_than_string_object() {
        let mut engine = Context::new();
        check_comparison!(engine, "new String('hello') < new String('hello')" => false);
        check_comparison!(engine, "new String('hell') < new String('hello')" => true);
        check_comparison!(engine, "new String('hello, world') < new String('world')" => true);
        check_comparison!(engine, "new String('aa') < new String('ab')" => true);
    }

    #[test]
    fn bigint_less_than_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "1n < 10" => true);
        check_comparison!(engine, "10n < 10" => false);
        check_comparison!(engine, "100n < 10" => false);
        check_comparison!(engine, "10n < 10.9" => true);
    }

    #[test]
    fn number_less_than_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "10 < 1n" => false);
        check_comparison!(engine, "1 < 1n" => false);
        check_comparison!(engine, "-1 < -1n" => false);
        check_comparison!(engine, "-1.9 < -1n" => true);
    }

    #[test]
    fn negative_infnity_less_than_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "-Infinity < -10000000000n" => true);
        check_comparison!(engine, "-Infinity < (-1n << 100n)" => true);
    }

    #[test]
    fn bigint_less_than_infinity() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n < NaN" => false);
        check_comparison!(engine, "(1n << 100n) < NaN" => false);
    }

    #[test]
    fn nan_less_than_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "NaN < -10000000000n" => false);
        check_comparison!(engine, "NaN < (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_less_than_nan() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n < Infinity" => true);
        check_comparison!(engine, "(1n << 100n) < Infinity" => true);
    }

    #[test]
    fn bigint_less_than_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n < '1000'" => false);
        check_comparison!(engine, "1000n < '2000'" => true);
        check_comparison!(engine, "1n < '-1'" => false);
        check_comparison!(engine, "2n < '-1'" => false);
        check_comparison!(engine, "-100n < 'InvalidBigInt'" => false);
    }

    #[test]
    fn string_less_than_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "'1000' < 1000n" => false);
        check_comparison!(engine, "'2000' < 1000n" => false);
        check_comparison!(engine, "'500' < 1000n" => true);
        check_comparison!(engine, "'-1' < 1n" => true);
        check_comparison!(engine, "'-1' < 2n" => true);
        check_comparison!(engine, "'InvalidBigInt' < -100n" => false);
    }

    // -------------------------------------------

    #[test]
    fn number_less_than_or_equal_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "1 <= 2" => true);
        check_comparison!(engine, "2 <= 2" => true);
        check_comparison!(engine, "3 <= 2" => false);
        check_comparison!(engine, "2 <= 2.5" => true);
        check_comparison!(engine, "2.5 <= 2" => false);
    }

    #[test]
    fn string_less_than_or_equal_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "'1' <= 2" => true);
        check_comparison!(engine, "'2' <= 2" => true);
        check_comparison!(engine, "'3' <= 2" => false);
        check_comparison!(engine, "'2' <= 2.5" => true);
        check_comparison!(engine, "'2.5' < 2" => false);
    }

    #[test]
    fn number_less_than_or_equal_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "1 <= '2'" => true);
        check_comparison!(engine, "2 <= '2'" => true);
        check_comparison!(engine, "3 <= '2'" => false);
        check_comparison!(engine, "2 <= '2.5'" => true);
        check_comparison!(engine, "2.5 <= '2'" => false);
    }

    #[test]
    fn number_object_less_than_or_equal_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "new Number(1) <= '2'" => true);
        check_comparison!(engine, "new Number(2) <= '2'" => true);
        check_comparison!(engine, "new Number(3) <= '2'" => false);
        check_comparison!(engine, "new Number(2) <= '2.5'" => true);
        check_comparison!(engine, "new Number(2.5) <= '2'" => false);
    }

    #[test]
    fn number_object_less_than_number_or_equal_object() {
        let mut engine = Context::new();
        check_comparison!(engine, "new Number(1) <= new Number(2)" => true);
        check_comparison!(engine, "new Number(2) <= new Number(2)" => true);
        check_comparison!(engine, "new Number(3) <= new Number(2)" => false);
        check_comparison!(engine, "new Number(2) <= new Number(2.5)" => true);
        check_comparison!(engine, "new Number(2.5) <= new Number(2)" => false);
    }

    #[test]
    fn string_less_than_or_equal_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "'hello' <= 'hello'" => true);
        check_comparison!(engine, "'hell' <= 'hello'" => true);
        check_comparison!(engine, "'hello, world' <= 'world'" => true);
        check_comparison!(engine, "'aa' <= 'ab'" => true);
    }

    #[test]
    fn string_object_less_than_or_equal_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "new String('hello') <= 'hello'" => true);
        check_comparison!(engine, "new String('hell') <= 'hello'" => true);
        check_comparison!(engine, "new String('hello, world') <= 'world'" => true);
        check_comparison!(engine, "new String('aa') <= 'ab'" => true);
    }

    #[test]
    fn string_object_less_than_string_or_equal_object() {
        let mut engine = Context::new();
        check_comparison!(engine, "new String('hello') <= new String('hello')" => true);
        check_comparison!(engine, "new String('hell') <= new String('hello')" => true);
        check_comparison!(engine, "new String('hello, world') <= new String('world')" => true);
        check_comparison!(engine, "new String('aa') <= new String('ab')" => true);
    }

    #[test]
    fn bigint_less_than_or_equal_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "1n <= 10" => true);
        check_comparison!(engine, "10n <= 10" => true);
        check_comparison!(engine, "100n <= 10" => false);
        check_comparison!(engine, "10n <= 10.9" => true);
    }

    #[test]
    fn number_less_than_or_equal_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "10 <= 1n" => false);
        check_comparison!(engine, "1 <= 1n" => true);
        check_comparison!(engine, "-1 <= -1n" => true);
        check_comparison!(engine, "-1.9 <= -1n" => true);
    }

    #[test]
    fn negative_infnity_less_than_or_equal_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "-Infinity <= -10000000000n" => true);
        check_comparison!(engine, "-Infinity <= (-1n << 100n)" => true);
    }

    #[test]
    fn bigint_less_than_or_equal_infinity() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n <= NaN" => false);
        check_comparison!(engine, "(1n << 100n) <= NaN" => false);
    }

    #[test]
    fn nan_less_than_or_equal_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "NaN <= -10000000000n" => false);
        check_comparison!(engine, "NaN <= (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_less_than_or_equal_nan() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n <= Infinity" => true);
        check_comparison!(engine, "(1n << 100n) <= Infinity" => true);
    }

    #[test]
    fn bigint_less_than_or_equal_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n <= '1000'" => true);
        check_comparison!(engine, "1000n <= '2000'" => true);
        check_comparison!(engine, "1n <= '-1'" => false);
        check_comparison!(engine, "2n <= '-1'" => false);
        check_comparison!(engine, "-100n <= 'InvalidBigInt'" => false);
    }

    #[test]
    fn string_less_than_or_equal_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "'1000' <= 1000n" => true);
        check_comparison!(engine, "'2000' <= 1000n" => false);
        check_comparison!(engine, "'500' <= 1000n" => true);
        check_comparison!(engine, "'-1' <= 1n" => true);
        check_comparison!(engine, "'-1' <= 2n" => true);
        check_comparison!(engine, "'InvalidBigInt' <= -100n" => false);
    }

    // -------------------------------------------

    #[test]
    fn number_greater_than_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "1 > 2" => false);
        check_comparison!(engine, "2 > 2" => false);
        check_comparison!(engine, "3 > 2" => true);
        check_comparison!(engine, "2 > 2.5" => false);
        check_comparison!(engine, "2.5 > 2" => true);
    }

    #[test]
    fn string_greater_than_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "'1' > 2" => false);
        check_comparison!(engine, "'2' > 2" => false);
        check_comparison!(engine, "'3' > 2" => true);
        check_comparison!(engine, "'2' > 2.5" => false);
        check_comparison!(engine, "'2.5' > 2" => true);
    }

    #[test]
    fn number_less_greater_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "1 > '2'" => false);
        check_comparison!(engine, "2 > '2'" => false);
        check_comparison!(engine, "3 > '2'" => true);
        check_comparison!(engine, "2 > '2.5'" => false);
        check_comparison!(engine, "2.5 > '2'" => true);
    }

    #[test]
    fn number_object_greater_than_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "new Number(1) > '2'" => false);
        check_comparison!(engine, "new Number(2) > '2'" => false);
        check_comparison!(engine, "new Number(3) > '2'" => true);
        check_comparison!(engine, "new Number(2) > '2.5'" => false);
        check_comparison!(engine, "new Number(2.5) > '2'" => true);
    }

    #[test]
    fn number_object_greater_than_number_object() {
        let mut engine = Context::new();
        check_comparison!(engine, "new Number(1) > new Number(2)" => false);
        check_comparison!(engine, "new Number(2) > new Number(2)" => false);
        check_comparison!(engine, "new Number(3) > new Number(2)" => true);
        check_comparison!(engine, "new Number(2) > new Number(2.5)" => false);
        check_comparison!(engine, "new Number(2.5) > new Number(2)" => true);
    }

    #[test]
    fn string_greater_than_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "'hello' > 'hello'" => false);
        check_comparison!(engine, "'hell' > 'hello'" => false);
        check_comparison!(engine, "'hello, world' > 'world'" => false);
        check_comparison!(engine, "'aa' > 'ab'" => false);
        check_comparison!(engine, "'ab' > 'aa'" => true);
    }

    #[test]
    fn string_object_greater_than_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "new String('hello') > 'hello'" => false);
        check_comparison!(engine, "new String('hell') > 'hello'" => false);
        check_comparison!(engine, "new String('hello, world') > 'world'" => false);
        check_comparison!(engine, "new String('aa') > 'ab'" => false);
        check_comparison!(engine, "new String('ab') > 'aa'" => true);
    }

    #[test]
    fn string_object_greater_than_string_object() {
        let mut engine = Context::new();
        check_comparison!(engine, "new String('hello') > new String('hello')" => false);
        check_comparison!(engine, "new String('hell') > new String('hello')" => false);
        check_comparison!(engine, "new String('hello, world') > new String('world')" => false);
        check_comparison!(engine, "new String('aa') > new String('ab')" => false);
        check_comparison!(engine, "new String('ab') > new String('aa')" => true);
    }

    #[test]
    fn bigint_greater_than_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "1n > 10" => false);
        check_comparison!(engine, "10n > 10" => false);
        check_comparison!(engine, "100n > 10" => true);
        check_comparison!(engine, "10n > 10.9" => false);
    }

    #[test]
    fn number_greater_than_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "10 > 1n" => true);
        check_comparison!(engine, "1 > 1n" => false);
        check_comparison!(engine, "-1 > -1n" => false);
        check_comparison!(engine, "-1.9 > -1n" => false);
    }

    #[test]
    fn negative_infnity_greater_than_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "-Infinity > -10000000000n" => false);
        check_comparison!(engine, "-Infinity > (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_greater_than_infinity() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n > NaN" => false);
        check_comparison!(engine, "(1n << 100n) > NaN" => false);
    }

    #[test]
    fn nan_greater_than_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "NaN > -10000000000n" => false);
        check_comparison!(engine, "NaN > (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_greater_than_nan() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n > Infinity" => false);
        check_comparison!(engine, "(1n << 100n) > Infinity" => false);
    }

    #[test]
    fn bigint_greater_than_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n > '1000'" => false);
        check_comparison!(engine, "1000n > '2000'" => false);
        check_comparison!(engine, "1n > '-1'" => true);
        check_comparison!(engine, "2n > '-1'" => true);
        check_comparison!(engine, "-100n > 'InvalidBigInt'" => false);
    }

    #[test]
    fn string_greater_than_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "'1000' > 1000n" => false);
        check_comparison!(engine, "'2000' > 1000n" => true);
        check_comparison!(engine, "'500' > 1000n" => false);
        check_comparison!(engine, "'-1' > 1n" => false);
        check_comparison!(engine, "'-1' > 2n" => false);
        check_comparison!(engine, "'InvalidBigInt' > -100n" => false);
    }

    // ----------------------------------------------

    #[test]
    fn number_greater_than_or_equal_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "1 >= 2" => false);
        check_comparison!(engine, "2 >= 2" => true);
        check_comparison!(engine, "3 >= 2" => true);
        check_comparison!(engine, "2 >= 2.5" => false);
        check_comparison!(engine, "2.5 >= 2" => true);
    }

    #[test]
    fn string_greater_than_or_equal_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "'1' >= 2" => false);
        check_comparison!(engine, "'2' >= 2" => true);
        check_comparison!(engine, "'3' >= 2" => true);
        check_comparison!(engine, "'2' >= 2.5" => false);
        check_comparison!(engine, "'2.5' >= 2" => true);
    }

    #[test]
    fn number_less_greater_or_equal_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "1 >= '2'" => false);
        check_comparison!(engine, "2 >= '2'" => true);
        check_comparison!(engine, "3 >= '2'" => true);
        check_comparison!(engine, "2 >= '2.5'" => false);
        check_comparison!(engine, "2.5 >= '2'" => true);
    }

    #[test]
    fn number_object_greater_than_or_equal_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "new Number(1) >= '2'" => false);
        check_comparison!(engine, "new Number(2) >= '2'" => true);
        check_comparison!(engine, "new Number(3) >= '2'" => true);
        check_comparison!(engine, "new Number(2) >= '2.5'" => false);
        check_comparison!(engine, "new Number(2.5) >= '2'" => true);
    }

    #[test]
    fn number_object_greater_than_or_equal_number_object() {
        let mut engine = Context::new();
        check_comparison!(engine, "new Number(1) >= new Number(2)" => false);
        check_comparison!(engine, "new Number(2) >= new Number(2)" => true);
        check_comparison!(engine, "new Number(3) >= new Number(2)" => true);
        check_comparison!(engine, "new Number(2) >= new Number(2.5)" => false);
        check_comparison!(engine, "new Number(2.5) >= new Number(2)" => true);
    }

    #[test]
    fn string_greater_than_or_equal_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "'hello' >= 'hello'" => true);
        check_comparison!(engine, "'hell' >= 'hello'" => false);
        check_comparison!(engine, "'hello, world' >= 'world'" => false);
        check_comparison!(engine, "'aa' >= 'ab'" => false);
        check_comparison!(engine, "'ab' >= 'aa'" => true);
    }

    #[test]
    fn string_object_greater_or_equal_than_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "new String('hello') >= 'hello'" => true);
        check_comparison!(engine, "new String('hell') >= 'hello'" => false);
        check_comparison!(engine, "new String('hello, world') >= 'world'" => false);
        check_comparison!(engine, "new String('aa') >= 'ab'" => false);
        check_comparison!(engine, "new String('ab') >= 'aa'" => true);
    }

    #[test]
    fn string_object_greater_than_or_equal_string_object() {
        let mut engine = Context::new();
        check_comparison!(engine, "new String('hello') >= new String('hello')" => true);
        check_comparison!(engine, "new String('hell') >= new String('hello')" => false);
        check_comparison!(engine, "new String('hello, world') >= new String('world')" => false);
        check_comparison!(engine, "new String('aa') >= new String('ab')" => false);
        check_comparison!(engine, "new String('ab') >= new String('aa')" => true);
    }

    #[test]
    fn bigint_greater_than_or_equal_number() {
        let mut engine = Context::new();
        check_comparison!(engine, "1n >= 10" => false);
        check_comparison!(engine, "10n >= 10" => true);
        check_comparison!(engine, "100n >= 10" => true);
        check_comparison!(engine, "10n >= 10.9" => false);
    }

    #[test]
    fn number_greater_than_or_equal_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "10 >= 1n" => true);
        check_comparison!(engine, "1 >= 1n" => true);
        check_comparison!(engine, "-1 >= -1n" => true);
        check_comparison!(engine, "-1.9 >= -1n" => false);
    }

    #[test]
    fn negative_infnity_greater_or_equal_than_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "-Infinity >= -10000000000n" => false);
        check_comparison!(engine, "-Infinity >= (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_greater_than_or_equal_infinity() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n >= NaN" => false);
        check_comparison!(engine, "(1n << 100n) >= NaN" => false);
    }

    #[test]
    fn nan_greater_than_or_equal_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "NaN >= -10000000000n" => false);
        check_comparison!(engine, "NaN >= (-1n << 100n)" => false);
    }

    #[test]
    fn bigint_greater_than_or_equal_nan() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n >= Infinity" => false);
        check_comparison!(engine, "(1n << 100n) >= Infinity" => false);
    }

    #[test]
    fn bigint_greater_than_or_equal_string() {
        let mut engine = Context::new();
        check_comparison!(engine, "1000n >= '1000'" => true);
        check_comparison!(engine, "1000n >= '2000'" => false);
        check_comparison!(engine, "1n >= '-1'" => true);
        check_comparison!(engine, "2n >= '-1'" => true);
        check_comparison!(engine, "-100n >= 'InvalidBigInt'" => false);
    }

    #[test]
    fn string_greater_than_or_equal_bigint() {
        let mut engine = Context::new();
        check_comparison!(engine, "'1000' >= 1000n" => true);
        check_comparison!(engine, "'2000' >= 1000n" => true);
        check_comparison!(engine, "'500' >= 1000n" => false);
        check_comparison!(engine, "'-1' >= 1n" => false);
        check_comparison!(engine, "'-1' >= 2n" => false);
        check_comparison!(engine, "'InvalidBigInt' >= -100n" => false);
    }
}
