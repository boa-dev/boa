use crate::{JsNativeErrorKind, TestAction, run_test_actions};

#[test]
fn add_has_delete_object() {
    run_test_actions([
        TestAction::run(
            r#"
            let ws = new WeakSet();
            let obj = {};
            ws.add(obj);
        "#,
        ),
        TestAction::assert("ws.has(obj)"),
        TestAction::assert("ws.delete(obj)"),
        TestAction::assert("!ws.has(obj)"),
    ]);
}

#[test]
fn symbol_add_has_delete() {
    run_test_actions([
        TestAction::run(
            r#"
            let ws = new WeakSet();
            let sym = Symbol("test");
            ws.add(sym);
        "#,
        ),
        TestAction::assert("ws.has(sym)"),
        TestAction::assert("ws.delete(sym)"),
        TestAction::assert("!ws.has(sym)"),
    ]);
}

#[test]
fn symbol_registered_rejected() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakSet().add(Symbol.for('registered'))",
        JsNativeErrorKind::Type,
        "WeakSet.add: invalid value type `symbol`: cannot be held weakly",
    )]);
}

#[test]
fn symbol_well_known_accepted() {
    run_test_actions([
        TestAction::run("let ws = new WeakSet(); ws.add(Symbol.iterator);"),
        TestAction::assert("ws.has(Symbol.iterator)"),
    ]);
}

#[test]
fn primitives_rejected() {
    run_test_actions([
        TestAction::assert_native_error(
            "new WeakSet().add(42)",
            JsNativeErrorKind::Type,
            "WeakSet.add: invalid value type `number`: cannot be held weakly",
        ),
        TestAction::assert_native_error(
            "new WeakSet().add('str')",
            JsNativeErrorKind::Type,
            "WeakSet.add: invalid value type `string`: cannot be held weakly",
        ),
        TestAction::assert_native_error(
            "new WeakSet().add(true)",
            JsNativeErrorKind::Type,
            "WeakSet.add: invalid value type `boolean`: cannot be held weakly",
        ),
    ]);
}

#[test]
fn delete_non_weakly_holdable_returns_false() {
    run_test_actions([TestAction::assert("!new WeakSet().delete(42)")]);
}

#[test]
fn has_non_weakly_holdable_returns_false() {
    run_test_actions([TestAction::assert("!new WeakSet().has('str')")]);
}
