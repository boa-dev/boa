use super::{IsoCursor, TemporalDateTimeString, TemporalMonthDayString, TemporalYearMonthString};

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

    let annotations = &result.annotations.unwrap();

    assert!(annotations.contains_key("u-ca"));
    assert_eq!(
        annotations.get("u-ca"),
        Some(&(true, "iso8601".to_string()))
    );
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

    if let Some(annotations) = &result.annotations {
        assert!(annotations.contains_key("u-ca"));
        assert_eq!(
            annotations.get("u-ca"),
            Some(&(false, "iso8601".to_string()))
        );
    }

    let omit_result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(omitted)).unwrap();

    assert!(&omit_result.tz_annotation.is_none());

    if let Some(annotations) = &omit_result.annotations {
        assert!(annotations.contains_key("u-ca"));
        assert_eq!(
            annotations.get("u-ca"),
            Some(&(false, "iso8601".to_string()))
        );
    }
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
    let possible_month_day = &["11-07", "1107[+04:00]", "--11-07", "--1107[+04:00]"];

    for md in possible_month_day {
        let result = TemporalMonthDayString::parse(&mut IsoCursor::new(md)).unwrap();

        assert_eq!(result.date.month, 11);
        assert_eq!(result.date.day, 7);
    }
}
