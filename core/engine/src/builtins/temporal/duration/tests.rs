use crate::{run_test_actions, TestAction};

#[test]
fn duration_constructor() {
    run_test_actions([
        TestAction::run("let dur = new Temporal.Duration(1, 1, 0, 1)"),
        TestAction::assert_eq("dur.years", 1),
        TestAction::assert_eq("dur.months", 1),
        TestAction::assert_eq("dur.weeks", 0),
        TestAction::assert_eq("dur.days", 1),
        TestAction::assert_eq("dur.milliseconds", 0),
    ]);
}

#[test]
fn duration_abs() {
    run_test_actions([
        TestAction::run("let dur = new Temporal.Duration(-1, -1, 0, -1)"),
        TestAction::assert_eq("dur.sign", -1),
        TestAction::run("let abs = dur.abs()"),
        TestAction::assert_eq("abs.years", 1),
        TestAction::assert_eq("abs.months", 1),
        TestAction::assert_eq("abs.weeks", 0),
        TestAction::assert_eq("abs.days", 1),
        TestAction::assert_eq("abs.milliseconds", 0),
    ]);
}
