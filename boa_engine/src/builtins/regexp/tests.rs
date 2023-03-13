use crate::{builtins::error::ErrorKind, object::JsObject, run_test_actions, JsValue, TestAction};
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
        TestAction::assert_eq("name.value", "get [Symbol.species]"),
        TestAction::assert("!name.enumerable"),
        TestAction::assert("!name.writable"),
        TestAction::assert("name.configurable"),
        // symbol-species
        TestAction::assert_eq("descriptor.set", JsValue::undefined()),
        TestAction::assert_with_op("accessor", |v, _| {
            v.as_object().map_or(false, JsObject::is_function)
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
            "#}),
        TestAction::assert("re_gi.global"),
        TestAction::assert("re_gi.ignoreCase"),
        TestAction::assert("!re_gi.multiline"),
        TestAction::assert("!re_gi.dotAll"),
        TestAction::assert("!re_gi.unicode"),
        TestAction::assert("!re_gi.sticky"),
        TestAction::assert_eq("re_gi.flags", "gi"),
        //
        TestAction::assert("!re_sm.global"),
        TestAction::assert("!re_sm.ignoreCase"),
        TestAction::assert("re_sm.multiline"),
        TestAction::assert("re_sm.dotAll"),
        TestAction::assert("!re_sm.unicode"),
        TestAction::assert("!re_sm.sticky"),
        TestAction::assert_eq("re_sm.flags", "ms"),
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
        TestAction::run(indoc! {r#"
                var re = /quick\s(brown).+?(jumps)/ig;
                var result = re.exec('The Quick Brown Fox Jumps Over The Lazy Dog');
            "#}),
        TestAction::assert(indoc! {r#"
            arrayEquals(
                result,
                ["Quick Brown Fox Jumps", "Brown", "Jumps"]
            )
        "#}),
        TestAction::assert_eq("result.index", 4),
        TestAction::assert_eq(
            "result.input",
            "The Quick Brown Fox Jumps Over The Lazy Dog",
        ),
    ]);
}

#[test]
fn to_string() {
    run_test_actions([
        TestAction::assert_eq("(new RegExp('a+b+c')).toString()", "/a+b+c/"),
        TestAction::assert_eq("(new RegExp('bar', 'g')).toString()", "/bar/g"),
        TestAction::assert_eq(r"(new RegExp('\\n', 'g')).toString()", r"/\n/g"),
        TestAction::assert_eq(r"/\n/g.toString()", r"/\n/g"),
        TestAction::assert_eq(r"/,\;/.toString()", r"/,\;/"),
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
        TestAction::assert_eq("name.value", "[Symbol.search]"),
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
        TestAction::assert_native_error("search.value.call()", ErrorKind::Type, ERROR),
        TestAction::assert_native_error("search.value.call(undefined)", ErrorKind::Type, ERROR),
        TestAction::assert_native_error("search.value.call(null)", ErrorKind::Type, ERROR),
        TestAction::assert_native_error("search.value.call(true)", ErrorKind::Type, ERROR),
        TestAction::assert_native_error("search.value.call('string')", ErrorKind::Type, ERROR),
        TestAction::assert_native_error("search.value.call(Symbol.search)", ErrorKind::Type, ERROR),
        TestAction::assert_native_error("search.value.call(86)", ErrorKind::Type, ERROR),
        // u-lastindex-advance
        TestAction::assert_eq(r"/\udf06/u[Symbol.search]('\ud834\udf06')", -1),
        TestAction::assert_eq("/a/[Symbol.search](\"a\")", 0),
        TestAction::assert_eq("/a/[Symbol.search](\"ba\")", 1),
        TestAction::assert_eq("/a/[Symbol.search](\"bb\")", -1),
        TestAction::assert_eq("/u/[Symbol.search](null)", 1),
        TestAction::assert_eq("/d/[Symbol.search](undefined)", 2),
    ]);
}
