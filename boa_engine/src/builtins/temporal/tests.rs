use super::date_equations::epoch_time_to_month_in_year;
use crate::{js_string, run_test_actions, JsValue, TestAction};

// Temporal Object tests.

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

// Date Equations

#[test]
fn time_to_month() {
    let milliseconds = [1696459917000_f64];

    for test_epochs in milliseconds {
        println!("{}", epoch_time_to_month_in_year(test_epochs));
        assert_eq!(epoch_time_to_month_in_year(test_epochs), 9);
    }
}
