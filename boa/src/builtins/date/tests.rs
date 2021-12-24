#![allow(clippy::zero_prefixed_literal)]

use crate::{forward, forward_val, Context, JsValue};
use chrono::prelude::*;

// NOTE: Javascript Uses 0-based months, where chrono uses 1-based months. Many of the assertions look wrong because of
// this.

fn forward_dt_utc(context: &mut Context, src: &str) -> Option<NaiveDateTime> {
    let date_time = if let Ok(v) = forward_val(context, src) {
        v
    } else {
        panic!("expected success")
    };

    if let JsValue::Object(ref date_time) = date_time {
        if let Some(date_time) = date_time.borrow().as_date() {
            date_time.0
        } else {
            panic!("expected date")
        }
    } else {
        panic!("expected object")
    }
}

fn forward_dt_local(context: &mut Context, src: &str) -> Option<NaiveDateTime> {
    let date_time = forward_dt_utc(context, src);

    // The timestamp is converted to UTC for internal representation
    date_time.map(|utc| {
        Local::now()
            .timezone()
            .from_utc_datetime(&utc)
            .naive_local()
    })
}

#[test]
fn date_display() {
    let dt = super::Date(None);
    assert_eq!("[Invalid Date]", format!("[{}]", dt));

    let cd = super::Date::default();
    assert_eq!(
        format!(
            "[{}]",
            cd.to_local().unwrap().format("%a %b %d %Y %H:%M:%S GMT%:z")
        ),
        format!("[{}]", cd)
    );
}

#[test]
fn date_this_time_value() {
    let mut context = Context::default();

    let error = forward_val(
        &mut context,
        "({toString: Date.prototype.toString}).toString()",
    )
    .expect_err("Expected error");
    let message_property = &error
        .get_property("message")
        .expect("Expected 'message' property")
        .expect_value()
        .clone();

    assert_eq!(JsValue::new("\'this\' is not a Date"), *message_property);
}

#[test]
fn date_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let dt1 = forward(&mut context, "Date()");

    std::thread::sleep(std::time::Duration::from_millis(1));

    let dt2 = forward(&mut context, "Date()");

    assert_ne!(dt1, dt2);
    Ok(())
}

#[test]
fn date_ctor_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let dt1 = forward_dt_local(&mut context, "new Date()");

    std::thread::sleep(std::time::Duration::from_millis(1));

    let dt2 = forward_dt_local(&mut context, "new Date()");

    assert_ne!(dt1, dt2);
    Ok(())
}

#[test]
fn date_ctor_call_string() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let date_time = forward_dt_utc(&mut context, "new Date('2020-06-08T09:16:15.779-06:30')");

    // Internal date is expressed as UTC
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 06, 08).and_hms_milli(15, 46, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_string_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let date_time = forward_dt_local(&mut context, "new Date('nope')");
    assert_eq!(None, date_time);
    Ok(())
}

#[test]
fn date_ctor_call_number() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let date_time = forward_dt_utc(&mut context, "new Date(1594199775779)");
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_date() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let date_time = forward_dt_utc(&mut context, "new Date(new Date(1594199775779))");

    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_multiple() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let date_time = forward_dt_local(&mut context, "new Date(2020, 06, 08, 09, 16, 15, 779)");

    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_multiple_90s() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let date_time = forward_dt_local(&mut context, "new Date(99, 06, 08, 09, 16, 15, 779)");

    assert_eq!(
        Some(NaiveDate::from_ymd(1999, 07, 08).and_hms_milli(09, 16, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_multiple_nan() -> Result<(), Box<dyn std::error::Error>> {
    fn check(src: &str) {
        let mut context = Context::default();
        let date_time = forward_dt_local(&mut context, src);
        assert_eq!(None, date_time);
    }

    check("new Date(1/0, 06, 08, 09, 16, 15, 779)");
    check("new Date(2020, 1/0, 08, 09, 16, 15, 779)");
    check("new Date(2020, 06, 1/0, 09, 16, 15, 779)");
    check("new Date(2020, 06, 08, 1/0, 16, 15, 779)");
    check("new Date(2020, 06, 08, 09, 1/0, 15, 779)");
    check("new Date(2020, 06, 08, 09, 16, 1/0, 779)");
    check("new Date(2020, 06, 08, 09, 16, 15, 1/0)");

    Ok(())
}

#[test]
fn date_ctor_now_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let date_time = forward(&mut context, "Date.now()");
    let dt1 = date_time.parse::<u64>()?;

    std::thread::sleep(std::time::Duration::from_millis(1));

    let date_time = forward(&mut context, "Date.now()");
    let dt2 = date_time.parse::<u64>()?;

    assert_ne!(dt1, dt2);
    Ok(())
}

#[test]
fn date_ctor_parse_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let date_time = forward_val(&mut context, "Date.parse('2020-06-08T09:16:15.779-07:30')");

    assert_eq!(Ok(JsValue::new(1591634775779f64)), date_time);
    Ok(())
}

#[test]
fn date_ctor_utc_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let date_time = forward_val(&mut context, "Date.UTC(2020, 06, 08, 09, 16, 15, 779)");

    assert_eq!(Ok(JsValue::new(1594199775779f64)), date_time);
    Ok(())
}

#[test]
fn date_ctor_utc_call_nan() -> Result<(), Box<dyn std::error::Error>> {
    fn check(src: &str) {
        let mut context = Context::default();
        let date_time = forward_val(&mut context, src).expect("Expected Success");
        assert_eq!(JsValue::nan(), date_time);
    }

    check("Date.UTC(1/0, 06, 08, 09, 16, 15, 779)");
    check("Date.UTC(2020, 1/0, 08, 09, 16, 15, 779)");
    check("Date.UTC(2020, 06, 1/0, 09, 16, 15, 779)");
    check("Date.UTC(2020, 06, 08, 1/0, 16, 15, 779)");
    check("Date.UTC(2020, 06, 08, 09, 1/0, 15, 779)");
    check("Date.UTC(2020, 06, 08, 09, 16, 1/0, 779)");
    check("Date.UTC(2020, 06, 08, 09, 16, 15, 1/0)");

    Ok(())
}

#[test]
fn date_proto_get_date_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getDate()",
    );
    assert_eq!(Ok(JsValue::new(08f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getDate()");
    assert_eq!(Ok(JsValue::nan()), actual);

    Ok(())
}

#[test]
fn date_proto_get_day_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getDay()",
    );
    assert_eq!(Ok(JsValue::new(3f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getDay()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_full_year_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getFullYear()",
    );
    assert_eq!(Ok(JsValue::new(2020f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getFullYear()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_hours_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getHours()",
    );
    assert_eq!(Ok(JsValue::new(09f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getHours()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_milliseconds_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getMilliseconds()",
    );
    assert_eq!(Ok(JsValue::new(779f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getMilliseconds()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_minutes_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getMinutes()",
    );
    assert_eq!(Ok(JsValue::new(16f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getMinutes()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_month() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getMonth()",
    );
    assert_eq!(Ok(JsValue::new(06f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getMonth()");
    assert_eq!(Ok(JsValue::nan()), actual);

    Ok(())
}

#[test]
fn date_proto_get_seconds() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getSeconds()",
    );
    assert_eq!(Ok(JsValue::new(15f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getSeconds()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_time() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getTime()",
    );

    let ts = Local
        .ymd(2020, 07, 08)
        .and_hms_milli(09, 16, 15, 779)
        .timestamp_millis() as f64;
    assert_eq!(Ok(JsValue::new(ts)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getTime()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_year() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 06, 08, 09, 16, 15, 779).getYear()",
    );
    assert_eq!(Ok(JsValue::new(120f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getYear()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_timezone_offset() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date('1975-08-19T23:15:30+07:00').getTimezoneOffset() === new Date('1975-08-19T23:15:30-02:00').getTimezoneOffset()",
    );

    // NB: Host Settings, not TZ specified in the DateTime.
    assert_eq!(Ok(JsValue::new(true)), actual);

    let actual = forward_val(
        &mut context,
        "new Date('1975-08-19T23:15:30+07:00').getTimezoneOffset()",
    );

    // The value of now().offset() depends on the host machine, so we have to replicate the method code here.
    let offset_seconds = chrono::Local::now().offset().local_minus_utc() as f64;
    let offset_minutes = -offset_seconds / 60f64;
    assert_eq!(Ok(JsValue::new(offset_minutes)), actual);

    let actual = forward_val(
        &mut context,
        "new Date('1975-08-19T23:15:30+07:00').getTimezoneOffset()",
    );
    assert_eq!(Ok(JsValue::new(offset_minutes)), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_date_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)).getUTCDate()",
    );
    assert_eq!(Ok(JsValue::new(08f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getUTCDate()");
    assert_eq!(Ok(JsValue::nan()), actual);

    Ok(())
}

#[test]
fn date_proto_get_utc_day_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)).getUTCDay()",
    );
    assert_eq!(Ok(JsValue::new(3f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getUTCDay()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_full_year_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)).getUTCFullYear()",
    );
    assert_eq!(Ok(JsValue::new(2020f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getUTCFullYear()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_hours_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)).getUTCHours()",
    );
    assert_eq!(Ok(JsValue::new(09f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getUTCHours()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_milliseconds_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)).getUTCMilliseconds()",
    );
    assert_eq!(Ok(JsValue::new(779f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getUTCMilliseconds()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_minutes_call() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)).getUTCMinutes()",
    );
    assert_eq!(Ok(JsValue::new(16f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getUTCMinutes()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_month() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)).getUTCMonth()",
    );
    assert_eq!(Ok(JsValue::new(06f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getUTCMonth()");
    assert_eq!(Ok(JsValue::nan()), actual);

    Ok(())
}

#[test]
fn date_proto_get_utc_seconds() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)).getUTCSeconds()",
    );
    assert_eq!(Ok(JsValue::new(15f64)), actual);

    let actual = forward_val(&mut context, "new Date(1/0).getUTCSeconds()");
    assert_eq!(Ok(JsValue::nan()), actual);
    Ok(())
}

#[test]
fn date_proto_set_date() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_local(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setDate(21); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 21).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    // Date wraps to previous month for 0.
    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setDate(0); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 06, 30).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setDate(1/0); dt",
    );
    assert_eq!(None, actual);

    Ok(())
}

#[test]
fn date_proto_set_full_year() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_local(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setFullYear(2012); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2012, 07, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setFullYear(2012, 8); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2012, 09, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setFullYear(2012, 8, 10); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2012, 09, 10).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    // Out-of-bounds

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 07, 08, 09, 16, 15, 779); dt.setFullYear(2012, 35); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2014, 12, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 07, 08, 09, 16, 15, 779); dt.setFullYear(2012, -35); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2009, 02, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 07, 08, 09, 16, 15, 779); dt.setFullYear(2012, 9, 950); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2015, 05, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 07, 08, 09, 16, 15, 779); dt.setFullYear(2012, 9, -950); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2010, 02, 23).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_hours() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_local(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setHours(11); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(11, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setHours(11, 35); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(11, 35, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setHours(11, 35, 23); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(11, 35, 23, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setHours(11, 35, 23, 537); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(11, 35, 23, 537)),
        actual
    );

    // Out-of-bounds

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setHours(10000, 20000, 30000, 40123); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2021, 09, 11).and_hms_milli(21, 40, 40, 123)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_milliseconds() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_local(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setMilliseconds(597); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 15, 597)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHours

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setMilliseconds(40123); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 55, 123)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_minutes() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_local(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setMinutes(11); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 11, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setMinutes(11, 35); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 11, 35, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setMinutes(11, 35, 537); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 11, 35, 537)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHours

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setMinutes(600000, 30000, 40123); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2021, 08, 29).and_hms_milli(09, 20, 40, 123)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_month() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_local(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setMonth(11); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 12, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setMonth(11, 16); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 12, 16).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setFullYear

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 07, 08, 09, 16, 15, 779); dt.setMonth(40, 83); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2023, 07, 22).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_seconds() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_local(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setSeconds(11); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 11, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setSeconds(11, 487); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 11, 487)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHour

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 07, 08, 09, 16, 15, 779); dt.setSeconds(40000000, 40123); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2021, 11, 14).and_hms_milli(08, 23, 20, 123)),
        actual
    );

    Ok(())
}

#[test]
fn set_year() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_local(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setYear(98); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(1998, 07, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_local(
        &mut context,
        "dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.setYear(2001); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2001, 07, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_time() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_local(
        &mut context,
        "let dt = new Date(); dt.setTime(new Date(2020, 06, 08, 09, 16, 15, 779).getTime()); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_utc_date() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_utc(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCDate(21); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 21).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    // Date wraps to previous month for 0.
    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCDate(0); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 06, 30).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCDate(1/0); dt",
    );
    assert_eq!(None, actual);

    Ok(())
}

#[test]
fn date_proto_set_utc_full_year() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_utc(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCFullYear(2012); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2012, 07, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCFullYear(2012, 8); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2012, 09, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCFullYear(2012, 8, 10); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2012, 09, 10).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    // Out-of-bounds

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)); dt.setUTCFullYear(2012, 35); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2014, 12, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)); dt.setUTCFullYear(2012, -35); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2009, 02, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)); dt.setUTCFullYear(2012, 9, 950); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2015, 05, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)); dt.setUTCFullYear(2012, 9, -950); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2010, 02, 23).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_utc_hours() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_utc(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCHours(11); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(11, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCHours(11, 35); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(11, 35, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCHours(11, 35, 23); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(11, 35, 23, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCHours(11, 35, 23, 537); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(11, 35, 23, 537)),
        actual
    );

    // Out-of-bounds

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCHours(10000, 20000, 30000, 40123); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2021, 09, 11).and_hms_milli(21, 40, 40, 123)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_utc_milliseconds() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_utc(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCMilliseconds(597); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 15, 597)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHours

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCMilliseconds(40123); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 55, 123)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_utc_minutes() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_utc(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCMinutes(11); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 11, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCMinutes(11, 35); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 11, 35, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCMinutes(11, 35, 537); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 11, 35, 537)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHours

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCMinutes(600000, 30000, 40123); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2021, 08, 29).and_hms_milli(09, 20, 40, 123)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_utc_month() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_utc(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCMonth(11); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 12, 08).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCMonth(11, 16); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 12, 16).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setFullYear

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)); dt.setUTCMonth(40, 83); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2023, 07, 22).and_hms_milli(09, 16, 15, 779)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_set_utc_seconds() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_dt_utc(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCSeconds(11); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 11, 779)),
        actual
    );

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.setUTCSeconds(11, 487); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 11, 487)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHour

    let actual = forward_dt_utc(
        &mut context,
        "dt = new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)); dt.setUTCSeconds(40000000, 40123); dt",
    );
    assert_eq!(
        Some(NaiveDate::from_ymd(2021, 11, 14).and_hms_milli(08, 23, 20, 123)),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_to_date_string() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.toDateString()",
    )
    .expect("Successful eval");
    assert_eq!(JsValue::new("Wed Jul 08 2020"), actual);

    Ok(())
}

#[test]
fn date_proto_to_gmt_string() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.toGMTString()",
    )
    .expect("Successful eval");
    assert_eq!(JsValue::new("Wed, 08 Jul 2020 09:16:15 GMT"), actual);

    Ok(())
}

#[test]
fn date_proto_to_iso_string() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.toISOString()",
    )
    .expect("Successful eval");
    assert_eq!(JsValue::new("2020-07-08T09:16:15.779Z"), actual);

    Ok(())
}

#[test]
fn date_proto_to_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.toJSON()",
    )
    .expect("Successful eval");
    assert_eq!(JsValue::new("2020-07-08T09:16:15.779Z"), actual);

    Ok(())
}

#[test]
fn date_proto_to_string() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.toString()",
    )
    .ok();

    assert_eq!(
        Some(JsValue::new(
            Local
                .from_local_datetime(&NaiveDateTime::new(
                    NaiveDate::from_ymd(2020, 6, 8),
                    NaiveTime::from_hms_milli(9, 16, 15, 779)
                ))
                .earliest()
                .unwrap()
                .format("Wed Jul 08 2020 09:16:15 GMT%z")
                .to_string()
        )),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_to_time_string() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(2020, 06, 08, 09, 16, 15, 779); dt.toTimeString()",
    )
    .ok();

    assert_eq!(
        Some(JsValue::new(
            Local
                .from_local_datetime(&NaiveDateTime::new(
                    NaiveDate::from_ymd(2020, 6, 8),
                    NaiveTime::from_hms_milli(9, 16, 15, 779)
                ))
                .earliest()
                .unwrap()
                .format("09:16:15 GMT%z")
                .to_string()
        )),
        actual
    );

    Ok(())
}

#[test]
fn date_proto_to_utc_string() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)); dt.toUTCString()",
    )
    .expect("Successful eval");
    assert_eq!(JsValue::new("Wed, 08 Jul 2020 09:16:15 GMT"), actual);

    Ok(())
}

#[test]
fn date_proto_value_of() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)).valueOf()",
    )
    .expect("Successful eval");
    assert_eq!(JsValue::new(1594199775779f64), actual);

    Ok(())
}

#[test]
fn date_neg() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "-new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779))",
    )
    .expect("Successful eval");
    assert_eq!(JsValue::new(-1594199775779f64), actual);

    Ok(())
}

#[test]
fn date_json() -> Result<(), Box<dyn std::error::Error>> {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "JSON.stringify({ date: new Date(Date.UTC(2020, 06, 08, 09, 16, 15, 779)) })",
    )
    .expect("Successful eval");
    assert_eq!(
        JsValue::new(r#"{"date":"2020-07-08T09:16:15.779Z"}"#),
        actual
    );

    Ok(())
}
