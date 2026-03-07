use crate::{JsNativeErrorKind, TestAction, run_test_actions};

#[test]
fn weak_set_symbol_support() {
    run_test_actions([
        TestAction::run("const ws = new WeakSet(); const s = Symbol(); ws.add(s);"),
        TestAction::assert("ws.has(s)"),
    ]);
}

#[test]
fn weak_set_global_symbol_rejection() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakSet().add(Symbol.for('sim'))",
        JsNativeErrorKind::Type,
        "WeakSet.add: expected target argument of type `object` or non-registered symbol, got target of type `symbol`",
    )]);
}
