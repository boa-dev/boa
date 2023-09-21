use crate::{object::JsObject, run_test_actions, JsNativeErrorKind, JsValue, TestAction};
use indoc::indoc;

#[test]
fn calendar_constructor() {
    run_test_actions([
        TestAction::run("let iso = new Temporal.Calendar('iso8601');"),
        TestAction::assert_eq("iso.id", "iso8601"),
    ]);
}

#[test]
fn calendar_methods() {
    run_test_actions([
        TestAction::run("let iso = new Temporal.Calendar('iso8601');"),
        TestAction::assert_eq("iso.inLeapYear('2020-11-20')", true),
        TestAction::assert_eq("iso.daysInYear('2020-11-20')", 366),
        TestAction::assert_eq("iso.daysInYear('2021-11-20')", 365),
        TestAction::assert_eq("iso.monthsInYear('2021-11-20')", 12),
        TestAction::assert_eq("iso.daysInWeek('2021-11-20')", 7),
    ]);
}
