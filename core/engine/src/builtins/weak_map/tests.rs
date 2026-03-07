use crate::{JsNativeErrorKind, TestAction, run_test_actions};
use boa_macros::js_str;

#[test]
fn get_or_insert_inserts_on_miss() {
    run_test_actions([
        TestAction::run("let wm = new WeakMap(); let k = {};"),
        TestAction::assert_eq("wm.getOrInsert(k, 42)", 42),
        TestAction::assert("wm.has(k)"),
        TestAction::assert_eq("wm.get(k)", 42),
    ]);
}

#[test]
fn get_or_insert_returns_existing_on_hit() {
    run_test_actions([
        TestAction::run("let k = {}; let wm = new WeakMap([[k, 99]]);"),
        TestAction::assert_eq("wm.getOrInsert(k, 123)", 99),
        TestAction::assert_eq("wm.get(k)", 99), // unchanged
    ]);
}

#[test]
fn get_or_insert_computed_requires_callable() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().getOrInsertComputed({}, undefined)",
        JsNativeErrorKind::Type,
        "Method WeakMap.prototype.getOrInsertComputed called with non-callable callback function",
    )]);
}

#[test]
fn get_or_insert_requires_object_key() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().getOrInsert('x', 1)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsert: expected target argument of type `object` or non-registered symbol, got target of type `string`",
    )]);
}

#[test]
fn get_or_insert_computed_requires_object_key() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().getOrInsertComputed('x', () => 1)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsertComputed: expected target argument of type `object` or non-registered symbol, got target of type `string`",
    )]);
}

#[test]
fn get_or_insert_computed_not_called_on_hit() {
    run_test_actions([
        TestAction::run("const k = {}; const wm = new WeakMap([[k, 7]]); let calls = 0;"),
        TestAction::assert_eq(
            "wm.getOrInsertComputed(k, (key) => { calls++; return 1; })",
            7,
        ),
        TestAction::assert_eq("calls", 0),
        TestAction::assert_eq("wm.get(k)", 7),
    ]);
}

#[test]
fn get_or_insert_computed_this_is_undefined_and_key_forwarded() {
    run_test_actions([
        TestAction::run(
            r#"
            const wm = new WeakMap();
            const k = {};
            let seenThis, seenKey;
            const v = wm.getOrInsertComputed(k, function(x) { 'use strict'; seenThis = this; seenKey = x; return 'ok'; });
        "#,
        ),
        // `this` inside callback is undefined
        TestAction::assert("seenThis === undefined"),
        // key argument is forwarded as the same object
        TestAction::assert("seenKey === k"),
        TestAction::assert_eq("v", js_str!("ok")),
        TestAction::assert("wm.has(k)"),
        TestAction::assert_eq("wm.get(k)", js_str!("ok")),
    ]);
}

#[test]
fn get_or_insert_computed_overwrites_race() {
    run_test_actions([
        TestAction::run(
            r#"
            const wm = new WeakMap();
            const k = {};
            const v = wm.getOrInsertComputed(k, function(x) { wm.set(k, 'other'); return 'computed'; });
        "#,
        ),
        TestAction::assert_eq("v", js_str!("computed")),
        TestAction::assert_eq("wm.get(k)", js_str!("computed")),
    ]);
}

#[test]
fn get_or_insert_this_not_weakmap() {
    run_test_actions([TestAction::assert_native_error(
        "WeakMap.prototype.getOrInsert.call({}, {}, 1)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsert: called with non-object value",
    )]);
}

#[test]
fn get_or_insert_computed_this_not_weakmap() {
    run_test_actions([TestAction::assert_native_error(
        "WeakMap.prototype.getOrInsertComputed.call({}, {}, x => x)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsertComputed: called with non-object value",
    )]);
}

#[test]
fn weak_map_symbol_support() {
    run_test_actions([
        TestAction::run("const wm = new WeakMap(); const s = Symbol(); wm.set(s, 10);"),
        TestAction::assert_eq("wm.get(s)", 10),
    ]);
}

#[test]
fn weak_map_global_symbol_rejection() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().set(Symbol.for('sim'), 1)",
        JsNativeErrorKind::Type,
        "WeakMap.set: expected target argument of type `object` or non-registered symbol, got target of type `symbol`",
    )]);
}

#[test]
fn weak_map_global_symbol_nonthrowing_ops() {
    run_test_actions([
        TestAction::run("const wm = new WeakMap(); const s = Symbol.for('sim');"),
        TestAction::assert("wm.get(s) === undefined"),
        TestAction::assert("wm.has(s) === false"),
        TestAction::assert("wm.delete(s) === false"),
    ]);
}
