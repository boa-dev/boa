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
        "WeakMap.getOrInsertComputed: callback is not a function",
    )]);
}

#[test]
fn get_or_insert_requires_weakly_holdable_key() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().getOrInsert('x', 1)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsert: invalid key type `string`: cannot be held weakly",
    )]);
}

#[test]
fn get_or_insert_computed_requires_weakly_holdable_key() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().getOrInsertComputed('x', () => 1)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsertComputed: invalid key type `string`: cannot be held weakly",
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

// --- Symbol key tests ---

#[test]
fn symbol_key_set_get_has_delete() {
    run_test_actions([
        TestAction::run(
            r#"
            let sym = Symbol("test");
            let wm = new WeakMap();
            wm.set(sym, 42);
        "#,
        ),
        TestAction::assert("wm.has(sym)"),
        TestAction::assert_eq("wm.get(sym)", 42),
        TestAction::assert("wm.delete(sym)"),
        TestAction::assert("!wm.has(sym)"),
        TestAction::assert_eq("wm.get(sym)", crate::JsValue::undefined()),
    ]);
}

#[test]
fn symbol_key_registered_rejected() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().set(Symbol.for('registered'), 1)",
        JsNativeErrorKind::Type,
        "WeakMap.set: invalid key type `symbol`: cannot be held weakly",
    )]);
}

#[test]
fn symbol_key_well_known_accepted() {
    run_test_actions([
        TestAction::run(
            r#"
            let wm = new WeakMap();
            wm.set(Symbol.iterator, "iter_value");
        "#,
        ),
        TestAction::assert("wm.has(Symbol.iterator)"),
        TestAction::assert_eq("wm.get(Symbol.iterator)", js_str!("iter_value")),
    ]);
}

#[test]
fn symbol_key_get_or_insert() {
    run_test_actions([
        TestAction::run(
            r#"
            let sym = Symbol("goi");
            let wm = new WeakMap();
        "#,
        ),
        TestAction::assert_eq("wm.getOrInsert(sym, 99)", 99),
        TestAction::assert("wm.has(sym)"),
        TestAction::assert_eq("wm.getOrInsert(sym, 200)", 99), // existing value returned
    ]);
}

#[test]
fn symbol_key_get_or_insert_computed() {
    run_test_actions([
        TestAction::run(
            r#"
            let sym = Symbol("goic");
            let wm = new WeakMap();
            let calls = 0;
        "#,
        ),
        TestAction::assert_eq(
            "wm.getOrInsertComputed(sym, (k) => { calls++; return 77; })",
            77,
        ),
        TestAction::assert_eq("calls", 1),
        TestAction::assert_eq(
            "wm.getOrInsertComputed(sym, (k) => { calls++; return 88; })",
            77,
        ),
        TestAction::assert_eq("calls", 1), // callback not called on hit
    ]);
}

#[test]
fn primitives_still_rejected() {
    run_test_actions([
        TestAction::assert_native_error(
            "new WeakMap().set(42, 'v')",
            JsNativeErrorKind::Type,
            "WeakMap.set: invalid key type `number`: cannot be held weakly",
        ),
        TestAction::assert_native_error(
            "new WeakMap().set('str', 'v')",
            JsNativeErrorKind::Type,
            "WeakMap.set: invalid key type `string`: cannot be held weakly",
        ),
        TestAction::assert_native_error(
            "new WeakMap().set(true, 'v')",
            JsNativeErrorKind::Type,
            "WeakMap.set: invalid key type `boolean`: cannot be held weakly",
        ),
    ]);
}
