use crate::{run_test_actions, JsValue, TestAction};
use indoc::indoc;

#[test]
fn call_symbol_and_check_return_type() {
    run_test_actions([TestAction::assert_with_op("Symbol()", |val, _| {
        val.is_symbol()
    })]);
}

#[test]
fn print_symbol_expect_description() {
    run_test_actions([TestAction::assert_eq(
        "String(Symbol('Hello'))",
        "Symbol(Hello)",
    )]);
}

#[test]
fn symbol_access() {
    run_test_actions([
        TestAction::run(indoc! {r#"
                var x = {};
                var sym1 = Symbol("Hello");
                var sym2 = Symbol("Hello");
                x[sym1] = 10;
                x[sym2] = 20;
            "#}),
        TestAction::assert_eq("x[sym1]", 10),
        TestAction::assert_eq("x[sym2]", 20),
        TestAction::assert_eq("x['Symbol(Hello)']", JsValue::undefined()),
    ]);
}
