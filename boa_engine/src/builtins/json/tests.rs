use indoc::indoc;

use crate::{builtins::error::ErrorKind, run_test, JsValue, TestAction};

#[test]
fn json_sanity() {
    run_test([
        TestAction::assert_eq(r#"JSON.parse('{"aaa":"bbb"}').aaa"#, "bbb"),
        TestAction::assert_eq(r#"JSON.stringify({aaa: 'bbb'})"#, r#"{"aaa":"bbb"}"#),
    ]);
}

#[test]
fn json_stringify_remove_undefined_values_from_objects() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({ aaa: undefined, bbb: 'ccc' })"#,
        r#"{"bbb":"ccc"}"#,
    )]);
}

#[test]
fn json_stringify_remove_function_values_from_objects() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({ aaa: () => {}, bbb: 'ccc' })"#,
        r#"{"bbb":"ccc"}"#,
    )]);
}

#[test]
fn json_stringify_remove_symbols_from_objects() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({ aaa: Symbol(), bbb: 'ccc' })"#,
        r#"{"bbb":"ccc"}"#,
    )]);
}

#[test]
fn json_stringify_replacer_array_strings() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({aaa: 'bbb', bbb: 'ccc', ccc: 'ddd'}, ['aaa', 'bbb'])"#,
        r#"{"aaa":"bbb","bbb":"ccc"}"#,
    )]);
}

#[test]
fn json_stringify_replacer_array_numbers() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({ 0: 'aaa', 1: 'bbb', 2: 'ccc'}, [1, 2])"#,
        r#"{"1":"bbb","2":"ccc"}"#,
    )]);
}

#[test]
fn json_stringify_replacer_function() {
    run_test([TestAction::assert_eq(
        indoc! {r#"
            JSON.stringify({ aaa: 1, bbb: 2}, (key, value) => {
                if (key === 'aaa') {
                    return undefined;
                }

                return value;
            })
        "#},
        r#"{"bbb":2}"#,
    )]);
}

#[test]
fn json_stringify_arrays() {
    run_test([TestAction::assert_eq(
        "JSON.stringify(['a', 'b'])",
        r#"["a","b"]"#,
    )]);
}

#[test]
fn json_stringify_object_array() {
    run_test([TestAction::assert_eq(
        "JSON.stringify([{a: 'b'}, {b: 'c'}])",
        r#"[{"a":"b"},{"b":"c"}]"#,
    )]);
}

#[test]
fn json_stringify_array_converts_undefined_to_null() {
    run_test([TestAction::assert_eq(
        "JSON.stringify([undefined])",
        "[null]",
    )]);
}

#[test]
fn json_stringify_array_converts_function_to_null() {
    run_test([TestAction::assert_eq(
        "JSON.stringify([() => {}])",
        "[null]",
    )]);
}

#[test]
fn json_stringify_array_converts_symbol_to_null() {
    run_test([TestAction::assert_eq(
        "JSON.stringify([Symbol()])",
        "[null]",
    )]);
}
#[test]
fn json_stringify_function_replacer_propagate_error() {
    run_test([TestAction::assert_opaque_error(
        "JSON.stringify({x: 1}, (key, value) => { throw 1 })",
        1,
    )]);
}

#[test]
fn json_stringify_function() {
    run_test([TestAction::assert_eq(
        "JSON.stringify(() => {})",
        JsValue::undefined(),
    )]);
}

#[test]
fn json_stringify_undefined() {
    run_test([TestAction::assert_eq(
        "JSON.stringify(undefined)",
        JsValue::undefined(),
    )]);
}

#[test]
fn json_stringify_symbol() {
    run_test([TestAction::assert_eq(
        "JSON.stringify(Symbol())",
        JsValue::undefined(),
    )]);
}

#[test]
fn json_stringify_no_args() {
    run_test([TestAction::assert_eq(
        "JSON.stringify()",
        JsValue::undefined(),
    )]);
}

#[test]
fn json_stringify_fractional_numbers() {
    run_test([TestAction::assert_eq("JSON.stringify(1.2)", "1.2")]);
}

#[test]
fn json_stringify_pretty_print() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, 4)"#,
        indoc! {r#"
            {
                "a": "b",
                "b": "c"
            }"#
        },
    )]);
}

#[test]
fn json_stringify_pretty_print_four_spaces() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, 4.3)"#,
        indoc! {r#"
            {
                "a": "b",
                "b": "c"
            }"#
        },
    )]);
}

#[test]
fn json_stringify_pretty_print_twenty_spaces() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, 20)"#,
        indoc! {r#"
            {
                      "a": "b",
                      "b": "c"
            }"#
        },
    )]);
}

#[test]
fn json_stringify_pretty_print_with_number_object() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, new Number(10))"#,
        indoc! {r#"
            {
                      "a": "b",
                      "b": "c"
            }"#
        },
    )]);
}

#[test]
fn json_stringify_pretty_print_bad_space_argument() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, [])"#,
        r#"{"a":"b","b":"c"}"#,
    )]);
}

#[test]
fn json_stringify_pretty_print_with_too_long_string() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, "abcdefghijklmn")"#,
        indoc! {r#"
            {
            abcdefghij"a": "b",
            abcdefghij"b": "c"
            }"#
        },
    )]);
}

#[test]
fn json_stringify_pretty_print_with_string_object() {
    run_test([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, new String("abcd"))"#,
        indoc! {r#"
            {
            abcd"a": "b",
            abcd"b": "c"
            }"#
        },
    )]);
}

#[test]
fn json_parse_array_with_reviver() {
    run_test([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
                function reviver(k, v){
                    if (typeof v == 'number') {
                        return v * 2;
                    } else {
                        return v;
                    }
                }
            "#}),
        TestAction::assert("arrayEquals(JSON.parse('[1,2,3,4]', reviver), [2,4,6,8])"),
    ]);
}

#[test]
fn json_parse_object_with_reviver() {
    run_test([
        TestAction::run(indoc! {r#"
                var jsonString = JSON.stringify({
                    firstname: "boa",
                    lastname: "snake"
                });

                function dataReviver(key, value) {
                    if (key == 'lastname') {
                        return 'interpreter';
                    } else {
                        return value;
                    }
                }

                var jsonObj = JSON.parse(jsonString, dataReviver);
            "#}),
        TestAction::assert_eq("jsonObj.firstname", "boa"),
        TestAction::assert_eq("jsonObj.lastname", "interpreter"),
    ]);
}

#[test]
fn json_parse_sets_prototypes() {
    run_test([
        TestAction::run(indoc! {r#"
                const jsonString = "{\"ob\":{\"ject\":1},\"arr\": [0,1]}";
                const jsonObj = JSON.parse(jsonString);
            "#}),
        TestAction::assert("Object.getPrototypeOf(jsonObj.ob) === Object.prototype"),
        TestAction::assert("Object.getPrototypeOf(jsonObj.arr) === Array.prototype"),
    ]);
}

#[test]
fn json_fields_should_be_enumerable() {
    run_test([
        TestAction::assert(indoc! {r#"
                var a = JSON.parse('{"x":0}');
                a.propertyIsEnumerable('x')
            "#}),
        TestAction::assert(indoc! {r#"
                var b = JSON.parse('[0, 1]');
                b.propertyIsEnumerable('0');
            "#}),
    ]);
}

#[test]
fn json_parse_with_no_args_throws_syntax_error() {
    run_test([TestAction::assert_native_error(
        "JSON.parse();",
        ErrorKind::Syntax,
        "expected value at line 1 column 1",
    )]);
}
