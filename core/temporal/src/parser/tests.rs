use std::str::FromStr;

use crate::{
    components::{DateTime, Duration},
    parser::{
        parse_date_time, Cursor, TemporalInstantString, TemporalMonthDayString,
        TemporalYearMonthString,
    },
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
    let possible_year_months = &[
        "+002020-11",
        "2020-11[u-ca=iso8601]",
        "+00202011",
        "202011[u-ca=iso8601]",
    ];

    for ym in possible_year_months {
        let result = TemporalYearMonthString::parse(&mut Cursor::new(ym)).unwrap();

        assert_eq!(result.year, 2020);
        assert_eq!(result.month, 11);

        if let Some(calendar) = result.calendar {
            assert_eq!(calendar, "iso8601");
        }
    }
}

#[test]
fn temporal_month_day() {
    let possible_month_day = ["11-07", "1107[+04:00]", "--11-07", "--1107[+04:00]"];

    for md in possible_month_day {
        let result = TemporalMonthDayString::parse(&mut Cursor::new(md)).unwrap();

        assert_eq!(result.month, 11);
        assert_eq!(result.day, 7);
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
        let err_result = TemporalMonthDayString::parse(&mut Cursor::new(invalid));
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
