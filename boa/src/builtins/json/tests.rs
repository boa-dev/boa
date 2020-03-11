use super::*;
use crate::{exec::Executor, forward, forward_val, realm::Realm};
use serde_json::{json, Value};

#[test]
fn check_json_global() {
    let global = ValueData::new_obj(None);
    let json_global = create_constructor(&global);

    assert!(json_global.is_object());

    let parse_function = json_global.get_prop("parse").unwrap();
    assert!(parse_function.value.as_ref().unwrap().is_function());
    let stringify_function = json_global.get_prop("stringify").unwrap();
    assert!(stringify_function.value.as_ref().unwrap().is_function());
}

#[test]
fn test_object_parse() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);

    let parsed = forward_val(
        &mut engine,
        r#"JSON.parse("{\"number\": 1, \"bool\": false, \"string\": \"text\"}")"#,
    )
    .unwrap();
    assert!(parsed.is_object());

    let parsed_number = parsed.get_prop("number").unwrap();
    let parsed_number_val = parsed_number.value.as_ref().unwrap();
    assert!(parsed_number_val.is_num());
    assert_eq!(parsed_number_val.to_int(), 1);

    let parsed_bool = parsed.get_prop("bool").unwrap();
    let parsed_bool_val = parsed_bool.value.as_ref().unwrap();
    assert!(parsed_bool_val.is_boolean());
    assert_eq!(parsed_bool_val.is_true(), false);

    let parsed_string = parsed.get_prop("string").unwrap();
    let parsed_string_val = parsed_string.value.as_ref().unwrap();
    assert!(parsed_string_val.is_string());
    assert_eq!(parsed_string_val.to_string(), "text");
}

#[test]
fn test_array_parse() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"JSON.parse("[1, true, \"text\"]")"#;
    let array = forward_val(&mut engine, init).unwrap();

    let first_prop = &array.get_prop("0").unwrap().value;
    let first_val = first_prop.as_ref().unwrap();
    assert!(first_val.is_num());
    assert_eq!(first_val.to_int(), 1);

    let second_prop = &array.get_prop("1").unwrap().value;
    let second_val = second_prop.as_ref().unwrap();
    assert!(second_val.is_boolean());
    assert!(second_val.is_true());

    let third_prop = &array.get_prop("2").unwrap().value;
    let third_val = third_prop.as_ref().unwrap();
    assert_eq!(third_val.to_string(), "text");
}

#[test]
fn test_stringify() {
    let realm = Realm::create();
    let mut engine = Executor::new(realm);
    let init = r#"
        var object = {
            array: [1, 2, 3],
            string: "text",
            number: 1,
            bool: true
        };
        "#;
    forward(&mut engine, init);

    let stringified = forward(&mut engine, "JSON.stringify(object)");
    let parsed_object: Value = serde_json::from_str(&stringified).unwrap();

    assert!(parsed_object.is_object());
    assert_eq!(parsed_object.get("string").unwrap(), &json!("text"));
    assert_eq!(parsed_object.get("number").unwrap(), &json!(1));
    assert_eq!(parsed_object.get("bool").unwrap(), &json!(true));
    assert_eq!(parsed_object.get("array").unwrap(), &json!([1, 2, 3]));
}
