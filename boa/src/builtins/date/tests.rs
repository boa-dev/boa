use crate::{
    builtins::{object::ObjectData, Value},
    forward, forward_val, Interpreter, Realm,
};
use chrono::prelude::*;

fn forward_dt_utc(engine: &mut Interpreter, src: &str) -> Option<NaiveDateTime> {
    let date_time = if let Ok(v) = forward_val(engine, src) {
        v
    } else {
        panic!("expected success")
    };

    let date_time = if let Value::Object(date_time) = &date_time {
        date_time
    } else {
        panic!("expected object")
    };

    let date_time = if let ObjectData::Date(date_time) = &date_time.borrow().data {
        date_time.0
    } else {
        panic!("expected date")
    };

    date_time.clone()
}

fn forward_dt_local(engine: &mut Interpreter, src: &str) -> Option<NaiveDateTime> {
    let date_time = forward_dt_utc(engine, src);

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
    assert_eq!(format!("[{}]", cd.to_local().unwrap()), format!("[{}]", cd));
}

#[test]
fn date_this_time_value() {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let error = forward_val(
        &mut engine,
        "({toString: Date.prototype.toString}).toString()",
    )
    .expect_err("Expected error");
    let message_property = &error
        .get_property("message")
        .expect("Expected 'message' property")
        .value;

    assert_eq!(
        &Some(Value::string("\'this\' is not a Date")),
        message_property
    );
}

#[test]
fn date_call() -> Result<(), Box<dyn std::error::Error>> {
    use chrono::prelude::*;

    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward(&mut engine, "Date()");
    let dt1 = DateTime::parse_from_rfc3339(&date_time)?;

    std::thread::sleep(std::time::Duration::from_millis(1));

    let date_time = forward(&mut engine, "Date()");
    let dt2 = DateTime::parse_from_rfc3339(&date_time)?;

    assert_ne!(dt1, dt2);
    Ok(())
}

#[test]
fn date_ctor_call() -> Result<(), Box<dyn std::error::Error>> {
    use chrono::prelude::*;

    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward(&mut engine, "new Date().toString()");
    let dt1 = DateTime::parse_from_rfc3339(&date_time)?;

    std::thread::sleep(std::time::Duration::from_millis(1));

    let date_time = forward(&mut engine, "new Date().toString()");
    let dt2 = DateTime::parse_from_rfc3339(&date_time)?;

    assert_ne!(dt1, dt2);
    Ok(())
}

#[test]
fn date_ctor_call_string() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward_dt_utc(&mut engine, "new Date('2020-07-08T09:16:15.779-07:30')");

    // Internal date is expressed as UTC
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(16, 46, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_string_invalid() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time =
        forward_val(&mut engine, "new Date('nope').toString()").expect("Expected Success");
    assert_eq!(Value::string("Invalid Date"), date_time);
    Ok(())
}

#[test]
fn date_ctor_call_number() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward_dt_utc(&mut engine, "new Date(1594199775779)");
    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_date() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward_dt_utc(&mut engine, "new Date(new Date(1594199775779))");

    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_multiple() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward_dt_local(&mut engine, "new Date(2020, 07, 08, 09, 16, 15, 779)");

    assert_eq!(
        Some(NaiveDate::from_ymd(2020, 07, 08).and_hms_milli(09, 16, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_multiple_90s() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward_dt_local(&mut engine, "new Date(99, 07, 08, 09, 16, 15, 779)");

    assert_eq!(
        Some(NaiveDate::from_ymd(1999, 07, 08).and_hms_milli(09, 16, 15, 779)),
        date_time
    );
    Ok(())
}

#[test]
fn date_ctor_call_multiple_nan() -> Result<(), Box<dyn std::error::Error>> {
    fn check(src: &str) {
        let realm = Realm::create();
        let mut engine = Interpreter::new(realm);
        let date_time = forward_val(&mut engine, src).expect("Expected Success");
        assert_eq!(Value::string("Invalid Date"), date_time);
    }

    check("new Date(1/0, 07, 08, 09, 16, 15, 779).toString()");
    check("new Date(2020, 1/0, 08, 09, 16, 15, 779).toString()");
    check("new Date(2020, 07, 1/0, 09, 16, 15, 779).toString()");
    check("new Date(2020, 07, 08, 1/0, 16, 15, 779).toString()");
    check("new Date(2020, 07, 08, 09, 1/0, 15, 779).toString()");
    check("new Date(2020, 07, 08, 09, 16, 1/0, 779).toString()");
    check("new Date(2020, 07, 08, 09, 16, 15, 1/0).toString()");

    Ok(())
}

#[test]
fn date_ctor_now_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward(&mut engine, "Date.now()");
    let dt1 = u64::from_str_radix(&date_time, 10)?;

    std::thread::sleep(std::time::Duration::from_millis(1));

    let date_time = forward(&mut engine, "Date.now()");
    let dt2 = u64::from_str_radix(&date_time, 10)?;

    assert_ne!(dt1, dt2);
    Ok(())
}

#[test]
fn date_ctor_parse_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward_val(&mut engine, "Date.parse('2020-07-08T09:16:15.779-07:30')");

    assert_eq!(Ok(Value::Rational(1594226775779f64)), date_time);
    Ok(())
}

#[test]
fn date_ctor_utc_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let date_time = forward_val(&mut engine, "Date.UTC(2020, 07, 08, 09, 16, 15, 779)");

    assert_eq!(Ok(Value::Rational(1594199775779f64)), date_time);
    Ok(())
}

#[test]
fn date_ctor_utc_call_nan() -> Result<(), Box<dyn std::error::Error>> {
    fn check(src: &str) {
        let realm = Realm::create();
        let mut engine = Interpreter::new(realm);
        let date_time = forward_val(&mut engine, src).expect("Expected Success");
        assert_eq!(Value::string("NaN"), date_time);
    }

    check("Date.UTC(1/0, 07, 08, 09, 16, 15, 779).toString()");
    check("Date.UTC(2020, 1/0, 08, 09, 16, 15, 779).toString()");
    check("Date.UTC(2020, 07, 1/0, 09, 16, 15, 779).toString()");
    check("Date.UTC(2020, 07, 08, 1/0, 16, 15, 779).toString()");
    check("Date.UTC(2020, 07, 08, 09, 1/0, 15, 779).toString()");
    check("Date.UTC(2020, 07, 08, 09, 16, 1/0, 779).toString()");
    check("Date.UTC(2020, 07, 08, 09, 16, 15, 1/0).toString()");

    Ok(())
}

#[test]
fn date_proto_get_date_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getDate()",
    );
    assert_eq!(Ok(Value::Rational(08f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getDate()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);

    Ok(())
}

#[test]
fn date_proto_get_day_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getDay()",
    );
    assert_eq!(Ok(Value::Rational(3f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getDay()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_full_year_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getFullYear()",
    );
    assert_eq!(Ok(Value::Rational(2020f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getFullYear()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_hours_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getHours()",
    );
    assert_eq!(Ok(Value::Rational(09f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getHours()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_milliseconds_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getMilliseconds()",
    );
    assert_eq!(Ok(Value::Rational(779f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getMilliseconds()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_minutes_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getMinutes()",
    );
    assert_eq!(Ok(Value::Rational(16f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getMinutes()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_month() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getMonth()",
    );
    assert_eq!(Ok(Value::Rational(07f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getMonth()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);

    Ok(())
}

#[test]
fn date_proto_get_seconds() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getSeconds()",
    );
    assert_eq!(Ok(Value::Rational(15f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getSeconds()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_time() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getTime()",
    );

    let ts = Local
        .ymd(2020, 07, 08)
        .and_hms_milli(09, 16, 15, 779)
        .timestamp_millis() as f64;
    assert_eq!(Ok(Value::Rational(ts)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getTime()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_year() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(2020, 07, 08, 09, 16, 15, 779).getYear()",
    );
    assert_eq!(Ok(Value::Rational(120f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getYear()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_timezone_offset() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date('August 19, 1975 23:15:30 GMT+07:00').getTimezoneOffset() === new Date('August 19, 1975 23:15:30 GMT-02:00').getTimezoneOffset()",
    );

    // NB: Host Settings, not TZ specified in the DateTime.
    assert_eq!(Ok(Value::Boolean(true)), actual);

    let actual = forward_val(
        &mut engine,
        "new Date('August 19, 1975 23:15:30 GMT+07:00').getTimezoneOffset()",
    );

    // The value of now().offset() depends on the host machine, so we have to replicate the method code here.
    let offset_seconds = chrono::Local::now().offset().local_minus_utc() as f64;
    let offset_minutes = offset_seconds / 60f64;
    assert_eq!(Ok(Value::Rational(offset_minutes)), actual);

    let actual = forward_val(
        &mut engine,
        "new Date(1/0, 07, 08, 09, 16, 15, 779).getTimezoneOffset()",
    );
    assert_eq!(Ok(Value::Rational(offset_minutes)), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_date_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)).getUTCDate()",
    );
    assert_eq!(Ok(Value::Rational(08f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getUTCDate()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);

    Ok(())
}

#[test]
fn date_proto_get_utc_day_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)).getUTCDay()",
    );
    assert_eq!(Ok(Value::Rational(3f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getUTCDay()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_full_year_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)).getUTCFullYear()",
    );
    assert_eq!(Ok(Value::Rational(2020f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getUTCFullYear()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_hours_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)).getUTCHours()",
    );
    assert_eq!(Ok(Value::Rational(09f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getUTCHours()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_milliseconds_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)).getUTCMilliseconds()",
    );
    assert_eq!(Ok(Value::Rational(779f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getUTCMilliseconds()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_minutes_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)).getUTCMinutes()",
    );
    assert_eq!(Ok(Value::Rational(16f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getUTCMinutes()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_get_utc_month() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)).getUTCMonth()",
    );
    assert_eq!(Ok(Value::Rational(07f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getUTCMonth()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);

    Ok(())
}

#[test]
fn date_proto_get_utc_seconds() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "new Date(Date.UTC(2020, 07, 08, 09, 16, 15, 779)).getUTCSeconds()",
    );
    assert_eq!(Ok(Value::Rational(15f64)), actual);

    let actual = forward_val(&mut engine, "new Date(1/0).getUTCSeconds()");
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);
    Ok(())
}

#[test]
fn date_proto_set_date() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(
        &mut engine,
        "let dt = new Date(2020, 07, 08, 09, 16, 15, 779); dt.setDate(21); dt.getDate()",
    );
    assert_eq!(Ok(Value::Rational(21f64)), actual);

    // Date wraps to previous month for 0.
    let actual = forward_val(
        &mut engine,
        "dt = new Date(2020, 07, 08, 09, 16, 15, 779); dt.setDate(0); dt.getDate()",
    );
    assert_eq!(Ok(Value::Rational(30f64)), actual);

    let actual = forward_val(
        &mut engine,
        "dt = new Date(2020, 07, 08, 09, 16, 15, 779); dt.setDate(1/0); dt.getDate()",
    );
    assert_eq!(Ok(Value::Rational(f64::NAN)), actual);

    Ok(())
}
