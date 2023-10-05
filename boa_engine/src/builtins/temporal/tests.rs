use super::date_equations::{epoch_time_to_month_in_year, mathematical_in_leap_year};
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
    let oct_2023 = 1_696_459_917_000_f64;
    let mar_1_2020 = 1_583_020_800_000_f64;
    let feb_29_2020 = 1_582_934_400_000_f64;
    let mar_1_2021 = 1_614_556_800_000_f64;

    assert_eq!(epoch_time_to_month_in_year(oct_2023), 9);
    assert_eq!(epoch_time_to_month_in_year(mar_1_2020), 2);
    assert_eq!(mathematical_in_leap_year(mar_1_2020), 1);
    assert_eq!(epoch_time_to_month_in_year(feb_29_2020), 1);
    assert_eq!(mathematical_in_leap_year(feb_29_2020), 1);
    assert_eq!(epoch_time_to_month_in_year(mar_1_2021), 2);
    assert_eq!(mathematical_in_leap_year(mar_1_2021), 0);
}
