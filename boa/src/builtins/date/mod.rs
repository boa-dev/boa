#[cfg(test)]
mod tests;

use crate::{
    builtins::BuiltIn,
    gc::{empty_trace, Finalize, Trace},
    object::{ConstructorBuilder, ObjectData, PROTOTYPE},
    property::Attribute,
    value::{JsValue, PreferredType},
    BoaProfiler, Context, Result,
};
use chrono::{prelude::*, Duration, LocalResult};
use std::fmt::Display;

/// The number of nanoseconds in a millisecond.
const NANOS_PER_MS: i64 = 1_000_000;
/// The number of milliseconds in an hour.
const MILLIS_PER_HOUR: i64 = 3_600_000;
/// The number of milliseconds in a minute.
const MILLIS_PER_MINUTE: i64 = 60_000;
/// The number of milliseconds in a second.
const MILLIS_PER_SECOND: i64 = 1000;

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

#[inline]
fn ignore_ambiguity<T>(result: LocalResult<T>) -> Option<T> {
    match result {
        LocalResult::Ambiguous(v, _) => Some(v),
        LocalResult::Single(v) => Some(v),
        LocalResult::None => None,
    }
}

macro_rules! getter_method {
    ($name:ident) => {{
        fn get_value(this: &JsValue, _: &[JsValue], context: &mut Context) -> Result<JsValue> {
            Ok(JsValue::new(this_time_value(this, context)?.$name()))
        }
        get_value
    }};
    (Self::$name:ident) => {{
        fn get_value(_: &JsValue, _: &[JsValue], _: &mut Context) -> Result<JsValue> {
            Ok(JsValue::new(Date::$name()))
        }
        get_value
    }};
}

macro_rules! setter_method {
    ($name:ident($($e:expr),* $(,)?)) => {{
        fn set_value(this: &JsValue, args: &[JsValue], context: &mut Context) -> Result<JsValue> {
            let mut result = this_time_value(this, context)?;
            result.$name(
                $(
                    args
                        .get($e)
                        .and_then(|value| {
                            value.to_numeric_number(context).map_or_else(
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
                ),*
            );

            this.set_data(ObjectData::Date(result));
            Ok(JsValue::new(result.get_time()))
        }
        set_value
    }};
}

#[derive(Debug, Finalize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(Option<NaiveDateTime>);

impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.to_local() {
            Some(v) => write!(f, "{}", v.format("%a %b %d %Y %H:%M:%S GMT%:z")),
            _ => write!(f, "Invalid Date"),
        }
    }
}

unsafe impl Trace for Date {
    // Date is a stack value, it doesn't require tracing.
    // only safe if `chrono` never implements `Trace` for `NaiveDateTime`
    empty_trace!();
}

impl Default for Date {
    fn default() -> Self {
        Self(Some(Utc::now().naive_utc()))
    }
}

impl BuiltIn for Date {
    const NAME: &'static str = "Date";

    fn attribute() -> Attribute {
        Attribute::WRITABLE | Attribute::NON_ENUMERABLE | Attribute::CONFIGURABLE
    }

    fn init(context: &mut Context) -> (&'static str, JsValue, Attribute) {
        let _timer = BoaProfiler::global().start_event(Self::NAME, "init");

        let date_object = ConstructorBuilder::new(context, Self::constructor)
            .name(Self::NAME)
            .length(Self::LENGTH)
            .method(getter_method!(get_date), "getDate", 0)
            .method(getter_method!(get_day), "getDay", 0)
            .method(getter_method!(get_full_year), "getFullYear", 0)
            .method(getter_method!(get_hours), "getHours", 0)
            .method(getter_method!(get_milliseconds), "getMilliseconds", 0)
            .method(getter_method!(get_minutes), "getMinutes", 0)
            .method(getter_method!(get_month), "getMonth", 0)
            .method(getter_method!(get_seconds), "getSeconds", 0)
            .method(getter_method!(get_time), "getTime", 0)
            .method(getter_method!(get_year), "getYear", 0)
            .method(
                getter_method!(Self::get_timezone_offset),
                "getTimezoneOffset",
                0,
            )
            .method(getter_method!(get_utc_date), "getUTCDate", 0)
            .method(getter_method!(get_utc_day), "getUTCDay", 0)
            .method(getter_method!(get_utc_full_year), "getUTCFullYear", 0)
            .method(getter_method!(get_utc_hours), "getUTCHours", 0)
            .method(
                getter_method!(get_utc_milliseconds),
                "getUTCMilliseconds",
                0,
            )
            .method(getter_method!(get_utc_minutes), "getUTCMinutes", 0)
            .method(getter_method!(get_utc_month), "getUTCMonth", 0)
            .method(getter_method!(get_utc_seconds), "getUTCSeconds", 0)
            .method(setter_method!(set_date(0)), "setDate", 1)
            .method(setter_method!(set_full_year(0, 1, 2)), "setFullYear", 1)
            .method(setter_method!(set_hours(0, 1, 2, 3)), "setHours", 1)
            .method(setter_method!(set_milliseconds(0)), "setMilliseconds", 1)
            .method(setter_method!(set_minutes(0, 1, 2)), "setMinutes", 1)
            .method(setter_method!(set_month(0, 1)), "setMonth", 1)
            .method(setter_method!(set_seconds(0, 1)), "setSeconds", 1)
            .method(setter_method!(set_year(0, 1, 2)), "setYear", 1)
            .method(setter_method!(set_time(0)), "setTime", 1)
            .method(setter_method!(set_utc_date(0)), "setUTCDate", 1)
            .method(
                setter_method!(set_utc_full_year(0, 1, 2)),
                "setUTCFullYear",
                1,
            )
            .method(setter_method!(set_utc_hours(0, 1, 2, 3)), "setUTCHours", 1)
            .method(
                setter_method!(set_utc_milliseconds(0)),
                "setUTCMilliseconds",
                1,
            )
            .method(setter_method!(set_utc_minutes(0, 1, 2)), "setUTCMinutes", 1)
            .method(setter_method!(set_utc_month(0, 1)), "setUTCMonth", 1)
            .method(setter_method!(set_utc_seconds(0, 1)), "setUTCSeconds", 1)
            .method(getter_method!(to_date_string), "toDateString", 0)
            .method(getter_method!(to_gmt_string), "toGMTString", 0)
            .method(getter_method!(to_iso_string), "toISOString", 0)
            .method(getter_method!(to_json), "toJSON", 0)
            // Locale strings
            .method(getter_method!(to_string), "toString", 0)
            .method(getter_method!(to_time_string), "toTimeString", 0)
            .method(getter_method!(to_utc_string), "toUTCString", 0)
            .method(getter_method!(value_of), "valueOf", 0)
            .static_method(Self::now, "now", 0)
            .static_method(Self::parse, "parse", 1)
            .static_method(Self::utc, "UTC", 7)
            .build();

        (Self::NAME, date_object.into(), Self::attribute())
    }
}

impl Date {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 7;

    /// Check if the time (number of miliseconds) is in the expected range.
    /// Returns None if the time is not in the range, otherwise returns the time itself in option.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-timeclip
    #[inline]
    pub fn time_clip(time: f64) -> Option<f64> {
        if time.abs() > 8.64e15 {
            None
        } else {
            Some(time)
        }
    }

    /// Converts the `Date` to a local `DateTime`.
    ///
    /// If the `Date` is invalid (i.e. NAN), this function will return `None`.
    pub fn to_local(self) -> Option<DateTime<Local>> {
        self.0
            .map(|utc| Local::now().timezone().from_utc_datetime(&utc))
    }

    /// Converts the `Date` to a UTC `DateTime`.
    ///
    /// If the `Date` is invalid (i.e. NAN), this function will return `None`.
    pub fn to_utc(self) -> Option<DateTime<Utc>> {
        self.0
            .map(|utc| Utc::now().timezone().from_utc_datetime(&utc))
    }

    /// Optionally sets the individual components of the `Date`.
    ///
    /// Each component does not have to be within the range of valid values. For example, if `month` is too large
    /// then `year` will be incremented by the required amount.
    #[allow(clippy::too_many_arguments)]
    pub fn set_components(
        &mut self,
        utc: bool,
        year: Option<f64>,
        month: Option<f64>,
        day: Option<f64>,
        hour: Option<f64>,
        minute: Option<f64>,
        second: Option<f64>,
        millisecond: Option<f64>,
    ) {
        #[inline]
        fn num_days_in(year: i32, month: u32) -> Option<u32> {
            let month = month + 1; // zero-based for calculations

            Some(
                NaiveDate::from_ymd_opt(
                    match month {
                        12 => year.checked_add(1)?,
                        _ => year,
                    },
                    match month {
                        12 => 1,
                        _ => month + 1,
                    },
                    1,
                )?
                .signed_duration_since(NaiveDate::from_ymd_opt(year, month, 1)?)
                .num_days() as u32,
            )
        }

        #[inline]
        fn fix_month(year: i32, month: i32) -> Option<(i32, u32)> {
            let year = year.checked_add(month / 12)?;

            if month < 0 {
                let year = year.checked_sub(1)?;
                let month = (11 + (month + 1) % 12) as u32;
                Some((year, month))
            } else {
                let month = (month % 12) as u32;
                Some((year, month))
            }
        }

        #[inline]
        fn fix_day(mut year: i32, mut month: i32, mut day: i32) -> Option<(i32, u32, u32)> {
            loop {
                if day < 0 {
                    let (fixed_year, fixed_month) = fix_month(year, month.checked_sub(1)?)?;

                    year = fixed_year;
                    month = fixed_month as i32;
                    day += num_days_in(fixed_year, fixed_month)? as i32;
                } else {
                    let (fixed_year, fixed_month) = fix_month(year, month)?;
                    let num_days = num_days_in(fixed_year, fixed_month)? as i32;

                    if day >= num_days {
                        day -= num_days;
                        month = month.checked_add(1)?;
                    } else {
                        break;
                    }
                }
            }

            let (fixed_year, fixed_month) = fix_month(year, month)?;
            Some((fixed_year, fixed_month, day as u32))
        }

        // If any of the args are infinity or NaN, return an invalid date.
        if !check_normal_opt!(year, month, day, hour, minute, second, millisecond) {
            self.0 = None;
            return;
        }

        let naive = if utc {
            self.to_utc().map(|dt| dt.naive_utc())
        } else {
            self.to_local().map(|dt| dt.naive_local())
        };

        self.0 = naive.and_then(|naive| {
            let year = year.unwrap_or_else(|| naive.year() as f64) as i32;
            let month = month.unwrap_or_else(|| naive.month0() as f64) as i32;
            let day = (day.unwrap_or_else(|| naive.day() as f64) as i32).checked_sub(1)?;
            let hour = hour.unwrap_or_else(|| naive.hour() as f64) as i64;
            let minute = minute.unwrap_or_else(|| naive.minute() as f64) as i64;
            let second = second.unwrap_or_else(|| naive.second() as f64) as i64;
            let millisecond = millisecond
                .unwrap_or_else(|| naive.nanosecond() as f64 / NANOS_PER_MS as f64)
                as i64;

            let (year, month, day) = fix_day(year, month, day)?;

            let duration_hour = Duration::milliseconds(hour.checked_mul(MILLIS_PER_HOUR)?);
            let duration_minute = Duration::milliseconds(minute.checked_mul(MILLIS_PER_MINUTE)?);
            let duration_second = Duration::milliseconds(second.checked_mul(MILLIS_PER_SECOND)?);
            let duration_milisecond = Duration::milliseconds(millisecond);

            let duration = duration_hour
                .checked_add(&duration_minute)?
                .checked_add(&duration_second)?
                .checked_add(&duration_milisecond)?;

            NaiveDate::from_ymd_opt(year, month + 1, day + 1)
                .and_then(|dt| dt.and_hms(0, 0, 0).checked_add_signed(duration))
                .and_then(|dt| {
                    if utc {
                        Some(Utc.from_utc_datetime(&dt).naive_utc())
                    } else {
                        ignore_ambiguity(Local.from_local_datetime(&dt)).map(|dt| dt.naive_utc())
                    }
                })
                .filter(|dt| Self::time_clip(dt.timestamp_millis() as f64).is_some())
        });
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
    pub(crate) fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> Result<JsValue> {
        if new_target.is_undefined() {
            Ok(Self::make_date_string())
        } else {
            let prototype = new_target
                .as_object()
                .and_then(|obj| {
                    obj.__get__(&PROTOTYPE.into(), obj.clone().into(), context)
                        .map(|o| o.as_object())
                        .transpose()
                })
                .transpose()?
                .unwrap_or_else(|| context.standard_objects().object_object().prototype());
            let obj = context.construct_object();
            obj.set_prototype_instance(prototype.into());
            let this = obj.into();
            if args.is_empty() {
                Ok(Self::make_date_now(&this))
            } else if args.len() == 1 {
                Self::make_date_single(&this, args, context)
            } else {
                Self::make_date_multiple(&this, args, context)
            }
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
    pub(crate) fn make_date_string() -> JsValue {
        JsValue::new(Local::now().to_rfc3339())
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
    pub(crate) fn make_date_now(this: &JsValue) -> JsValue {
        let date = Date::default();
        this.set_data(ObjectData::Date(date));
        this.clone()
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
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> Result<JsValue> {
        let value = &args[0];
        let tv = match this_time_value(value, context) {
            Ok(dt) => dt.0,
            _ => match value.to_primitive(context, PreferredType::Default)? {
                JsValue::String(ref str) => match chrono::DateTime::parse_from_rfc3339(str) {
                    Ok(dt) => Some(dt.naive_utc()),
                    _ => None,
                },
                tv => {
                    let tv = tv.to_number(context)?;
                    let secs = (tv / 1_000f64) as i64;
                    let nsecs = ((tv % 1_000f64) * 1_000_000f64) as u32;
                    NaiveDateTime::from_timestamp_opt(secs, nsecs)
                }
            },
        };

        let tv = tv.filter(|time| Self::time_clip(time.timestamp_millis() as f64).is_some());
        let date = Date(tv);
        this.set_data(ObjectData::Date(date));
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
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> Result<JsValue> {
        let year = args[0].to_number(context)?;
        let month = args[1].to_number(context)?;
        let day = args
            .get(2)
            .map_or(Ok(1f64), |value| value.to_number(context))?;
        let hour = args
            .get(3)
            .map_or(Ok(0f64), |value| value.to_number(context))?;
        let min = args
            .get(4)
            .map_or(Ok(0f64), |value| value.to_number(context))?;
        let sec = args
            .get(5)
            .map_or(Ok(0f64), |value| value.to_number(context))?;
        let milli = args
            .get(6)
            .map_or(Ok(0f64), |value| value.to_number(context))?;

        // If any of the args are infinity or NaN, return an invalid date.
        if !check_normal_opt!(year, month, day, hour, min, sec, milli) {
            let date = Date(None);
            this.set_data(ObjectData::Date(date));
            return Ok(this.clone());
        }

        let year = year as i32;
        let month = month as u32;
        let day = day as u32;
        let hour = hour as u32;
        let min = min as u32;
        let sec = sec as u32;
        let milli = milli as u32;

        let year = if (0..=99).contains(&year) {
            1900 + year
        } else {
            year
        };

        let final_date = NaiveDate::from_ymd_opt(year, month + 1, day)
            .and_then(|naive_date| naive_date.and_hms_milli_opt(hour, min, sec, milli))
            .and_then(|local| ignore_ambiguity(Local.from_local_datetime(&local)))
            .map(|local| local.naive_utc())
            .filter(|time| Self::time_clip(time.timestamp_millis() as f64).is_some());

        let date = Date(final_date);
        this.set_data(ObjectData::Date(date));
        Ok(this.clone())
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
    pub fn get_date(&self) -> f64 {
        self.to_local().map_or(f64::NAN, |dt| dt.day() as f64)
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
    pub fn get_day(&self) -> f64 {
        self.to_local().map_or(f64::NAN, |dt| {
            let weekday = dt.weekday() as u32;
            let weekday = (weekday + 1) % 7; // 0 represents Monday in Chrono
            weekday as f64
        })
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
    pub fn get_full_year(&self) -> f64 {
        self.to_local().map_or(f64::NAN, |dt| dt.year() as f64)
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
    pub fn get_hours(&self) -> f64 {
        self.to_local().map_or(f64::NAN, |dt| dt.hour() as f64)
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
    pub fn get_milliseconds(&self) -> f64 {
        self.to_local()
            .map_or(f64::NAN, |dt| dt.nanosecond() as f64 / NANOS_PER_MS as f64)
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
    pub fn get_minutes(&self) -> f64 {
        self.to_local().map_or(f64::NAN, |dt| dt.minute() as f64)
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
    pub fn get_month(&self) -> f64 {
        self.to_local().map_or(f64::NAN, |dt| dt.month0() as f64)
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
    pub fn get_seconds(&self) -> f64 {
        self.to_local().map_or(f64::NAN, |dt| dt.second() as f64)
    }

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
    pub fn get_year(&self) -> f64 {
        self.to_local()
            .map_or(f64::NAN, |dt| dt.year() as f64 - 1900f64)
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
    pub fn get_time(&self) -> f64 {
        self.to_utc()
            .map_or(f64::NAN, |dt| dt.timestamp_millis() as f64)
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
    pub fn get_timezone_offset() -> f64 {
        let offset_seconds = chrono::Local::now().offset().local_minus_utc() as f64;
        offset_seconds / 60f64
    }

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
    pub fn get_utc_date(&self) -> f64 {
        self.to_utc().map_or(f64::NAN, |dt| dt.day() as f64)
    }

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
    pub fn get_utc_day(&self) -> f64 {
        self.to_utc().map_or(f64::NAN, |dt| {
            let weekday = dt.weekday() as u32;
            let weekday = (weekday + 1) % 7; // 0 represents Monday in Chrono
            weekday as f64
        })
    }

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
    pub fn get_utc_full_year(&self) -> f64 {
        self.to_utc().map_or(f64::NAN, |dt| dt.year() as f64)
    }

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
    pub fn get_utc_hours(&self) -> f64 {
        self.to_utc().map_or(f64::NAN, |dt| dt.hour() as f64)
    }

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
    pub fn get_utc_milliseconds(&self) -> f64 {
        self.to_utc()
            .map_or(f64::NAN, |dt| dt.nanosecond() as f64 / NANOS_PER_MS as f64)
    }

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
    pub fn get_utc_minutes(&self) -> f64 {
        self.to_utc().map_or(f64::NAN, |dt| dt.minute() as f64)
    }

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
    pub fn get_utc_month(&self) -> f64 {
        self.to_utc().map_or(f64::NAN, |dt| dt.month0() as f64)
    }

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
    pub fn get_utc_seconds(&self) -> f64 {
        self.to_utc().map_or(f64::NAN, |dt| dt.second() as f64)
    }

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
    pub fn set_date(&mut self, day: Option<f64>) {
        if let Some(day) = day {
            self.set_components(false, None, None, Some(day), None, None, None, None)
        } else {
            self.0 = None
        }
    }

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
    pub fn set_full_year(&mut self, year: Option<f64>, month: Option<f64>, day: Option<f64>) {
        if let Some(year) = year {
            self.set_components(false, Some(year), month, day, None, None, None, None)
        } else {
            self.0 = None
        }
    }

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
    pub fn set_hours(
        &mut self,
        hour: Option<f64>,
        minute: Option<f64>,
        second: Option<f64>,
        millisecond: Option<f64>,
    ) {
        if let Some(hour) = hour {
            self.set_components(
                false,
                None,
                None,
                None,
                Some(hour),
                minute,
                second,
                millisecond,
            )
        } else {
            self.0 = None
        }
    }

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
    pub fn set_milliseconds(&mut self, millisecond: Option<f64>) {
        if let Some(millisecond) = millisecond {
            self.set_components(false, None, None, None, None, None, None, Some(millisecond))
        } else {
            self.0 = None
        }
    }

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
    pub fn set_minutes(
        &mut self,
        minute: Option<f64>,
        second: Option<f64>,
        millisecond: Option<f64>,
    ) {
        if let Some(minute) = minute {
            self.set_components(
                false,
                None,
                None,
                None,
                None,
                Some(minute),
                second,
                millisecond,
            )
        } else {
            self.0 = None
        }
    }

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
    pub fn set_month(&mut self, month: Option<f64>, day: Option<f64>) {
        if let Some(month) = month {
            self.set_components(false, None, Some(month), day, None, None, None, None)
        } else {
            self.0 = None
        }
    }

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
    pub fn set_seconds(&mut self, second: Option<f64>, millisecond: Option<f64>) {
        if let Some(second) = second {
            self.set_components(
                false,
                None,
                None,
                None,
                None,
                None,
                Some(second),
                millisecond,
            )
        } else {
            self.0 = None
        }
    }

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
    pub fn set_year(&mut self, year: Option<f64>, month: Option<f64>, day: Option<f64>) {
        if let Some(mut year) = year {
            year += if (0f64..100f64).contains(&year) {
                1900f64
            } else {
                0f64
            };
            self.set_components(false, Some(year), month, day, None, None, None, None)
        } else {
            self.0 = None
        }
    }

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
    pub fn set_time(&mut self, time: Option<f64>) {
        if let Some(time) = time {
            let secs = (time / 1_000f64) as i64;
            let nsecs = ((time % 1_000f64) * 1_000_000f64) as u32;
            self.0 = ignore_ambiguity(Local.timestamp_opt(secs, nsecs)).map(|dt| dt.naive_utc());
        } else {
            self.0 = None
        }
    }

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
    pub fn set_utc_date(&mut self, day: Option<f64>) {
        if let Some(day) = day {
            self.set_components(true, None, None, Some(day), None, None, None, None)
        } else {
            self.0 = None
        }
    }

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
    pub fn set_utc_full_year(&mut self, year: Option<f64>, month: Option<f64>, day: Option<f64>) {
        if let Some(year) = year {
            self.set_components(true, Some(year), month, day, None, None, None, None)
        } else {
            self.0 = None
        }
    }

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
    pub fn set_utc_hours(
        &mut self,
        hour: Option<f64>,
        minute: Option<f64>,
        second: Option<f64>,
        millisecond: Option<f64>,
    ) {
        if let Some(hour) = hour {
            self.set_components(
                true,
                None,
                None,
                None,
                Some(hour),
                minute,
                second,
                millisecond,
            )
        } else {
            self.0 = None
        }
    }

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
    pub fn set_utc_milliseconds(&mut self, millisecond: Option<f64>) {
        if let Some(millisecond) = millisecond {
            self.set_components(true, None, None, None, None, None, None, Some(millisecond))
        } else {
            self.0 = None
        }
    }

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
    pub fn set_utc_minutes(
        &mut self,
        minute: Option<f64>,
        second: Option<f64>,
        millisecond: Option<f64>,
    ) {
        if let Some(minute) = minute {
            self.set_components(
                true,
                None,
                None,
                None,
                None,
                Some(minute),
                second,
                millisecond,
            )
        } else {
            self.0 = None
        }
    }

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
    pub fn set_utc_month(&mut self, month: Option<f64>, day: Option<f64>) {
        if let Some(month) = month {
            self.set_components(true, None, Some(month), day, None, None, None, None)
        } else {
            self.0 = None
        }
    }

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
    pub fn set_utc_seconds(&mut self, second: Option<f64>, millisecond: Option<f64>) {
        if let Some(second) = second {
            self.set_components(
                true,
                None,
                None,
                None,
                None,
                None,
                Some(second),
                millisecond,
            )
        } else {
            self.0 = None
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
    pub fn to_date_string(self) -> String {
        self.to_local()
            .map(|date_time| date_time.format("%a %b %d %Y").to_string())
            .unwrap_or_else(|| "Invalid Date".to_string())
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
    pub fn to_gmt_string(self) -> String {
        self.to_utc_string()
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
    pub fn to_iso_string(self) -> String {
        self.to_utc()
            // RFC 3389 uses +0.00 for UTC, where JS expects Z, so we can't use the built-in chrono function.
            .map(|f| f.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string())
            .unwrap_or_else(|| "Invalid Date".to_string())
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
    pub fn to_json(self) -> String {
        self.to_iso_string()
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
    pub fn to_time_string(self) -> String {
        self.to_local()
            .map(|date_time| date_time.format("%H:%M:%S GMT%:z").to_string())
            .unwrap_or_else(|| "Invalid Date".to_string())
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
    pub fn to_utc_string(self) -> String {
        self.to_utc()
            .map(|date_time| date_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string())
            .unwrap_or_else(|| "Invalid Date".to_string())
    }

    /// `Date.prototype.valueOf()`
    ///
    /// The `valueOf()` method returns the primitive value of a `Date` object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.valueof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/valueOf
    pub fn value_of(&self) -> f64 {
        self.get_time()
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
    pub(crate) fn now(_: &JsValue, _: &[JsValue], _: &mut Context) -> Result<JsValue> {
        Ok(JsValue::new(Utc::now().timestamp_millis() as f64))
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
    pub(crate) fn parse(_: &JsValue, args: &[JsValue], context: &mut Context) -> Result<JsValue> {
        // This method is implementation-defined and discouraged, so we just require the same format as the string
        // constructor.

        if args.is_empty() {
            return Ok(JsValue::nan());
        }

        match DateTime::parse_from_rfc3339(&args[0].to_string(context)?) {
            Ok(v) => Ok(JsValue::new(v.naive_utc().timestamp_millis() as f64)),
            _ => Ok(JsValue::new(f64::NAN)),
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
    pub(crate) fn utc(_: &JsValue, args: &[JsValue], context: &mut Context) -> Result<JsValue> {
        let year = args
            .get(0)
            .map_or(Ok(f64::NAN), |value| value.to_number(context))?;
        let month = args
            .get(1)
            .map_or(Ok(1f64), |value| value.to_number(context))?;
        let day = args
            .get(2)
            .map_or(Ok(1f64), |value| value.to_number(context))?;
        let hour = args
            .get(3)
            .map_or(Ok(0f64), |value| value.to_number(context))?;
        let min = args
            .get(4)
            .map_or(Ok(0f64), |value| value.to_number(context))?;
        let sec = args
            .get(5)
            .map_or(Ok(0f64), |value| value.to_number(context))?;
        let milli = args
            .get(6)
            .map_or(Ok(0f64), |value| value.to_number(context))?;

        if !check_normal_opt!(year, month, day, hour, min, sec, milli) {
            return Ok(JsValue::nan());
        }

        let year = year as i32;
        let month = month as u32;
        let day = day as u32;
        let hour = hour as u32;
        let min = min as u32;
        let sec = sec as u32;
        let milli = milli as u32;

        let year = if (0..=99).contains(&year) {
            1900 + year
        } else {
            year
        };

        NaiveDate::from_ymd_opt(year, month + 1, day)
            .and_then(|f| f.and_hms_milli_opt(hour, min, sec, milli))
            .and_then(|f| Self::time_clip(f.timestamp_millis() as f64))
            .map_or(Ok(JsValue::nan()), |time| Ok(JsValue::new(time)))
    }
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
pub fn this_time_value(value: &JsValue, context: &mut Context) -> Result<Date> {
    if let JsValue::Object(ref object) = value {
        if let ObjectData::Date(ref date) = object.borrow().data {
            return Ok(*date);
        }
    }
    Err(context.construct_type_error("'this' is not a Date"))
}
