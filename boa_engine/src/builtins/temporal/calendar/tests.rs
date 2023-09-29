use crate::{run_test_actions, TestAction};
use indoc::indoc;

#[test]
fn calendar_constructor() {
    // TODO: Add other BuiltinCalendars
    run_test_actions([TestAction::assert_eq(
        "new Temporal.Calendar('iso8601').id",
        "iso8601",
    )]);
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
