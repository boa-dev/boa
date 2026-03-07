use crate::{Context, JsNativeErrorKind, JsValue, Source, TestAction, run_test_actions};
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
        "WeakMap.getOrInsert: expected target argument of type `object`, got target of type `string`",
    )]);
}

#[test]
fn get_or_insert_computed_requires_object_key() {
    run_test_actions([TestAction::assert_native_error(
        "new WeakMap().getOrInsertComputed('x', () => 1)",
        JsNativeErrorKind::Type,
        "WeakMap.getOrInsertComputed: expected target argument of type `object`, got target of type `string`",
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
fn weakmap_set_and_get() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            const obj = {};
            wm.set(obj, 42);
            wm.get(obj)
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(42));
}

#[test]
fn weakmap_overwrite_value() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            const obj = {};
            wm.set(obj, 1);
            wm.set(obj, 2);
            wm.get(obj)
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(2));
}

#[test]
fn weakmap_has() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            const obj = {};
            wm.set(obj, 10);
            wm.has(obj)
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(true));
}

#[test]
fn weakmap_delete() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            const obj = {};
            wm.set(obj, 1);
            wm.delete(obj)
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(true));
}

#[test]
fn weakmap_delete_twice() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            const obj = {};
            wm.set(obj, 1);
            wm.delete(obj);
            wm.delete(obj)
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(false));
}

#[test]
fn weakmap_get_missing_key() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            wm.get({})
        "#,
        ))
        .unwrap();
    assert!(result.is_undefined());
}

#[test]
fn weakmap_multiple_keys() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            const a = {};
            const b = {};
            wm.set(a, 1);
            wm.set(b, 2);
            wm.get(a) + wm.get(b)
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(3));
}

#[test]
fn weakmap_set_returns_this() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            const obj = {};
            wm.set(obj, 1) === wm
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(true));
}

#[test]
fn weakmap_set_rejects_number() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            try {
                wm.set(42, "value");
                false
            } catch (e) {
                e instanceof TypeError
            }
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(true));
}

#[test]
fn weakmap_set_rejects_string() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            try {
                wm.set("string", "value");
                false
            } catch (e) {
                e instanceof TypeError
            }
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(true));
}

#[test]
fn weakmap_set_rejects_boolean() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            try {
                wm.set(true, "value");
                false
            } catch (e) {
                e instanceof TypeError
            }
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(true));
}

#[test]
fn weakmap_set_rejects_null() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            try {
                wm.set(null, "value");
                false
            } catch (e) {
                e instanceof TypeError
            }
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(true));
}

#[test]
fn weakmap_set_rejects_undefined() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            try {
                wm.set(undefined, "value");
                false
            } catch (e) {
                e instanceof TypeError
            }
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(true));
}

#[test]
fn weakmap_set_rejects_symbol() {
    let mut context = Context::default();
    let result = context
        .eval(Source::from_bytes(
            r#"
            const wm = new WeakMap();
            try {
                wm.set(Symbol("sim"), "value");
                false
            } catch (e) {
                e instanceof TypeError
            }
        "#,
        ))
        .unwrap();
    assert_eq!(result, JsValue::new(true));
}
