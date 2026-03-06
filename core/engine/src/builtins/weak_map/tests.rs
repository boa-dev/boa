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
fn get_or_insert_requires_weakly_holdable_key() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().getOrInsert('x', 1)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsert: invalid key type `string`, expected an object or non-registered symbol",
    )]);
}

#[test]
fn get_or_insert_computed_requires_weakly_holdable_key() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().getOrInsertComputed('x', () => 1)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsertComputed: invalid key type `string`, expected an object or non-registered symbol",
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
fn weak_map_symbol_key_set_get() {
    run_test_actions([
        TestAction::run("const wm = new WeakMap(); const sym = Symbol('s');"),
        TestAction::run("wm.set(sym, 99);"),
        TestAction::assert("wm.has(sym)"),
        TestAction::assert_eq("wm.get(sym)", 99),
    ]);
}

#[test]
fn weak_map_symbol_key_delete() {
    run_test_actions([
        TestAction::run("const wm = new WeakMap(); const sym = Symbol();"),
        TestAction::run("wm.set(sym, 1);"),
        TestAction::assert("wm.delete(sym)"),
        TestAction::assert("!wm.has(sym)"),
    ]);
}

#[test]
fn weak_map_registered_symbol_throws() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().set(Symbol.for('x'), 1)",
        JsNativeErrorKind::Type,
        "WeakMap.set: invalid key type `symbol`, expected an object or non-registered symbol",
    )]);
}

#[test]
fn weak_map_get_or_insert_with_symbol_key() {
    run_test_actions([
        TestAction::run("const wm = new WeakMap(); const sym = Symbol('s');"),
        TestAction::assert_eq("wm.getOrInsert(sym, 42)", 42),
        TestAction::assert("wm.has(sym)"),
        TestAction::assert_eq("wm.getOrInsert(sym, 99)", 42), // returns existing
    ]);
}

#[test]
fn weak_map_get_or_insert_computed_with_symbol_key() {
    run_test_actions([
        TestAction::run("const wm = new WeakMap(); const sym = Symbol();"),
        TestAction::assert_eq("wm.getOrInsertComputed(sym, k => 7)", 7),
        TestAction::assert_eq("wm.get(sym)", 7),
    ]);
}

#[test]
fn weak_map_well_known_symbol_allowed() {
    // well known symbols are NOT in the global registry,
    // so CanBeHeldWeakly returns true
    run_test_actions([
        TestAction::run("const wm = new WeakMap();"),
        TestAction::run("wm.set(Symbol.iterator, 'iter');"),
        TestAction::assert_eq("wm.get(Symbol.iterator)", js_str!("iter")),
    ]);
}
