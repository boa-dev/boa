#[cfg(test)]
mod tests;

use crate::{
    builtins::{
        function::{make_builtin_fn, make_constructor_fn},
        object::ObjectData,
        value::RcDate,
        ResultValue, Value,
    },
    exec::PreferredType,
    BoaProfiler, Interpreter,
};
use chrono::{prelude::*, LocalResult};
use std::fmt::Display;

#[inline]
fn is_zero_or_normal_opt(value: Option<f64>) -> bool {
    value
        .map(|value| value == 0f64 || value.is_normal())
        .unwrap_or(true)
}
macro_rules! check_normal_opt {
    ($($v:expr),+) => {
        $(is_zero_or_normal_opt($v.into()) &&)+ true
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(Option<NaiveDateTime>);

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.to_local() {
            Some(v) => write!(f, "{}", v),
            _ => write!(f, "Invalid Date"),
        }
    }
}

impl Default for Date {
    fn default() -> Self {
        Self(Some(Utc::now().naive_utc()))
    }
}

impl Date {
    /// The name of the object.
    pub(crate) const NAME: &'static str = "Date";

    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 7;

    /// The local date time.
    pub fn to_local(&self) -> Option<DateTime<Local>> {
        self.0
            .map(|utc| Local::now().timezone().from_utc_datetime(&utc))
    }

    // The UTC date time.
    pub fn to_utc(&self) -> Option<DateTime<Utc>> {
        self.0
            .map(|utc| Utc::now().timezone().from_utc_datetime(&utc))
    }

    /// The abstract operation `thisTimeValue` takes argument value.
    ///
    /// In following descriptions of functions that are properties of the Date prototype object, the phrase “this
    /// Date object” refers to the object that is the this value for the invocation of the function. If the `Type` of
    /// the this value is not `Object`, a `TypeError` exception is thrown. The phrase “this time value” within the
    /// specification of a method refers to the result returned by calling the abstract operation `thisTimeValue` with
    /// the this value of the method invocation passed as the argument.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thistimevalue
    #[inline]
    fn this_time_value_opt(value: &Value, _: &mut Interpreter) -> Option<RcDate> {
        match value {
            // 1. If Type(value) is Date, return value.
            Value::Date(ref date) => return Some(date.clone()),

            // 2. If Type(value) is Object and value has a [[DateData]] internal slot, then
            //    a. Assert: Type(value.[[DateData]]) is Date.
            //    b. Return value.[[DateData]].
            Value::Object(ref object) => {
                if let ObjectData::Date(ref date) = object.borrow().data {
                    return Some(date.clone());
                }
            }
            _ => {}
        }
        None
    }

    /// The abstract operation `thisTimeValue` takes argument value.
    ///
    /// In following descriptions of functions that are properties of the Date prototype object, the phrase “this
    /// Date object” refers to the object that is the this value for the invocation of the function. If the `Type` of
    /// the this value is not `Object`, a `TypeError` exception is thrown. The phrase “this time value” within the
    /// specification of a method refers to the result returned by calling the abstract operation `thisTimeValue` with
    /// the this value of the method invocation passed as the argument.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-thistimevalue
    #[inline]
    fn this_time_value(value: &Value, ctx: &mut Interpreter) -> Result<Date, Value> {
        match Self::this_time_value_opt(value, ctx) {
            Some(date) => Ok(Date(date.0)),
            _ => Err(ctx.construct_type_error("'this' is not a Date")),
        }
    }

    /// `Date()`
    ///
    /// Creates a JavaScript `Date` instance that represents a single moment in time in a platform-independent format.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date-constructor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/Date
    pub(crate) fn make_date(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        if this.is_global() {
            Self::make_date_string()
        } else if args.is_empty() {
            Self::make_date_now(this)
        } else if args.len() == 1 {
            Self::make_date_single(this, args, ctx)
        } else {
            Self::make_date_multiple(this, args, ctx)
        }
    }

    /// `Date()`
    ///
    /// The `Date()` function is used to create a string that represent the current date and time.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date-constructor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/Date
    pub(crate) fn make_date_string() -> ResultValue {
        Ok(Value::from(Local::now().to_rfc3339()))
    }

    /// `Date()`
    ///
    /// The newly-created `Date` object represents the current date and time as of the time of instantiation.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date-constructor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/Date
    pub(crate) fn make_date_now(this: &Value) -> ResultValue {
        let date = Date::default();
        this.set_data(ObjectData::Date(RcDate::from(date)));
        Ok(this.clone())
    }

    /// `Date(value)`
    ///
    /// The newly-created `Date` object represents the value provided to the constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date-constructor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/Date
    pub(crate) fn make_date_single(
        this: &Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        let value = &args[0];
        let tv = match Self::this_time_value_opt(value, ctx) {
            Some(dt) => {
                println!("{:?}", dt);
                dt.0
            }
            None => match &ctx.to_primitive(value, PreferredType::Default)? {
                Value::String(str) => match chrono::DateTime::parse_from_rfc3339(&str) {
                    Ok(dt) => Some(dt.naive_utc()),
                    _ => None,
                },
                tv => {
                    let tv = ctx.to_number(&tv)?;
                    let secs = (tv / 1_000f64) as i64;
                    let nsecs = ((tv % 1_000f64) * 1_000_000f64) as u32;
                    NaiveDateTime::from_timestamp_opt(secs, nsecs)
                }
            },
        };

        let date = Date(tv);
        this.set_data(ObjectData::Date(RcDate::from(date)));
        Ok(this.clone())
    }

    /// `Date(year, month [ , date [ , hours [ , minutes [ , seconds [ , ms ] ] ] ] ])`
    ///
    /// The newly-created `Date` object represents the date components provided to the constructor.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date-constructor
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/Date
    pub(crate) fn make_date_multiple(
        this: &Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        let year = ctx.to_number(&args[0])?;
        let month = ctx.to_number(&args[1])?;
        let day = args.get(2).map_or(Ok(1f64), |value| ctx.to_number(value))?;
        let hour = args.get(3).map_or(Ok(0f64), |value| ctx.to_number(value))?;
        let min = args.get(4).map_or(Ok(0f64), |value| ctx.to_number(value))?;
        let sec = args.get(5).map_or(Ok(0f64), |value| ctx.to_number(value))?;
        let milli = args.get(6).map_or(Ok(0f64), |value| ctx.to_number(value))?;

        // If any of the args are infinity or NaN, return an invalid date.
        if !check_normal_opt!(year, month, day, hour, min, sec, milli) {
            let date = Date(None);
            this.set_data(ObjectData::Date(RcDate::from(date)));
            return Ok(this.clone());
        }

        let year = year as i32;
        let month = month as u32;
        let day = day as u32;
        let hour = hour as u32;
        let min = min as u32;
        let sec = sec as u32;
        let milli = milli as u32;

        let year = if 0 <= year && year <= 99 {
            1900 + year
        } else {
            year
        };

        let final_date = NaiveDate::from_ymd_opt(year, month, day)
            .map(|naive_date| naive_date.and_hms_milli_opt(hour, min, sec, milli))
            .flatten()
            .map(
                |local| match Local::now().timezone().from_local_datetime(&local) {
                    LocalResult::Single(v) => Some(v.naive_utc()),
                    // JS simply hopes for the best
                    LocalResult::Ambiguous(v, _) => Some(v.naive_utc()),
                    _ => None,
                },
            )
            .flatten();

        let date = Date(final_date);
        this.set_data(ObjectData::Date(RcDate::from(date)));
        Ok(this.clone())
    }

    /// `Date.prototype.toString()`
    ///
    /// The `toString()` method returns a string representing the specified Date object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_string(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let dt_str = Self::this_time_value(this, ctx)?
            .to_local()
            .map(|f| f.to_rfc3339())
            .unwrap_or_else(|| "Invalid Date".to_string());
        Ok(Value::from(dt_str))
    }

    /// `Date.prototype.getDate()`
    ///
    /// The `getDate()` method returns the day of the month for the specified date according to local time.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getdate
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getDate
    #[inline]
    fn get_date(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::number(
            Self::this_time_value(this, ctx)?
                .to_local()
                .map_or(f64::NAN, |dt| dt.day() as f64),
        ))
    }

    /// `Date.prototype.getDay()`
    ///
    /// The `getDay()` method returns the day of the week for the specified date according to local time, where 0
    /// represents Sunday.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getday
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getDay
    #[inline]
    fn get_day(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::number(
            Self::this_time_value(this, ctx)?
                .to_local()
                .map_or(f64::NAN, |dt| {
                    let weekday = dt.weekday() as u32;
                    let weekday = (weekday + 1) % 7; // 0 represents Monday in Chrono
                    weekday as f64
                }),
        ))
    }

    /// `Date.prototype.getFullYear()`
    ///
    /// The `getFullYear()` method returns the year of the specified date according to local time.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getfullyear
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getFullYear
    #[inline]
    fn get_full_year(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::number(
            Self::this_time_value(this, ctx)?
                .to_local()
                .map_or(f64::NAN, |dt| dt.year() as f64),
        ))
    }

    /// `Date.prototype.getHours()`
    ///
    /// The `getHours()` method returns the hour for the specified date, according to local time.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.gethours
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getHours
    #[inline]
    fn get_hours(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::number(
            Self::this_time_value(this, ctx)?
                .to_local()
                .map_or(f64::NAN, |dt| dt.hour() as f64),
        ))
    }

    /// `Date.prototype.getMilliseconds()`
    ///
    /// The `getMilliseconds()` method returns the milliseconds in the specified date according to local time.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getmilliseconds
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getMilliseconds
    #[inline]
    fn get_milliseconds(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::number(
            Self::this_time_value(this, ctx)?
                .to_local()
                .map_or(f64::NAN, |dt| dt.nanosecond() as f64 / 1_000_000f64),
        ))
    }

    /// `Date.prototype.getMinutes()`
    ///
    /// The `getMinutes()` method returns the minutes in the specified date according to local time.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getminutes
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getMinutes
    #[inline]
    fn get_minutes(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::number(
            Self::this_time_value(this, ctx)?
                .to_local()
                .map_or(f64::NAN, |dt| dt.minute() as f64),
        ))
    }

    /// `Date.prototype.getMonth()`
    ///
    /// The `getMonth()` method returns the month in the specified date according to local time, as a zero-based value
    /// (where zero indicates the first month of the year).
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getmonth
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getMonth
    #[inline]
    fn get_month(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::number(
            Self::this_time_value(this, ctx)?
                .to_local()
                .map_or(f64::NAN, |dt| dt.month() as f64),
        ))
    }

    /// `Date.prototype.getSeconds()`
    ///
    /// The `getSeconds()` method returns the seconds in the specified date according to local time.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getseconds
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getSeconds
    #[inline]
    fn get_seconds(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::number(
            Self::this_time_value(this, ctx)?
                .to_local()
                .map_or(f64::NAN, |dt| dt.second() as f64),
        ))
    }

    /// `Date.prototype.getTime()`
    ///
    /// The `getTime()` method returns the number of milliseconds since the Unix Epoch.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.gettime
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getTime
    #[inline]
    fn get_time(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Ok(Value::number(
            Self::this_time_value(this, ctx)?
                .to_utc()
                .map_or(f64::NAN, |dt| dt.timestamp_millis() as f64),
        ))
    }

    /// `Date.prototype.getTimeZoneOffset()`
    ///
    /// The getTimezoneOffset() method returns the time zone difference, in minutes, from current locale (host system
    /// settings) to UTC.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.gettimezoneoffset
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getTimezoneOffset
    #[inline]
    fn get_timezone_offset(_: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
        let offset_seconds = chrono::Local::now().offset().local_minus_utc() as f64;
        let offset_minutes = offset_seconds / 60f64;
        Ok(Value::number(offset_minutes))
    }

    /// `Date.now()`
    ///
    /// The static `Date.now()` method returns the number of milliseconds elapsed since January 1, 1970 00:00:00 UTC.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.now
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/now
    pub(crate) fn now(_: &Value, _: &[Value], _: &mut Interpreter) -> ResultValue {
        Ok(Value::from(Utc::now().timestamp_millis() as f64))
    }

    /// `Date.parse()`
    ///
    /// The `Date.parse()` method parses a string representation of a date, and returns the number of milliseconds since
    /// January 1, 1970, 00:00:00 UTC or NaN if the string is unrecognized or, in some cases, contains illegal date
    /// values.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.parse
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/parse
    pub(crate) fn parse(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        // This method is implementation-defined and discouraged, so we just require the same format as the string
        // constructor.

        if args.is_empty() {
            return Ok(Value::number(f64::NAN));
        }

        match DateTime::parse_from_rfc3339(&ctx.to_string(&args[0])?) {
            Ok(v) => Ok(Value::number(v.naive_utc().timestamp_millis() as f64)),
            _ => Ok(Value::number(f64::NAN)),
        }
    }

    /// `Date.UTC()`
    ///
    /// The `Date.UTC()` method accepts parameters similar to the `Date` constructor, but treats them as UTC.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.utc
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/UTC
    pub(crate) fn utc(_: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let year = args
            .get(0)
            .map_or(Ok(f64::NAN), |value| ctx.to_number(value))?;
        let month = args.get(1).map_or(Ok(1f64), |value| ctx.to_number(value))?;
        let day = args.get(2).map_or(Ok(1f64), |value| ctx.to_number(value))?;
        let hour = args.get(3).map_or(Ok(0f64), |value| ctx.to_number(value))?;
        let min = args.get(4).map_or(Ok(0f64), |value| ctx.to_number(value))?;
        let sec = args.get(5).map_or(Ok(0f64), |value| ctx.to_number(value))?;
        let milli = args.get(6).map_or(Ok(0f64), |value| ctx.to_number(value))?;

        if !check_normal_opt!(year, month, day, hour, min, sec, milli) {
            return Ok(Value::number(f64::NAN));
        }

        let year = year as i32;
        let month = month as u32;
        let day = day as u32;
        let hour = hour as u32;
        let min = min as u32;
        let sec = sec as u32;
        let milli = milli as u32;

        let year = if 0 <= year && year <= 99 {
            1900 + year
        } else {
            year
        };

        NaiveDate::from_ymd_opt(year, month, day)
            .map(|f| f.and_hms_milli_opt(hour, min, sec, milli))
            .flatten()
            .map_or(Ok(Value::number(f64::NAN)), |f| {
                Ok(Value::number(f.timestamp_millis() as f64))
            })
    }

    pub(crate) fn create(global: &Value) -> Value {
        let prototype = Value::new_object(Some(global));

        make_builtin_fn(Self::to_string, "toString", &prototype, 0);
        make_builtin_fn(Self::get_date, "getDate", &prototype, 0);
        make_builtin_fn(Self::get_day, "getDay", &prototype, 0);
        make_builtin_fn(Self::get_full_year, "getFullYear", &prototype, 0);
        make_builtin_fn(Self::get_hours, "getHours", &prototype, 0);
        make_builtin_fn(Self::get_milliseconds, "getMilliseconds", &prototype, 0);
        make_builtin_fn(Self::get_minutes, "getMinutes", &prototype, 0);
        make_builtin_fn(Self::get_month, "getMonth", &prototype, 0);
        make_builtin_fn(Self::get_seconds, "getSeconds", &prototype, 0);
        make_builtin_fn(Self::get_time, "getTime", &prototype, 0);
        make_builtin_fn(
            Self::get_timezone_offset,
            "getTimezoneOffset",
            &prototype,
            0,
        );

        let constructor = make_constructor_fn(
            Self::NAME,
            Self::LENGTH,
            Self::make_date,
            global,
            prototype,
            true,
            true,
        );

        make_builtin_fn(Self::now, "now", &constructor, 0);
        make_builtin_fn(Self::parse, "parse", &constructor, 1);
        make_builtin_fn(Self::utc, "UTC", &constructor, 7);
        constructor
    }

    /// Initialise the `Date` object on the global object.
    #[inline]
    pub(crate) fn init(global: &Value) -> (&str, Value) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        (Self::NAME, Self::create(global))
    }
}
