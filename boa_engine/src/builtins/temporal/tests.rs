use crate::{object::JsObject, run_test_actions, JsNativeErrorKind, JsValue, TestAction};
use indoc::indoc;

#[test]
fn temporal_object() {
    // Temporal Builtin tests.
    run_test_actions([
        TestAction::assert_eq(
            "Object.prototype.toString.call(Temporal)",
            "[object Temporal]",
        ),
        TestAction::assert_eq("String(Temporal)", "[object Temporal]"),
        TestAction::assert_eq("Object.keys(Temporal).length === 0", true),
    ])
}

#[test]
fn now_object() {
    // Now Builtin tests.
    run_test_actions([
        TestAction::assert_eq("Object.isExtensible(Temporal.Now)", true),
        TestAction::assert_eq(
            "Object.prototype.toString.call(Temporal.Now)",
            "[object Temporal.Now]",
        ),
        TestAction::assert_eq(
            "Object.getPrototypeOf(Temporal.Now) === Object.prototype",
            true,
        ),
        TestAction::assert_eq("Temporal.Now.prototype", JsValue::undefined()),
    ])
}
