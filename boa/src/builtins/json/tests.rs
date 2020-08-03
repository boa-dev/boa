use crate::{
    builtins::{object::PROTOTYPE, value::same_value},
    exec::Interpreter,
    forward, forward_val,
    realm::Realm,
};

#[test]
fn json_sanity() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    assert_eq!(
        forward(&mut engine, r#"JSON.parse('{"aaa":"bbb"}').aaa == 'bbb'"#),
        "true"
    );
    assert_eq!(
        forward(
            &mut engine,
            r#"JSON.stringify({aaa: 'bbb'}) == '{"aaa":"bbb"}'"#
        ),
        "true"
    );
}

#[test]
fn json_stringify_remove_undefined_values_from_objects() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward(
        &mut engine,
        r#"JSON.stringify({ aaa: undefined, bbb: 'ccc' })"#,
    );
    let expected = r#""{"bbb":"ccc"}""#;

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_remove_function_values_from_objects() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward(
        &mut engine,
        r#"JSON.stringify({ aaa: () => {}, bbb: 'ccc' })"#,
    );
    let expected = r#""{"bbb":"ccc"}""#;

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_remove_symbols_from_objects() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward(
        &mut engine,
        r#"JSON.stringify({ aaa: Symbol(), bbb: 'ccc' })"#,
    );
    let expected = r#""{"bbb":"ccc"}""#;

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_replacer_array_strings() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual = forward(
        &mut engine,
        r#"JSON.stringify({aaa: 'bbb', bbb: 'ccc', ccc: 'ddd'}, ['aaa', 'bbb'])"#,
    );
    let expected = forward(&mut engine, r#"'{"aaa":"bbb","bbb":"ccc"}'"#);
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_replacer_array_numbers() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual = forward(
        &mut engine,
        r#"JSON.stringify({ 0: 'aaa', 1: 'bbb', 2: 'ccc'}, [1, 2])"#,
    );
    let expected = forward(&mut engine, r#"'{"1":"bbb","2":"ccc"}'"#);
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_replacer_function() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual = forward(
        &mut engine,
        r#"JSON.stringify({ aaa: 1, bbb: 2}, (key, value) => {
            if (key === 'aaa') {
                return undefined;
            }

            return value;
        })"#,
    );
    let expected = forward(&mut engine, r#"'{"bbb":2}'"#);
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_arrays() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual = forward(&mut engine, r#"JSON.stringify(['a', 'b'])"#);
    let expected = forward(&mut engine, r#"'["a","b"]'"#);

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_object_array() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual = forward(&mut engine, r#"JSON.stringify([{a: 'b'}, {b: 'c'}])"#);
    let expected = forward(&mut engine, r#"'[{"a":"b"},{"b":"c"}]'"#);

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_array_converts_undefined_to_null() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual = forward(&mut engine, r#"JSON.stringify([undefined])"#);
    let expected = forward(&mut engine, r#"'[null]'"#);

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_array_converts_function_to_null() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual = forward(&mut engine, r#"JSON.stringify([() => {}])"#);
    let expected = forward(&mut engine, r#"'[null]'"#);

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_array_converts_symbol_to_null() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual = forward(&mut engine, r#"JSON.stringify([Symbol()])"#);
    let expected = forward(&mut engine, r#"'[null]'"#);

    assert_eq!(actual, expected);
}
#[test]
fn json_stringify_function_replacer_propogate_error() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward(
        &mut engine,
        r#"
        let thrown = 0;
        try {
            JSON.stringify({x: 1}, (key, value) => { throw 1 })
        } catch (err) {
            thrown = err;
        }
        thrown
        "#,
    );
    let expected = forward(&mut engine, "1");

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_function() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual_function = forward(&mut engine, r#"JSON.stringify(() => {})"#);
    let expected = forward(&mut engine, r#"undefined"#);

    assert_eq!(actual_function, expected);
}

#[test]
fn json_stringify_undefined() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual_undefined = forward(&mut engine, r#"JSON.stringify(undefined)"#);
    let expected = forward(&mut engine, r#"undefined"#);

    assert_eq!(actual_undefined, expected);
}

#[test]
fn json_stringify_symbol() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual_symbol = forward(&mut engine, r#"JSON.stringify(Symbol())"#);
    let expected = forward(&mut engine, r#"undefined"#);

    assert_eq!(actual_symbol, expected);
}

#[test]
fn json_stringify_no_args() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual_no_args = forward(&mut engine, r#"JSON.stringify()"#);
    let expected = forward(&mut engine, r#"undefined"#);

    assert_eq!(actual_no_args, expected);
}

#[test]
fn json_parse_array_with_reviver() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let result = forward_val(
        &mut engine,
        r#"JSON.parse('[1,2,3,4]', function(k, v){
            if (typeof v == 'number') {
                return v * 2;
            } else {
                v
        }})"#,
    )
    .unwrap();
    assert_eq!(result.get_field("0").to_number() as u8, 2u8);
    assert_eq!(result.get_field("1").to_number() as u8, 4u8);
    assert_eq!(result.get_field("2").to_number() as u8, 6u8);
    assert_eq!(result.get_field("3").to_number() as u8, 8u8);
}

#[test]
fn json_parse_object_with_reviver() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let result = forward(
        &mut engine,
        r#"
        var myObj = new Object();
        myObj.firstname = "boa";
        myObj.lastname = "snake";
        var jsonString = JSON.stringify(myObj);

        function dataReviver(key, value) {
            if (key == 'lastname') {
                return 'interpreter';
            } else {
                return value;
            }
        }

        var jsonObj = JSON.parse(jsonString, dataReviver);

        JSON.stringify(jsonObj);"#,
    );
    assert_eq!(result, r#""{"firstname":"boa","lastname":"interpreter"}""#);
}

#[test]
fn json_parse_sets_prototypes() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let init = r#"
        const jsonString = "{
            \"ob\":{\"ject\":1},
            \"arr\": [0,1]
        }";
        const jsonObj = JSON.parse(jsonString);
    "#;
    eprintln!("{}", forward(&mut engine, init));
    let object_prototype = forward_val(&mut engine, r#"jsonObj.ob"#)
        .unwrap()
        .as_object()
        .unwrap()
        .prototype()
        .clone();
    let array_prototype = forward_val(&mut engine, r#"jsonObj.arr"#)
        .unwrap()
        .as_object()
        .unwrap()
        .prototype()
        .clone();
    let global_object_prototype = engine
        .realm
        .global_obj
        .get_field("Object")
        .get_field(PROTOTYPE);
    let global_array_prototype = engine
        .realm
        .global_obj
        .get_field("Array")
        .get_field(PROTOTYPE);
    assert_eq!(
        same_value(&object_prototype, &global_object_prototype),
        true
    );
    assert_eq!(same_value(&array_prototype, &global_array_prototype), true);
}

#[test]
fn json_fields_should_be_enumerable() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);
    let actual_object = forward(
        &mut engine,
        r#"
        var a = JSON.parse('{"x":0}');
        a.propertyIsEnumerable('x');
    "#,
    );
    let actual_array_index = forward(
        &mut engine,
        r#"
        var b = JSON.parse('[0, 1]');
        b.propertyIsEnumerable('0');
        "#,
    );
    let expected = forward(&mut engine, r#"true"#);

    assert_eq!(actual_object, expected);
    assert_eq!(actual_array_index, expected);
}
