use crate::{builtins::error::ErrorKind, run_test_actions, JsValue, TestAction};
use indoc::indoc;

#[test]
fn function_declaration_returns_undefined() {
    run_test_actions([TestAction::assert_eq(
        "function abc() {}",
        JsValue::undefined(),
    )]);
}

#[test]
fn empty_function_returns_undefined() {
    run_test_actions([TestAction::assert_eq(
        "(function () {}) ()",
        JsValue::undefined(),
    )]);
}

#[test]
fn property_accessor_member_expression_dot_notation_on_function() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            function asd () {};
            asd.name;
        "#},
        "asd",
    )]);
}

#[test]
fn property_accessor_member_expression_bracket_notation_on_function() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            function asd () {};
            asd['name'];
        "#},
        "asd",
    )]);
}

#[test]
fn early_return() {
    run_test_actions([
        TestAction::assert(indoc! {r#"
                function early_return() {
                    if (true) {
                        return true;
                    }
                    return false;
                }
                early_return()
            "#}),
        TestAction::assert_eq(
            indoc! {r#"
                function nested_fnct() {
                    return "nested";
                }
                function outer_fnct() {
                    nested_fnct();
                    return "outer";
                }
                outer_fnct()
            "#},
            "outer",
        ),
    ]);
}

#[test]
fn should_set_this_value() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                function Foo() {
                    this.a = "a";
                    this.b = "b";
                }

                var bar = new Foo();
            "#}),
        TestAction::assert_eq("bar.a", "a"),
        TestAction::assert_eq("bar.b", "b"),
    ]);
}

#[test]
fn should_type_error_when_new_is_not_constructor() {
    run_test_actions([TestAction::assert_native_error(
        "new ''()",
        ErrorKind::Type,
        "not a constructor",
    )]);
}

#[test]
fn new_instance_should_point_to_prototype() {
    // A new instance should point to a prototype object created with the constructor function
    run_test_actions([
        TestAction::run(indoc! {r#"
                function Foo() {}
                var bar = new Foo();
            "#}),
        TestAction::assert("Object.getPrototypeOf(bar) == Foo.prototype "),
    ]);
}

#[test]
fn calling_function_with_unspecified_arguments() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            function test(a, b) {
                return b;
            }

            test(10)
        "#},
        JsValue::undefined(),
    )]);
}

#[test]
fn not_a_function() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                let a = {};
                let b = true;
            "#}),
        TestAction::assert_native_error("a()", ErrorKind::Type, "not a callable function"),
        TestAction::assert_native_error("a.a()", ErrorKind::Type, "not a callable function"),
        TestAction::assert_native_error("b()", ErrorKind::Type, "not a callable function"),
    ]);
}

#[test]
fn strict_mode_dup_func_parameters() {
    // Checks that a function cannot contain duplicate parameter
    // names in strict mode code as per https://tc39.es/ecma262/#sec-function-definitions-static-semantics-early-errors.
    run_test_actions([TestAction::assert_native_error(
        indoc! {r#"
            'use strict';
            function f(a, b, b) {}
        "#},
        ErrorKind::Syntax,
        "Duplicate parameter name not allowed in this context at position: 2:12",
    )]);
}

#[test]
fn duplicate_function_name() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            function f () {}
            function f () {return 12;}
            f()
        "#},
        12,
    )]);
}
