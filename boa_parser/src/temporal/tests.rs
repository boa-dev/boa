use super::{
    IsoCursor, TemporalDateTimeString, TemporalDurationString, TemporalInstantString,
    TemporalMonthDayString, TemporalYearMonthString,
};

#[test]
fn temporal_parser_basic() {
    let basic = "20201108";
    let basic_separated = "2020-11-08";

    let basic_result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(basic)).unwrap();

    let sep_result =
        TemporalDateTimeString::parse(false, &mut IsoCursor::new(basic_separated)).unwrap();

    assert_eq!(basic_result.date.year, 2020);
    assert_eq!(basic_result.date.month, 11);
    assert_eq!(basic_result.date.day, 8);
    assert_eq!(basic_result.date.year, sep_result.date.year);
    assert_eq!(basic_result.date.month, sep_result.date.month);
    assert_eq!(basic_result.date.day, sep_result.date.day);
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn temporal_date_time_max() {
    // Fractions not accurate, but for testing purposes.
    let date_time = "+002020-11-08T12:28:32.329402834-03:00:00.123456789[!America/Argentina/ComodRivadavia][!u-ca=iso8601]";

    let result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(date_time)).unwrap();

    let time_results = &result.time.unwrap();

    assert_eq!(time_results.hour, 12);
    assert_eq!(time_results.minute, 28);
    assert_eq!(
        time_results.second.mul_add(f64::from(100_000), 0.0).trunc() as i64,
        32.329_402_834_f64.mul_add(100_000_f64, 0.0).trunc() as i64
    );

    let offset_results = &result.offset.unwrap();

    assert_eq!(offset_results.sign, -1);
    assert_eq!(offset_results.hour, 3);
    assert_eq!(offset_results.minute, 0);
    assert_eq!(
        offset_results
            .second
            .mul_add(f64::from(1_000_000), 0.0)
            .trunc() as i64,
        0.123_456_789_f64.mul_add(1_000_000_f64, 0.0).trunc() as i64
    );

    let tz = &result.tz_annotation.unwrap();

    assert!(tz.critical);

    match &tz.tz {
        boa_ast::temporal::TzIdentifier::TzIANAName(id) => {
            assert_eq!(id, "America/Argentina/ComodRivadavia");
        }
        boa_ast::temporal::TzIdentifier::UtcOffset(_) => unreachable!(),
    }

    assert_eq!(&result.calendar, &Some("iso8601".to_string()));
}

#[test]
fn temporal_year_parsing() {
    let long = "+002020-11-08";
    let bad_year = "-000000-11-08";

    let result_good = TemporalDateTimeString::parse(false, &mut IsoCursor::new(long)).unwrap();
    assert_eq!(result_good.date.year, 2020);

    let err_result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(bad_year));
    assert!(err_result.is_err());
}

#[test]
fn temporal_annotated_date_time() {
    let basic = "2020-11-08[America/Argentina/ComodRivadavia][u-ca=iso8601][foo=bar]";
    let omitted = "+0020201108[u-ca=iso8601][f-1a2b=a0sa-2l4s]";

    let result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(basic)).unwrap();

    if let Some(tz) = &result.tz_annotation {
        match &tz.tz {
            boa_ast::temporal::TzIdentifier::TzIANAName(id) => {
                assert_eq!(id, "America/Argentina/ComodRivadavia");
            }
            boa_ast::temporal::TzIdentifier::UtcOffset(_) => unreachable!(),
        }
    }

    assert_eq!(&result.calendar, &Some("iso8601".to_string()));

    let omit_result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(omitted)).unwrap();

    assert!(&omit_result.tz_annotation.is_none());

    assert_eq!(&omit_result.calendar, &Some("iso8601".to_string()));
}

#[test]
fn temporal_year_month() {
    use boa_ast::temporal::TzIdentifier;

    let possible_year_months = &[
        "+002020-11",
        "2020-11[+04:00]",
        "+00202011",
        "202011[+04:00]",
    ];

    for ym in possible_year_months {
        let result = TemporalYearMonthString::parse(&mut IsoCursor::new(ym)).unwrap();

        assert_eq!(result.date.year, 2020);
        assert_eq!(result.date.month, 11);

        if let Some(annotation) = &result.tz_annotation {
            match &annotation.tz {
                TzIdentifier::UtcOffset(utc) => assert_eq!(utc.hour, 4),
                TzIdentifier::TzIANAName(_) => unreachable!(),
            }
        }
    }
}

#[test]
fn temporal_month_day() {
    let possible_month_day = ["11-07", "1107[+04:00]", "--11-07", "--1107[+04:00]"];

    for md in possible_month_day {
        let result = TemporalMonthDayString::parse(&mut IsoCursor::new(md)).unwrap();

        assert_eq!(result.date.month, 11);
        assert_eq!(result.date.day, 7);
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
        let err_result = TemporalMonthDayString::parse(&mut IsoCursor::new(invalid));
        assert!(err_result.is_err())
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
        let result = TemporalInstantString::parse(&mut IsoCursor::new(test));
        assert!(result.is_ok());
    }
}

#[test]
fn temporal_duration_parsing() {
    let durations = [
        "p1y1m1dt1h1m1s",
        "P1Y1M1W1DT1H1M1.1S",
        "P1Y1M1W1DT1H1M1.123456789S",
        "-P1Y3wT0,5H",
    ];

    for dur in durations {
        let _ok_result = TemporalDurationString::parse(&mut IsoCursor::new(dur)).unwrap();
        //assert!(ok_result.is_ok())
    }

    let dur = durations[3];
    let test_result = TemporalDurationString::parse(&mut IsoCursor::new(dur)).unwrap();

    assert!(!test_result.sign);
    assert_eq!(test_result.date.years, 1);
    assert_eq!(test_result.date.weeks, 3);
    assert_eq!(test_result.time.hours.mul_add(10.0, 0.0), 5.0);
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
        let err = TemporalDurationString::parse(&mut IsoCursor::new(test));
        assert!(err.is_err());
    }
}
