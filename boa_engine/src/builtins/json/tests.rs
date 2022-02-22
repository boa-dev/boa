use crate::{forward, forward_val, Context};

#[test]
fn json_sanity() {
    let mut context = Context::default();
    assert_eq!(
        forward(&mut context, r#"JSON.parse('{"aaa":"bbb"}').aaa == 'bbb'"#),
        "true"
    );
    assert_eq!(
        forward(
            &mut context,
            r#"JSON.stringify({aaa: 'bbb'}) == '{"aaa":"bbb"}'"#
        ),
        "true"
    );
}

#[test]
fn json_stringify_remove_undefined_values_from_objects() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({ aaa: undefined, bbb: 'ccc' })"#,
    );
    let expected = r#""{"bbb":"ccc"}""#;

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_remove_function_values_from_objects() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({ aaa: () => {}, bbb: 'ccc' })"#,
    );
    let expected = r#""{"bbb":"ccc"}""#;

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_remove_symbols_from_objects() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({ aaa: Symbol(), bbb: 'ccc' })"#,
    );
    let expected = r#""{"bbb":"ccc"}""#;

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_replacer_array_strings() {
    let mut context = Context::default();
    let actual = forward(
        &mut context,
        r#"JSON.stringify({aaa: 'bbb', bbb: 'ccc', ccc: 'ddd'}, ['aaa', 'bbb'])"#,
    );
    let expected = forward(&mut context, r#"'{"aaa":"bbb","bbb":"ccc"}'"#);
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_replacer_array_numbers() {
    let mut context = Context::default();
    let actual = forward(
        &mut context,
        r#"JSON.stringify({ 0: 'aaa', 1: 'bbb', 2: 'ccc'}, [1, 2])"#,
    );
    let expected = forward(&mut context, r#"'{"1":"bbb","2":"ccc"}'"#);
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_replacer_function() {
    let mut context = Context::default();
    let actual = forward(
        &mut context,
        r#"JSON.stringify({ aaa: 1, bbb: 2}, (key, value) => {
            if (key === 'aaa') {
                return undefined;
            }

            return value;
        })"#,
    );
    let expected = forward(&mut context, r#"'{"bbb":2}'"#);
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_arrays() {
    let mut context = Context::default();
    let actual = forward(&mut context, r#"JSON.stringify(['a', 'b'])"#);
    let expected = forward(&mut context, r#"'["a","b"]'"#);

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_object_array() {
    let mut context = Context::default();
    let actual = forward(&mut context, r#"JSON.stringify([{a: 'b'}, {b: 'c'}])"#);
    let expected = forward(&mut context, r#"'[{"a":"b"},{"b":"c"}]'"#);

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_array_converts_undefined_to_null() {
    let mut context = Context::default();
    let actual = forward(&mut context, r#"JSON.stringify([undefined])"#);
    let expected = forward(&mut context, r#"'[null]'"#);

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_array_converts_function_to_null() {
    let mut context = Context::default();
    let actual = forward(&mut context, r#"JSON.stringify([() => {}])"#);
    let expected = forward(&mut context, r#"'[null]'"#);

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_array_converts_symbol_to_null() {
    let mut context = Context::default();
    let actual = forward(&mut context, r#"JSON.stringify([Symbol()])"#);
    let expected = forward(&mut context, r#"'[null]'"#);

    assert_eq!(actual, expected);
}
#[test]
fn json_stringify_function_replacer_propogate_error() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
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
    let expected = forward(&mut context, "1");

    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_function() {
    let mut context = Context::default();

    let actual_function = forward(&mut context, r#"JSON.stringify(() => {})"#);
    let expected = forward(&mut context, r#"undefined"#);

    assert_eq!(actual_function, expected);
}

#[test]
fn json_stringify_undefined() {
    let mut context = Context::default();
    let actual_undefined = forward(&mut context, r#"JSON.stringify(undefined)"#);
    let expected = forward(&mut context, r#"undefined"#);

    assert_eq!(actual_undefined, expected);
}

#[test]
fn json_stringify_symbol() {
    let mut context = Context::default();

    let actual_symbol = forward(&mut context, r#"JSON.stringify(Symbol())"#);
    let expected = forward(&mut context, r#"undefined"#);

    assert_eq!(actual_symbol, expected);
}

#[test]
fn json_stringify_no_args() {
    let mut context = Context::default();

    let actual_no_args = forward(&mut context, r#"JSON.stringify()"#);
    let expected = forward(&mut context, r#"undefined"#);

    assert_eq!(actual_no_args, expected);
}

#[test]
fn json_stringify_fractional_numbers() {
    let mut context = Context::default();

    let actual = forward(&mut context, r#"JSON.stringify(Math.round(1.0))"#);
    let expected = forward(&mut context, r#""1""#);
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_pretty_print() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({a: "b", b: "c"}, undefined, 4)"#,
    );
    let expected = forward(
        &mut context,
        r#"'{\n'
            +'    "a": "b",\n'
            +'    "b": "c"\n'
            +'}'"#,
    );
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_pretty_print_four_spaces() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({a: "b", b: "c"}, undefined, 4.3)"#,
    );
    let expected = forward(
        &mut context,
        r#"'{\n'
            +'    "a": "b",\n'
            +'    "b": "c"\n'
            +'}'"#,
    );
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_pretty_print_twenty_spaces() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({a: "b", b: "c"}, ["a", "b"], 20)"#,
    );
    let expected = forward(
        &mut context,
        r#"'{\n'
            +'          "a": "b",\n'
            +'          "b": "c"\n'
            +'}'"#,
    );
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_pretty_print_with_number_object() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({a: "b", b: "c"}, undefined, new Number(10))"#,
    );
    let expected = forward(
        &mut context,
        r#"'{\n'
        +'          "a": "b",\n'
        +'          "b": "c"\n'
        +'}'"#,
    );
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_pretty_print_bad_space_argument() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({a: "b", b: "c"}, ["a", "b"], [])"#,
    );
    let expected = forward(&mut context, r#"'{"a":"b","b":"c"}'"#);
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_pretty_print_with_too_long_string() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({a: "b", b: "c"}, undefined, "abcdefghijklmn")"#,
    );
    let expected = forward(
        &mut context,
        r#"'{\n'
            +'abcdefghij"a": "b",\n'
            +'abcdefghij"b": "c"\n'
            +'}'"#,
    );
    assert_eq!(actual, expected);
}

#[test]
fn json_stringify_pretty_print_with_string_object() {
    let mut context = Context::default();

    let actual = forward(
        &mut context,
        r#"JSON.stringify({a: "b", b: "c"}, undefined, new String("abcd"))"#,
    );
    let expected = forward(
        &mut context,
        r#"'{\n'
            +'abcd"a": "b",\n'
            +'abcd"b": "c"\n'
            +'}'"#,
    );
    assert_eq!(actual, expected);
}

#[test]
fn json_parse_array_with_reviver() {
    let mut context = Context::default();
    let result = forward_val(
        &mut context,
        r#"JSON.parse('[1,2,3,4]', function(k, v){
            if (typeof v == 'number') {
                return v * 2;
            } else {
                return v;
            }
        })"#,
    )
    .unwrap();
    assert_eq!(
        result
            .get_field("0", &mut context)
            .unwrap()
            .to_number(&mut context)
            .unwrap() as u8,
        2u8
    );
    assert_eq!(
        result
            .get_field("1", &mut context)
            .unwrap()
            .to_number(&mut context)
            .unwrap() as u8,
        4u8
    );
    assert_eq!(
        result
            .get_field("2", &mut context)
            .unwrap()
            .to_number(&mut context)
            .unwrap() as u8,
        6u8
    );
    assert_eq!(
        result
            .get_field("3", &mut context)
            .unwrap()
            .to_number(&mut context)
            .unwrap() as u8,
        8u8
    );
}

#[test]
fn json_parse_object_with_reviver() {
    let mut context = Context::default();
    let result = forward(
        &mut context,
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
    let mut context = Context::default();
    let init = r#"
        const jsonString = "{\"ob\":{\"ject\":1},\"arr\": [0,1]}";
        const jsonObj = JSON.parse(jsonString);
    "#;
    eprintln!("{}", forward(&mut context, init));
    let object_prototype = forward_val(&mut context, r#"jsonObj.ob"#)
        .unwrap()
        .as_object()
        .unwrap()
        .prototype()
        .clone();
    let array_prototype = forward_val(&mut context, r#"jsonObj.arr"#)
        .unwrap()
        .as_object()
        .unwrap()
        .prototype()
        .clone();
    let global_object_prototype = context
        .standard_objects()
        .object_object()
        .prototype()
        .into();
    let global_array_prototype = context.standard_objects().array_object().prototype().into();
    assert_eq!(object_prototype, global_object_prototype);
    assert_eq!(array_prototype, global_array_prototype);
}

#[test]
fn json_fields_should_be_enumerable() {
    let mut context = Context::default();
    let actual_object = forward(
        &mut context,
        r#"
        var a = JSON.parse('{"x":0}');
        a.propertyIsEnumerable('x');
    "#,
    );
    let actual_array_index = forward(
        &mut context,
        r#"
        var b = JSON.parse('[0, 1]');
        b.propertyIsEnumerable('0');
        "#,
    );
    let expected = forward(&mut context, r#"true"#);

    assert_eq!(actual_object, expected);
    assert_eq!(actual_array_index, expected);
}

#[test]
fn json_parse_with_no_args_throws_syntax_error() {
    let mut context = Context::default();
    let result = forward(&mut context, "JSON.parse();");
    assert!(result.contains("SyntaxError"));
}
