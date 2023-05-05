use crate::{builtins::error::ErrorKind, run_test_actions, TestAction};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use indoc::indoc;

// NOTE: Javascript Uses 0-based months, where chrono uses 1-based months. Many of the assertions look wrong because of
// this.

fn timestamp_from_local(
    year: i32,
    month: u32,
    date: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
) -> i64 {
    Local
        .from_local_datetime(
            &NaiveDate::from_ymd_opt(year, month, date)
                .unwrap()
                .and_hms_milli_opt(hour, minute, second, millisecond)
                .unwrap(),
        )
        .earliest()
        .unwrap()
        .naive_utc()
        .timestamp_millis()
}

fn timestamp_from_utc(
    year: i32,
    month: u32,
    date: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
) -> i64 {
    NaiveDate::from_ymd_opt(year, month, date)
        .unwrap()
        .and_hms_milli_opt(hour, minute, second, millisecond)
        .unwrap()
        .timestamp_millis()
}

#[test]
fn date_this_time_value() {
    run_test_actions([TestAction::assert_native_error(
        "({toString: Date.prototype.toString}).toString()",
        ErrorKind::Type,
        "'this' is not a Date",
    )]);
}

#[test]
fn date_ctor_call() {
    run_test_actions([
        TestAction::run("let a = new Date()"),
        TestAction::inspect_context(|_| std::thread::sleep(std::time::Duration::from_millis(1))),
        TestAction::assert("a.getTime() != new Date().getTime()"),
    ]);
}

#[test]
fn date_ctor_call_string() {
    run_test_actions([TestAction::assert_eq(
        "new Date('2020-06-08T09:16:15.779-06:30').getTime()",
        timestamp_from_utc(2020, 6, 8, 15, 46, 15, 779),
    )]);
}

#[test]
fn date_ctor_call_string_invalid() {
    run_test_actions([TestAction::assert_eq(
        "new Date('nope').getTime()",
        f64::NAN,
    )]);
}

#[test]
fn date_ctor_call_number() {
    run_test_actions([TestAction::assert_eq(
        "new Date(1594199775779).getTime()",
        timestamp_from_utc(2020, 7, 8, 9, 16, 15, 779),
    )]);
}

#[test]
fn date_ctor_call_date() {
    run_test_actions([TestAction::assert_eq(
        "new Date(new Date(1594199775779)).getTime()",
        timestamp_from_utc(2020, 7, 8, 9, 16, 15, 779),
    )]);
}

#[test]
fn date_ctor_call_multiple() {
    run_test_actions([TestAction::assert_eq(
        "new Date(2020, 6, 8, 9, 16, 15, 779).getTime()",
        timestamp_from_local(2020, 7, 8, 9, 16, 15, 779),
    )]);
}

#[test]
fn date_ctor_call_multiple_90s() {
    run_test_actions([TestAction::assert_eq(
        "new Date(99, 6, 8, 9, 16, 15, 779).getTime()",
        timestamp_from_local(1999, 7, 8, 9, 16, 15, 779),
    )]);
}

#[test]
fn date_ctor_call_multiple_nan() {
    run_test_actions([
        TestAction::assert_eq("new Date(1/0, 6, 8, 9, 16, 15, 779).getTime()", f64::NAN),
        TestAction::assert_eq("new Date(2020, 1/0, 8, 9, 16, 15, 779).getTime()", f64::NAN),
        TestAction::assert_eq("new Date(2020, 6, 1/0, 9, 16, 15, 779).getTime()", f64::NAN),
        TestAction::assert_eq("new Date(2020, 6, 8, 1/0, 16, 15, 779).getTime()", f64::NAN),
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 1/0, 15, 779).getTime()", f64::NAN),
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 1/0, 779).getTime()", f64::NAN),
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 15, 1/0).getTime()", f64::NAN),
    ]);
}

#[test]
fn date_ctor_now_call() {
    run_test_actions([
        TestAction::run("let a = Date.now()"),
        TestAction::inspect_context(|_| std::thread::sleep(std::time::Duration::from_millis(1))),
        TestAction::assert("a != Date.now()"),
    ]);
}

#[test]
fn date_ctor_parse_call() {
    run_test_actions([TestAction::assert_eq(
        "Date.parse('2020-06-08T09:16:15.779-07:30')",
        1_591_634_775_779_i64,
    )]);
}

#[test]
fn date_ctor_utc_call() {
    run_test_actions([TestAction::assert_eq(
        "Date.UTC(2020, 6, 8, 9, 16, 15, 779)",
        1_594_199_775_779_i64,
    )]);
}

#[test]
fn date_ctor_utc_call_nan() {
    run_test_actions([
        TestAction::assert_eq("Date.UTC(1/0, 6, 8, 9, 16, 15, 779)", f64::NAN),
        TestAction::assert_eq("Date.UTC(2020, 1/0, 8, 9, 16, 15, 779)", f64::NAN),
        TestAction::assert_eq("Date.UTC(2020, 6, 1/0, 9, 16, 15, 779)", f64::NAN),
        TestAction::assert_eq("Date.UTC(2020, 6, 8, 1/0, 16, 15, 779)", f64::NAN),
        TestAction::assert_eq("Date.UTC(2020, 6, 8, 9, 1/0, 15, 779)", f64::NAN),
        TestAction::assert_eq("Date.UTC(2020, 6, 8, 9, 16, 1/0, 779)", f64::NAN),
        TestAction::assert_eq("Date.UTC(2020, 6, 8, 9, 16, 15, 1/0)", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_date_call() {
    run_test_actions([
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 15, 779).getDate()", 8),
        TestAction::assert_eq("new Date(1/0).getDate()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_day_call() {
    run_test_actions([
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 15, 779).getDay()", 3),
        TestAction::assert_eq("new Date(1/0).getDay()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_full_year_call() {
    run_test_actions([
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 15, 779).getFullYear()", 2020),
        TestAction::assert_eq("new Date(1/0).getFullYear()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_hours_call() {
    run_test_actions([
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 15, 779).getHours()", 9),
        TestAction::assert_eq("new Date(1/0).getHours()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_milliseconds_call() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).getMilliseconds()",
            779,
        ),
        TestAction::assert_eq("new Date(1/0).getMilliseconds()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_minutes_call() {
    run_test_actions([
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 15, 779).getMinutes()", 16),
        TestAction::assert_eq("new Date(1/0).getMinutes()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_month() {
    run_test_actions([
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 15, 779).getMonth()", 6),
        TestAction::assert_eq("new Date(1/0).getMonth()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_seconds() {
    run_test_actions([
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 15, 779).getSeconds()", 15),
        TestAction::assert_eq("new Date(1/0).getSeconds()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_time() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).getTime()",
            timestamp_from_local(2020, 7, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq("new Date(1/0).getTime()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_year() {
    run_test_actions([
        TestAction::assert_eq("new Date(2020, 6, 8, 9, 16, 15, 779).getYear()", 120),
        TestAction::assert_eq("new Date(1/0).getYear()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_timezone_offset() {
    run_test_actions([
        TestAction::assert(indoc! {r#"
                new Date('1975-08-19T23:15:30+07:00').getTimezoneOffset() ===
                new Date('1975-08-19T23:15:30-02:00').getTimezoneOffset()
            "#}),
        // NB: Host Settings, not TZ specified in the DateTime.
        TestAction::assert_eq(
            "new Date('1975-08-19T23:15:30+07:00').getTimezoneOffset()",
            {
                // The value of now().offset() depends on the host machine, so we have to replicate the method code here.
                let dt = Local
                    .from_local_datetime(
                        &NaiveDate::from_ymd_opt(1975, 8, 19)
                            .unwrap()
                            .and_hms_opt(23, 15, 30)
                            .unwrap(),
                    )
                    .earliest()
                    .unwrap();
                let offset_seconds = dt.offset().local_minus_utc();
                -offset_seconds / 60
            },
        ),
    ]);
}

#[test]
fn date_proto_get_utc_date_call() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCDate()",
            8,
        ),
        TestAction::assert_eq("new Date(1/0).getUTCDate()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_utc_day_call() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCDay()",
            3,
        ),
        TestAction::assert_eq("new Date(1/0).getUTCDay()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_utc_full_year_call() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCFullYear()",
            2020,
        ),
        TestAction::assert_eq("new Date(1/0).getUTCFullYear()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_utc_hours_call() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCHours()",
            9,
        ),
        TestAction::assert_eq("new Date(1/0).getUTCHours()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_utc_milliseconds_call() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCMilliseconds()",
            779,
        ),
        TestAction::assert_eq("new Date(1/0).getUTCMilliseconds()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_utc_minutes_call() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCMinutes()",
            16,
        ),
        TestAction::assert_eq("new Date(1/0).getUTCMinutes()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_utc_month() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCMonth()",
            6,
        ),
        TestAction::assert_eq("new Date(1/0).getUTCMonth()", f64::NAN),
    ]);
}

#[test]
fn date_proto_get_utc_seconds() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCSeconds()",
            15,
        ),
        TestAction::assert_eq("new Date(1/0).getUTCSeconds()", f64::NAN),
    ]);
}

#[test]
fn date_proto_set_date() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setDate(21)",
            timestamp_from_local(2020, 7, 21, 9, 16, 15, 779),
        ),
        // Date wraps to previous month for 0.
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setDate(0)",
            timestamp_from_local(2020, 6, 30, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setDate(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_full_year() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setFullYear(2012)",
            timestamp_from_local(2012, 7, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setFullYear(2012, 8)",
            timestamp_from_local(2012, 9, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setFullYear(2012, 8, 10)",
            timestamp_from_local(2012, 9, 10, 9, 16, 15, 779),
        ),
        // Out-of-bounds
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setFullYear(2012, 35)",
            timestamp_from_local(2014, 12, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setFullYear(2012, -35)",
            timestamp_from_local(2009, 2, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setFullYear(2012, 9, 950)",
            timestamp_from_local(2015, 5, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setFullYear(2012, 9, -950)",
            timestamp_from_local(2010, 2, 23, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setFullYear(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_hours() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setHours(11)",
            timestamp_from_local(2020, 7, 8, 11, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setHours(11, 35)",
            timestamp_from_local(2020, 7, 8, 11, 35, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setHours(11, 35, 23)",
            timestamp_from_local(2020, 7, 8, 11, 35, 23, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setHours(11, 35, 23, 537)",
            timestamp_from_local(2020, 7, 8, 11, 35, 23, 537),
        ),
        // Out-of-bounds
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setHours(10000, 20000, 30000, 40123)",
            timestamp_from_local(2021, 9, 11, 21, 40, 40, 123),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setHours(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_milliseconds() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMilliseconds(597)",
            timestamp_from_local(2020, 7, 8, 9, 16, 15, 597),
        ),
        // Out-of-bounds
        // Thorough tests are done by setHours
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMilliseconds(40123)",
            timestamp_from_local(2020, 7, 8, 9, 16, 55, 123),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMilliseconds(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_minutes() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMinutes(11)",
            timestamp_from_local(2020, 7, 8, 9, 11, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMinutes(11, 35)",
            timestamp_from_local(2020, 7, 8, 9, 11, 35, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMinutes(11, 35, 537)",
            timestamp_from_local(2020, 7, 8, 9, 11, 35, 537),
        ),
        // Out-of-bounds
        // Thorough tests are done by setHours
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMinutes(600000, 30000, 40123)",
            timestamp_from_local(2021, 8, 29, 9, 20, 40, 123),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMinutes(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_month() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMonth(11)",
            timestamp_from_local(2020, 12, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMonth(11, 16)",
            timestamp_from_local(2020, 12, 16, 9, 16, 15, 779),
        ),
        // Out-of-bounds
        // Thorough tests are done by setFullYear
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMonth(40, 83)",
            timestamp_from_local(2023, 7, 22, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setMonth(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_seconds() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setSeconds(11)",
            timestamp_from_local(2020, 7, 8, 9, 16, 11, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setSeconds(11, 487)",
            timestamp_from_local(2020, 7, 8, 9, 16, 11, 487),
        ),
        // Out-of-bounds
        // Thorough tests are done by setHour
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setSeconds(40000000, 40123)",
            timestamp_from_local(2021, 10, 14, 8, 23, 20, 123),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setSeconds(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn set_year() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setYear(98)",
            timestamp_from_local(1998, 7, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setYear(2001)",
            timestamp_from_local(2001, 7, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setYear(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_time() {
    run_test_actions([TestAction::assert_eq(
        "new Date().setTime(new Date(2020, 6, 8, 9, 16, 15, 779).getTime())",
        timestamp_from_local(2020, 7, 8, 9, 16, 15, 779),
    )]);
}

#[test]
fn date_proto_set_utc_date() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCDate(21)",
            timestamp_from_utc(2020, 7, 21, 9, 16, 15, 779),
        ),
        // Date wraps to previous month for 0.
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCDate(0)",
            timestamp_from_utc(2020, 6, 30, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCDate(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_utc_full_year() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCFullYear(2012)",
            timestamp_from_utc(2012, 7, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCFullYear(2012, 8)",
            timestamp_from_utc(2012, 9, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCFullYear(2012, 8, 10)",
            timestamp_from_utc(2012, 9, 10, 9, 16, 15, 779),
        ),
        // Out-of-bounds
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCFullYear(2012, 35)",
            timestamp_from_utc(2014, 12, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCFullYear(2012, -35)",
            timestamp_from_utc(2009, 2, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCFullYear(2012, 9, 950)",
            timestamp_from_utc(2015, 5, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCFullYear(2012, 9, -950)",
            timestamp_from_utc(2010, 2, 23, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCFullYear(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_utc_hours() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setUTCHours(11)",
            timestamp_from_utc(2020, 7, 8, 11, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setUTCHours(11, 35)",
            timestamp_from_utc(2020, 7, 8, 11, 35, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setUTCHours(11, 35, 23)",
            timestamp_from_utc(2020, 7, 8, 11, 35, 23, 779),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setUTCHours(11, 35, 23, 537)",
            timestamp_from_utc(2020, 7, 8, 11, 35, 23, 537),
        ),
        // Out-of-bounds
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setUTCHours(10000, 20000, 30000, 40123)",
            timestamp_from_utc(2021, 9, 11, 21, 40, 40, 123),
        ),
        TestAction::assert_eq(
            "new Date(2020, 6, 8, 9, 16, 15, 779).setUTCHours(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_utc_milliseconds() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMilliseconds(597)",
            timestamp_from_utc(2020, 7, 8, 9, 16, 15, 597),
        ),
        // Out-of-bounds
        // Thorough tests are done by setHours
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMilliseconds(40123)",
            timestamp_from_utc(2020, 7, 8, 9, 16, 55, 123),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMilliseconds(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_utc_minutes() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMinutes(11)",
            timestamp_from_utc(2020, 7, 8, 9, 11, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMinutes(11, 35)",
            timestamp_from_utc(2020, 7, 8, 9, 11, 35, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMinutes(11, 35, 537)",
            timestamp_from_utc(2020, 7, 8, 9, 11, 35, 537),
        ),
        // Out-of-bounds
        // Thorough tests are done by setHours
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMinutes(600000, 30000, 40123)",
            timestamp_from_utc(2021, 8, 29, 9, 20, 40, 123),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMinutes(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_utc_month() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMonth(11)",
            timestamp_from_utc(2020, 12, 8, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMonth(11, 16)",
            timestamp_from_utc(2020, 12, 16, 9, 16, 15, 779),
        ),
        // Out-of-bounds
        // Thorough tests are done by setFullYear
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMonth(40, 83)",
            timestamp_from_utc(2023, 7, 22, 9, 16, 15, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCMonth(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_set_utc_seconds() {
    run_test_actions([
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCSeconds(11)",
            timestamp_from_utc(2020, 7, 8, 9, 16, 11, 779),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCSeconds(11, 487)",
            timestamp_from_utc(2020, 7, 8, 9, 16, 11, 487),
        ),
        // Out-of-bounds
        // Thorough tests are done by setHour
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCSeconds(40000000, 40123)",
            timestamp_from_utc(2021, 10, 14, 8, 23, 20, 123),
        ),
        TestAction::assert_eq(
            "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).setUTCSeconds(1/0)",
            f64::NAN,
        ),
    ]);
}

#[test]
fn date_proto_to_date_string() {
    run_test_actions([TestAction::assert_eq(
        "new Date(2020, 6, 8, 9, 16, 15, 779).toDateString()",
        "Wed Jul 08 2020",
    )]);
}

#[test]
fn date_proto_to_gmt_string() {
    run_test_actions([TestAction::assert_eq(
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).toGMTString()",
        "Wed, 08 Jul 2020 09:16:15 GMT",
    )]);
}

#[test]
fn date_proto_to_iso_string() {
    run_test_actions([TestAction::assert_eq(
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).toISOString()",
        "2020-07-08T09:16:15.779Z",
    )]);
}

#[test]
fn date_proto_to_json() {
    run_test_actions([TestAction::assert_eq(
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).toJSON()",
        "2020-07-08T09:16:15.779Z",
    )]);
}

#[test]
fn date_proto_to_string() {
    run_test_actions([TestAction::assert_eq(
        "new Date(2020, 6, 8, 9, 16, 15, 779).toString()",
        Local
            .from_local_datetime(&NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2020, 7, 8).unwrap(),
                NaiveTime::from_hms_milli_opt(9, 16, 15, 779).unwrap(),
            ))
            .earliest()
            .unwrap()
            .format("Wed Jul 08 2020 09:16:15 GMT%z")
            .to_string(),
    )]);
}

#[test]
fn date_proto_to_time_string() {
    run_test_actions([TestAction::assert_eq(
        "new Date(2020, 6, 8, 9, 16, 15, 779).toTimeString()",
        Local
            .from_local_datetime(&NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2020, 7, 8).unwrap(),
                NaiveTime::from_hms_milli_opt(9, 16, 15, 779).unwrap(),
            ))
            .earliest()
            .unwrap()
            .format("09:16:15 GMT%z")
            .to_string(),
    )]);
}

#[test]
fn date_proto_to_utc_string() {
    run_test_actions([TestAction::assert_eq(
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).toUTCString()",
        "Wed, 08 Jul 2020 09:16:15 GMT",
    )]);
}

#[test]
fn date_proto_value_of() {
    run_test_actions([TestAction::assert_eq(
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).valueOf()",
        1_594_199_775_779_i64,
    )]);
}

#[test]
fn date_neg() {
    run_test_actions([TestAction::assert_eq(
        "-new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779))",
        -1_594_199_775_779_i64,
    )]);
}

#[test]
fn date_json() {
    run_test_actions([TestAction::assert_eq(
        "JSON.stringify({ date: new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)) })",
        r#"{"date":"2020-07-08T09:16:15.779Z"}"#,
    )]);
}
