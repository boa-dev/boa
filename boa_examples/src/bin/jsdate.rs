use boa_engine::{
    context::HostHooks, js_string, object::builtins::JsDate, Context, JsResult, JsValue,
};
use chrono::{DateTime, FixedOffset, LocalResult, NaiveDateTime, TimeZone};

struct CustomTimezone;

// This pins the local timezone to a system-agnostic value; in this case, UTC+3
impl HostHooks for CustomTimezone {
    fn local_from_utc(&self, utc: NaiveDateTime) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(3 * 3600)
            .unwrap()
            .from_utc_datetime(&utc)
    }

    fn local_from_naive_local(&self, local: NaiveDateTime) -> LocalResult<DateTime<FixedOffset>> {
        FixedOffset::east_opt(3 * 3600)
            .unwrap()
            .from_local_datetime(&local)
    }
}

fn main() -> JsResult<()> {
    let hooks: &dyn HostHooks = &CustomTimezone;
    let context = &mut Context::builder().host_hooks(hooks).build().unwrap();

    let timestamp = JsDate::utc(
        &[
            JsValue::new(96),
            JsValue::new(1),
            JsValue::new(2),
            JsValue::new(3),
            JsValue::new(4),
            JsValue::new(5),
        ],
        context,
    )?
    .as_number()
    .unwrap();

    assert_eq!(timestamp, 823230245000.0);

    // Gets the current time in UTC time.
    let date = JsDate::new(context);

    // sets day of the month to 24
    date.set_date(24, context)?;

    // sets date to 1st of January 2000
    date.set_full_year(&[2000.into(), 0.into(), 1.into()], context)?;

    // sets time to 10H:10M:10S:10mS
    date.set_hours(&[23.into(), 23.into(), 23.into(), 23.into()], context)?;

    // sets milliseconds to 999
    date.set_milliseconds(999, context)?;

    // sets time to 12M:12S:12ms
    date.set_minutes(&[12.into(), 12.into(), 12.into()], context)?;

    // sets month to 9 (the 10th) and day to 9
    date.set_month(&[9.into(), 9.into()], context)?;

    // set seconds to 59 and ms to 59
    date.set_seconds(&[59.into(), 59.into()], context)?;

    assert_eq!(
        date.to_json(context)?,
        JsValue::from(js_string!("2000-10-09T20:12:59.059Z"))
    );

    assert_eq!(
        date.to_date_string(context)?,
        JsValue::from(js_string!("Mon Oct 09 2000"))
    );

    assert_eq!(
        date.to_iso_string(context)?,
        JsValue::from(js_string!("2000-10-09T20:12:59.059Z"))
    );

    assert_eq!(
        date.to_time_string(context)?,
        JsValue::from(js_string!("23:12:59 GMT+0300"))
    );

    assert_eq!(
        date.to_string(context)?,
        JsValue::from(js_string!("Mon Oct 09 2000 23:12:59 GMT+0300"))
    );

    Ok(())
}
