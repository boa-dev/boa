use crate::{
    js_string, object::JsObject, run_test_actions, JsNativeErrorKind, JsValue, TestAction,
};
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
        TestAction::assert_eq("name.value", js_string!("get [Symbol.species]")),
        TestAction::assert("!name.enumerable"),
        TestAction::assert("!name.writable"),
        TestAction::assert("name.configurable"),
        // symbol-species
        TestAction::assert_eq("descriptor.set", JsValue::undefined()),
        TestAction::assert_with_op("accessor", |v, _| {
            v.as_object().map_or(false, JsObject::is_native_function)
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
        TestAction::assert_eq("re_gi.flags", js_string!("gi")),
        //
        TestAction::assert("!re_sm.global"),
        TestAction::assert("!re_sm.ignoreCase"),
        TestAction::assert("re_sm.multiline"),
        TestAction::assert("re_sm.dotAll"),
        TestAction::assert("!re_sm.unicode"),
        TestAction::assert("!re_sm.sticky"),
        TestAction::assert_eq("re_sm.flags", js_string!("ms")),
        //
        TestAction::assert("!re_u.global"),
        TestAction::assert("!re_u.ignoreCase"),
        TestAction::assert("!re_u.multiline"),
        TestAction::assert("!re_u.dotAll"),
        TestAction::assert("re_u.unicode"),
        TestAction::assert("!re_u.sticky"),
        TestAction::assert_eq("re_u.flags", js_string!("u")),
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
            js_string!("The Quick Brown Fox Jumps Over The Lazy Dog"),
        ),
    ]);
}

#[test]
fn no_panic_on_parse_fail() {
    run_test_actions([
        TestAction::assert_native_error(
            r"var re = /]/u;",
            JsNativeErrorKind::Syntax,
            "Invalid regular expression literal: Unbalanced bracket at line 1, col 10",
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
        TestAction::assert_eq("(new RegExp('a+b+c')).toString()", js_string!("/a+b+c/")),
        TestAction::assert_eq("(new RegExp('bar', 'g')).toString()", js_string!("/bar/g")),
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
        TestAction::assert_eq("name.value", js_string!("[Symbol.search]")),
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
fn regular_expression_construction_independant_of_global_reg_exp() {
    let regex = "/abc/";
    run_test_actions([
        TestAction::run(regex),
        TestAction::run("RegExp = null"),
        TestAction::run(regex),
    ]);
}
