#[cfg(test)]
mod tests;

use super::JsArgs;
use crate::{
    builtins::BuiltIn,
    context::intrinsics::StandardConstructors,
    error::JsNativeError,
    js_string,
    object::{
        internal_methods::get_prototype_from_constructor, ConstructorBuilder, JsObject, ObjectData,
    },
    string::utf16,
    symbol::WellKnownSymbols,
    value::{JsValue, PreferredType},
    Context, JsResult,
};
use boa_profiler::Profiler;
use chrono::{prelude::*, Duration, LocalResult};
use std::fmt::Display;
use tap::{Conv, Pipe};

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
    value.map_or(true, |value| value == 0f64 || value.is_normal())
}

macro_rules! check_normal_opt {
    ($($v:expr),+) => {
        $(is_zero_or_normal_opt($v.into()) &&)+ true
    };
}

#[inline]
fn ignore_ambiguity<T>(result: LocalResult<T>) -> Option<T> {
    match result {
        LocalResult::Ambiguous(v, _) | LocalResult::Single(v) => Some(v),
        LocalResult::None => None,
    }
}

macro_rules! getter_method {
    ($name:ident) => {{
        fn get_value(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
            Ok(JsValue::new(this_time_value(this)?.$name()))
        }
        get_value
    }};
    (Self::$name:ident) => {{
        fn get_value(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
            Ok(JsValue::new(Date::$name()))
        }
        get_value
    }};
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

impl BuiltIn for Date {
    const NAME: &'static str = "Date";

    fn init(context: &mut Context) -> Option<JsValue> {
        let _timer = Profiler::global().start_event(Self::NAME, "init");

        ConstructorBuilder::with_standard_constructor(
            context,
            Self::constructor,
            context.intrinsics().constructors().date().clone(),
        )
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
        .method(Self::get_timezone_offset, "getTimezoneOffset", 0)
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
        .method(Self::set_date, "setDate", 1)
        .method(Self::set_full_year, "setFullYear", 3)
        .method(Self::set_hours, "setHours", 4)
        .method(Self::set_milliseconds, "setMilliseconds", 1)
        .method(Self::set_minutes, "setMinutes", 3)
        .method(Self::set_month, "setMonth", 2)
        .method(Self::set_seconds, "setSeconds", 2)
        .method(Self::set_year, "setYear", 1)
        .method(Self::set_time, "setTime", 1)
        .method(Self::set_utc_date, "setUTCDate", 1)
        .method(Self::set_utc_full_year, "setUTCFullYear", 3)
        .method(Self::set_utc_hours, "setUTCHours", 4)
        .method(Self::set_utc_milliseconds, "setUTCMilliseconds", 1)
        .method(Self::set_utc_minutes, "setUTCMinutes", 3)
        .method(Self::set_utc_month, "setUTCMonth", 2)
        .method(Self::set_utc_seconds, "setUTCSeconds", 2)
        .method(Self::to_date_string, "toDateString", 0)
        .method(getter_method!(to_gmt_string), "toGMTString", 0)
        .method(Self::to_iso_string, "toISOString", 0)
        .method(Self::to_json, "toJSON", 1)
        // Locale strings
        .method(Self::to_string, "toString", 0)
        .method(Self::to_time_string, "toTimeString", 0)
        .method(getter_method!(to_utc_string), "toUTCString", 0)
        .method(getter_method!(value_of), "valueOf", 0)
        .method(
            Self::to_primitive,
            (WellKnownSymbols::to_primitive(), "[Symbol.toPrimitive]"),
            1,
        )
        .static_method(Self::now, "now", 0)
        .static_method(Self::parse, "parse", 1)
        .static_method(Self::utc, "UTC", 7)
        .build()
        .conv::<JsValue>()
        .pipe(Some)
    }
}

impl Date {
    /// The amount of arguments this function object takes.
    pub(crate) const LENGTH: usize = 7;

    /// Check if the time (number of milliseconds) is in the expected range.
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
    #[inline]
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
            let year = year.unwrap_or_else(|| f64::from(naive.year())) as i32;
            let month = month.unwrap_or_else(|| f64::from(naive.month0())) as i32;
            let day = (day.unwrap_or_else(|| f64::from(naive.day())) as i32).checked_sub(1)?;
            let hour = hour.unwrap_or_else(|| f64::from(naive.hour())) as i64;
            let minute = minute.unwrap_or_else(|| f64::from(naive.minute())) as i64;
            let second = second.unwrap_or_else(|| f64::from(naive.second())) as i64;
            let millisecond = millisecond
                .unwrap_or_else(|| f64::from(naive.nanosecond()) / NANOS_PER_MS as f64)
                as i64;

            let (year, month, day) = fix_day(year, month, day)?;

            let duration_hour = Duration::milliseconds(hour.checked_mul(MILLIS_PER_HOUR)?);
            let duration_minute = Duration::milliseconds(minute.checked_mul(MILLIS_PER_MINUTE)?);
            let duration_second = Duration::milliseconds(second.checked_mul(MILLIS_PER_SECOND)?);
            let duration_millisecond = Duration::milliseconds(millisecond);

            let duration = duration_hour
                .checked_add(&duration_minute)?
                .checked_add(&duration_second)?
                .checked_add(&duration_millisecond)?;

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
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            Ok(Self::make_date_string())
        } else {
            let prototype =
                get_prototype_from_constructor(new_target, StandardConstructors::date, context)?;
            Ok(if args.is_empty() {
                Self::make_date_now(prototype)
            } else if args.len() == 1 {
                Self::make_date_single(prototype, args, context)?
            } else {
                Self::make_date_multiple(prototype, args, context)?
            }
            .into())
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
    pub(crate) fn make_date_now(prototype: JsObject) -> JsObject {
        JsObject::from_proto_and_data(prototype, ObjectData::date(Self::default()))
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
        prototype: JsObject,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let value = &args[0];
        let tv = match this_time_value(value) {
            Ok(dt) => dt.0,
            _ => match value.to_primitive(context, PreferredType::Default)? {
                JsValue::String(ref str) => str
                    .to_std_string()
                    .ok()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s.as_str()).ok())
                    .map(|dt| dt.naive_utc()),
                tv => {
                    let tv = tv.to_number(context)?;
                    if tv.is_nan() {
                        None
                    } else {
                        let secs = (tv / 1_000f64) as i64;
                        let nano_secs = ((tv % 1_000f64) * 1_000_000f64) as u32;
                        NaiveDateTime::from_timestamp_opt(secs, nano_secs)
                    }
                }
            },
        };

        let tv = tv.filter(|time| Self::time_clip(time.timestamp_millis() as f64).is_some());
        Ok(JsObject::from_proto_and_data(
            prototype,
            ObjectData::date(Self(tv)),
        ))
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
        prototype: JsObject,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsObject> {
        let mut year = args[0].to_number(context)?;
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
            return Ok(JsObject::from_proto_and_data(
                prototype,
                ObjectData::date(Self(None)),
            ));
        }

        if (0.0..=99.0).contains(&year) {
            year += 1900.0;
        }

        let mut date = Self(
            NaiveDateTime::from_timestamp_opt(0, 0)
                .and_then(|local| ignore_ambiguity(Local.from_local_datetime(&local)))
                .map(|local| local.naive_utc())
                .filter(|time| Self::time_clip(time.timestamp_millis() as f64).is_some()),
        );

        date.set_components(
            false,
            Some(year),
            Some(month),
            Some(day),
            Some(hour),
            Some(min),
            Some(sec),
            Some(milli),
        );

        Ok(JsObject::from_proto_and_data(
            prototype,
            ObjectData::date(date),
        ))
    }

    /// `Date.prototype[@@toPrimitive]`
    ///
    /// The [@@toPrimitive]() method converts a Date object to a primitive value.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype-@@toprimitive
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/@@toPrimitive
    #[allow(clippy::wrong_self_convention)]
    pub(crate) fn to_primitive(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let O be the this value.
        // 2. If Type(O) is not Object, throw a TypeError exception.
        let o = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Date.prototype[@@toPrimitive] called on non object")
        })?;

        let hint = args.get_or_undefined(0);

        let try_first = match hint.as_string() {
            // 3. If hint is "string" or "default", then
            // a. Let tryFirst be string.
            Some(string) if string == utf16!("string") || string == utf16!("default") => {
                PreferredType::String
            }
            // 4. Else if hint is "number", then
            // a. Let tryFirst be number.
            Some(number) if number == utf16!("number") => PreferredType::Number,
            // 5. Else, throw a TypeError exception.
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("Date.prototype[@@toPrimitive] called with invalid hint")
                    .into())
            }
        };

        // 6. Return ? OrdinaryToPrimitive(O, tryFirst).
        o.ordinary_to_primitive(context, try_first)
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
        self.to_local().map_or(f64::NAN, |dt| f64::from(dt.day()))
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
            f64::from(weekday)
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
        self.to_local().map_or(f64::NAN, |dt| f64::from(dt.year()))
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
        self.to_local().map_or(f64::NAN, |dt| f64::from(dt.hour()))
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
        self.to_local().map_or(f64::NAN, |dt| {
            f64::from(dt.nanosecond()) / NANOS_PER_MS as f64
        })
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
        self.to_local()
            .map_or(f64::NAN, |dt| f64::from(dt.minute()))
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
        self.to_local()
            .map_or(f64::NAN, |dt| f64::from(dt.month0()))
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
        self.to_local()
            .map_or(f64::NAN, |dt| f64::from(dt.second()))
    }

    /// `Date.prototype.getYear()`
    ///
    /// The `getYear()` method returns the year in the specified date according to local time.
    /// Because `getYear()` does not return full years ("year 2000 problem"), it is no longer used
    /// and has been replaced by the `getFullYear()` method.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.getyear
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getYear
    pub fn get_year(&self) -> f64 {
        self.to_local()
            .map_or(f64::NAN, |dt| f64::from(dt.year()) - 1900f64)
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
    /// The `getTimezoneOffset()` method returns the time zone difference, in minutes, from current locale (host system
    /// settings) to UTC.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.gettimezoneoffset
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/getTimezoneOffset
    #[inline]
    pub fn get_timezone_offset(
        this: &JsValue,
        _: &[JsValue],
        _: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let t = this_time_value(this)?;

        // 2. If t is NaN, return NaN.
        if t.0.is_none() {
            return Ok(JsValue::nan());
        }

        // 3. Return (t - LocalTime(t)) / msPerMinute.
        Ok(JsValue::new(
            f64::from(-Local::now().offset().local_minus_utc()) / 60f64,
        ))
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
        self.to_utc().map_or(f64::NAN, |dt| f64::from(dt.day()))
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
            f64::from(weekday)
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
        self.to_utc().map_or(f64::NAN, |dt| f64::from(dt.year()))
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
        self.to_utc().map_or(f64::NAN, |dt| f64::from(dt.hour()))
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
        self.to_utc().map_or(f64::NAN, |dt| {
            f64::from(dt.nanosecond()) / NANOS_PER_MS as f64
        })
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
        self.to_utc().map_or(f64::NAN, |dt| f64::from(dt.minute()))
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
        self.to_utc().map_or(f64::NAN, |dt| f64::from(dt.month0()))
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
        self.to_utc().map_or(f64::NAN, |dt| f64::from(dt.second()))
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
    pub fn set_date(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let t be LocalTime(? thisTimeValue(this value)).
        let mut t = this_time_value(this)?;

        // 2. Let dt be ? ToNumber(date).
        let dt = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. Let newDate be MakeDate(MakeDay(YearFromTime(t), MonthFromTime(t), dt), TimeWithinDay(t)).
        t.set_components(false, None, None, Some(dt), None, None, None, None);

        // 4. Let u be TimeClip(UTC(newDate)).
        let u = t.get_time();

        // 5. Set the [[DateValue]] internal slot of this Date object to u.
        this.set_data(ObjectData::date(t));

        // 6. Return u.
        Ok(u.into())
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
    pub fn set_full_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = this_time_value(this)?;

        // 2. If t is NaN, set t to +0𝔽; otherwise, set t to LocalTime(t).
        if t.0.is_none() {
            t.0 = NaiveDateTime::from_timestamp_opt(0, 0)
                .and_then(|local| ignore_ambiguity(Local.from_local_datetime(&local)))
                .map(|local| local.naive_utc())
                .filter(|time| Self::time_clip(time.timestamp_millis() as f64).is_some());
        }

        // 3. Let y be ? ToNumber(year).
        let y = args.get_or_undefined(0).to_number(context)?;

        // 4. If month is not present, let m be MonthFromTime(t); otherwise, let m be ? ToNumber(month).
        let m = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 5. If date is not present, let dt be DateFromTime(t); otherwise, let dt be ? ToNumber(date).
        let dt = args.get(2).map(|v| v.to_number(context)).transpose()?;

        // 6. Let newDate be MakeDate(MakeDay(y, m, dt), TimeWithinDay(t)).
        t.set_components(false, Some(y), m, dt, None, None, None, None);

        // 7. Let u be TimeClip(UTC(newDate)).
        let u = t.get_time();

        // 8. Set the [[DateValue]] internal slot of this Date object to u.
        this.set_data(ObjectData::date(t));

        // 9. Return u.
        Ok(u.into())
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
    pub fn set_hours(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let t be LocalTime(? thisTimeValue(this value)).
        let mut t = this_time_value(this)?;

        // 2. Let h be ? ToNumber(hour).
        let h = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. If min is not present, let m be MinFromTime(t); otherwise, let m be ? ToNumber(min).
        let m = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 4. If sec is not present, let s be SecFromTime(t); otherwise, let s be ? ToNumber(sec).
        let sec = args.get(2).map(|v| v.to_number(context)).transpose()?;

        // 5. If ms is not present, let milli be msFromTime(t); otherwise, let milli be ? ToNumber(ms).
        let milli = args.get(3).map(|v| v.to_number(context)).transpose()?;

        // 6. Let date be MakeDate(Day(t), MakeTime(h, m, s, milli)).
        t.set_components(false, None, None, None, Some(h), m, sec, milli);

        // 7. Let u be TimeClip(UTC(date)).
        let u = t.get_time();

        // 8. Set the [[DateValue]] internal slot of this Date object to u.
        this.set_data(ObjectData::date(t));

        // 9. Return u.
        Ok(u.into())
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
    pub fn set_milliseconds(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be LocalTime(? thisTimeValue(this value)).
        let mut t = this_time_value(this)?;

        // 2. Set ms to ? ToNumber(ms).
        let ms = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. Let time be MakeTime(HourFromTime(t), MinFromTime(t), SecFromTime(t), ms).
        t.set_components(false, None, None, None, None, None, None, Some(ms));

        // 4. Let u be TimeClip(UTC(MakeDate(Day(t), time))).
        let u = t.get_time();

        // 5. Set the [[DateValue]] internal slot of this Date object to u.
        this.set_data(ObjectData::date(t));

        // 6. Return u.
        Ok(u.into())
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
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be LocalTime(? thisTimeValue(this value)).
        let mut t = this_time_value(this)?;

        // 2. Let m be ? ToNumber(min).
        let m = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. If sec is not present, let s be SecFromTime(t); otherwise, let s be ? ToNumber(sec).
        let s = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 4. If ms is not present, let milli be msFromTime(t); otherwise, let milli be ? ToNumber(ms).
        let milli = args.get(2).map(|v| v.to_number(context)).transpose()?;

        // 5. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), m, s, milli)).
        t.set_components(false, None, None, None, None, Some(m), s, milli);

        // 6. Let u be TimeClip(UTC(date)).
        let u = t.get_time();

        // 7. Set the [[DateValue]] internal slot of this Date object to u.
        this.set_data(ObjectData::date(t));

        // 8. Return u.
        Ok(u.into())
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
    pub fn set_month(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let t be LocalTime(? thisTimeValue(this value)).
        let mut t = this_time_value(this)?;

        // 2. Let m be ? ToNumber(month).
        let m = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. If date is not present, let dt be DateFromTime(t); otherwise, let dt be ? ToNumber(date).
        let dt = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 4. Let newDate be MakeDate(MakeDay(YearFromTime(t), m, dt), TimeWithinDay(t)).
        t.set_components(false, None, Some(m), dt, None, None, None, None);

        // 5. Let u be TimeClip(UTC(newDate)).
        let u = t.get_time();

        // 6. Set the [[DateValue]] internal slot of this Date object to u.
        this.set_data(ObjectData::date(t));

        // 7. Return u.
        Ok(u.into())
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
    pub fn set_seconds(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be LocalTime(? thisTimeValue(this value)).
        let mut t = this_time_value(this)?;

        // 2. Let s be ? ToNumber(sec).
        let s = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. If ms is not present, let milli be msFromTime(t); otherwise, let milli be ? ToNumber(ms).
        let milli = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 4. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), MinFromTime(t), s, milli)).
        t.set_components(false, None, None, None, None, None, Some(s), milli);

        // 5. Let u be TimeClip(UTC(date)).
        let u = t.get_time();

        // 6. Set the [[DateValue]] internal slot of this Date object to u.
        this.set_data(ObjectData::date(t));

        // 7. Return u.
        Ok(u.into())
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
    pub fn set_year(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = this_time_value(this)?;

        // 2. If t is NaN, set t to +0𝔽; otherwise, set t to LocalTime(t).
        if t.0.is_none() {
            t.0 = NaiveDateTime::from_timestamp_opt(0, 0)
                .and_then(|local| ignore_ambiguity(Local.from_local_datetime(&local)))
                .map(|local| local.naive_utc())
                .filter(|time| Self::time_clip(time.timestamp_millis() as f64).is_some());
        }

        // 3. Let y be ? ToNumber(year).
        let mut y = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 4. If y is NaN, then
        if y.is_nan() {
            // a. Set the [[DateValue]] internal slot of this Date object to NaN.
            this.set_data(ObjectData::date(Self(None)));

            // b. Return NaN.
            return Ok(JsValue::nan());
        }

        // 5. Let yi be ! ToIntegerOrInfinity(y).
        // 6. If 0 ≤ yi ≤ 99, let yyyy be 1900𝔽 + 𝔽(yi).
        // 7. Else, let yyyy be y.
        if (0f64..=99f64).contains(&y) {
            y += 1900f64;
        }

        // 8. Let d be MakeDay(yyyy, MonthFromTime(t), DateFromTime(t)).
        // 9. Let date be UTC(MakeDate(d, TimeWithinDay(t))).
        t.set_components(false, Some(y), None, None, None, None, None, None);

        // 10. Set the [[DateValue]] internal slot of this Date object to TimeClip(date).
        this.set_data(ObjectData::date(t));

        // 11. Return the value of the [[DateValue]] internal slot of this Date object.
        Ok(t.get_time().into())
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
    pub fn set_time(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Perform ? thisTimeValue(this value).
        this_time_value(this)?;

        // 2. Let t be ? ToNumber(time).
        let t = if let Some(t) = args.get(0) {
            let t = t.to_number(context)?;
            let seconds = (t / 1_000f64) as i64;
            let nanoseconds = ((t % 1_000f64) * 1_000_000f64) as u32;
            Self(
                ignore_ambiguity(Local.timestamp_opt(seconds, nanoseconds))
                    .map(|dt| dt.naive_utc()),
            )
        } else {
            Self(None)
        };

        // 3. Let v be TimeClip(t).
        let v = t.get_time();

        // 4. Set the [[DateValue]] internal slot of this Date object to v.
        this.set_data(ObjectData::date(t));

        // 5. Return v.
        Ok(v.into())
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
    pub fn set_utc_date(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = this_time_value(this)?;

        // 2. Let dt be ? ToNumber(date).
        let dt = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. Let newDate be MakeDate(MakeDay(YearFromTime(t), MonthFromTime(t), dt), TimeWithinDay(t)).
        t.set_components(true, None, None, Some(dt), None, None, None, None);

        // 4. Let v be TimeClip(newDate).
        let v = t.get_time();

        // 5. Set the [[DateValue]] internal slot of this Date object to v.
        this.set_data(ObjectData::date(t));

        // 6. Return v.
        Ok(v.into())
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
    pub fn set_utc_full_year(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = this_time_value(this)?;

        // 2. If t is NaN, set t to +0𝔽.
        if t.0.is_none() {
            t.0 = NaiveDateTime::from_timestamp_opt(0, 0)
                .and_then(|local| ignore_ambiguity(Local.from_local_datetime(&local)))
                .map(|local| local.naive_utc())
                .filter(|time| Self::time_clip(time.timestamp_millis() as f64).is_some());
        }

        // 3. Let y be ? ToNumber(year).
        let y = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 4. If month is not present, let m be MonthFromTime(t); otherwise, let m be ? ToNumber(month).
        let m = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 5. If date is not present, let dt be DateFromTime(t); otherwise, let dt be ? ToNumber(date).
        let dt = args.get(2).map(|v| v.to_number(context)).transpose()?;

        // 6. Let newDate be MakeDate(MakeDay(y, m, dt), TimeWithinDay(t)).
        t.set_components(true, Some(y), m, dt, None, None, None, None);

        // 7. Let v be TimeClip(newDate).
        let v = t.get_time();

        // 8. Set the [[DateValue]] internal slot of this Date object to v.
        this.set_data(ObjectData::date(t));

        // 9. Return v.
        Ok(v.into())
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
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = this_time_value(this)?;

        // 2. Let h be ? ToNumber(hour).
        let h = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. If min is not present, let m be MinFromTime(t); otherwise, let m be ? ToNumber(min).
        let m = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 4. If sec is not present, let s be SecFromTime(t); otherwise, let s be ? ToNumber(sec).
        let sec = args.get(2).map(|v| v.to_number(context)).transpose()?;

        // 5. If ms is not present, let milli be msFromTime(t); otherwise, let milli be ? ToNumber(ms).
        let ms = args.get(3).map(|v| v.to_number(context)).transpose()?;

        // 6. Let newDate be MakeDate(Day(t), MakeTime(h, m, s, milli)).
        t.set_components(true, None, None, None, Some(h), m, sec, ms);

        // 7. Let v be TimeClip(newDate).
        let v = t.get_time();

        // 8. Set the [[DateValue]] internal slot of this Date object to v.
        this.set_data(ObjectData::date(t));

        // 9. Return v.
        Ok(v.into())
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
    pub fn set_utc_milliseconds(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = this_time_value(this)?;

        // 2. Let milli be ? ToNumber(ms).
        let ms = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. Let time be MakeTime(HourFromTime(t), MinFromTime(t), SecFromTime(t), milli).
        t.set_components(true, None, None, None, None, None, None, Some(ms));

        // 4. Let v be TimeClip(MakeDate(Day(t), time)).
        let v = t.get_time();

        // 5. Set the [[DateValue]] internal slot of this Date object to v.
        this.set_data(ObjectData::date(t));

        // 6. Return v.
        Ok(v.into())
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
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = this_time_value(this)?;

        // 2. Let m be ? ToNumber(min).
        let m = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. If sec is not present, let s be SecFromTime(t).
        // 4. Else,
        // a. Let s be ? ToNumber(sec).
        let s = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 5. If ms is not present, let milli be msFromTime(t).
        // 6. Else,
        // a. Let milli be ? ToNumber(ms).
        let milli = args.get(2).map(|v| v.to_number(context)).transpose()?;

        // 7. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), m, s, milli)).
        t.set_components(true, None, None, None, None, Some(m), s, milli);

        // 8. Let v be TimeClip(date).
        let v = t.get_time();

        // 9. Set the [[DateValue]] internal slot of this Date object to v.
        this.set_data(ObjectData::date(t));

        // 10. Return v.
        Ok(v.into())
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
    pub fn set_utc_month(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = this_time_value(this)?;

        // 2. Let m be ? ToNumber(month).
        let m = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. If date is not present, let dt be DateFromTime(t).
        // 4. Else,
        // a. Let dt be ? ToNumber(date).
        let dt = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 5. Let newDate be MakeDate(MakeDay(YearFromTime(t), m, dt), TimeWithinDay(t)).
        t.set_components(true, None, Some(m), dt, None, None, None, None);

        // 6. Let v be TimeClip(newDate).
        let v = t.get_time();

        // 7. Set the [[DateValue]] internal slot of this Date object to v.
        this.set_data(ObjectData::date(t));

        // 8. Return v.
        Ok(v.into())
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
    pub fn set_utc_seconds(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // 1. Let t be ? thisTimeValue(this value).
        let mut t = this_time_value(this)?;

        // 2. Let s be ? ToNumber(sec).
        let s = args
            .get(0)
            .cloned()
            .unwrap_or_default()
            .to_number(context)?;

        // 3. If ms is not present, let milli be msFromTime(t).
        // 4. Else,
        // a. Let milli be ? ToNumber(ms).
        let milli = args.get(1).map(|v| v.to_number(context)).transpose()?;

        // 5. Let date be MakeDate(Day(t), MakeTime(HourFromTime(t), MinFromTime(t), s, milli)).
        t.set_components(true, None, None, None, None, None, Some(s), milli);

        // 6. Let v be TimeClip(date).
        let v = t.get_time();

        // 7. Set the [[DateValue]] internal slot of this Date object to v.
        this.set_data(ObjectData::date(t));

        // 8. Return v.
        Ok(v.into())
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
    pub fn to_date_string(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be this Date object.
        // 2. Let tv be ? thisTimeValue(O).
        let tv = this_time_value(this)?;

        // 3. If tv is NaN, return "Invalid Date".
        // 4. Let t be LocalTime(tv).
        // 5. Return DateString(t).
        if let Some(t) = tv.0 {
            Ok(Local::now()
                .timezone()
                .from_utc_datetime(&t)
                .format("%a %b %d %Y")
                .to_string()
                .into())
        } else {
            Ok(js_string!("Invalid Date").into())
        }
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
    #[allow(clippy::wrong_self_convention)]
    pub fn to_iso_string(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        if let Some(t) = this_time_value(this)?.0 {
            Ok(Utc::now()
                .timezone()
                .from_utc_datetime(&t)
                .format("%Y-%m-%dT%H:%M:%S.%3fZ")
                .to_string()
                .into())
        } else {
            Err(JsNativeError::range()
                .with_message("Invalid time value")
                .into())
        }
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
    pub fn to_json(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be ? ToObject(this value).
        let o = this.to_object(context)?;

        // 2. Let tv be ? ToPrimitive(O, number).
        let tv = this.to_primitive(context, PreferredType::Number)?;

        // 3. If Type(tv) is Number and tv is not finite, return null.
        if let Some(number) = tv.as_number() {
            if !number.is_finite() {
                return Ok(JsValue::null());
            }
        }

        // 4. Return ? Invoke(O, "toISOString").
        let func = o.get("toISOString", context)?;
        context.call(&func, &o.into(), &[])
    }

    /// `Date.prototype.toString()`
    ///
    /// The toString() method returns a string representing the specified Date object.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.prototype.tostring
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toString
    #[allow(clippy::wrong_self_convention)]
    pub fn to_string(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let tv be ? thisTimeValue(this value).
        let tv = this_time_value(this)?;

        // 2. Return ToDateString(tv).
        if let Some(t) = tv.0 {
            Ok(Local::now()
                .timezone()
                .from_utc_datetime(&t)
                .format("%a %b %d %Y %H:%M:%S GMT%z")
                .to_string()
                .into())
        } else {
            Ok(js_string!("Invalid Date").into())
        }
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
    pub fn to_time_string(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        // 1. Let O be this Date object.
        // 2. Let tv be ? thisTimeValue(O).
        let tv = this_time_value(this)?;

        // 3. If tv is NaN, return "Invalid Date".
        // 4. Let t be LocalTime(tv).
        // 5. Return the string-concatenation of TimeString(t) and TimeZoneString(tv).
        if let Some(t) = tv.0 {
            Ok(Local::now()
                .timezone()
                .from_utc_datetime(&t)
                .format("%H:%M:%S GMT%z")
                .to_string()
                .into())
        } else {
            Ok(js_string!("Invalid Date").into())
        }
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
        self.to_utc().map_or_else(
            || "Invalid Date".to_string(),
            |date_time| date_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string(),
        )
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
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn now(_: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::new(Utc::now().timestamp_millis() as f64))
    }

    /// `Date.parse()`
    ///
    /// The `Date.parse()` method parses a string representation of a date, and returns the number of milliseconds since
    /// January 1, 1970, 00:00:00 UTC or `NaN` if the string is unrecognized or, in some cases, contains illegal date
    /// values.
    ///
    /// More information:
    ///  - [ECMAScript reference][spec]
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma262/#sec-date.parse
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/parse
    pub(crate) fn parse(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // This method is implementation-defined and discouraged, so we just require the same format as the string
        // constructor.

        let Some(date) = args.get(0) else {
            return Ok(JsValue::nan());
        };

        let date = date.to_string(context)?;

        Ok(JsValue::new(
            date.to_std_string()
                .ok()
                .and_then(|s| DateTime::parse_from_rfc3339(s.as_str()).ok())
                .map_or(f64::NAN, |v| v.naive_utc().timestamp_millis() as f64),
        ))
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
    pub(crate) fn utc(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let year = args
            .get(0)
            .map_or(Ok(f64::NAN), |value| value.to_number(context))?;
        let month = args
            .get(1)
            .map_or(Ok(0f64), |value| value.to_number(context))?;
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
pub fn this_time_value(value: &JsValue) -> JsResult<Date> {
    value
        .as_object()
        .and_then(|obj| obj.borrow().as_date().copied())
        .ok_or_else(|| {
            JsNativeError::typ()
                .with_message("'this' is not a Date")
                .into()
        })
}
