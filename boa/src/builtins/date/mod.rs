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
use chrono::{prelude::*, Duration, LocalResult};
use std::fmt::Display;

const NANOS_IN_MS: f64 = 1_000_000f64;

#[inline]
fn is_zero_or_normal_opt(value: Option<f64>) -> bool {
    value
        .map(|value| value == 0f64 || value.is_normal())
        .unwrap_or(true)
}

#[inline]
fn ignore_ambiguity<T>(result: LocalResult<T>) -> Option<T> {
    match result {
        LocalResult::Ambiguous(v, _) => Some(v),
        LocalResult::Single(v) => Some(v),
        LocalResult::None => None,
    }
}

/// Some JS functions allow completely invalid dates, and the runtime is expected to make sense of this. This function
/// constrains a date to correct values.
fn fix_date(year: &mut i32, month: &mut i32, day: &mut i32) {
    #[inline]
    fn num_days_in(year: i32, month: u32) -> i32 {
        let month = month + 1; // zero-based for calculations
        NaiveDate::from_ymd(
            match month {
                12 => year + 1,
                _ => year,
            },
            match month {
                12 => 1,
                _ => month + 1,
            },
            1,
        )
        .signed_duration_since(NaiveDate::from_ymd(year, month, 1))
        .num_days() as i32
    }

    #[inline]
    fn fix_month(year: &mut i32, month: &mut i32) {
        *year += *month / 12;
        *month = if *month < 0 {
            *year -= 1;
            11 + (*month + 1) % 12
        } else {
            *month % 12
        }
    }

    #[inline]
    fn fix_day(year: &mut i32, month: &mut i32, day: &mut i32) {
        fix_month(year, month);
        loop {
            if *day < 0 {
                *month -= 1;
                fix_month(year, month);
                *day = num_days_in(*year, *month as u32) + *day;
            } else {
                let num_days = num_days_in(*year, *month as u32);
                if *day >= num_days {
                    *day -= num_days_in(*year, *month as u32);
                    *month += 1;
                    fix_month(year, month);
                } else {
                    break;
                }
            }
        }
    }

    fix_day(year, month, day);
}

macro_rules! check_normal_opt {
    ($($v:expr),+) => {
        $(is_zero_or_normal_opt($v.into()) &&)+ true
    };
}

macro_rules! getter_method {
    ($(#[$outer:meta])* fn $name:ident ($tz:ident, $var:ident) $get:expr) => {
        $(#[$outer])*
        fn $name(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
            Ok(Value::number(
                Self::this_time_value(this, ctx)?
                    .$tz()
                    .map_or(f64::NAN, |$var| $get),
            ))
        }
    };
}

macro_rules! setter_method {
    ($(#[$outer:meta])* fn $name:ident ($tz: ident, $date_time:ident, $var:ident[$count:literal]) $mutate:expr) => {
        $(#[$outer])*
        fn $name(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
            // If the first arg is not present or NaN, the Date becomes NaN itself.
            fn get_arg(i: usize, args: &[Value], ctx: &mut Interpreter) -> Option<f64> {
                args
                .get(i)
                .map(|value| {
                    ctx.to_numeric_number(value).map_or_else(
                        |_| None,
                        |value| {
                            if value == 0f64 || value.is_normal() {
                                Some(value)
                            } else {
                                None
                            }
                        },
                    )
                })
                .flatten()
            }

            let mut $var = [None; $count];
            for i in 0..$count {
                $var[i] = get_arg(i, args, ctx);
            }

            let inner = Date::this_time_value(this, ctx)?.$tz();
            let new_value = inner.map(|$date_time| $mutate).flatten();
            let new_value = new_value.map(|date_time| date_time.naive_utc());
            this.set_data(ObjectData::Date(RcDate::from(Date(new_value))));

            Ok(Value::number(
                Self::this_time_value(this, ctx)?
                    .to_utc()
                    .map_or(f64::NAN, |f| f.timestamp_millis() as f64),
            ))
        }
    };
    ($(#[$outer:meta])* fn $name:ident ($var:ident[$count:literal]) $mutate:expr) => {
        $(#[$outer])*
        fn $name(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
            // If the first arg is not present or NaN, the Date becomes NaN itself.
            fn get_arg(i: usize, args: &[Value], ctx: &mut Interpreter) -> Option<f64> {
                args
                .get(i)
                .map(|value| {
                    ctx.to_numeric_number(value).map_or_else(
                        |_| None,
                        |value| {
                            if value == 0f64 || value.is_normal() {
                                Some(value)
                            } else {
                                None
                            }
                        },
                    )
                })
                .flatten()
            }

            let mut $var = [None; $count];
            for i in 0..$count {
                $var[i] = get_arg(i, args, ctx);
            }

            let new_value = $mutate;
            let new_value = new_value.map(|date_time| date_time.naive_utc());
            this.set_data(ObjectData::Date(RcDate::from(Date(new_value))));

            Ok(Value::number(
                Self::this_time_value(this, ctx)?
                    .to_utc()
                    .map_or(f64::NAN, |f| f.timestamp_millis() as f64),
            ))
        }
    };
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(Option<NaiveDateTime>);

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.to_local() {
            Some(v) => write!(f, "{}", v.format("%a %b %d %Y %H:%M:%S GMT%:z")),
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
    fn this_time_value(value: &Value, ctx: &mut Interpreter) -> Result<RcDate, Value> {
        match value {
            // 1. If Type(value) is Date, return value.
            Value::Date(ref date) => Ok(date.clone()),

            // 2. If Type(value) is Object and value has a [[DateData]] internal slot, then
            //    a. Assert: Type(value.[[DateData]]) is Date.
            //    b. Return value.[[DateData]].
            Value::Object(ref object) => {
                if let ObjectData::Date(ref date) = object.borrow().data {
                    Ok(date.clone())
                } else {
                    Err(ctx.construct_type_error("'this' is not a Date"))
                }
            }
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
        let tv = match Self::this_time_value(value, ctx) {
            Ok(dt) => dt.0,
            _ => match &ctx.to_primitive(value, PreferredType::Default)? {
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

        let final_date = NaiveDate::from_ymd_opt(year, month + 1, day)
            .map(|naive_date| naive_date.and_hms_milli_opt(hour, min, sec, milli))
            .flatten()
            .map(|local| ignore_ambiguity(Local.from_local_datetime(&local)))
            .flatten()
            .map(|local| local.naive_utc());

        let date = Date(final_date);
        this.set_data(ObjectData::Date(RcDate::from(date)));
        Ok(this.clone())
    }

    getter_method! {
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
        fn get_date(to_local, dt) { dt.day() as f64 }
    }

    getter_method! {
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
        fn get_day(to_local, dt) {
            let weekday = dt.weekday() as u32;
            let weekday = (weekday + 1) % 7; // 0 represents Monday in Chrono
            weekday as f64
        }
    }

    getter_method! {
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
        fn get_full_year(to_local, dt) { dt.year() as f64 }
    }

    getter_method! {
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
        fn get_hours(to_local, dt) { dt.hour() as f64 }
    }

    getter_method! {
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
        fn get_milliseconds(to_local, dt) { dt.nanosecond() as f64 / NANOS_IN_MS }
    }

    getter_method! {
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
        fn get_minutes(to_local, dt) { dt.minute() as f64 }
    }

    getter_method! {
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
        fn get_month(to_local, dt) { dt.month0() as f64 }
    }

    getter_method! {
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
        fn get_seconds(to_local, dt) { dt.second() as f64 }
    }

    getter_method! {
        /// `Date.prototype.getYear()`
        ///
        /// The getYear() method returns the year in the specified date according to local time. Because getYear() does not
        /// return full years ("year 2000 problem"), it is no longer used and has been replaced by the getFullYear() method.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getyear
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getYear
        fn get_year(to_local, dt) { dt.year() as f64 - 1900f64 }
    }

    getter_method! {
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
        fn get_time(to_utc, dt) { dt.timestamp_millis() as f64 }
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

    getter_method! {
        /// `Date.prototype.getUTCDate()`
        ///
        /// The `getUTCDate()` method returns the day (date) of the month in the specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getutcdate
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCDate
        fn get_utc_date(to_utc, dt) { dt.day() as f64 }
    }

    getter_method! {
        /// `Date.prototype.getUTCDay()`
        ///
        /// The `getUTCDay()` method returns the day of the week in the specified date according to universal time, where 0
        /// represents Sunday.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getutcday
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCDay
        fn get_utc_day(to_utc, dt) {
            let weekday = dt.weekday() as u32;
            let weekday = (weekday + 1) % 7; // 0 represents Monday in Chrono
            weekday as f64
        }
    }

    getter_method! {
        /// `Date.prototype.getUTCFullYear()`
        ///
        /// The `getUTCFullYear()` method returns the year in the specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getutcfullyear
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCFullYear
        fn get_utc_full_year(to_utc, dt) { dt.year() as f64 }
    }

    getter_method! {
        /// `Date.prototype.getUTCHours()`
        ///
        /// The `getUTCHours()` method returns the hours in the specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getutchours
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCHours
        fn get_utc_hours(to_utc, dt) { dt.hour() as f64 }
    }

    getter_method! {
        /// `Date.prototype.getUTCMilliseconds()`
        ///
        /// The `getUTCMilliseconds()` method returns the milliseconds portion of the time object's value.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getutcmilliseconds
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCMilliseconds
        fn get_utc_milliseconds(to_utc, dt) { dt.nanosecond() as f64 / NANOS_IN_MS }
    }

    getter_method! {
        /// `Date.prototype.getUTCMinutes()`
        ///
        /// The `getUTCMinutes()` method returns the minutes in the specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getutcminutes
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCMinutes
        fn get_utc_minutes(to_utc, dt) { dt.minute() as f64 }
    }

    getter_method! {
        /// `Date.prototype.getUTCMonth()`
        ///
        /// The `getUTCMonth()` returns the month of the specified date according to universal time, as a zero-based value
        /// (where zero indicates the first month of the year).
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getutcmonth
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCMonth
        fn get_utc_month(to_utc, dt) { dt.month0() as f64 }
    }

    getter_method! {
        /// `Date.prototype.getUTCSeconds()`
        ///
        /// The `getUTCSeconds()` method returns the seconds in the specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getutcseconds
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getUTCSeconds
        fn get_utc_seconds(to_utc, dt) { dt.second() as f64 }
    }

    setter_method! {
        /// `Date.prototype.setDate()`
        ///
        /// The `setDate()` method sets the day of the `Date` object relative to the beginning of the currently set
        /// month.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setdate
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setDate
        fn set_date (to_local, date_time, args[1]) {
            args[0].map_or(None, |day| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let local = date_time.naive_local();
                let mut year = local.year();
                let mut month = local.month0() as i32;
                let mut day = day as i32 - 1;

                fix_date(&mut year, &mut month, &mut day);
                ignore_ambiguity(Local.ymd_opt(year, month as u32 + 1, day as u32 + 1).and_time(local.time()))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setFullYear()`
        ///
        /// The `setFullYear()` method sets the full year for a specified date according to local time. Returns new
        /// timestamp.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setfullyear
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setFullYear
        fn set_full_year (to_local, date_time, args[3]) {
            args[0].map_or(None, |year| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let local = date_time.naive_local();
                let mut year = year as i32;
                let mut month = args[1].unwrap_or_else(|| local.month0() as f64) as i32;
                let mut day = args[2].unwrap_or_else(|| local.day() as f64) as i32 - 1;

                fix_date(&mut year, &mut month, &mut day);
                ignore_ambiguity(Local.ymd_opt(year, month as u32 + 1, day as u32 + 1).and_time(local.time()))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setHours()`
        ///
        /// The `setHours()` method sets the hours for a specified date according to local time, and returns the number
        /// of milliseconds since January 1, 1970 00:00:00 UTC until the time represented by the updated `Date`
        /// instance.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.sethours
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setHours
        fn set_hours (to_local, date_time, args[4]) {
            args[0].map_or(None, |hour| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let local = date_time.naive_local();
                let hour = hour as i64;
                let minute = args[1].map_or_else(|| local.minute() as i64, |minute| minute as i64);
                let second = args[2].map_or_else(|| local.second() as i64, |second| second as i64);
                let ms = args[3].map_or_else(|| (local.nanosecond() as f64 / NANOS_IN_MS) as i64, |ms| ms as i64);

                let duration = Duration::hours(hour) + Duration::minutes(minute) + Duration::seconds(second) + Duration::milliseconds(ms);
                let local = local.date().and_hms(0, 0, 0).checked_add_signed(duration);
                local.map_or(None, |local| ignore_ambiguity(Local.from_local_datetime(&local)))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setMilliseconds()`
        ///
        /// The `setMilliseconds()` method sets the milliseconds for a specified date according to local time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setmilliseconds
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setMilliseconds
        fn set_milliseconds (to_local, date_time, args[1]) {
            args[0].map_or(None, |ms| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let local = date_time.naive_local();
                let hour = local.hour() as i64;
                let minute = local.minute() as i64;
                let second = local.second() as i64;
                let ms = ms as i64;

                let duration = Duration::hours(hour) + Duration::minutes(minute) + Duration::seconds(second) + Duration::milliseconds(ms);
                let local = local.date().and_hms(0, 0, 0).checked_add_signed(duration);
                local.map_or(None, |local| ignore_ambiguity(Local.from_local_datetime(&local)))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setMinutes()`
        ///
        /// The `setMinutes()` method sets the minutes for a specified date according to local time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setminutes
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setMinutes
        fn set_minutes (to_local, date_time, args[3]) {
            args[0].map_or(None, |minute| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let local = date_time.naive_local();
                let hour = local.hour() as i64;
                let minute = minute as i64;
                let second = args[1].map_or_else(|| local.second() as i64, |second| second as i64);
                let ms = args[2].map_or_else(|| (local.nanosecond() as f64 / NANOS_IN_MS) as i64, |ms| ms as i64);

                let duration = Duration::hours(hour) + Duration::minutes(minute) + Duration::seconds(second) + Duration::milliseconds(ms);
                let local = local.date().and_hms(0, 0, 0).checked_add_signed(duration);
                local.map_or(None, |local| ignore_ambiguity(Local.from_local_datetime(&local)))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setMonth()`
        ///
        /// The `setMonth()` method sets the month for a specified date according to the currently set year.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setmonth
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setMonth
        fn set_month (to_local, date_time, args[2]) {
            args[0].map_or(None, |month| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let local = date_time.naive_local();
                let mut year = local.year();
                let mut month = month as i32;
                let mut day = args[1].unwrap_or_else(|| local.day() as f64) as i32 - 1;

                fix_date(&mut year, &mut month, &mut day);
                ignore_ambiguity(Local.ymd_opt(year, month as u32 + 1, day as u32 + 1).and_time(local.time()))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setSeconds()`
        ///
        /// The `setSeconds()` method sets the seconds for a specified date according to local time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setseconds
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setSeconds
        fn set_seconds (to_local, date_time, args[2]) {
            args[0].map_or(None, |second| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let local = date_time.naive_local();
                let hour = local.hour() as i64;
                let minute = local.minute() as i64;
                let second = second as i64;
                let ms = args[1].map_or_else(|| (local.nanosecond() as f64 / NANOS_IN_MS) as i64, |ms| ms as i64);

                let duration = Duration::hours(hour) + Duration::minutes(minute) + Duration::seconds(second) + Duration::milliseconds(ms);
                let local = local.date().and_hms(0, 0, 0).checked_add_signed(duration);
                local.map_or(None, |local| ignore_ambiguity(Local.from_local_datetime(&local)))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setYear()`
        ///
        /// The `setYear()` method sets the year for a specified date according to local time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setyear
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setYear
        fn set_year (to_local, date_time, args[3]) {
            args[0].map_or(None, |year| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let local = date_time.naive_local();
                let mut year = year as i32;
                year += if 0 <= year && year <= 99 {
                    1900
                } else {
                    0
                };

                local.with_year(year).map(|local| ignore_ambiguity(Local.from_local_datetime(&local))).flatten()
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setTime()`
        ///
        /// The `setTime()` method sets the Date object to the time represented by a number of milliseconds since
        /// January 1, 1970, 00:00:00 UTC.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.settime
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setTime
        fn set_time (args[1]) {
            args[0].map_or(None, |tv| {
                let secs = (tv / 1_000f64) as i64;
                let nsecs = ((tv % 1_000f64) * 1_000_000f64) as u32;
                ignore_ambiguity(Local.timestamp_opt(secs, nsecs))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setUTCDate()`
        ///
        /// The `setUTCDate()` method sets the day of the month for a specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setutcdate
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCDate
        fn set_utc_date (to_utc, date_time, args[1]) {
            args[0].map_or(None, |day| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let utc = date_time.naive_utc();
                let mut year = utc.year();
                let mut month = utc.month0() as i32;
                let mut day = day as i32 - 1;

                fix_date(&mut year, &mut month, &mut day);
                ignore_ambiguity(Utc.ymd_opt(year, month as u32 + 1, day as u32 + 1).and_time(utc.time()))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setFullYear()`
        ///
        /// The `setFullYear()` method sets the full year for a specified date according to local time. Returns new
        /// timestamp.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setutcfullyear
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCFullYear
        fn set_utc_full_year (to_utc, date_time, args[3]) {
            args[0].map_or(None, |year| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let utc = date_time.naive_utc();
                let mut year = year as i32;
                let mut month = args[1].unwrap_or_else(|| utc.month0() as f64) as i32;
                let mut day = args[2].unwrap_or_else(|| utc.day() as f64) as i32 - 1;

                fix_date(&mut year, &mut month, &mut day);
                ignore_ambiguity(Utc.ymd_opt(year, month as u32 + 1, day as u32 + 1).and_time(utc.time()))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setUTCHours()`
        ///
        /// The `setUTCHours()` method sets the hour for a specified date according to universal time, and returns the
        /// number of milliseconds since  January 1, 1970 00:00:00 UTC until the time represented by the updated `Date`
        /// instance.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setutchours
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCHours
        fn set_utc_hours (to_utc, date_time, args[4]) {
            args[0].map_or(None, |hour| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let utc = date_time.naive_utc();
                let hour = hour as i64;
                let minute = args[1].map_or_else(|| utc.minute() as i64, |minute| minute as i64);
                let second = args[2].map_or_else(|| utc.second() as i64, |second| second as i64);
                let ms = args[3].map_or_else(|| (utc.nanosecond() as f64 / NANOS_IN_MS) as i64, |ms| ms as i64);

                let duration = Duration::hours(hour) + Duration::minutes(minute) + Duration::seconds(second) + Duration::milliseconds(ms);
                let utc = utc.date().and_hms(0, 0, 0).checked_add_signed(duration);
                utc.map(|utc| Utc.from_utc_datetime(&utc))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setUTCMilliseconds()`
        ///
        /// The `setUTCMilliseconds()` method sets the milliseconds for a specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setutcmilliseconds
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCMilliseconds
        fn set_utc_milliseconds (to_utc, date_time, args[1]) {
            args[0].map_or(None, |ms| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let utc = date_time.naive_utc();
                let hour = utc.hour() as i64;
                let minute = utc.minute() as i64;
                let second = utc.second() as i64;
                let ms = ms as i64;

                let duration = Duration::hours(hour) + Duration::minutes(minute) + Duration::seconds(second) + Duration::milliseconds(ms);
                let utc = utc.date().and_hms(0, 0, 0).checked_add_signed(duration);
                utc.map(|utc| Utc.from_utc_datetime(&utc))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setUTCMinutes()`
        ///
        /// The `setUTCMinutes()` method sets the minutes for a specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setutcminutes
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCMinutes
        fn set_utc_minutes (to_utc, date_time, args[3]) {
            args[0].map_or(None, |minute| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let utc = date_time.naive_utc();
                let hour = utc.hour() as i64;
                let minute = minute as i64;
                let second = args[1].map_or_else(|| utc.second() as i64, |second| second as i64);
                let ms = args[2].map_or_else(|| (utc.nanosecond() as f64 / NANOS_IN_MS) as i64, |ms| ms as i64);

                let duration = Duration::hours(hour) + Duration::minutes(minute) + Duration::seconds(second) + Duration::milliseconds(ms);
                let utc = utc.date().and_hms(0, 0, 0).checked_add_signed(duration);
                utc.map(|utc| Utc.from_utc_datetime(&utc))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setUTCMonth()`
        ///
        /// The `setUTCMonth()` method sets the month for a specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setutcmonth
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCMonth
        fn set_utc_month (to_utc, date_time, args[2]) {
            args[0].map_or(None, |month| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let utc = date_time.naive_utc();
                let mut year = utc.year();
                let mut month = month as i32;
                let mut day = args[1].unwrap_or_else(|| utc.day() as f64) as i32 - 1;

                fix_date(&mut year, &mut month, &mut day);
                ignore_ambiguity(Utc.ymd_opt(year, month as u32 + 1, day as u32 + 1).and_time(utc.time()))
            })
        }
    }

    setter_method! {
        /// `Date.prototype.setUTCSeconds()`
        ///
        /// The `setUTCSeconds()` method sets the seconds for a specified date according to universal time.
        ///
        /// More information:
        ///  - [ECMAScript reference][spec]
        ///  - [MDN documentation][mdn]
        ///
        /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.setutcseconds
        /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/setUTCSeconds
        fn set_utc_seconds (to_utc, date_time, args[2]) {
            args[0].map_or(None, |second| {
                // Setters have to work in naive time because chrono [correctly] deals with DST, where JS does not.
                let utc = date_time.naive_utc();
                let hour = utc.hour() as i64;
                let minute = utc.minute() as i64;
                let second = second as i64;
                let ms = args[1].map_or_else(|| (utc.nanosecond() as f64 / NANOS_IN_MS) as i64, |ms| ms as i64);

                let duration = Duration::hours(hour) + Duration::minutes(minute) + Duration::seconds(second) + Duration::milliseconds(ms);
                let utc = utc.date().and_hms(0, 0, 0).checked_add_signed(duration);
                utc.map(|utc| Utc.from_utc_datetime(&utc))
            })
        }
    }

    /// `Date.prototype.toDateString()`
    ///
    /// The `toDateString()` method returns the date portion of a Date object in English.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.todatestring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toDateString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_date_string(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let dt_str = Self::this_time_value(this, ctx)?
            .to_local()
            .map(|date_time| date_time.format("%a %b %d %Y").to_string())
            .unwrap_or_else(|| "Invalid Date".to_string());
        Ok(Value::from(dt_str))
    }

    /// `Date.prototype.toGMTString()`
    ///
    /// The `toGMTString()` method converts a date to a string, using Internet Greenwich Mean Time (GMT) conventions.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.togmtstring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toGMTString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_gmt_string(
        this: &Value,
        args: &[Value],
        ctx: &mut Interpreter,
    ) -> ResultValue {
        Self::to_utc_string(this, args, ctx)
    }

    /// `Date.prototype.toISOString()`
    ///
    /// The `toISOString()` method returns a string in simplified extended ISO format (ISO 8601).
    ///
    /// More information:
    ///  - [ISO 8601][iso8601]
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [iso8601]: http://en.wikipedia.org/wiki/ISO_8601
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.toisostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toISOString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_iso_string(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let dt_str = Self::this_time_value(this, ctx)?
            .to_utc()
            // RFC 3389 uses +0.00 for UTC, where JS expects Z, so we can't use the built-in chrono function.
            .map(|f| f.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string())
            .unwrap_or_else(|| "Invalid Date".to_string());
        Ok(Value::from(dt_str))
    }

    /// `Date.prototype.toJSON()`
    ///
    /// The `toJSON()` method returns a string representation of the `Date` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.tojson
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toJSON
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_json(this: &Value, args: &[Value], ctx: &mut Interpreter) -> ResultValue {
        Self::to_iso_string(this, args, ctx)
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
            .map(|date_time| date_time.format("%a %b %d %Y %H:%M:%S GMT%:z").to_string())
            .unwrap_or_else(|| "Invalid Date".to_string());
        Ok(Value::from(dt_str))
    }

    /// `Date.prototype.toTimeString()`
    ///
    /// The `toTimeString()` method returns the time portion of a Date object in human readable form in American
    /// English.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.totimestring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toTimeString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_time_string(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let dt_str = Self::this_time_value(this, ctx)?
            .to_local()
            .map(|date_time| date_time.format("%H:%M:%S GMT%:z").to_string())
            .unwrap_or_else(|| "Invalid Date".to_string());
        Ok(Value::from(dt_str))
    }

    /// `Date.prototype.toUTCString()`
    ///
    /// The `toUTCString()` method returns a string representing the specified Date object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.toutcstring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toUTCString
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_utc_string(this: &Value, _: &[Value], ctx: &mut Interpreter) -> ResultValue {
        let dt_str = Self::this_time_value(this, ctx)?
            .to_utc()
            .map(|date_time| date_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .unwrap_or_else(|| "Invalid Date".to_string());
        Ok(Value::from(dt_str))
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

        NaiveDate::from_ymd_opt(year, month + 1, day)
            .map(|f| f.and_hms_milli_opt(hour, min, sec, milli))
            .flatten()
            .map_or(Ok(Value::number(f64::NAN)), |f| {
                Ok(Value::number(f.timestamp_millis() as f64))
            })
    }

    /// Initialise the `Date` object on the global object.
    #[inline]
    pub(crate) fn init(interpreter: &mut Interpreter) -> (&'static str, Value) {
        let global = interpreter.global();
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let prototype = Value::new_object(Some(global));

        make_builtin_fn(Self::get_date, "getDate", &prototype, 0);
        make_builtin_fn(Self::get_day, "getDay", &prototype, 0);
        make_builtin_fn(Self::get_full_year, "getFullYear", &prototype, 0);
        make_builtin_fn(Self::get_hours, "getHours", &prototype, 0);
        make_builtin_fn(Self::get_milliseconds, "getMilliseconds", &prototype, 0);
        make_builtin_fn(Self::get_minutes, "getMinutes", &prototype, 0);
        make_builtin_fn(Self::get_month, "getMonth", &prototype, 0);
        make_builtin_fn(Self::get_seconds, "getSeconds", &prototype, 0);
        make_builtin_fn(Self::get_time, "getTime", &prototype, 0);
        make_builtin_fn(Self::get_year, "getYear", &prototype, 0);
        make_builtin_fn(
            Self::get_timezone_offset,
            "getTimezoneOffset",
            &prototype,
            0,
        );
        make_builtin_fn(Self::get_utc_date, "getUTCDate", &prototype, 0);
        make_builtin_fn(Self::get_utc_day, "getUTCDay", &prototype, 0);
        make_builtin_fn(Self::get_utc_full_year, "getUTCFullYear", &prototype, 0);
        make_builtin_fn(Self::get_utc_hours, "getUTCHours", &prototype, 0);
        make_builtin_fn(
            Self::get_utc_milliseconds,
            "getUTCMilliseconds",
            &prototype,
            0,
        );
        make_builtin_fn(Self::get_utc_minutes, "getUTCMinutes", &prototype, 0);
        make_builtin_fn(Self::get_utc_month, "getUTCMonth", &prototype, 0);
        make_builtin_fn(Self::get_utc_seconds, "getUTCSeconds", &prototype, 0);
        make_builtin_fn(Self::set_date, "setDate", &prototype, 1);
        make_builtin_fn(Self::set_full_year, "setFullYear", &prototype, 1);
        make_builtin_fn(Self::set_hours, "setHours", &prototype, 1);
        make_builtin_fn(Self::set_milliseconds, "setMilliseconds", &prototype, 1);
        make_builtin_fn(Self::set_minutes, "setMinutes", &prototype, 1);
        make_builtin_fn(Self::set_month, "setMonth", &prototype, 1);
        make_builtin_fn(Self::set_seconds, "setSeconds", &prototype, 1);
        make_builtin_fn(Self::set_year, "setYear", &prototype, 1);
        make_builtin_fn(Self::set_time, "setTime", &prototype, 1);
        make_builtin_fn(Self::set_utc_date, "setUTCDate", &prototype, 1);
        make_builtin_fn(Self::set_utc_full_year, "setUTCFullYear", &prototype, 1);
        make_builtin_fn(Self::set_utc_hours, "setUTCHours", &prototype, 1);
        make_builtin_fn(
            Self::set_utc_milliseconds,
            "setUTCMilliseconds",
            &prototype,
            1,
        );
        make_builtin_fn(Self::set_utc_minutes, "setUTCMinutes", &prototype, 1);
        make_builtin_fn(Self::set_utc_month, "setUTCMonth", &prototype, 1);
        make_builtin_fn(Self::set_utc_seconds, "setUTCSeconds", &prototype, 1);
        make_builtin_fn(Self::to_date_string, "toDateString", &prototype, 0);
        make_builtin_fn(Self::to_gmt_string, "toGMTString", &prototype, 0);
        make_builtin_fn(Self::to_iso_string, "toISOString", &prototype, 0);
        make_builtin_fn(Self::to_json, "toJSON", &prototype, 0);
        // Locale strings
        make_builtin_fn(Self::to_string, "toString", &prototype, 0);
        make_builtin_fn(Self::to_time_string, "toTimeString", &prototype, 0);
        make_builtin_fn(Self::to_utc_string, "toUTCString", &prototype, 0);

        let date_time_object = make_constructor_fn(
            Self::NAME,
            Self::LENGTH,
            Self::make_date,
            global,
            prototype,
            true,
            true,
        );

        make_builtin_fn(Self::now, "now", &date_time_object, 0);
        make_builtin_fn(Self::parse, "parse", &date_time_object, 1);
        make_builtin_fn(Self::utc, "UTC", &date_time_object, 7);
        (Self::NAME, date_time_object)
    }
}
