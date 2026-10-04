use boa_macros::js_str;
use indoc::indoc;

use crate::{JsNativeErrorKind, JsValue, TestAction, js_string, run_test_actions};

#[test]
fn json_sanity() {
    run_test_actions([
        TestAction::assert_eq(r#"JSON.parse('{"aaa":"bbb"}').aaa"#, js_str!("bbb")),
        TestAction::assert_eq(
            r#"JSON.stringify({aaa: 'bbb'})"#,
            js_string!(r#"{"aaa":"bbb"}"#),
        ),
    ]);
}

#[test]
fn json_stringify_remove_undefined_values_from_objects() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({ aaa: undefined, bbb: 'ccc' })"#,
        js_string!(r#"{"bbb":"ccc"}"#),
    )]);
}

#[test]
fn json_stringify_remove_function_values_from_objects() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({ aaa: () => {}, bbb: 'ccc' })"#,
        js_string!(r#"{"bbb":"ccc"}"#),
    )]);
}

#[test]
fn json_stringify_remove_symbols_from_objects() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({ aaa: Symbol(), bbb: 'ccc' })"#,
        js_string!(r#"{"bbb":"ccc"}"#),
    )]);
}

#[test]
fn json_stringify_replacer_array_strings() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({aaa: 'bbb', bbb: 'ccc', ccc: 'ddd'}, ['aaa', 'bbb'])"#,
        js_string!(r#"{"aaa":"bbb","bbb":"ccc"}"#),
    )]);
}

#[test]
fn json_stringify_replacer_array_numbers() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({ 0: 'aaa', 1: 'bbb', 2: 'ccc'}, [1, 2])"#,
        js_string!(r#"{"1":"bbb","2":"ccc"}"#),
    )]);
}

#[test]
fn json_stringify_replacer_function() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            JSON.stringify({ aaa: 1, bbb: 2}, (key, value) => {
                if (key === 'aaa') {
                    return undefined;
                }

                return value;
            })
        "#},
        js_string!(r#"{"bbb":2}"#),
    )]);
}

#[test]
fn json_stringify_arrays() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(['a', 'b'])",
        js_string!(r#"["a","b"]"#),
    )]);
}

#[test]
fn json_stringify_object_array() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify([{a: 'b'}, {b: 'c'}])",
        js_string!(r#"[{"a":"b"},{"b":"c"}]"#),
    )]);
}

#[test]
fn json_stringify_array_converts_undefined_to_null() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify([undefined])",
        js_str!("[null]"),
    )]);
}

#[test]
fn json_stringify_array_converts_function_to_null() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify([() => {}])",
        js_str!("[null]"),
    )]);
}

#[test]
fn json_stringify_array_converts_symbol_to_null() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify([Symbol()])",
        js_str!("[null]"),
    )]);
}
#[test]
fn json_stringify_function_replacer_propagate_error() {
    run_test_actions([TestAction::assert_opaque_error(
        "JSON.stringify({x: 1}, (key, value) => { throw 1 })",
        1,
    )]);
}

#[test]
fn json_stringify_function() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(() => {})",
        JsValue::undefined(),
    )]);
}

#[test]
fn json_stringify_undefined() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(undefined)",
        JsValue::undefined(),
    )]);
}

#[test]
fn json_stringify_symbol() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify(Symbol())",
        JsValue::undefined(),
    )]);
}

#[test]
fn json_stringify_no_args() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify()",
        JsValue::undefined(),
    )]);
}

#[test]
fn json_stringify_fractional_numbers() {
    run_test_actions([TestAction::assert_eq("JSON.stringify(1.2)", js_str!("1.2"))]);
}

#[test]
fn json_stringify_pretty_print() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, 4)"#,
        js_string!(indoc! {r#"
            {
                "a": "b",
                "b": "c"
            }"#
        }),
    )]);
}

#[test]
fn json_stringify_pretty_print_four_spaces() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, 4.3)"#,
        js_string!(indoc! {r#"
            {
                "a": "b",
                "b": "c"
            }"#
        }),
    )]);
}

#[test]
fn json_stringify_pretty_print_twenty_spaces() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, 20)"#,
        js_string!(indoc! {r#"
            {
                      "a": "b",
                      "b": "c"
            }"#
        }),
    )]);
}

#[test]
fn json_stringify_pretty_print_with_number_object() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, new Number(10))"#,
        js_string!(indoc! {r#"
            {
                      "a": "b",
                      "b": "c"
            }"#
        }),
    )]);
}

#[test]
fn json_stringify_pretty_print_bad_space_argument() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, [])"#,
        js_string!(r#"{"a":"b","b":"c"}"#),
    )]);
}

#[test]
fn json_stringify_pretty_print_with_too_long_string() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, "abcdefghijklmn")"#,
        js_string!(indoc! {r#"
            {
            abcdefghij"a": "b",
            abcdefghij"b": "c"
            }"#
        }),
    )]);
}

#[test]
fn json_stringify_pretty_print_with_string_object() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify({a: "b", b: "c"}, undefined, new String("abcd"))"#,
        js_string!(indoc! {r#"
            {
            abcd"a": "b",
            abcd"b": "c"
            }"#
        }),
    )]);
}

#[test]
fn json_parse_array_with_reviver() {
    run_test_actions([
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
    run_test_actions([
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
        TestAction::assert_eq("jsonObj.firstname", js_str!("boa")),
        TestAction::assert_eq("jsonObj.lastname", js_str!("interpreter")),
    ]);
}

#[test]
fn json_parse_sets_prototypes() {
    run_test_actions([
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
    run_test_actions([
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
    run_test_actions([TestAction::assert_native_error(
        "JSON.parse();",
        JsNativeErrorKind::Syntax,
        "expected value at line 1 column 1",
    )]);
}

#[test]
fn json_stringify_cyclic_object_throws_type_error() {
    run_test_actions([TestAction::assert_native_error(
        "var a = {}; a.a = a; JSON.stringify(a);",
        JsNativeErrorKind::Type,
        "cyclic object value",
    )]);
}

#[test]
fn json_stringify_cyclic_array_throws_type_error() {
    run_test_actions([TestAction::assert_native_error(
        "var a = []; a[0] = a; JSON.stringify(a);",
        JsNativeErrorKind::Type,
        "cyclic object value",
    )]);
}

#[test]
fn json_stringify_cyclic_nested_object_throws_type_error() {
    run_test_actions([TestAction::assert_native_error(
        "var a = {}; var b = { a }; a.b = b; JSON.stringify(a);",
        JsNativeErrorKind::Type,
        "cyclic object value",
    )]);
}

#[test]
fn json_parse_number_outside_of_f64_range() {
    run_test_actions([
        TestAction::assert_eq("JSON.parse('1e1000')", f64::INFINITY),
        TestAction::assert_eq("JSON.parse('-1e1000')", f64::NEG_INFINITY),
        TestAction::assert_eq("JSON.parse('[1E+400]')[0]", f64::INFINITY),
        TestAction::assert_eq(r#"JSON.parse('{"a":1e999}').a"#, f64::INFINITY),
        TestAction::assert_eq("JSON.parse('9'.repeat(400))", f64::INFINITY),
        TestAction::assert_eq("JSON.parse('1e-1000')", 0),
        TestAction::assert_eq(
            "JSON.parse('[1e1000]', (key, value) => value)[0]",
            f64::INFINITY,
        ),
    ]);
}

#[test]
fn json_parse_escaped_lone_surrogate() {
    run_test_actions([
        TestAction::assert_eq(r#"JSON.parse('"\\uD800"').length"#, 1),
        TestAction::assert_eq(r#"JSON.parse('"\\uD800"').charCodeAt(0)"#, 0xD800),
        TestAction::assert_eq(r#"JSON.parse('"\\uDC00"').charCodeAt(0)"#, 0xDC00),
        TestAction::assert_eq(r#"JSON.parse('"\\uD83D\\uDE00"').length"#, 2),
    ]);
}

#[test]
fn json_parse_rejects_invalid_json_text() {
    run_test_actions([TestAction::assert(indoc! {r#"
        [
            '01', '1.', '.5', '1e', '+1', 'Infinity', '1 2', '"\\u12"', '"\\x41"', '"a\nb"',
            "'a'", '[1,]', '[1', '',
        ].every((text) => {
            try {
                JSON.parse(text);
                return false;
            } catch (e) {
                return e instanceof SyntaxError;
            }
        })
    "#})]);
}

#[test]
fn json_parse_nesting_limit() {
    run_test_actions([
        TestAction::assert_native_error(
            "JSON.parse('['.repeat(128) + ']'.repeat(128))",
            JsNativeErrorKind::Syntax,
            "JSON text is nested too deeply",
        ),
        TestAction::assert_native_error(
            r#"JSON.parse('{"a":'.repeat(128) + '1' + '}'.repeat(128))"#,
            JsNativeErrorKind::Syntax,
            "JSON text is nested too deeply",
        ),
        // Brackets inside of strings are not nesting.
        TestAction::assert_eq(r#"JSON.parse('"' + '['.repeat(300) + '"').length"#, 300),
        TestAction::assert_eq(r#"JSON.parse('["a\\"[[[", "\\\\"]').length"#, 2),
    ]);
}

#[test]
fn json_raw_json_number_outside_of_f64_range() {
    run_test_actions([
        TestAction::assert_eq(
            "JSON.stringify({ a: JSON.rawJSON('1e1000') })",
            js_string!(r#"{"a":1e1000}"#),
        ),
        TestAction::assert_native_error(
            "JSON.rawJSON('[1]')",
            JsNativeErrorKind::Syntax,
            "JSON.rawJSON text must not be an object or array",
        ),
        TestAction::assert_native_error(
            "JSON.rawJSON('{}')",
            JsNativeErrorKind::Syntax,
            "JSON.rawJSON text must not be an object or array",
        ),
    ]);
}
