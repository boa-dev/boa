use crate::{
    JsNativeErrorKind, JsValue, TestAction, js_string, native_function::NativeFunctionObject,
    run_test_actions,
};
use boa_macros::js_str;
use indoc::indoc;

#[test]
fn constructors() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                var constructed = new RegExp("[0-9]+(\\.[0-9]+)?");
                var literal = /[0-9]+(\.[0-9]+)?/;
                var ctor_literal = new RegExp(/[0-9]+(\.[0-9]+)?/);
            "#}),
        TestAction::assert("constructed.test('1.0')"),
        TestAction::assert("literal.test('1.0')"),
        TestAction::assert("ctor_literal.test('1.0')"),
    ]);
}

#[test]
fn unicode_sets_word_boundaries_ignore_case() {
    run_test_actions([
        TestAction::assert(r"/^(?:\D\b)$/iv.test('\u212a')"),
        TestAction::assert(r"/^(?:\D\b)$/iv.test('\u017f')"),
        TestAction::assert(r"/^(?:\D\b)$/iu.test('\u212a')"),
        TestAction::assert(r"!/^(?:\D\b)$/v.test('\u212a')"),
        TestAction::assert(r"!/^(?:\D\b)$/i.test('\u212a')"),
        TestAction::assert(r"/\w/iv.test('\u212a')"),
        TestAction::assert(r"!/\W/iv.test('\u212a')"),
        TestAction::assert(r"!/\W/iv.test('\u017f')"),
        TestAction::assert("!/a/v.unicode"),
        TestAction::assert("/a/v.unicodeSets"),
    ]);
}

#[test]
fn species() {
    run_test_actions([
        TestAction::run(indoc! {r#"
        var descriptor = Object.getOwnPropertyDescriptor(RegExp, Symbol.species);
        var accessor = descriptor.get;
        var name = Object.getOwnPropertyDescriptor(accessor, "name");
        var length = Object.getOwnPropertyDescriptor(accessor, "length");
        var thisVal = {};
        "#}),
        // length
        TestAction::assert_eq("length.value", 0),
        TestAction::assert("!length.enumerable"),
        TestAction::assert("!length.writable"),
        TestAction::assert("length.configurable"),
        // return-value
        TestAction::assert("Object.is(accessor.call(thisVal), thisVal)"),
        // symbol-species-name
        TestAction::assert_eq("name.value", js_str!("get [Symbol.species]")),
        TestAction::assert("!name.enumerable"),
        TestAction::assert("!name.writable"),
        TestAction::assert("name.configurable"),
        // symbol-species
        TestAction::assert_eq("descriptor.set", JsValue::undefined()),
        TestAction::assert_with_op("accessor", |v, _| {
            v.as_object()
                .is_some_and(|o| o.is::<NativeFunctionObject>())
        }),
        TestAction::assert("!descriptor.enumerable"),
        TestAction::assert("descriptor.configurable"),
    ]);
}

#[test]
fn flags() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                var re_gi = /test/gi;
                var re_sm = /test/sm;
                var re_u = /test/u;
            "#}),
        TestAction::assert("re_gi.global"),
        TestAction::assert("re_gi.ignoreCase"),
        TestAction::assert("!re_gi.multiline"),
        TestAction::assert("!re_gi.dotAll"),
        TestAction::assert("!re_gi.unicode"),
        TestAction::assert("!re_gi.sticky"),
        TestAction::assert_eq("re_gi.flags", js_str!("gi")),
        //
        TestAction::assert("!re_sm.global"),
        TestAction::assert("!re_sm.ignoreCase"),
        TestAction::assert("re_sm.multiline"),
        TestAction::assert("re_sm.dotAll"),
        TestAction::assert("!re_sm.unicode"),
        TestAction::assert("!re_sm.sticky"),
        TestAction::assert_eq("re_sm.flags", js_str!("ms")),
        //
        TestAction::assert("!re_u.global"),
        TestAction::assert("!re_u.ignoreCase"),
        TestAction::assert("!re_u.multiline"),
        TestAction::assert("!re_u.dotAll"),
        TestAction::assert("re_u.unicode"),
        TestAction::assert("!re_u.sticky"),
        TestAction::assert_eq("re_u.flags", js_str!("u")),
    ]);
}

#[test]
fn escape() {
    run_test_actions([
        TestAction::assert_eq(
            r"RegExp.escape('The Quick Brown Fox')",
            js_str!("\\x54he\\x20Quick\\x20Brown\\x20Fox"),
        ),
        TestAction::assert_eq(
            r"RegExp.escape('Buy it. use it. break it. fix it.')",
            js_str!("\\x42uy\\x20it\\.\\x20use\\x20it\\.\\x20break\\x20it\\.\\x20fix\\x20it\\."),
        ),
        TestAction::assert_eq(r"RegExp.escape('(*.*)')", js_str!("\\(\\*\\.\\*\\)")),
        TestAction::assert_eq(r"RegExp.escape('｡^･ｪ･^｡')", js_str!("｡\\^･ｪ･\\^｡")),
        TestAction::assert_eq(
            r"RegExp.escape('😊 *_* +_+ ... 👍')",
            js_str!("😊\\x20\\*_\\*\\x20\\+_\\+\\x20\\.\\.\\.\\x20👍"),
        ),
        TestAction::assert_eq(
            r"RegExp.escape('\\d \\D (?:)')",
            js_str!("\\\\d\\x20\\\\D\\x20\\(\\?\\x3a\\)"),
        ),
        TestAction::assert_native_error(
            "RegExp.escape(1)",
            JsNativeErrorKind::Type,
            "RegExp.escape requires a string argument",
        ),
    ]);
}

#[test]
fn last_index() {
    run_test_actions([
        TestAction::run(r"var regex = /[0-9]+(\.[0-9]+)?/g;"),
        TestAction::assert_eq("regex.lastIndex", 0),
        TestAction::assert("regex.test('1.0foo')"),
        TestAction::assert_eq("regex.lastIndex", 3),
        TestAction::assert("!regex.test('1.0foo')"),
        TestAction::assert_eq("regex.lastIndex", 0),
    ]);
}

#[test]
fn exec() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r"
                var re = /quick\s(brown).+?(jumps)/ig;
                var result = re.exec('The Quick Brown Fox Jumps Over The Lazy Dog');
            "}),
        TestAction::assert(indoc! {r#"
            arrayEquals(
                result,
                ["Quick Brown Fox Jumps", "Brown", "Jumps"]
            )
        "#}),
        TestAction::assert_eq("result.index", 4),
        TestAction::assert_eq(
            "result.input",
            js_str!("The Quick Brown Fox Jumps Over The Lazy Dog"),
        ),
    ]);
}

#[test]
fn no_panic_on_parse_fail() {
    run_test_actions([
        TestAction::assert_native_error(
            r"var re = /]/u;",
            JsNativeErrorKind::Syntax,
            "Invalid regular expression literal: Invalid atom character at line 1, col 10",
        ),
        TestAction::assert_native_error(
            r"var re = /a{/u;",
            JsNativeErrorKind::Syntax,
            "Invalid regular expression literal: Invalid quantifier at line 1, col 10",
        ),
    ]);
}

#[test]
fn to_string() {
    run_test_actions([
        TestAction::assert_eq("(new RegExp('a+b+c')).toString()", js_str!("/a+b+c/")),
        TestAction::assert_eq("(new RegExp('bar', 'g')).toString()", js_str!("/bar/g")),
        TestAction::assert_eq(r"(new RegExp('\\n', 'g')).toString()", js_string!(r"/\n/g")),
        TestAction::assert_eq(r"/\n/g.toString()", js_string!(r"/\n/g")),
        TestAction::assert_eq(r"/,\;/.toString()", js_string!(r"/,\;/")),
    ]);
}
#[test]
fn search() {
    const ERROR: &str = "RegExp.prototype[Symbol.search] method called on incompatible value";
    run_test_actions([
        TestAction::run(indoc! {r#"
                var search = Object.getOwnPropertyDescriptor(RegExp.prototype, Symbol.search);
                var length = Object.getOwnPropertyDescriptor(search.value, 'length');
                var name = Object.getOwnPropertyDescriptor(search.value, 'name');
            "#}),
        // prop-desc
        TestAction::assert("!search.enumerable"),
        TestAction::assert("search.writable"),
        TestAction::assert("search.configurable"),
        // length
        TestAction::assert_eq("length.value", 1),
        TestAction::assert("!length.enumerable"),
        TestAction::assert("!length.writable"),
        TestAction::assert("length.configurable"),
        // name
        TestAction::assert_eq("name.value", js_str!("[Symbol.search]")),
        TestAction::assert("!name.enumerable"),
        TestAction::assert("!name.writable"),
        TestAction::assert("name.configurable"),
        // success-return-val
        TestAction::assert_eq("/a/[Symbol.search]('abc')", 0),
        TestAction::assert_eq("/b/[Symbol.search]('abc')", 1),
        TestAction::assert_eq("/c/[Symbol.search]('abc')", 2),
        // failure-return-val
        TestAction::assert_eq("/z/[Symbol.search]('a')", -1),
        // coerce-string
        TestAction::assert_eq(
            indoc! {r#"
                /ring/[Symbol.search]({
                    toString: function() {
                        return 'toString value';
                    }
                });
            "#},
            4,
        ),
        // this-val-non-obj
        TestAction::assert_native_error("search.value.call()", JsNativeErrorKind::Type, ERROR),
        TestAction::assert_native_error(
            "search.value.call(undefined)",
            JsNativeErrorKind::Type,
            ERROR,
        ),
        TestAction::assert_native_error("search.value.call(null)", JsNativeErrorKind::Type, ERROR),
        TestAction::assert_native_error("search.value.call(true)", JsNativeErrorKind::Type, ERROR),
        TestAction::assert_native_error(
            "search.value.call('string')",
            JsNativeErrorKind::Type,
            ERROR,
        ),
        TestAction::assert_native_error(
            "search.value.call(Symbol.search)",
            JsNativeErrorKind::Type,
            ERROR,
        ),
        TestAction::assert_native_error("search.value.call(86)", JsNativeErrorKind::Type, ERROR),
        // u-lastindex-advance
        TestAction::assert_eq(r"/\udf06/u[Symbol.search]('\ud834\udf06')", -1),
        TestAction::assert_eq("/a/[Symbol.search](\"a\")", 0),
        TestAction::assert_eq("/a/[Symbol.search](\"ba\")", 1),
        TestAction::assert_eq("/a/[Symbol.search](\"bb\")", -1),
        TestAction::assert_eq("/u/[Symbol.search](null)", 1),
        TestAction::assert_eq("/d/[Symbol.search](undefined)", 2),
    ]);
}

#[test]
fn regular_expression_construction_independent_of_global_reg_exp() {
    let regex = "/abc/";
    run_test_actions([
        TestAction::run(regex),
        TestAction::run("RegExp = null"),
        TestAction::run(regex),
    ]);
}

#[test]
fn regexp_no_panic_on_empty_class_quantifier() {
    // Regression test for https://github.com/boa-dev/boa/issues/2718
    // `/[]*1/u.exec()` previously caused a panic in the regress crate.
    // It should return null without panicking.
    run_test_actions([TestAction::assert_eq("/[]*1/u.exec()", JsValue::null())]);
}

#[test]
fn split_with_captures_and_limit() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::assert(r#"arrayEquals("a1b2c".split(/(\d)/), ["a", "1", "b", "2", "c"])"#),
        TestAction::assert(r#"arrayEquals("a-b-c".split(/-/, 2), ["a", "b"])"#),
        TestAction::assert(r#"arrayEquals("a-b-c".split(/-/, 0), [])"#),
        TestAction::assert(r#"arrayEquals("abc".split(/x/), ["abc"])"#),
        TestAction::assert(r#"arrayEquals("".split(/a/), [""])"#),
        TestAction::assert(r#"arrayEquals("".split(/(?:)/), [])"#),
    ]);
}

#[test]
fn split_empty_matches() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::assert(r#"arrayEquals("foo".split(/(?:)/), ["f", "o", "o"])"#),
        TestAction::assert(r#"arrayEquals("aba".split(/a/), ["", "b", ""])"#),
        TestAction::assert(r#"arrayEquals("xax".split(/a*/), ["x", "x"])"#),
        TestAction::assert(r#"arrayEquals("x".split(/$/), ["x"])"#),
        TestAction::assert(r#"arrayEquals("x".split(/^/), ["x"])"#),
        TestAction::assert(r#"arrayEquals("x".split(/\b/), ["x"])"#),
    ]);
}

#[test]
fn split_unicode_and_latin1() {
    run_test_actions([
        TestAction::run_harness(),
        // Latin-1 stored string.
        TestAction::assert(r#"arrayEquals("café".split(/é/), ["caf", ""])"#),
        TestAction::assert(r#"arrayEquals("aa".split(/a/), ["", "", ""])"#),
        // UTF-16 / surrogate pair: without `u`, empty matches split by code unit.
        TestAction::assert(r#"arrayEquals("😀".split(/(?:)/), ["\uD83D", "\uDE00"])"#),
        TestAction::assert(r#"arrayEquals("😀".split(/(?:)/u), ["😀"])"#),
        TestAction::assert(r#"arrayEquals("😀".split(/(?:)/v), ["😀"])"#),
    ]);
}

#[test]
fn split_overridden_exec_is_called_at_every_index() {
    // A user-defined `exec` is observable, so @@split must take the spec path
    // and call it at every index instead of searching forward once per piece.
    run_test_actions([
        TestAction::run(indoc! {r#"
            var calls = [];
            var orig = RegExp.prototype.exec;
            RegExp.prototype.exec = function(s) {
                calls.push(this.lastIndex);
                return orig.call(this, s);
            };
            var result = "aba".split(/a/);
            RegExp.prototype.exec = orig;
        "#}),
        TestAction::run_harness(),
        TestAction::assert(r#"arrayEquals(result, ["", "b", ""])"#),
        TestAction::assert(r#"arrayEquals(calls, [0, 1, 2])"#),
    ]);
}

#[test]
fn sticky_exec() {
    run_test_actions([
        TestAction::run_harness(),
        TestAction::run(indoc! {r#"
            var re = /a/y;
            var first = re.exec("aaa");
            var second = re.exec("aaa");
            var third = re.exec("aaa");
            var done = re.exec("aaa");
        "#}),
        TestAction::assert(r#"arrayEquals(first, ["a"])"#),
        TestAction::assert_eq("first.index", 0),
        TestAction::assert(r#"arrayEquals(second, ["a"])"#),
        TestAction::assert_eq("second.index", 1),
        TestAction::assert(r#"arrayEquals(third, ["a"])"#),
        TestAction::assert_eq("third.index", 2),
        TestAction::assert_eq("done", JsValue::null()),
        TestAction::assert_eq("re.lastIndex", 0),
        TestAction::assert_eq("/a/y.exec('baa')", JsValue::null()),
    ]);
}
