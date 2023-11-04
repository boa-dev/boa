use crate::{js_string, run_test_actions, TestAction};

#[test]
fn calendar_constructor() {
    // TODO: Add other BuiltinCalendars
    run_test_actions([TestAction::assert_eq(
        "new Temporal.Calendar('iso8601').id",
        js_string!("iso8601"),
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

#[test]
fn run_custom_calendar() {
    run_test_actions([
        TestAction::run(
            r#"const custom = {
            dateAdd() {},
            dateFromFields() {},
            dateUntil() {},
            day() {},
            dayOfWeek() {},
            dayOfYear() {},
            daysInMonth() { return 14 },
            daysInWeek() {return 6},
            daysInYear() {return 360},
            fields() {},
            id: "custom-calendar",
            inLeapYear() {},
            mergeFields() {},
            month() {},
            monthCode() {},
            monthDayFromFields() {},
            monthsInYear() {},
            weekOfYear() {},
            year() {},
            yearMonthFromFields() {},
            yearOfWeek() {},
          };

          let cal = Temporal.Calendar.from(custom);
          let date = "1972-05-01";
        "#,
        ),
        TestAction::assert_eq("cal.id", js_string!("custom-calendar")),
        TestAction::assert_eq("cal.daysInMonth(date)", 14),
        TestAction::assert_eq("cal.daysInWeek(date)", 6),
        TestAction::assert_eq("cal.daysInYear(date)", 360),
    ])
}
