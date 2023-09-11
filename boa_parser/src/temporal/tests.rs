
#[test]
fn temporal_parser_basic() {
    use super::{IsoCursor, TemporalDateTimeString};
    let basic = "20201108";
    let basic_separated = "2020-11-08";

    let basic_result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(basic.to_string())).unwrap();

    let sep_result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(basic_separated.to_string())).unwrap();

    assert_eq!(basic_result.date_time.date.year, 2020);
    assert_eq!(basic_result.date_time.date.month, 11);
    assert_eq!(basic_result.date_time.date.day, 8);
    assert_eq!(basic_result.date_time.date.year, sep_result.date_time.date.year);
    assert_eq!(basic_result.date_time.date.month, sep_result.date_time.date.month);
    assert_eq!(basic_result.date_time.date.day, sep_result.date_time.date.day);
}

#[test]
fn temporal_date_time_max() {
    use super::{IsoCursor, TemporalDateTimeString};
    // Fractions not accurate, but for testing purposes.
    let date_time = "+002020-11-08T12:28:32.329402834-03:00:00.123456789[!America/Argentina/ComodRivadavia][!u-ca=iso8601]";

    let result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(date_time.to_string())).unwrap();

    let time_results = &result.date_time.time.unwrap();

    assert_eq!(time_results.hour, 12);
    assert_eq!(time_results.minute, 28);
    assert_eq!(time_results.second, 32.329402834);

    let offset_results = &result.date_time.offset.unwrap();

    assert_eq!(offset_results.sign, -1);
    assert_eq!(offset_results.hour, 3);
    assert_eq!(offset_results.minute, 0);
    assert_eq!(offset_results.second, 0.123456789);

    let tz = &result.tz_annotation.unwrap();

    assert!(tz.critical);

    match &tz.tz {
        boa_ast::temporal::TzIdentifier::TzIANAName(id) => assert_eq!(id, "America/Argentina/ComodRivadavia"),
        _=> unreachable!(),
    }

    let annotations = &result.annotations.unwrap();

    assert!(annotations.contains_key("u-ca"));
    assert_eq!(annotations.get("u-ca"), Some(&(true, "iso8601".to_string())));
}

#[test]
fn temporal_year_parsing() {
    use super::{IsoCursor, TemporalDateTimeString};
    let long = "+002020-11-08";
    let bad_year = "-000000-11-08";

    let result_good = TemporalDateTimeString::parse(false, &mut IsoCursor::new(long.to_string())).unwrap();
    assert_eq!(result_good.date_time.date.year, 2020);

    let err_result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(bad_year.to_string()));
    assert!(err_result.is_err());
}

#[test]
fn temporal_annotated_date_time() {
    use super::{IsoCursor, TemporalDateTimeString};
    let basic = "2020-11-08[America/Argentina/ComodRivadavia][u-ca=iso8601][foo=bar]";
    let omitted = "+0020201108[u-ca=iso8601][f-1a2b=a0sa-2l4s]";

    let result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(basic.to_string())).unwrap();

    if let Some(tz) = &result.tz_annotation {
        match &tz.tz {
            boa_ast::temporal::TzIdentifier::TzIANAName(id) => assert_eq!(id, "America/Argentina/ComodRivadavia"),
            _=> unreachable!(),
        }
    }

    if let Some(annotations) = &result.annotations {
        assert!(annotations.contains_key("u-ca"));
        assert_eq!(annotations.get("u-ca"), Some(&(false, "iso8601".to_string())))
    }

    let omit_result = TemporalDateTimeString::parse(false, &mut IsoCursor::new(omitted.to_string())).unwrap();

    assert!(&omit_result.tz_annotation.is_none());

    if let Some(annotations) = &omit_result.annotations {
        assert!(annotations.contains_key("u-ca"));
        assert_eq!(annotations.get("u-ca"), Some(&(false, "iso8601".to_string())))
    }

}
