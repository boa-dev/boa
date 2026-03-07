use crate::{TestAction, run_test_actions};

#[test]
fn weakset_add_and_has() {
    run_test_actions([
        TestAction::run("var ws = new WeakSet(); var obj = {}; ws.add(obj);"),
        TestAction::assert("ws.has(obj)"),
    ]);
}

#[test]
fn weakset_add_chaining() {
    run_test_actions([
        TestAction::run("var ws = new WeakSet(); var obj = {};"),
        TestAction::assert("ws.add(obj) === ws"),
    ]);
}

#[test]
fn weakset_delete_behavior() {
    run_test_actions([
        TestAction::run("var ws = new WeakSet(); var obj = {}; ws.add(obj);"),
        TestAction::assert("ws.delete(obj) === true"),
        TestAction::assert("ws.delete(obj) === false"),
        TestAction::assert("ws.has(obj) === false"),
    ]);
}
