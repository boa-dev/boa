use std::str::FromStr;

use crate::{
    components::{DateTime, Duration, MonthDay, YearMonth},
    parser::{parse_date_time, Cursor, TemporalInstantString},
};

#[test]
fn temporal_parser_basic() {
    let basic = "20201108";
    let basic_separated = "2020-11-08";

    let basic_result = basic.parse::<DateTime>().unwrap();

    let sep_result = basic_separated.parse::<DateTime>().unwrap();

    assert_eq!(basic_result.iso_date().year(), 2020);
    assert_eq!(basic_result.iso_date().month(), 11);
    assert_eq!(basic_result.iso_date().day(), 8);
    assert_eq!(basic_result.iso_date().year(), sep_result.iso_date().year());
    assert_eq!(
        basic_result.iso_date().month(),
        sep_result.iso_date().month()
    );
    assert_eq!(basic_result.iso_date().day(), sep_result.iso_date().day());
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn temporal_date_time_max() {
    // Fractions not accurate, but for testing purposes.
    let date_time =
        "+002020-11-08T12:28:32.329402834[!America/Argentina/ComodRivadavia][!u-ca=iso8601]";

    let result = date_time.parse::<DateTime>().unwrap();

    let time_results = result.iso_time();

    assert_eq!(time_results.hour, 12);
    assert_eq!(time_results.minute, 28);
    assert_eq!(time_results.second, 32);
    assert_eq!(time_results.millisecond, 329);
    assert_eq!(time_results.microsecond, 402);
    assert_eq!(time_results.nanosecond, 834);
}

#[test]
fn temporal_year_parsing() {
    let long = "+002020-11-08";
    let bad_year = "-000000-11-08";

    let result_good = long.parse::<DateTime>().unwrap();
    assert_eq!(result_good.iso_date().year(), 2020);

    let err_result = bad_year.parse::<DateTime>();
    assert!(err_result.is_err());
}

#[test]
fn temporal_annotated_date_time() {
    let basic = "2020-11-08[America/Argentina/ComodRivadavia][u-ca=iso8601][foo=bar]";
    let omitted = "+0020201108[u-ca=iso8601][f-1a2b=a0sa-2l4s]";

    let result = parse_date_time(basic).unwrap();

    let tz = &result.tz.unwrap().name.unwrap();

    assert_eq!(tz, "America/Argentina/ComodRivadavia");

    assert_eq!(&result.calendar, &Some("iso8601".to_string()));

    let omit_result = parse_date_time(omitted).unwrap();

    assert!(&omit_result.tz.is_none());

    assert_eq!(&omit_result.calendar, &Some("iso8601".to_string()));
}

#[test]
fn temporal_year_month() {
    let possible_year_months = [
        "+002020-11",
        "2020-11[u-ca=iso8601]",
        "+00202011",
        "202011[u-ca=iso8601]",
    ];

    for ym in possible_year_months {
        let result = ym.parse::<YearMonth>().unwrap();

        assert_eq!(result.year(), 2020);
        assert_eq!(result.month(), 11);
    }
}

#[test]
fn temporal_month_day() {
    let possible_month_day = ["11-07", "1107[+04:00]", "--11-07", "--1107[+04:00]"];

    for md in possible_month_day {
        let result = md.parse::<MonthDay>().unwrap();

        assert_eq!(result.month(), 11);
        assert_eq!(result.day(), 7);
    }
}

#[test]
fn temporal_invalid_annotations() {
    let invalid_annotations = [
        "2020-11-11[!u-ca=iso8601][u-ca=iso8601]",
        "2020-11-11[u-ca=iso8601][!u-ca=iso8601]",
        "2020-11-11[u-ca=iso8601][!rip=this-invalid-annotation]",
    ];

    for invalid in invalid_annotations {
        let err_result = invalid.parse::<MonthDay>();
        assert!(err_result.is_err());
    }
}

#[test]
fn temporal_valid_instant_strings() {
    let instants = [
        "1970-01-01T00:00+00:00[!Africa/Abidjan]",
        "1970-01-01T00:00+00:00[UTC]",
        "1970-01-01T00:00Z[!Europe/Vienna]",
    ];

    for test in instants {
        let result = TemporalInstantString::parse(&mut Cursor::new(test));
        assert!(result.is_ok());
    }
}

#[test]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::float_cmp)]
fn temporal_duration_parsing() {
    let durations = [
        "p1y1m1dt1h1m1s",
        "P1Y1M1W1DT1H1M1.1S",
        "-P1Y1M1W1DT1H1M1.123456789S",
        "-P1Y3wT0,5H",
    ];

    for dur in durations {
        let ok_result = Duration::from_str(dur);
        assert!(ok_result.is_ok());
    }

    let sub_second = durations[2].parse::<Duration>().unwrap();

    assert_eq!(sub_second.time().milliseconds(), -123.0);
    assert_eq!(sub_second.time().microseconds(), -456.0);
    assert_eq!(sub_second.time().nanoseconds(), -789.0);

    let test_result = durations[3].parse::<Duration>().unwrap();

    assert_eq!(test_result.date().years(), -1f64);
    assert_eq!(test_result.date().weeks(), -3f64);
    assert_eq!(test_result.time().minutes(), -30.0);
}

#[test]
fn temporal_invalid_durations() {
    let invalids = [
        "P1Y1M1W0,5D",
        "P1Y1M1W1DT1H1M1.123456789123S",
        "+PT",
        "P1Y1M1W1DT1H0.5M0.5S",
    ];

    for test in invalids {
        let err = test.parse::<Duration>();
        assert!(err.is_err());
    }
}

#[test]
fn temporal_invalid_iso_datetime_strings() {
    // NOTE: The below tests were initially pulled from test262's `argument-string-invalid`
    const INVALID_DATETIME_STRINGS: [&str; 34] = [
        "", // 1
        "invalid iso8601",
        "2020-01-00",
        "2020-01-32",
        "2020-02-30",
        "2021-02-29",
        "2020-00-01",
        "2020-13-01",
        "2020-01-01T",
        "2020-01-01T25:00:00",
        "2020-01-01T01:60:00",
        "2020-01-01T01:60:61",
        "2020-01-01junk",
        "2020-01-01T00:00:00junk",
        "2020-01-01T00:00:00+00:00junk",
        "2020-01-01T00:00:00+00:00[UTC]junk",
        "2020-01-01T00:00:00+00:00[UTC][u-ca=iso8601]junk",
        "02020-01-01",
        "2020-001-01",
        "2020-01-001",
        "2020-01-01T001",
        "2020-01-01T01:001",
        "2020-01-01T01:01:001",
        "2020-W01-1",
        "2020-001",
        "+0002020-01-01",
        // TODO: Add the non-existent calendar test back to the test cases.
        // may be valid in other contexts, but insufficient information for PlainDate:
        "2020-01",
        "+002020-01",
        "01-01",
        "2020-W01",
        "P1Y",
        "-P12Y",
        // valid, but outside the supported range:
        "-999999-01-01",
        "+999999-01-01",
    ];

    for invalid_target in INVALID_DATETIME_STRINGS {
        let error_result = invalid_target.parse::<DateTime>();
        assert!(error_result.is_err())
    }
}
