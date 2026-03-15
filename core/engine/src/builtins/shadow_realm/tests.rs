#![cfg(feature = "experimental")]
use crate::{run_test_actions, TestAction};

#[test]
fn constructor() {
    run_test_actions([
        TestAction::assert("new ShadowRealm() instanceof ShadowRealm"),
        TestAction::assert("typeof ShadowRealm.prototype.evaluate === 'function'"),
        TestAction::assert("typeof ShadowRealm.prototype.importValue === 'function'"),
    ]);
}

#[test]
fn evaluate_isolation() {
    run_test_actions([
        TestAction::run("const realm = new ShadowRealm();"),
        TestAction::run("realm.evaluate('globalThis.x = 42;');"),
        TestAction::assert("globalThis.x === undefined"),
        TestAction::assert("realm.evaluate('globalThis.x') === 42"),
        TestAction::assert("realm.evaluate('globalThis.x = 100;'); realm.evaluate('globalThis.x') === 100"),
    ]);
}
