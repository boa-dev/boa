use indoc::indoc;

use crate::{TestAction, run_test_actions};

// Intl.DateTimeFormat tests

#[test]
fn icu4x_test() {
    use icu_datetime::DateTimeFormatter;
    use icu_datetime::input::{Date, DateTime, Time};
    use icu_locale::locale;

    let field_set_with_options = icu_datetime::fieldsets::YMD::medium().with_time_hm();
    let locale = locale!("en-US");
    let dtf = DateTimeFormatter::try_new_with_buffer_provider(
        &boa_icu_provider::buffer(),
        locale.into(),
        field_set_with_options,
    )
    .unwrap();

    let datetime = DateTime {
        date: Date::try_new_iso(2025, 1, 15).unwrap(),
        time: Time::try_new(16, 9, 35, 0).unwrap(),
    };
    let formatted_date = dtf.format(&datetime);

    assert_eq!(formatted_date.to_string(), "Jan 15, 2025, 4:09\u{202f}PM");
}

#[cfg(feature = "intl_bundled")]
#[test]
fn dtf_basic() {
    run_test_actions([
        TestAction::run(indoc! {"
            let dtf = new Intl.DateTimeFormat('en-US');
            let result = dtf.format();
        "}),
        TestAction::assert_eq("result === 'TODO: implement formatting.'", true),
    ]);
}
