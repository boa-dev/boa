use crate::{run_test_actions, TestAction};
use boa_macros::js_str;
use indoc::indoc;

// === Arrow expression body (single return) ===

#[test]
fn inline_arrow_expression_body() {
    run_test_actions([TestAction::assert_eq(
        "((a, b) => a + b)(3, 4)",
        7,
    )]);
}

#[test]
fn inline_arrow_with_outer_variable() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            const x = 10;
            ((a) => a + x)(5)
        "#},
        15,
    )]);
}

#[test]
fn inline_arrow_no_args() {
    run_test_actions([TestAction::assert_eq("(() => 42)()", 42)]);
}

#[test]
fn inline_arrow_missing_args() {
    run_test_actions([TestAction::assert(
        "Number.isNaN(((a, b) => a + b)(1))",
    )]);
}

#[test]
fn inline_arrow_extra_args() {
    run_test_actions([TestAction::assert_eq("((a) => a)(1, 2, 3)", 1)]);
}

#[test]
fn inline_arrow_nested() {
    run_test_actions([TestAction::assert_eq(
        "((a) => ((b) => a + b)(20))(3)",
        23,
    )]);
}

#[test]
fn inline_arrow_string_ops() {
    run_test_actions([TestAction::assert_eq(
        r#"((a, b) => a + " " + b)("hello", "world")"#,
        js_str!("hello world"),
    )]);
}

#[test]
fn inline_arrow_with_object() {
    run_test_actions([TestAction::assert_eq(
        "((obj) => obj.x + obj.y)({ x: 3, y: 4 })",
        7,
    )]);
}

#[test]
fn inline_arrow_side_effects_in_args() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let counter = 0;
            const result = ((a, b) => a + b)(
                (counter++, 10),
                (counter++, 20)
            );
            result + counter
        "#},
        32,
    )]);
}

#[test]
fn inline_arrow_used_in_expression() {
    run_test_actions([TestAction::assert_eq(
        "1 + ((x) => x * 2)(10) + 3",
        24,
    )]);
}

#[test]
fn inline_arrow_boolean_logic() {
    run_test_actions([TestAction::assert("((a, b) => a > b)(5, 3)")]);
}

#[test]
fn inline_arrow_conditional() {
    run_test_actions([TestAction::assert_eq(
        "((x) => x > 0 ? x : -x)(-5)",
        5,
    )]);
}

#[test]
fn inline_arrow_shadowing() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            const a = 100;
            ((a) => a)(42)
        "#},
        42,
    )]);
}

#[test]
fn inline_arrow_no_mutation_leak() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            const x = 1;
            const result = ((x) => x + 10)(5);
            result + x
        "#},
        16,
    )]);
}

// === Function expression with single return ===

#[test]
fn inline_function_expression() {
    run_test_actions([TestAction::assert_eq(
        "(function(a, b) { return a * b; })(6, 7)",
        42,
    )]);
}

// === No return statement (void/side-effect bodies) ===

#[test]
fn inline_arrow_void_body() {
    // Arrow with block body, no return → result is undefined
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let x = 0;
            const result = (() => { x = 42; })();
            result
        "#},
        JsValue::undefined(),
    )]);
}

#[test]
fn inline_arrow_void_body_side_effect() {
    // Verify side effects actually happen
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let x = 0;
            (() => { x = 42; })();
            x
        "#},
        42,
    )]);
}

#[test]
fn inline_function_void_body() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let x = 0;
            (function() { x = 99; })();
            x
        "#},
        99,
    )]);
}

#[test]
fn inline_void_body_multiple_statements() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let x = 0;
            let y = 0;
            ((a, b) => { x = a; y = b; })(10, 20);
            x + y
        "#},
        30,
    )]);
}

#[test]
fn inline_empty_body() {
    run_test_actions([TestAction::assert_eq(
        "(() => {})()",
        JsValue::undefined(),
    )]);
}

// === Statements with trailing return ===

#[test]
fn inline_statements_with_trailing_return() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            ((a, b) => {
                const sum = a + b;
                return sum * 2;
            })(3, 4)
        "#},
        14,
    )]);
}

#[test]
fn inline_function_statements_with_trailing_return() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            (function(x) {
                const doubled = x * 2;
                const tripled = x * 3;
                return doubled + tripled;
            })(10)
        "#},
        50,
    )]);
}

#[test]
fn inline_statements_with_side_effects_and_return() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let sideEffect = 0;
            const result = ((x) => {
                sideEffect = x * 10;
                return x + 1;
            })(5);
            result + sideEffect
        "#},
        // result = 6, sideEffect = 50, total = 56
        56,
    )]);
}

// === Cases that should NOT be inlined (fall back to normal call) ===

#[test]
fn no_inline_early_return() {
    // Early return in an if-statement means the body has a return not at the end.
    // Should still work correctly (just not inlined).
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            (function(x) {
                if (x > 0) return x;
                return -x;
            })(5)
        "#},
        5,
    )]);
}

#[test]
fn no_inline_early_return_negative() {
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            (function(x) {
                if (x > 0) return x;
                return -x;
            })(-5)
        "#},
        5,
    )]);
}

#[test]
fn no_inline_return_in_nested_function_is_ok() {
    // A return inside a nested function should not prevent inlining of the outer body.
    run_test_actions([TestAction::assert_eq(
        indoc! {r#"
            let result = 0;
            (() => {
                const inner = function() { return 42; };
                result = inner();
            })();
            result
        "#},
        42,
    )]);
}

use crate::JsValue;
