use crate::{JsNativeErrorKind, TestAction, run_test_actions};

#[test]
fn weak_set_add_object() {
    run_test_actions([
        TestAction::run("const ws = new WeakSet(); const o = {};"),
        TestAction::run("ws.add(o);"),
        TestAction::assert("ws.has(o)"),
    ]);
}

#[test]
fn weak_set_delete_object() {
    run_test_actions([
        TestAction::run("const ws = new WeakSet([{}]);"),
        TestAction::run("const o = {}; ws.add(o);"),
        TestAction::assert("ws.delete(o)"),
        TestAction::assert("!ws.has(o)"),
    ]);
}

#[test]
fn weak_set_add_non_holdable_throws() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakSet().add('hello')",
        JsNativeErrorKind::Type,
        "WeakSet.add: invalid value type `string`, expected an object or non-registered symbol",
    )]);
}

#[test]
fn symbol_add_has() {
    run_test_actions([
        TestAction::run("const s = new WeakSet(); const sym = Symbol();"),
        TestAction::run("s.add(sym);"),
        TestAction::assert("s.has(sym)"),
    ]);
}

#[test]
fn symbol_delete() {
    run_test_actions([
        TestAction::run("const s = new WeakSet(); const sym = Symbol();"),
        TestAction::run("s.add(sym);"),
        TestAction::assert("s.delete(sym)"),
        TestAction::assert("!s.has(sym)"),
    ]);
}

#[test]
fn registered_symbol_throws() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakSet().add(Symbol.for('key'))",
        JsNativeErrorKind::Type,
        "WeakSet.add: invalid value type `symbol`, expected an object or non-registered symbol",
    )]);
}

#[test]
fn well_known_symbol_allowed() {
    // well known symbols are not in the registry, so CanBeHeldWeakly is true
    run_test_actions([
        TestAction::run("const s = new WeakSet();"),
        TestAction::run("s.add(Symbol.iterator);"),
        TestAction::assert("s.has(Symbol.iterator)"),
    ]);
}
