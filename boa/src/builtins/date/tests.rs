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
fn date_proto_get_date_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(&mut engine, "new Date(2020, 07, 08, 09).getDate()");

    assert_eq!(Ok(Value::Rational(08f64)), actual);
    Ok(())
}

#[test]
fn date_proto_get_day_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(&mut engine, "new Date(2020, 07, 08, 09).getDay()");

    assert_eq!(Ok(Value::Rational(3f64)), actual);
    Ok(())
}

#[test]
fn date_proto_get_full_year_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(&mut engine, "new Date(2020, 07).getFullYear()");

    assert_eq!(Ok(Value::Rational(2020f64)), actual);
    Ok(())
}

#[test]
fn date_proto_get_hours_call() -> Result<(), Box<dyn std::error::Error>> {
    let realm = Realm::create();
    let mut engine = Interpreter::new(realm);

    let actual = forward_val(&mut engine, "new Date(2020, 07, 08, 09, 16).getHours()");

    assert_eq!(Ok(Value::Rational(09f64)), actual);
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
    Ok(())
}
