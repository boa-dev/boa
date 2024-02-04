use crate::{run_test_actions, TestAction};

#[test]
fn pdt_year_of_week_basic() {
    run_test_actions([
        TestAction::run("let calendar = Temporal.Calendar.from('iso8601')"),
        TestAction::run("let pdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789, calendar)"),
        TestAction::assert_eq("pdt.yearOfWeek", 1976),
    ]);
}
