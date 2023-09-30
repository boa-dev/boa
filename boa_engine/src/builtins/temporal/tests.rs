use crate::{js_string, run_test_actions, JsValue, TestAction};

#[test]
fn temporal_object() {
    // Temporal Builtin tests.
    run_test_actions([
        TestAction::assert_eq(
            "Object.prototype.toString.call(Temporal)",
            js_string!("[object Temporal]"),
        ),
        TestAction::assert_eq("String(Temporal)", js_string!("[object Temporal]")),
        TestAction::assert_eq("Object.keys(Temporal).length === 0", true),
    ]);
}

#[test]
fn now_object() {
    // Now Builtin tests.
    run_test_actions([
        TestAction::assert_eq("Object.isExtensible(Temporal.Now)", true),
        TestAction::assert_eq(
            "Object.prototype.toString.call(Temporal.Now)",
            js_string!("[object Temporal.Now]"),
        ),
        TestAction::assert_eq(
            "Object.getPrototypeOf(Temporal.Now) === Object.prototype",
            true,
        ),
        TestAction::assert_eq("Temporal.Now.prototype", JsValue::undefined()),
    ]);
}
