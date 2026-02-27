use crate::{JsNativeErrorKind, TestAction, run_test_actions};
use boa_macros::js_str;

// ── Symbol-as-key tests ───────────────────────────────────────────────────────

#[test]
fn symbol_key_get_missing_returns_undefined() {
    run_test_actions([
        TestAction::run("const sym = Symbol('missing'); const wm = new WeakMap();"),
        TestAction::assert("wm.get(sym) === undefined"),
    ]);
}

#[test]
fn symbol_key_get_or_insert_computed_race() {
    run_test_actions([
        TestAction::run(
            r#"
            const wm = new WeakMap()
            const sym = Symbol('race');
            const v = wm.getOrInsertComputed(sym, function() {
                wm.set(sym, 'other');
                return 'computed';
            });
        "#,
        ),
        // computed value wins over the one set inside the callback
        TestAction::assert_eq("v", js_str!("computed")),
        TestAction::assert_eq("wm.get(sym)", js_str!("computed")),
    ]);
}

#[test]
fn symbol_key_set_get_has() {
    run_test_actions([
        TestAction::run("const sym = Symbol('test'); const wm = new WeakMap(); wm.set(sym, 42);"),
        TestAction::assert("wm.has(sym)"),
        TestAction::assert_eq("wm.get(sym)", 42),
    ]);
}

#[test]
fn symbol_key_delete() {
    run_test_actions([
        TestAction::run("const sym = Symbol(); const wm = new WeakMap([[sym, 'hi']]);"),
        TestAction::assert("wm.has(sym)"),
        TestAction::assert("wm.delete(sym)"),
        TestAction::assert("!wm.has(sym)"),
    ]);
}

#[test]
fn registered_symbol_rejected_by_set() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().set(Symbol.for('reg'), 1)",
        JsNativeErrorKind::Type,
        "WeakMap.set: expected target argument of type `object` or non-registered `symbol`, got target of type `symbol`",
    )]);
}

#[test]
fn registered_symbol_returns_false_from_has() {
    run_test_actions([TestAction::assert_eq(
        "new WeakMap().has(Symbol.for('reg'))",
        false,
    )]);
}

#[test]
fn well_known_symbol_allowed_as_key() {
    run_test_actions([
        TestAction::run("const wm = new WeakMap(); wm.set(Symbol.iterator, 42);"),
        TestAction::assert("wm.has(Symbol.iterator)"),
        TestAction::assert_eq("wm.get(Symbol.iterator)", 42),
        TestAction::assert("wm.delete(Symbol.iterator)"),
        TestAction::assert("!wm.has(Symbol.iterator)"),
    ]);
}

#[test]
fn symbol_key_get_or_insert() {
    run_test_actions([
        TestAction::run("const sym = Symbol('x'); const wm = new WeakMap();"),
        TestAction::assert_eq("wm.getOrInsert(sym, 99)", 99),
        TestAction::assert("wm.has(sym)"),
        TestAction::assert_eq("wm.getOrInsert(sym, 0)", 99), // returns existing
    ]);
}

#[test]
fn symbol_key_get_or_insert_computed() {
    run_test_actions([
        TestAction::run("const sym = Symbol(); const wm = new WeakMap(); let called = 0;"),
        TestAction::assert_eq(
            "wm.getOrInsertComputed(sym, () => { called++; return 'v'; })",
            js_str!("v"),
        ),
        TestAction::assert_eq("called", 1),
        // Second call should NOT invoke callback
        TestAction::assert_eq(
            "wm.getOrInsertComputed(sym, () => { called++; return 'other'; })",
            js_str!("v"),
        ),
        TestAction::assert_eq("called", 1),
    ]);
}

#[test]
fn symbol_and_object_keys_coexist() {
    run_test_actions([
        TestAction::run(
            "const sym = Symbol(); const obj = {}; const wm = new WeakMap();
             wm.set(sym, 'sym-val'); wm.set(obj, 'obj-val');",
        ),
        TestAction::assert_eq("wm.get(sym)", js_str!("sym-val")),
        TestAction::assert_eq("wm.get(obj)", js_str!("obj-val")),
        TestAction::assert("wm.has(sym)"),
        TestAction::assert("wm.has(obj)"),
    ]);
}

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
        "WeakMap.getOrInsert: expected target argument of type `object` or non-registered `symbol`, got target of type `string`",
    )]);
}

#[test]
fn get_or_insert_computed_requires_object_key() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().getOrInsertComputed('x', () => 1)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsertComputed: expected target argument of type `object` or non-registered `symbol`, got target of type `string`",
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
