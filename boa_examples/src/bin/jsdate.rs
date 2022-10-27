use boa_engine::{object::builtins::JsDate, Context, JsResult, JsValue};

fn main() -> JsResult<()> {
    let context = &mut Context::default();

    let date = JsDate::new(context);

    // 823230245000.0
    JsDate::utc(
        &[
            JsValue::new(96),
            JsValue::new(1),
            JsValue::new(2),
            JsValue::new(3),
            JsValue::new(4),
            JsValue::new(5),
        ],
        context,
    )?;
    // reference date: 2022-07-16T06:27:32.087241439

    // sets day of the month to 24
    date.set_date(24, context)?;
    // 2022-07-24T06:27:11.567

    // sets date to 1st of January 2000
    date.set_full_year(&[2000.into(), 0.into(), 1.into()], context)?;
    // 2000-01-01T06:26:53.984

    // sets time to 10H:10M:10S:10mS
    date.set_hours(&[23.into(), 23.into(), 23.into(), 23.into()], context)?;
    // Is           2000-01-01T17:53:23.023
    // Should be    2000-01-01T23:23:23.023

    // sets milliseconds to 999
    date.set_milliseconds(999, context)?;
    // 2000-01-01T17:40:10.999

    // sets time to 12M:12S:12ms
    date.set_minutes(&[12.into(), 12.into(), 12.into()], context)?;
    // Is           2000-01-01T17:42:12.012
    // Should be    2000-01-01T17:12:12:012

    // sets month to 9 and day to 9
    date.set_month(&[9.into(), 9.into()], context)?;
    // 2000-10-09T04:42:12.012

    // set seconds to 59 and ms to 59
    date.set_seconds(&[59.into(), 59.into()], context)?;
    // 2000-10-09T04:42:59.059

    assert_eq!(
        date.to_json(context)?,
        JsValue::from("2000-10-09T17:42:59.059Z")
    );

    assert_eq!(
        date.to_date_string(context)?,
        JsValue::from("Mon Oct 09 2000")
    );

    assert_eq!(
        date.to_iso_string(context)?,
        JsValue::from("2000-10-09T17:42:59.059Z")
    );

    assert_eq!(
        date.to_time_string(context)?,
        JsValue::from("23:12:59 GMT+0530")
    );

    assert_eq!(
        date.to_string(context)?,
        JsValue::from("Mon Oct 09 2000 23:12:59 GMT+0530")
    );

    Ok(())
}
