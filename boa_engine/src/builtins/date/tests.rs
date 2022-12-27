use crate::{forward, forward_val, Context, JsValue};
use chrono::{prelude::*, Duration};

// NOTE: Javascript Uses 0-based months, where chrono uses 1-based months. Many of the assertions look wrong because of
// this.

fn datetime_from_local(
    year: i32,
    month: u32,
    date: u32,
    hour: u32,
    minute: u32,
    second: u32,
    millisecond: u32,
) -> NaiveDateTime {
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
}

fn forward_dt(context: &mut Context<'_>, src: &str) -> Option<NaiveDateTime> {
    let date_time = forward_val(context, src).unwrap();
    let date_time = date_time.as_object().unwrap();
    let date_time = date_time.borrow();
    date_time.as_date().unwrap().0
}

#[test]
fn date_display() {
    let dt = super::Date(None);
    assert_eq!("[Invalid Date]", format!("[{dt}]"));

    let cd = super::Date::default();
    assert_eq!(
        format!(
            "[{}]",
            cd.to_local().unwrap().format("%a %b %d %Y %H:%M:%S GMT%:z")
        ),
        format!("[{cd}]")
    );
}

#[test]
fn date_this_time_value() {
    let mut context = Context::default();

    let error = forward_val(
        &mut context,
        "({toString: Date.prototype.toString}).toString()",
    )
    .unwrap_err();
    let error = error.as_native().unwrap();
    assert_eq!("\'this\' is not a Date", error.message());
}

#[test]
fn date_ctor_call() {
    let mut context = Context::default();

    let dt1 = forward_dt(&mut context, "new Date()");

    std::thread::sleep(std::time::Duration::from_millis(1));

    let dt2 = forward_dt(&mut context, "new Date()");

    assert_ne!(dt1, dt2);
}

#[test]
fn date_ctor_call_string() {
    let mut context = Context::default();

    let date_time = forward_dt(&mut context, "new Date('2020-06-08T09:16:15.779-06:30')");

    // Internal date is expressed as UTC
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 6, 8)
                .unwrap()
                .and_hms_milli_opt(15, 46, 15, 779)
                .unwrap()
        ),
        date_time
    );
}

#[test]
fn date_ctor_call_string_invalid() {
    let mut context = Context::default();

    let date_time = forward_dt(&mut context, "new Date('nope')");
    assert_eq!(None, date_time);
}

#[test]
fn date_ctor_call_number() {
    let mut context = Context::default();

    let date_time = forward_dt(&mut context, "new Date(1594199775779)");
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        date_time
    );
}

#[test]
fn date_ctor_call_date() {
    let mut context = Context::default();

    let date_time = forward_dt(&mut context, "new Date(new Date(1594199775779))");

    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        date_time
    );
}

#[test]
fn date_ctor_call_multiple() {
    let mut context = Context::default();

    let date_time = forward_dt(&mut context, "new Date(2020, 6, 8, 9, 16, 15, 779)");

    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 9, 16, 15, 779)),
        date_time
    );
}

#[test]
fn date_ctor_call_multiple_90s() {
    let mut context = Context::default();

    let date_time = forward_dt(&mut context, "new Date(99, 6, 8, 9, 16, 15, 779)");

    assert_eq!(
        Some(datetime_from_local(1999, 7, 8, 9, 16, 15, 779)),
        date_time
    );
}

#[test]
fn date_ctor_call_multiple_nan() {
    fn check(src: &str) {
        let mut context = Context::default();
        let date_time = forward_dt(&mut context, src);
        assert_eq!(None, date_time);
    }

    check("new Date(1/0, 6, 8, 9, 16, 15, 779)");
    check("new Date(2020, 1/0, 8, 9, 16, 15, 779)");
    check("new Date(2020, 6, 1/0, 9, 16, 15, 779)");
    check("new Date(2020, 6, 8, 1/0, 16, 15, 779)");
    check("new Date(2020, 6, 8, 9, 1/0, 15, 779)");
    check("new Date(2020, 6, 8, 9, 16, 1/0, 779)");
    check("new Date(2020, 6, 8, 9, 16, 15, 1/0)");
}

#[test]
fn date_ctor_now_call() {
    let mut context = Context::default();

    let date_time = forward(&mut context, "Date.now()");
    let dt1 = date_time.parse::<u64>().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(1));

    let date_time = forward(&mut context, "Date.now()");
    let dt2 = date_time.parse::<u64>().unwrap();

    assert_ne!(dt1, dt2);
}

#[test]
fn date_ctor_parse_call() {
    let mut context = Context::default();

    let date_time = forward_val(&mut context, "Date.parse('2020-06-08T09:16:15.779-07:30')");

    assert_eq!(JsValue::new(1_591_634_775_779_i64), date_time.unwrap());
}

#[test]
fn date_ctor_utc_call() {
    let mut context = Context::default();

    let date_time = forward_val(&mut context, "Date.UTC(2020, 6, 8, 9, 16, 15, 779)");

    assert_eq!(JsValue::new(1_594_199_775_779_i64), date_time.unwrap());
}

#[test]
fn date_ctor_utc_call_nan() {
    fn check(src: &str) {
        let mut context = Context::default();
        let date_time = forward_val(&mut context, src).unwrap();
        assert_eq!(JsValue::nan(), date_time);
    }

    check("Date.UTC(1/0, 6, 8, 9, 16, 15, 779)");
    check("Date.UTC(2020, 1/0, 8, 9, 16, 15, 779)");
    check("Date.UTC(2020, 6, 1/0, 9, 16, 15, 779)");
    check("Date.UTC(2020, 6, 8, 1/0, 16, 15, 779)");
    check("Date.UTC(2020, 6, 8, 9, 1/0, 15, 779)");
    check("Date.UTC(2020, 6, 8, 9, 16, 1/0, 779)");
    check("Date.UTC(2020, 6, 8, 9, 16, 15, 1/0)");
}

#[test]
fn date_proto_get_date_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getDate()",
    );
    assert_eq!(JsValue::new(8), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getDate()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_day_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getDay()",
    );
    assert_eq!(JsValue::new(3), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getDay()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_full_year_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getFullYear()",
    );
    assert_eq!(JsValue::new(2020), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getFullYear()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_hours_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getHours()",
    );
    assert_eq!(JsValue::new(9), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getHours()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_milliseconds_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getMilliseconds()",
    );
    assert_eq!(JsValue::new(779), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getMilliseconds()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_minutes_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getMinutes()",
    );
    assert_eq!(JsValue::new(16), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getMinutes()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_month() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getMonth()",
    );
    assert_eq!(JsValue::new(6), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getMonth()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_seconds() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getSeconds()",
    );
    assert_eq!(JsValue::new(15), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getSeconds()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_time() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getTime()",
    );

    let ts = Local.with_ymd_and_hms(2020, 7, 8, 9, 16, 15).unwrap() + Duration::milliseconds(779);

    assert_eq!(JsValue::new(ts.timestamp_millis()), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getTime()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_year() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(2020, 6, 8, 9, 16, 15, 779).getYear()",
    );
    assert_eq!(JsValue::new(120), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getYear()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_timezone_offset() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date('1975-08-19T23:15:30+07:00').getTimezoneOffset() === new Date('1975-08-19T23:15:30-02:00').getTimezoneOffset()",
    );

    // NB: Host Settings, not TZ specified in the DateTime.
    assert_eq!(JsValue::new(true), actual.unwrap());

    let actual = forward_val(
        &mut context,
        "new Date('1975-08-19T23:15:30+07:00').getTimezoneOffset()",
    );

    // The value of now().offset() depends on the host machine, so we have to replicate the method code here.
    let offset_seconds = chrono::Local::now().offset().local_minus_utc();
    let offset_minutes = -offset_seconds / 60;
    assert_eq!(JsValue::new(offset_minutes), actual.unwrap());

    let actual = forward_val(
        &mut context,
        "new Date('1975-08-19T23:15:30+07:00').getTimezoneOffset()",
    );
    assert_eq!(JsValue::new(offset_minutes), actual.unwrap());
}

#[test]
fn date_proto_get_utc_date_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCDate()",
    );
    assert_eq!(JsValue::new(8), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getUTCDate()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_utc_day_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCDay()",
    );
    assert_eq!(JsValue::new(3), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getUTCDay()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_utc_full_year_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCFullYear()",
    );
    assert_eq!(JsValue::new(2020), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getUTCFullYear()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_utc_hours_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCHours()",
    );
    assert_eq!(JsValue::new(9), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getUTCHours()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_utc_milliseconds_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCMilliseconds()",
    );
    assert_eq!(JsValue::new(779), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getUTCMilliseconds()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_utc_minutes_call() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCMinutes()",
    );
    assert_eq!(JsValue::new(16), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getUTCMinutes()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_utc_month() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCMonth()",
    );
    assert_eq!(JsValue::new(6), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getUTCMonth()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_get_utc_seconds() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).getUTCSeconds()",
    );
    assert_eq!(JsValue::new(15), actual.unwrap());

    let actual = forward_val(&mut context, "new Date(1/0).getUTCSeconds()");
    assert_eq!(JsValue::nan(), actual.unwrap());
}

#[test]
fn date_proto_set_date() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setDate(21); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 21, 9, 16, 15, 779)),
        actual
    );

    // Date wraps to previous month for 0.
    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setDate(0); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 6, 30, 9, 16, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setDate(1/0); dt",
    );
    assert_eq!(None, actual);
}

#[test]
fn date_proto_set_full_year() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setFullYear(2012); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2012, 7, 8, 9, 16, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setFullYear(2012, 8); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2012, 9, 8, 9, 16, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setFullYear(2012, 8, 10); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2012, 9, 10, 9, 16, 15, 779)),
        actual
    );

    // Out-of-bounds

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 7, 8, 9, 16, 15, 779); dt.setFullYear(2012, 35); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2014, 12, 8, 9, 16, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 7, 8, 9, 16, 15, 779); dt.setFullYear(2012, -35); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2009, 2, 8, 9, 16, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 7, 8, 9, 16, 15, 779); dt.setFullYear(2012, 9, 950); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2015, 5, 8, 9, 16, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 7, 8, 9, 16, 15, 779); dt.setFullYear(2012, 9, -950); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2010, 2, 23, 9, 16, 15, 779)),
        actual
    );
}

#[test]
fn date_proto_set_hours() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setHours(11); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 11, 16, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setHours(11, 35); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 11, 35, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setHours(11, 35, 23); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 11, 35, 23, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setHours(11, 35, 23, 537); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 11, 35, 23, 537)),
        actual
    );

    // Out-of-bounds

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setHours(10000, 20000, 30000, 40123); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2021, 9, 11, 21, 40, 40, 123)),
        actual
    );
}

#[test]
fn date_proto_set_milliseconds() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setMilliseconds(597); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 9, 16, 15, 597)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHours

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setMilliseconds(40123); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 9, 16, 55, 123)),
        actual
    );
}

#[test]
fn date_proto_set_minutes() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setMinutes(11); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 9, 11, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setMinutes(11, 35); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 9, 11, 35, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setMinutes(11, 35, 537); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 9, 11, 35, 537)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHours

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setMinutes(600000, 30000, 40123); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2021, 8, 29, 9, 20, 40, 123)),
        actual
    );
}

#[test]
fn date_proto_set_month() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setMonth(11); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 12, 8, 9, 16, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setMonth(11, 16); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 12, 16, 9, 16, 15, 779)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setFullYear

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 7, 8, 9, 16, 15, 779); dt.setMonth(40, 83); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2023, 7, 22, 9, 16, 15, 779)),
        actual
    );
}

#[test]
fn date_proto_set_seconds() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setSeconds(11); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 9, 16, 11, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setSeconds(11, 487); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 9, 16, 11, 487)),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHour

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 7, 8, 9, 16, 15, 779); dt.setSeconds(40000000, 40123); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2021, 11, 14, 8, 23, 20, 123)),
        actual
    );
}

#[test]
fn set_year() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setYear(98); dt",
    );
    assert_eq!(
        Some(datetime_from_local(1998, 7, 8, 9, 16, 15, 779)),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.setYear(2001); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2001, 7, 8, 9, 16, 15, 779)),
        actual
    );
}

#[test]
fn date_proto_set_time() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(); dt.setTime(new Date(2020, 6, 8, 9, 16, 15, 779).getTime()); dt",
    );
    assert_eq!(
        Some(datetime_from_local(2020, 7, 8, 9, 16, 15, 779)),
        actual
    );
}

#[test]
fn date_proto_set_utc_date() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCDate(21); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 21)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    // Date wraps to previous month for 0.
    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCDate(0); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 6, 30)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCDate(1/0); dt",
    );
    assert_eq!(None, actual);
}

#[test]
fn date_proto_set_utc_full_year() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCFullYear(2012); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2012, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCFullYear(2012, 8); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2012, 9, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCFullYear(2012, 8, 10); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2012, 9, 10)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    // Out-of-bounds

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 7, 8, 9, 16, 15, 779)); dt.setUTCFullYear(2012, 35); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2014, 12, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 7, 8, 9, 16, 15, 779)); dt.setUTCFullYear(2012, -35); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2009, 2, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 7, 8, 9, 16, 15, 779)); dt.setUTCFullYear(2012, 9, 950); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2015, 5, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 7, 8, 9, 16, 15, 779)); dt.setUTCFullYear(2012, 9, -950); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2010, 2, 23)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );
}

#[test]
fn date_proto_set_utc_hours() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCHours(11); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(11, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCHours(11, 35); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(11, 35, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCHours(11, 35, 23); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(11, 35, 23, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCHours(11, 35, 23, 537); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(11, 35, 23, 537)
                .unwrap()
        ),
        actual
    );

    // Out-of-bounds

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCHours(10000, 20000, 30000, 40123); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2021, 9, 11)
                .unwrap()
                .and_hms_milli_opt(21, 40, 40, 123)
                .unwrap()
        ),
        actual
    );
}

#[test]
fn date_proto_set_utc_milliseconds() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCMilliseconds(597); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 597)
                .unwrap()
        ),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHours

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCMilliseconds(40123); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 55, 123)
                .unwrap()
        ),
        actual
    );
}

#[test]
fn date_proto_set_utc_minutes() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCMinutes(11); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 11, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCMinutes(11, 35); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 11, 35, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCMinutes(11, 35, 537); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 11, 35, 537)
                .unwrap()
        ),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHours

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCMinutes(600000, 30000, 40123); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2021, 8, 29)
                .unwrap()
                .and_hms_milli_opt(9, 20, 40, 123)
                .unwrap()
        ),
        actual
    );
}

#[test]
fn date_proto_set_utc_month() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCMonth(11); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 12, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCMonth(11, 16); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 12, 16)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setFullYear

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 7, 8, 9, 16, 15, 779)); dt.setUTCMonth(40, 83); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2023, 7, 22)
                .unwrap()
                .and_hms_milli_opt(9, 16, 15, 779)
                .unwrap()
        ),
        actual
    );
}

#[test]
fn date_proto_set_utc_seconds() {
    let mut context = Context::default();

    let actual = forward_dt(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCSeconds(11); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 11, 779)
                .unwrap()
        ),
        actual
    );

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.setUTCSeconds(11, 487); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2020, 7, 8)
                .unwrap()
                .and_hms_milli_opt(9, 16, 11, 487)
                .unwrap()
        ),
        actual
    );

    // Out-of-bounds
    // Thorough tests are done by setHour

    let actual = forward_dt(
        &mut context,
        "dt = new Date(Date.UTC(2020, 7, 8, 9, 16, 15, 779)); dt.setUTCSeconds(40000000, 40123); dt",
    );
    assert_eq!(
        Some(
            NaiveDate::from_ymd_opt(2021, 11, 14)
                .unwrap()
                .and_hms_milli_opt(8, 23, 20, 123)
                .unwrap()
        ),
        actual
    );
}

#[test]
fn date_proto_to_date_string() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.toDateString()",
    )
    .unwrap();
    assert_eq!(JsValue::new("Wed Jul 08 2020"), actual);
}

#[test]
fn date_proto_to_gmt_string() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.toGMTString()",
    )
    .unwrap();
    assert_eq!(JsValue::new("Wed, 08 Jul 2020 09:16:15 GMT"), actual);
}

#[test]
fn date_proto_to_iso_string() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.toISOString()",
    )
    .unwrap();
    assert_eq!(JsValue::new("2020-07-08T09:16:15.779Z"), actual);
}

#[test]
fn date_proto_to_json() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.toJSON()",
    )
    .unwrap();
    assert_eq!(JsValue::new("2020-07-08T09:16:15.779Z"), actual);
}

#[test]
fn date_proto_to_string() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.toString()",
    )
    .ok();

    assert_eq!(
        Some(JsValue::new(
            Local
                .from_local_datetime(&NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 7, 8).unwrap(),
                    NaiveTime::from_hms_milli_opt(9, 16, 15, 779).unwrap()
                ))
                .earliest()
                .unwrap()
                .format("Wed Jul 08 2020 09:16:15 GMT%z")
                .to_string()
        )),
        actual
    );
}

#[test]
fn date_proto_to_time_string() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(2020, 6, 8, 9, 16, 15, 779); dt.toTimeString()",
    )
    .ok();

    assert_eq!(
        Some(JsValue::new(
            Local
                .from_local_datetime(&NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(2020, 7, 8).unwrap(),
                    NaiveTime::from_hms_milli_opt(9, 16, 15, 779).unwrap()
                ))
                .earliest()
                .unwrap()
                .format("09:16:15 GMT%z")
                .to_string()
        )),
        actual
    );
}

#[test]
fn date_proto_to_utc_string() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "let dt = new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)); dt.toUTCString()",
    )
    .unwrap();
    assert_eq!(JsValue::new("Wed, 08 Jul 2020 09:16:15 GMT"), actual);
}

#[test]
fn date_proto_value_of() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)).valueOf()",
    )
    .unwrap();
    assert_eq!(JsValue::new(1_594_199_775_779_i64), actual);
}

#[test]
fn date_neg() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "-new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779))",
    )
    .unwrap();
    assert_eq!(JsValue::new(-1_594_199_775_779_i64), actual);
}

#[test]
fn date_json() {
    let mut context = Context::default();

    let actual = forward_val(
        &mut context,
        "JSON.stringify({ date: new Date(Date.UTC(2020, 6, 8, 9, 16, 15, 779)) })",
    )
    .unwrap();
    assert_eq!(
        JsValue::new(r#"{"date":"2020-07-08T09:16:15.779Z"}"#),
        actual
    );
}
