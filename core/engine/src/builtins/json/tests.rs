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
fn json_parse_lone_surrogate_escapes() {
    run_test_actions([
        TestAction::assert_eq(r#"JSON.parse('"\\ud83d"').length"#, 1),
        TestAction::assert_eq(r#"JSON.parse('"\\ud83d"').charCodeAt(0)"#, 0xd83d),
        TestAction::assert_eq(r#"JSON.parse('"\\udc00x"').charCodeAt(0)"#, 0xdc00),
        TestAction::assert_eq(r#"JSON.parse('["a\\ud83d"]')[0].length"#, 2),
        TestAction::assert_eq(r#"JSON.parse('"\\ud83d\\ude00"') === "\u{1F600}""#, true),
    ]);
}

#[test]
fn json_parse_invalid_text_still_throws_syntax_error() {
    run_test_actions([TestAction::assert_native_error(
        r#"JSON.parse('"\\ud83"');"#,
        JsNativeErrorKind::Syntax,
        "invalid escape at line 1 column 7",
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
fn json_parse_proto_key_is_an_own_property() {
    run_test_actions([
        TestAction::assert(
            r#"Object.getPrototypeOf(JSON.parse('{"__proto__": 1}')) === Object.prototype"#,
        ),
        TestAction::assert(
            r#"Object.getOwnPropertyNames(JSON.parse('{"__proto__": 1}'))[0] === "__proto__""#,
        ),
        TestAction::assert_eq(
            r#"JSON.parse('{"__proto__": {"x": 1}}').x"#,
            JsValue::undefined(),
        ),
    ]);
}

#[test]
fn json_parse_duplicate_keys_keep_first_position_and_last_value() {
    run_test_actions([TestAction::assert_eq(
        r#"JSON.stringify(JSON.parse('{"a": 1, "b": 2, "a": 3}'))"#,
        js_str!(r#"{"a":3,"b":2}"#),
    )]);
}

#[test]
fn json_parse_numbers() {
    run_test_actions([
        TestAction::assert("Object.is(JSON.parse('-0'), -0)"),
        TestAction::assert_eq(
            "JSON.parse('123456789012345678901234567890')",
            1.234_567_890_123_456_8e29,
        ),
        TestAction::assert_eq("JSON.parse('1e-7')", 1e-7),
        TestAction::assert_eq("JSON.parse('-12.5E+2')", -1250.0),
        TestAction::assert_eq("JSON.parse(' [ 2147483648 ] ')[0]", 2_147_483_648.0),
        TestAction::assert_native_error(
            "JSON.parse('01')",
            JsNativeErrorKind::Syntax,
            "trailing characters at line 1 column 2",
        ),
        TestAction::assert_native_error(
            "JSON.parse('1.')",
            JsNativeErrorKind::Syntax,
            "EOF while parsing a value at line 1 column 2",
        ),
        TestAction::assert_native_error(
            "JSON.parse('+1')",
            JsNativeErrorKind::Syntax,
            "expected value at line 1 column 1",
        ),
    ]);
}

#[test]
fn json_parse_strings() {
    run_test_actions([
        TestAction::assert_eq(r#"JSON.parse('"a\\u00e9\\n\\"b"')"#, js_str!("aé\n\"b")),
        // A lone surrogate, escaped or raw, is a valid JSON string.
        TestAction::assert_eq(r#"JSON.parse('"\\ud800"').charCodeAt(0)"#, 0xD800),
        TestAction::assert_eq(
            r#"JSON.parse('"' + String.fromCharCode(0xDC00) + '"').charCodeAt(0)"#,
            0xDC00,
        ),
        TestAction::assert_eq(r#"JSON.parse('"ünïcödé ✓"')"#, js_str!("ünïcödé ✓")),
        TestAction::assert_native_error(
            r#"JSON.parse('"a\nb"')"#,
            JsNativeErrorKind::Syntax,
            "control character (\\u0000-\\u001F) found while parsing a string at line 1 column 3",
        ),
        TestAction::assert_native_error(
            r#"JSON.parse('"\\x41"')"#,
            JsNativeErrorKind::Syntax,
            "invalid escape at line 1 column 3",
        ),
        TestAction::assert_native_error(
            r#"JSON.parse('"abc')"#,
            JsNativeErrorKind::Syntax,
            "EOF while parsing a string at line 1 column 4",
        ),
    ]);
}

#[test]
fn json_parse_structure_errors() {
    run_test_actions([
        TestAction::assert_native_error(
            "JSON.parse('[1, 2,]')",
            JsNativeErrorKind::Syntax,
            "trailing comma at line 1 column 7",
        ),
        TestAction::assert_native_error(
            r#"JSON.parse('{"a": 1,}')"#,
            JsNativeErrorKind::Syntax,
            "trailing comma at line 1 column 9",
        ),
        TestAction::assert_native_error(
            "JSON.parse('[1 2]')",
            JsNativeErrorKind::Syntax,
            "expected `,` or `]` at line 1 column 4",
        ),
        TestAction::assert_native_error(
            "JSON.parse('{a: 1}')",
            JsNativeErrorKind::Syntax,
            "key must be a string at line 1 column 2",
        ),
        TestAction::assert_native_error(
            r#"JSON.parse('{"a" 1}')"#,
            JsNativeErrorKind::Syntax,
            "expected `:` at line 1 column 6",
        ),
        TestAction::assert_native_error(
            "JSON.parse('[\\n1,\\n')",
            JsNativeErrorKind::Syntax,
            "EOF while parsing a list at line 3 column 0",
        ),
        TestAction::assert_native_error(
            "JSON.parse('tru')",
            JsNativeErrorKind::Syntax,
            "EOF while parsing a value at line 1 column 3",
        ),
        TestAction::assert_native_error(
            "JSON.parse('nul1')",
            JsNativeErrorKind::Syntax,
            "expected ident at line 1 column 4",
        ),
        TestAction::assert_native_error(
            "JSON.parse('\\u00a0 1')",
            JsNativeErrorKind::Syntax,
            "expected value at line 1 column 1",
        ),
    ]);
}

#[test]
fn json_parse_deep_nesting_does_not_overflow() {
    run_test_actions([
        TestAction::assert_eq(
            "let s = '['.repeat(100000) + ']'.repeat(100000); let v = JSON.parse(s); let d = 0; while (Array.isArray(v)) { d++; v = v[0]; } d",
            100_000,
        ),
        TestAction::assert_eq(
            r#"let o = '{"a":'.repeat(50000) + '1' + '}'.repeat(50000); let w = JSON.parse(o); let e = 0; while (typeof w === "object") { e++; w = w.a; } e"#,
            50_000,
        ),
    ]);
}

#[test]
fn json_parse_reviver_source_after_nesting() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            const seen = [];
            JSON.parse('{"a": [1.50, {"b": -0.0}], "c": "x"}', function (key, value, context) {
                if (typeof value !== "object") seen.push(key + "=" + context.source);
                return value;
            });
            seen.join(" ")
        "#},
        js_str!("0=1.50 b=-0.0 c=\"x\""),
    )]);
}

#[test]
fn json_parse_floats_from_utf16_and_long_input() {
    run_test_actions([
        // The `\u20ac` makes the text a UTF-16 string.
        TestAction::assert_eq(r#"JSON.parse('["\u20ac", 1.5e3]')[1]"#, 1500.0),
        TestAction::assert_eq(r#"JSON.parse('["\u20ac", -0.25]')[1]"#, -0.25),
        // Longer than the 64-byte stack buffer, Latin-1 and UTF-16.
        TestAction::assert_eq("JSON.parse('1.' + '0'.repeat(100) + '1')", 1.0),
        TestAction::assert_eq(
            r#"JSON.parse('["\u20ac", 1.' + '0'.repeat(100) + '1]')[1]"#,
            1.0,
        ),
        TestAction::assert_eq("JSON.parse('1e400')", f64::INFINITY),
    ]);
}
